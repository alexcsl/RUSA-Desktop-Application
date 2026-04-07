// commands/da_operations.rs — Data Analyst: browse, statistical operations, security report
//
// Use cases:
// - UC-DA-03 (supplement): Read all non-medical data (da_browse_data)
// - UC-DA-05: Apply statistical operations (da_compute_operation)
// - UC-DA-06: Submit security incident report (da_submit_security_report)
//
// Security: DataAnalyst role guard on all commands.
// Redis: browse results cached 60s; invalidated on table writes (out-of-scope for reads).
// No N+1: browse uses single SELECT row_to_json; compute is pure Rust.
// Medical exclusion: patient_records, treatment_logs, medical_inventory are NOT in whitelist.

use std::collections::HashMap;

use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;
use uuid::Uuid;

use crate::{
    audit::{write_audit_log, AuditOperation},
    error::AppError,
    state::{AppState, Role},
};

// ── Whitelist & helpers ───────────────────────────────────────────────────────

/// Non-medical tables the Data Analyst may browse.
/// Medical tables (patient_records, treatment_logs, medical_inventory) are explicitly excluded.
const ALLOWED_BROWSE_TABLES: &[&str] = &[
    "users",
    "roles",
    "base_locations",
    "experiments",
    "experiment_daily_logs",
    "science_archive",
    "species_archive",
    "approved_tests",
    "test_proposals",
    "missions",
    "territory_names",
    "events",
    "event_documents",
    "incident_reports",
    "daily_security_reports",
    "data_requests",
    "data_responses",
    "progress_reports",
    "audit_log",
    "tasks",
    "vote_sessions",
    "vote_records",
    "settlements",
    "sanitary_tasks",
    "sanitary_shifts",
    "sanitary_divisions",
];

/// Validates that a string is a safe SQL identifier (alphanumeric + underscore only).
pub fn is_valid_identifier(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

// ── Structs ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct BrowseDataPayload {
    pub table_name: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub filter_column: Option<String>,
    pub filter_value: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ComputeOperationPayload {
    /// One of: avg, mode, median, variance, max, min, filter, pivot, graph
    pub operation: String,
    pub data: Vec<serde_json::Value>,
    pub column: String,
    /// For filter: column to match. For pivot/graph: grouping column.
    pub filter_column: Option<String>,
    /// For filter: value substring to match.
    pub filter_value: Option<String>,
    /// For pivot/graph: optional secondary column to aggregate.
    pub agg_column: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ComputeResult {
    pub operation: String,
    pub column: String,
    pub result: serde_json::Value,
    pub row_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct DaSecurityReportPayload {
    pub incident_type: String,
    pub location: String,
    pub description: String,
    pub severity: String,
    pub occurred_at: Option<String>,
    pub recommended_action: Option<String>,
}

// ── Statistical helpers (pure Rust, no DB) ────────────────────────────────────

fn extract_numeric(val: &serde_json::Value) -> Option<f64> {
    match val {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

fn get_column_numerics(data: &[serde_json::Value], col: &str) -> Vec<f64> {
    data.iter()
        .filter_map(|row| row.as_object()?.get(col))
        .filter_map(extract_numeric)
        .collect()
}

fn compute_avg(nums: &[f64]) -> f64 {
    if nums.is_empty() {
        return 0.0;
    }
    nums.iter().sum::<f64>() / nums.len() as f64
}

fn compute_median(nums: &mut [f64]) -> f64 {
    if nums.is_empty() {
        return 0.0;
    }
    nums.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = nums.len() / 2;
    if nums.len() % 2 == 0 {
        (nums[mid - 1] + nums[mid]) / 2.0
    } else {
        nums[mid]
    }
}

fn compute_mode(nums: &[f64]) -> f64 {
    let mut counts: HashMap<i64, usize> = HashMap::new();
    for &n in nums {
        *counts.entry((n * 1_000_000.0) as i64).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(k, _)| k as f64 / 1_000_000.0)
        .unwrap_or(0.0)
}

fn compute_variance(nums: &[f64]) -> f64 {
    if nums.len() < 2 {
        return 0.0;
    }
    let mean = compute_avg(nums);
    nums.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (nums.len() - 1) as f64
}

// ── UC-DA-03 (data access): Browse non-medical data ──────────────────────────

/// Returns rows from any non-medical table (with optional ILIKE filter).
/// Results are cached in Redis for 60 seconds.
///
/// **Access:** DataAnalyst or Administrator.
#[tauri::command]
pub async fn da_browse_data(
    state: State<'_, AppState>,
    payload: BrowseDataPayload,
) -> Result<Vec<serde_json::Value>, AppError> {
    let user = crate::require_auth_any!(state, [Role::DataAnalyst, Role::TheStatistician, Role::TheLibrarian, Role::TheDirector, Role::Administrator]);

    // Validate table name:
    // TheLibrarian, TheDirector, and Administrator can access all tables (including medical).
    // DataAnalyst and TheStatistician are restricted to the non-medical whitelist.
    let is_privileged = matches!(
        user.role,
        Role::TheLibrarian | Role::TheDirector | Role::Administrator
    );
    if !is_privileged && !ALLOWED_BROWSE_TABLES.contains(&payload.table_name.as_str()) {
        return Err(AppError::Forbidden);
    }
    // All roles: validate identifier for SQL injection protection
    if !is_valid_identifier(&payload.table_name) {
        return Err(AppError::Forbidden);
    }

    let limit = payload.limit.unwrap_or(50).clamp(1, 500);
    let offset = payload.offset.unwrap_or(0).max(0);

    let cache_key = format!(
        "da_browse:{}:{}:{}:{}:{}",
        payload.table_name,
        limit,
        offset,
        payload.filter_column.as_deref().unwrap_or(""),
        payload.filter_value.as_deref().unwrap_or(""),
    );

    // Redis cache check
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(parsed) = serde_json::from_str::<Vec<serde_json::Value>>(&cached) {
                return Ok(parsed);
            }
        }
    }

    // Build safe query — table name is whitelisted; filter column validated; value is bound
    let rows: Vec<serde_json::Value> = match (&payload.filter_column, &payload.filter_value) {
        (Some(fc), Some(fv)) => {
            if !is_valid_identifier(fc) {
                return Err(AppError::Internal("Invalid filter column name.".into()));
            }
            let sql = format!(
                "SELECT row_to_json(t) AS row_data \
                 FROM (SELECT * FROM {} WHERE {}::text ILIKE $1 LIMIT $2 OFFSET $3) t",
                payload.table_name, fc
            );
            sqlx::query(&sql)
                .bind(format!("%{}%", fv))
                .bind(limit)
                .bind(offset)
                .fetch_all(&state.db_pool)
                .await?
                .into_iter()
                .map(|r| r.try_get::<serde_json::Value, _>(0).unwrap_or(serde_json::Value::Null))
                .collect()
        }
        _ => {
            let sql = format!(
                "SELECT row_to_json(t) AS row_data \
                 FROM (SELECT * FROM {} LIMIT $1 OFFSET $2) t",
                payload.table_name
            );
            sqlx::query(&sql)
                .bind(limit)
                .bind(offset)
                .fetch_all(&state.db_pool)
                .await?
                .into_iter()
                .map(|r| r.try_get::<serde_json::Value, _>(0).unwrap_or(serde_json::Value::Null))
                .collect()
        }
    };

    // Cache 60 seconds
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(json) = serde_json::to_string(&rows) {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json, 60).await;
        }
    }

    Ok(rows)
}

// ── UC-DA-05: Statistical Operations ──────────────────────────────────────────

/// Apply a statistical operation to a JSON dataset (pure Rust, no DB query).
/// Supports: avg, mode, median, variance, max, min, filter, pivot, graph.
///
/// **Access:** DataAnalyst or Administrator.
#[tauri::command]
pub async fn da_compute_operation(
    state: State<'_, AppState>,
    payload: ComputeOperationPayload,
) -> Result<ComputeResult, AppError> {
    let _user = crate::require_auth_any!(state, [Role::DataAnalyst, Role::TheStatistician, Role::TheLibrarian, Role::TheDirector, Role::Administrator]);

    let row_count = payload.data.len();

    let result = match payload.operation.as_str() {
        "avg" => {
            let nums = get_column_numerics(&payload.data, &payload.column);
            serde_json::json!({ "value": compute_avg(&nums), "count": nums.len() })
        }
        "mode" => {
            let nums = get_column_numerics(&payload.data, &payload.column);
            serde_json::json!({ "value": compute_mode(&nums), "count": nums.len() })
        }
        "median" => {
            let mut nums = get_column_numerics(&payload.data, &payload.column);
            serde_json::json!({ "value": compute_median(&mut nums), "count": nums.len() })
        }
        "variance" => {
            let nums = get_column_numerics(&payload.data, &payload.column);
            let var = compute_variance(&nums);
            serde_json::json!({ "variance": var, "std_dev": var.sqrt(), "count": nums.len() })
        }
        "max" => {
            let nums = get_column_numerics(&payload.data, &payload.column);
            let max = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            serde_json::json!({ "value": if nums.is_empty() { serde_json::Value::Null } else { max.into() }, "count": nums.len() })
        }
        "min" => {
            let nums = get_column_numerics(&payload.data, &payload.column);
            let min = nums.iter().cloned().fold(f64::INFINITY, f64::min);
            serde_json::json!({ "value": if nums.is_empty() { serde_json::Value::Null } else { min.into() }, "count": nums.len() })
        }
        "filter" => {
            let fc = payload.filter_column.as_deref().unwrap_or(&payload.column);
            let fv = payload.filter_value.as_deref().unwrap_or("").to_lowercase();
            let filtered: Vec<_> = payload
                .data
                .iter()
                .filter(|row| {
                    row.as_object()
                        .and_then(|o| o.get(fc))
                        .map(|v| {
                            let s = match v {
                                serde_json::Value::String(s) => s.to_lowercase(),
                                other => other.to_string().to_lowercase(),
                            };
                            s.contains(&fv)
                        })
                        .unwrap_or(false)
                })
                .cloned()
                .collect();
            let matched = filtered.len();
            serde_json::json!({ "rows": filtered, "matched": matched, "is_table": true })
        }
        "pivot" | "graph" => {
            // Group by column, aggregate (count or sum agg_column) per group
            let mut groups: HashMap<String, Vec<f64>> = HashMap::new();
            for row in &payload.data {
                if let Some(obj) = row.as_object() {
                    let key = obj
                        .get(&payload.column)
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string().trim_matches('"').to_string(),
                        })
                        .unwrap_or_else(|| "null".to_string());
                    let num_val = payload
                        .agg_column
                        .as_deref()
                        .and_then(|ac| obj.get(ac))
                        .and_then(extract_numeric)
                        .unwrap_or(1.0);
                    groups.entry(key).or_default().push(num_val);
                }
            }
            let mut labels: Vec<String> = groups.keys().cloned().collect();
            labels.sort();
            let values: Vec<f64> = labels.iter().map(|l| groups[l].iter().sum()).collect();
            let agg_label = payload.agg_column.as_deref().unwrap_or("count");
            serde_json::json!({
                "labels": labels,
                "values": values,
                "agg_column": agg_label,
                "is_chart": true,
            })
        }
        unknown => {
            return Err(AppError::Internal(format!(
                "Unknown operation '{}'. Valid: avg, mode, median, variance, max, min, filter, pivot, graph.",
                unknown
            )));
        }
    };

    Ok(ComputeResult {
        operation: payload.operation,
        column: payload.column,
        result,
        row_count,
    })
}

// ── UC-DA-04: Write non-medical data ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct WriteDataPayload {
    pub table_name: String,
    /// JSON object: column name → value pairs for the new row.
    pub row_data: serde_json::Value,
}

/// Insert a new row into any non-medical whitelisted table.
/// Column names are validated as safe SQL identifiers; values are bound as parameters.
///
/// **Access:** DataAnalyst, TheStatistician, or Administrator.
#[tauri::command]
pub async fn da_write_data(
    state: State<'_, AppState>,
    payload: WriteDataPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::DataAnalyst, Role::TheStatistician, Role::Administrator]);

    // Validate table
    if !ALLOWED_BROWSE_TABLES.contains(&payload.table_name.as_str()) {
        return Err(AppError::Forbidden);
    }
    if !is_valid_identifier(&payload.table_name) {
        return Err(AppError::Forbidden);
    }

    let obj = payload
        .row_data
        .as_object()
        .ok_or_else(|| AppError::Internal("row_data must be a JSON object.".into()))?;

    if obj.is_empty() {
        return Err(AppError::Internal("row_data must have at least one field.".into()));
    }

    // Validate all column names
    for col in obj.keys() {
        if !is_valid_identifier(col) {
            return Err(AppError::Internal(format!("Invalid column name: {col}")));
        }
    }

    // Build: INSERT INTO table (col1, col2, ...) VALUES ($1, $2, ...) RETURNING id
    let cols: Vec<&str> = obj.keys().map(String::as_str).collect();
    let placeholders: Vec<String> = (1..=cols.len()).map(|i| format!("${i}")).collect();
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        payload.table_name,
        cols.join(", "),
        placeholders.join(", ")
    );

    let mut query = sqlx::query(&sql);
    for val in obj.values() {
        query = match val {
            serde_json::Value::String(s) => query.bind(s.clone()),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    query.bind(i)
                } else {
                    query.bind(n.as_f64().unwrap_or(0.0))
                }
            }
            serde_json::Value::Bool(b) => query.bind(*b),
            serde_json::Value::Null => query.bind(Option::<String>::None),
            other => query.bind(other.to_string()),
        };
    }

    query.execute(&state.db_pool).await?;

    // Invalidate Redis browse cache for this table (pattern delete not available in all drivers;
    // we simply let the 60s TTL expire — acceptable for analyst write-then-browse workflow).

    write_audit_log(
        &state.db_pool,
        &payload.table_name,
        Uuid::nil(), // new row id unknown without RETURNING; nil UUID marks DA inserts
        AuditOperation::Create,
        user.id,
        None,
        Some(payload.row_data.clone()),
    )
    .await?;

    Ok(())
}

// ── UC-DA-06: Security Incident Report ────────────────────────────────────────

/// Submit a security incident report to the Galactic Security team.
/// Inserts into incident_reports and notifies all active GalacticSecurityHead users.
///
/// **Access:** DataAnalyst or Administrator.
#[tauri::command]
pub async fn da_submit_security_report(
    state: State<'_, AppState>,
    payload: DaSecurityReportPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::DataAnalyst, Role::Administrator]);

    let valid_severities = ["low", "medium", "high", "critical"];
    if !valid_severities.contains(&payload.severity.as_str()) {
        return Err(AppError::Internal(
            "Severity must be one of: low, medium, high, critical.".into(),
        ));
    }

    let occurred_at: Option<chrono::DateTime<chrono::Utc>> = payload
        .occurred_at
        .as_deref()
        .and_then(|s| s.parse().ok());

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO incident_reports
          (reported_by, source, incident_type, location, description,
           severity, occurred_at, recommended_action)
        VALUES ($1, 'external_report', $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.incident_type)
    .bind(&payload.location)
    .bind(&payload.description)
    .bind(&payload.severity)
    .bind(occurred_at)
    .bind(&payload.recommended_action)
    .fetch_one(&state.db_pool)
    .await?;

    let report_id = row.0;

    // Notify all active GalacticSecurityHead users
    let security_heads: Vec<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT u.id FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE r.name = 'GalacticSecurityHead'
          AND u.deleted_at IS NULL
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    for (head_id,) in &security_heads {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'security_report:submitted', $2::jsonb)
            "#,
        )
        .bind(head_id)
        .bind(serde_json::json!({
            "report_id": report_id,
            "reporter_name": user.full_name,
            "incident_type": payload.incident_type,
            "severity": payload.severity,
            "location": payload.location,
            "source": "data_analyst",
        }))
        .execute(&state.db_pool)
        .await?;
    }

    write_audit_log(
        &state.db_pool,
        "incident_reports",
        report_id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "incident_type": payload.incident_type,
            "severity": payload.severity,
            "location": payload.location,
            "source": "external_report",
        })),
    )
    .await?;

    Ok(report_id)
}
