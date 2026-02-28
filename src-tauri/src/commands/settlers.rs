// commands/settlers.rs — Exoplanet Settlers subsystem commands
// Source of truth: 06_EXOPLANET_SETTLERS.md, 00_MASTER_GUIDE.md
//
// Use cases implemented:
//   Shared (PS/TS):
//     UC-PS-01: Receive Task Assignment           (stl_get_my_tasks)
//     UC-PS-02: Submit Progress Report             (stl_submit_progress_report)
//     UC-PS-03 / UC-TS-01: Submit Anomaly Report   (stl_submit_anomaly_report)
//     UC-PS-04 / UC-TS-02: Submit Complaint        (stl_submit_complaint)
//     UC-PS-05 / UC-TS-03: Submit Supply Request   (stl_submit_supply_request)
//     UC-PS-06: View Settlement Inventory           (stl_get_settlement_inventory)
//
//   Settler Commander:
//     UC-SC-01: Assign Task                        (stl_assign_task)
//     UC-SC-02: Review Incoming Queue               (stl_get_incoming_queue)
//     UC-SC-02: Reject Incoming                     (stl_reject_incoming)
//     UC-SC-02: Forward to Directors                (stl_forward_to_directors)
//     UC-SC-02A: Commander Anomaly Report           (stl_submit_commander_anomaly)
//     UC-SC-03: Request Abandonment                 (stl_request_abandonment)
//     UC-SC-04: Request Repatriation                (stl_request_repatriation)
//     UC-SC-05: Set House Arrest                    (stl_set_house_arrest)
//     UC-SC-06: Settlement Dashboard                (stl_get_dashboard)
//     UC-SC-07: Commander Supply Request            (stl_submit_commander_supply)
//     UC-SC-08: Manage Inventory                    (stl_manage_inventory)
//
//   Civil Engineer:
//     UC-CE-01: Construction Progress Report        (stl_submit_construction_report)
//     UC-CE-02: Request Materials                   (stl_request_materials)
//     UC-CE-03: Log Building Health                 (stl_log_building_health)
//     UC-CE-04: View Residence                      (stl_get_residence)
//
//   Farmer:
//     UC-FA-01: Request Farming Supplies            (stl_request_farming_supplies)
//     UC-FA-02: Log Farm Health Check               (stl_log_farm_health)

use chrono::{DateTime, NaiveDate, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::State;
use uuid::Uuid;

use crate::{
    audit::{write_audit_log, AuditOperation},
    error::AppError,
    state::{AppState, Role},
};

// ── Role sets ─────────────────────────────────────────────────────────────────

const SETTLER_ROLES: [Role; 4] = [
    Role::SettlerCommander,
    Role::CivilEngineer,
    Role::Farmer,
    Role::TemporarySetter,
];

const PERMANENT_SETTLER_ROLES: [Role; 3] = [
    Role::SettlerCommander,
    Role::CivilEngineer,
    Role::Farmer,
];

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Look up the settlement the user is currently assigned to.
async fn get_user_settlement(pool: &sqlx::PgPool, user_id: Uuid) -> Result<Uuid, AppError> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT settlement_id FROM settler_assignments WHERE user_id = $1 AND departed_at IS NULL",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    row.map(|r| r.0)
        .ok_or_else(|| AppError::Internal("No active settlement assignment found.".into()))
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  STRUCTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// ── Task inbox ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SettlerTaskSummary {
    pub id: Uuid,
    pub title: String,
    pub scope: Option<String>,
    pub urgency: Option<String>,
    pub deadline: Option<NaiveDate>,
    pub status: String,
    pub assigned_by_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SettlerTaskDetail {
    pub id: Uuid,
    pub settlement_id: Uuid,
    pub assigned_by: Uuid,
    pub assigned_by_name: String,
    pub assigned_to: Uuid,
    pub assigned_to_name: String,
    pub title: String,
    pub description: Option<String>,
    pub scope: Option<String>,
    pub urgency: Option<String>,
    pub deadline: Option<NaiveDate>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

// ── Progress report ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ProgressReportPayload {
    pub task_id: Uuid,
    pub week: Option<String>,
    pub rag_status: Option<String>,
    pub progress_made: String,
    pub materials_equipment: Option<String>,
}

// ── Anomaly report ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AnomalyReportPayload {
    pub description: String,
    pub location_in_settlement: Option<String>,
    pub danger_level: Option<String>,
    pub severity: String,
    pub category: Option<serde_json::Value>,
}

// ── Complaint ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ComplaintPayload {
    pub subject_user_id: Uuid,
    pub incident_description: String,
}

// ── Supply request ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SupplyRequestPayload {
    pub items: serde_json::Value, // JSONB array
    pub justification: Option<String>,
}

// ── Inventory item ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct InventoryItem {
    pub id: Uuid,
    pub item_name: String,
    pub category: Option<String>,
    pub quantity: i32,
    pub unit: Option<String>,
    pub min_threshold: i32,
    pub updated_at: DateTime<Utc>,
}

// ── Commander structs ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AssignTaskPayload {
    pub assigned_to: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub scope: Option<String>,
    pub urgency: Option<String>,
    pub deadline: Option<String>, // YYYY-MM-DD
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncomingQueueItem {
    pub id: Uuid,
    pub item_type: String,
    pub submitted_by: Uuid,
    pub submitted_by_name: String,
    pub summary: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RejectPayload {
    pub item_id: Uuid,
    pub item_type: String, // 'supply_request' | 'anomaly_report' | 'complaint'
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ForwardPayload {
    pub item_id: Uuid,
    pub item_type: String, // 'supply_request' | 'anomaly_report' | 'complaint'
    pub topic: Option<String>,
    pub context: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AbandonmentPayload {
    pub anomaly_report_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct RepatriationPayload {
    pub settler_id: Uuid,
    pub complaint_id: Uuid,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct HouseArrestPayload {
    pub settler_id: Uuid,
    pub arrest: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardData {
    pub settlement_name: String,
    pub planet: String,
    pub status: String,
    pub settlers: Vec<DashboardSettler>,
    pub open_anomalies: i64,
    pub pending_requests: i64,
    pub house_arrests: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DashboardSettler {
    pub user_id: Uuid,
    pub full_name: String,
    pub role_name: String,
    #[sqlx(rename = "type")]
    pub assignment_type: String,
    pub residence_unit: Option<String>,
    pub house_arrest: bool,
    pub assigned_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ManageInventoryPayload {
    pub action: String, // 'add' | 'update' | 'delete'
    pub id: Option<Uuid>,
    pub item_name: Option<String>,
    pub category: Option<String>,
    pub quantity: Option<i32>,
    pub unit: Option<String>,
    pub min_threshold: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CommanderSupplyPayload {
    pub items: serde_json::Value,
    pub justification: Option<String>,
}

// ── Civil Engineer structs ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ConstructionReportPayload {
    pub task_id: Uuid,
    pub week: Option<String>,
    pub rag_status: Option<String>,
    pub materials_used: Option<serde_json::Value>,
    pub construction_progress: String,
    pub issues: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BuildingHealthPayload {
    pub building_name: String,
    pub check_date: String, // YYYY-MM-DD
    pub findings: Option<String>,
    pub status: String, // 'pass' | 'fail' | 'needs_repair'
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ResidenceInfo {
    pub settlement_name: String,
    pub planet: String,
    pub residence_unit: Option<String>,
    pub arrived_at: Option<DateTime<Utc>>,
}

// ── Farmer structs ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SettlementMember {
    pub user_id: Uuid,
    pub full_name: String,
    pub role_name: String,
}

#[derive(Debug, Deserialize)]
pub struct FarmHealthPayload {
    pub log_date: String, // YYYY-MM-DD
    pub subject_type: String, // 'plant' | 'livestock'
    pub subject_name: String,
    pub condition: String,
    pub treatment: Option<String>,
    pub notes: Option<String>,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  SHARED SETTLER COMMANDS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// ═══════════════════════════════════════════════════════════════════════════════
// UC-PS-01: Get My Tasks (settler task inbox)
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_get_my_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<SettlerTaskSummary>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::SettlerCommander, Role::CivilEngineer, Role::Farmer, Role::TemporarySetter
    ]);

    let tasks = sqlx::query_as::<_, SettlerTaskSummary>(
        r#"
        SELECT st.id, st.title, st.scope, st.urgency, st.deadline, st.status,
               u.full_name AS assigned_by_name, st.created_at
        FROM settler_tasks st
        JOIN users u ON u.id = st.assigned_by
        WHERE st.assigned_to = $1 AND st.deleted_at IS NULL
        ORDER BY st.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(tasks)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-PS-01 (detail): Get Task Detail
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_get_task_detail(
    state: State<'_, AppState>,
    task_id: Uuid,
) -> Result<SettlerTaskDetail, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::SettlerCommander, Role::CivilEngineer, Role::Farmer, Role::TemporarySetter
    ]);

    let task = sqlx::query_as::<_, SettlerTaskDetail>(
        r#"
        SELECT st.id, st.settlement_id, st.assigned_by,
               ub.full_name AS assigned_by_name,
               st.assigned_to,
               ua.full_name AS assigned_to_name,
               st.title, st.description, st.scope, st.urgency, st.deadline,
               st.status, st.created_at
        FROM settler_tasks st
        JOIN users ub ON ub.id = st.assigned_by
        JOIN users ua ON ua.id = st.assigned_to
        WHERE st.id = $1 AND st.deleted_at IS NULL
        "#,
    )
    .bind(task_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Task not found.".into()))?;

    // Settlers can only see their own tasks; commander can see all in their settlement
    if user.role != Role::SettlerCommander && task.assigned_to != user.id {
        return Err(AppError::Forbidden);
    }

    Ok(task)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-PS-02: Submit Progress Report
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_submit_progress_report(
    state: State<'_, AppState>,
    payload: ProgressReportPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::SettlerCommander, Role::CivilEngineer, Role::Farmer, Role::TemporarySetter
    ]);

    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    // Verify task belongs to this user
    let task_check: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM settler_tasks WHERE id = $1 AND assigned_to = $2 AND deleted_at IS NULL",
    )
    .bind(payload.task_id)
    .bind(user.id)
    .fetch_optional(&state.db_pool)
    .await?;

    if task_check.is_none() {
        return Err(AppError::Internal("Task not found or not assigned to you.".into()));
    }

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO settler_progress_reports
            (task_id, settlement_id, submitted_by, week, rag_status, progress_made, materials_equipment)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(payload.task_id)
    .bind(settlement_id)
    .bind(user.id)
    .bind(&payload.week)
    .bind(&payload.rag_status)
    .bind(&payload.progress_made)
    .bind(&payload.materials_equipment)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "settler_progress_reports",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "task_id": payload.task_id })),
    )
    .await?;

    // Notify commander
    let commander_id: Option<(Uuid,)> = sqlx::query_as(
        "SELECT commander_id FROM settlements WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(settlement_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some((cid,)) = commander_id {
        let _ = sqlx::query(
            r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'report:received', $2::jsonb)"#,
        )
        .bind(cid)
        .bind(serde_json::json!({
            "report_type": "progress_report",
            "task_id": payload.task_id,
            "submitted_by": user.id,
        }))
        .execute(&state.db_pool)
        .await;
    }

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-PS-03 / UC-TS-01: Submit Anomaly Report
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_submit_anomaly_report(
    state: State<'_, AppState>,
    payload: AnomalyReportPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::SettlerCommander, Role::CivilEngineer, Role::Farmer, Role::TemporarySetter
    ]);

    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO anomaly_reports
            (settlement_id, reported_by, description, location_in_settlement,
             danger_level, severity, category, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'submitted')
        RETURNING id
        "#,
    )
    .bind(settlement_id)
    .bind(user.id)
    .bind(&payload.description)
    .bind(&payload.location_in_settlement)
    .bind(&payload.danger_level)
    .bind(&payload.severity)
    .bind(&payload.category)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "anomaly_reports",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "severity": payload.severity })),
    )
    .await?;

    // Notify commander
    let commander_id: Option<(Uuid,)> = sqlx::query_as(
        "SELECT commander_id FROM settlements WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(settlement_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some((cid,)) = commander_id {
        let _ = sqlx::query(
            r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'report:received', $2::jsonb)"#,
        )
        .bind(cid)
        .bind(serde_json::json!({
            "report_type": "anomaly_report",
            "report_id": row.0,
            "severity": payload.severity,
        }))
        .execute(&state.db_pool)
        .await;
    }

    // Invalidate anomaly cache
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(format!("anomaly_reports:open:{}", settlement_id)).await;
    }

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-PS-04 / UC-TS-02: Submit Complaint (Troublesome Settler Report)
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_submit_complaint(
    state: State<'_, AppState>,
    payload: ComplaintPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::SettlerCommander, Role::CivilEngineer, Role::Farmer, Role::TemporarySetter
    ]);

    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO settler_complaints
            (settlement_id, reported_by, subject_user_id, incident_description, status)
        VALUES ($1, $2, $3, $4, 'submitted')
        RETURNING id
        "#,
    )
    .bind(settlement_id)
    .bind(user.id)
    .bind(payload.subject_user_id)
    .bind(&payload.incident_description)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "settler_complaints",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "subject_user_id": payload.subject_user_id })),
    )
    .await?;

    // Notify commander
    let commander_id: Option<(Uuid,)> = sqlx::query_as(
        "SELECT commander_id FROM settlements WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(settlement_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some((cid,)) = commander_id {
        let _ = sqlx::query(
            r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'report:received', $2::jsonb)"#,
        )
        .bind(cid)
        .bind(serde_json::json!({
            "report_type": "complaint",
            "complaint_id": row.0,
        }))
        .execute(&state.db_pool)
        .await;
    }

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-PS-05 / UC-TS-03: Submit Supply Request
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_submit_supply_request(
    state: State<'_, AppState>,
    payload: SupplyRequestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::SettlerCommander, Role::CivilEngineer, Role::Farmer, Role::TemporarySetter
    ]);

    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    // Determine source_type from role
    let source_type = match user.role {
        Role::CivilEngineer => "civil_engineer",
        Role::Farmer => "farmer",
        Role::SettlerCommander => "commander",
        _ => "settler",
    };

    // All supply requests from the shared endpoint go to commander review.
    // Commanders should use stl_submit_commander_supply for direct→Directors routing.
    let status = "pending_commander";

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO supply_requests
            (settlement_id, submitted_by, source_type, items, justification, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(settlement_id)
    .bind(user.id)
    .bind(source_type)
    .bind(&payload.items)
    .bind(&payload.justification)
    .bind(status)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "supply_requests",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "source_type": source_type, "status": status })),
    )
    .await?;

    // Notify commander (unless submitter IS the commander)
    if user.role != Role::SettlerCommander {
        let commander_id: Option<(Uuid,)> = sqlx::query_as(
            "SELECT commander_id FROM settlements WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(settlement_id)
        .fetch_optional(&state.db_pool)
        .await?;

        if let Some((cid,)) = commander_id {
            let _ = sqlx::query(
                r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'report:received', $2::jsonb)"#,
            )
            .bind(cid)
            .bind(serde_json::json!({
                "report_type": "supply_request",
                "request_id": row.0,
            }))
            .execute(&state.db_pool)
            .await;
        }
    }

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-PS-06: View Settlement Inventory (read-only)
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_get_settlement_inventory(
    state: State<'_, AppState>,
) -> Result<Vec<InventoryItem>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::SettlerCommander, Role::CivilEngineer, Role::Farmer, Role::TemporarySetter
    ]);

    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    // Try Redis cache
    let cache_key = format!("settlement_inventory:{}", settlement_id);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(parsed) = serde_json::from_str::<Vec<InventoryItem>>(&cached) {
                return Ok(parsed);
            }
        }
    }

    let items = sqlx::query_as::<_, InventoryItem>(
        r#"
        SELECT id, item_name, category, quantity, unit, min_threshold, updated_at
        FROM settlement_inventory
        WHERE settlement_id = $1 AND deleted_at IS NULL
        ORDER BY category, item_name
        "#,
    )
    .bind(settlement_id)
    .fetch_all(&state.db_pool)
    .await?;

    // Cache 15 min
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(json) = serde_json::to_string(&items) {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json, 900).await;
        }
    }

    Ok(items)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Shared: Get Settlement Members (any settler can list fellow settlers)
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_get_settlement_members(
    state: State<'_, AppState>,
) -> Result<Vec<SettlementMember>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::SettlerCommander, Role::CivilEngineer, Role::Farmer, Role::TemporarySetter
    ]);

    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    let members = sqlx::query_as::<_, SettlementMember>(
        r#"
        SELECT sa.user_id, u.full_name, r.name AS role_name
        FROM settler_assignments sa
        JOIN users u ON u.id = sa.user_id
        JOIN roles r ON r.id = u.role_id
        WHERE sa.settlement_id = $1 AND sa.departed_at IS NULL
        ORDER BY u.full_name
        "#,
    )
    .bind(settlement_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(members)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  SETTLER COMMANDER COMMANDS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SC-01: Assign Task to Settler
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_assign_task(
    state: State<'_, AppState>,
    payload: AssignTaskPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::SettlerCommander]);

    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    // Verify target settler is assigned to this settlement
    let target_check: Option<(Uuid,)> = sqlx::query_as(
        "SELECT user_id FROM settler_assignments WHERE user_id = $1 AND settlement_id = $2 AND departed_at IS NULL",
    )
    .bind(payload.assigned_to)
    .bind(settlement_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if target_check.is_none() {
        return Err(AppError::Internal("Target settler is not assigned to your settlement.".into()));
    }

    let deadline = payload
        .deadline
        .as_deref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO settler_tasks
            (settlement_id, assigned_by, assigned_to, title, description, scope, urgency, deadline, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'assigned')
        RETURNING id
        "#,
    )
    .bind(settlement_id)
    .bind(user.id)
    .bind(payload.assigned_to)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.scope)
    .bind(&payload.urgency)
    .bind(deadline)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "settler_tasks",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "assigned_to": payload.assigned_to })),
    )
    .await?;

    // Notify the assigned settler
    let _ = sqlx::query(
        r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'task:assigned', $2::jsonb)"#,
    )
    .bind(payload.assigned_to)
    .bind(serde_json::json!({
        "task_id": row.0,
        "title": payload.title,
    }))
    .execute(&state.db_pool)
    .await;

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SC-02: Get Incoming Queue
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_get_incoming_queue(
    state: State<'_, AppState>,
) -> Result<Vec<IncomingQueueItem>, AppError> {
    let user = crate::require_auth_any!(state, [Role::SettlerCommander]);

    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    // UNION ALL across the three queue sources
    let rows = sqlx::query_as::<_, IncomingQueueItemRow>(
        r#"
        SELECT id, 'supply_request' AS item_type, submitted_by, summary, status, created_at
        FROM (
            SELECT sr.id, sr.submitted_by,
                   LEFT(sr.justification, 100) AS summary,
                   sr.status, sr.created_at
            FROM supply_requests sr
            WHERE sr.settlement_id = $1 AND sr.status = 'pending_commander'
                  AND sr.deleted_at IS NULL
        ) sr_sub

        UNION ALL

        SELECT id, 'anomaly_report' AS item_type, reported_by AS submitted_by,
               LEFT(description, 100) AS summary, status, created_at
        FROM anomaly_reports
        WHERE settlement_id = $1 AND status = 'submitted' AND deleted_at IS NULL

        UNION ALL

        SELECT id, 'complaint' AS item_type, reported_by AS submitted_by,
               LEFT(incident_description, 100) AS summary, status, created_at
        FROM settler_complaints
        WHERE settlement_id = $1 AND status = 'submitted' AND deleted_at IS NULL

        ORDER BY created_at DESC
        "#,
    )
    .bind(settlement_id)
    .fetch_all(&state.db_pool)
    .await?;

    // Resolve submitter names
    let mut result = Vec::with_capacity(rows.len());
    for r in rows {
        let name: Option<(String,)> = sqlx::query_as(
            "SELECT full_name FROM users WHERE id = $1",
        )
        .bind(r.submitted_by)
        .fetch_optional(&state.db_pool)
        .await?;

        result.push(IncomingQueueItem {
            id: r.id,
            item_type: r.item_type,
            submitted_by: r.submitted_by,
            submitted_by_name: name.map(|n| n.0).unwrap_or_else(|| "Unknown".to_string()),
            summary: r.summary.unwrap_or_default(),
            status: r.status,
            created_at: r.created_at,
        });
    }

    Ok(result)
}

/// Internal row type for the UNION query.
#[derive(Debug, FromRow)]
struct IncomingQueueItemRow {
    id: Uuid,
    item_type: String,
    submitted_by: Uuid,
    summary: Option<String>,
    status: String,
    created_at: DateTime<Utc>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SC-02 (action): Reject Incoming Item
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_reject_incoming(
    state: State<'_, AppState>,
    payload: RejectPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::SettlerCommander]);
    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    match payload.item_type.as_str() {
        "supply_request" => {
            sqlx::query(
                r#"UPDATE supply_requests SET status = 'rejected_by_commander', commander_note = $1
                   WHERE id = $2 AND settlement_id = $3 AND status = 'pending_commander'"#,
            )
            .bind(&payload.reason)
            .bind(payload.item_id)
            .bind(settlement_id)
            .execute(&state.db_pool)
            .await?;
        }
        "anomaly_report" => {
            sqlx::query(
                r#"UPDATE anomaly_reports SET status = 'resolved', commander_note = $1
                   WHERE id = $2 AND settlement_id = $3 AND status = 'submitted'"#,
            )
            .bind(&payload.reason)
            .bind(payload.item_id)
            .bind(settlement_id)
            .execute(&state.db_pool)
            .await?;
        }
        "complaint" => {
            sqlx::query(
                r#"UPDATE settler_complaints SET status = 'rejected', commander_note = $1
                   WHERE id = $2 AND settlement_id = $3 AND status = 'submitted'"#,
            )
            .bind(&payload.reason)
            .bind(payload.item_id)
            .bind(settlement_id)
            .execute(&state.db_pool)
            .await?;
        }
        _ => return Err(AppError::Internal("Invalid item type.".into())),
    }

    write_audit_log(
        &state.db_pool,
        &payload.item_type,
        payload.item_id,
        AuditOperation::Update,
        user.id,
        None,
        Some(serde_json::json!({ "action": "reject", "reason": payload.reason })),
    )
    .await?;

    // Notify the original submitter
    let submitter_id: Option<(Uuid,)> = match payload.item_type.as_str() {
        "supply_request" => {
            sqlx::query_as("SELECT submitted_by FROM supply_requests WHERE id = $1")
                .bind(payload.item_id)
                .fetch_optional(&state.db_pool)
                .await?
        }
        "anomaly_report" => {
            sqlx::query_as("SELECT reported_by FROM anomaly_reports WHERE id = $1")
                .bind(payload.item_id)
                .fetch_optional(&state.db_pool)
                .await?
        }
        "complaint" => {
            sqlx::query_as("SELECT reported_by FROM settler_complaints WHERE id = $1")
                .bind(payload.item_id)
                .fetch_optional(&state.db_pool)
                .await?
        }
        _ => None,
    };

    if let Some((sid,)) = submitter_id {
        let _ = sqlx::query(
            r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'request:rejected', $2::jsonb)"#,
        )
        .bind(sid)
        .bind(serde_json::json!({
            "item_type": payload.item_type,
            "item_id": payload.item_id,
            "reason": payload.reason,
        }))
        .execute(&state.db_pool)
        .await;
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SC-02 (action): Forward to Directors
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_forward_to_directors(
    state: State<'_, AppState>,
    payload: ForwardPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::SettlerCommander]);
    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    let topic = payload.topic.clone().unwrap_or_else(|| {
        format!("Settler {} forwarded by Commander", payload.item_type.replace('_', " "))
    });
    let context = payload.context.clone().unwrap_or_else(|| {
        format!("Item ID: {}", payload.item_id)
    });

    // 1. Create a requests entry
    let req_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO requests (type, status, requester_id, title, description, payload)
        VALUES ($1, 'in_vote', $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(format!("settler_{}", payload.item_type))
    .bind(user.id)
    .bind(&topic)
    .bind(&context)
    .bind(serde_json::json!({
        "source_item_type": payload.item_type,
        "source_item_id": payload.item_id,
        "settlement_id": settlement_id,
    }))
    .fetch_one(&state.db_pool)
    .await?;

    // 2. Create a vote session (opens immediately)
    let session_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO vote_sessions (request_id, topic, context, status, opens_at, created_by)
        VALUES ($1, $2, $3, 'open', NOW(), $4)
        RETURNING id
        "#,
    )
    .bind(req_row.0)
    .bind(&topic)
    .bind(&context)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // 3. Update the source item's status
    match payload.item_type.as_str() {
        "supply_request" => {
            sqlx::query(
                r#"UPDATE supply_requests SET status = 'pending_vote', vote_session_id = $1
                   WHERE id = $2 AND settlement_id = $3"#,
            )
            .bind(session_row.0)
            .bind(payload.item_id)
            .bind(settlement_id)
            .execute(&state.db_pool)
            .await?;
        }
        "anomaly_report" => {
            sqlx::query(
                r#"UPDATE anomaly_reports SET status = 'forwarded_to_directors', vote_session_id = $1
                   WHERE id = $2 AND settlement_id = $3"#,
            )
            .bind(session_row.0)
            .bind(payload.item_id)
            .bind(settlement_id)
            .execute(&state.db_pool)
            .await?;
        }
        "complaint" => {
            sqlx::query(
                r#"UPDATE settler_complaints SET status = 'forwarded_to_directors', vote_session_id = $1
                   WHERE id = $2 AND settlement_id = $3"#,
            )
            .bind(session_row.0)
            .bind(payload.item_id)
            .bind(settlement_id)
            .execute(&state.db_pool)
            .await?;
        }
        _ => return Err(AppError::Internal("Invalid item type.".into())),
    }

    write_audit_log(
        &state.db_pool,
        "vote_sessions",
        session_row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "source": payload.item_type,
            "source_id": payload.item_id,
            "type": "settler_forward",
        })),
    )
    .await?;

    // 4. Cache vote state in Redis
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let cache_key = format!("vote_state:{}", session_row.0);
        let _: Result<(), _> = conn
            .set::<_, _, ()>(
                &cache_key,
                serde_json::json!({
                    "id": session_row.0,
                    "status": "open",
                    "total_yay": 0,
                    "total_nay": 0,
                    "total_abstain": 0,
                })
                .to_string(),
            )
            .await;
    }

    // 5. Notify all directors
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        SELECT u.id, 'vote:new', $2::jsonb
        FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE r.name IN (
            'GeneralDirector','TheDirector','TheAccountant','TheLibrarian',
            'TheNomad','TheArtificer','TheObserver','TheWanderer',
            'TheTaskmaster','TheGuardian','TheStatistician','TheCoordinator',
            'TheOverseer','TheAnchorman','Administrator'
        )
        AND u.deleted_at IS NULL AND u.is_active = TRUE
        "#,
    )
    .bind(session_row.0)
    .bind(serde_json::json!({
        "session_id": session_row.0,
        "topic": topic,
    }))
    .execute(&state.db_pool)
    .await?;

    // 6. Notify original submitter
    let submitter_id: Option<(Uuid,)> = match payload.item_type.as_str() {
        "supply_request" => {
            sqlx::query_as("SELECT submitted_by FROM supply_requests WHERE id = $1")
                .bind(payload.item_id)
                .fetch_optional(&state.db_pool)
                .await?
        }
        "anomaly_report" => {
            sqlx::query_as("SELECT reported_by FROM anomaly_reports WHERE id = $1")
                .bind(payload.item_id)
                .fetch_optional(&state.db_pool)
                .await?
        }
        "complaint" => {
            sqlx::query_as("SELECT reported_by FROM settler_complaints WHERE id = $1")
                .bind(payload.item_id)
                .fetch_optional(&state.db_pool)
                .await?
        }
        _ => None,
    };

    if let Some((sid,)) = submitter_id {
        let _ = sqlx::query(
            r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'request:forwarded', $2::jsonb)"#,
        )
        .bind(sid)
        .bind(serde_json::json!({
            "item_type": payload.item_type,
            "item_id": payload.item_id,
        }))
        .execute(&state.db_pool)
        .await;
    }

    Ok(session_row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SC-02A: Submit Commander Anomaly Report (directly to Directors)
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_submit_commander_anomaly(
    state: State<'_, AppState>,
    payload: AnomalyReportPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::SettlerCommander]);

    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    // Insert anomaly report
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO anomaly_reports
            (settlement_id, reported_by, description, location_in_settlement,
             danger_level, severity, category, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'forwarded_to_directors')
        RETURNING id
        "#,
    )
    .bind(settlement_id)
    .bind(user.id)
    .bind(&payload.description)
    .bind(&payload.location_in_settlement)
    .bind(&payload.danger_level)
    .bind(&payload.severity)
    .bind(&payload.category)
    .fetch_one(&state.db_pool)
    .await?;

    // Create vote session for directors
    let topic = format!("Commander Anomaly: {} — severity {}", 
        payload.description.chars().take(60).collect::<String>(), payload.severity);

    let req_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO requests (type, status, requester_id, title, description, payload)
        VALUES ('settler_anomaly_report', 'in_vote', $1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&topic)
    .bind(&payload.description)
    .bind(serde_json::json!({
        "anomaly_report_id": row.0,
        "settlement_id": settlement_id,
        "severity": payload.severity,
    }))
    .fetch_one(&state.db_pool)
    .await?;

    let session_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO vote_sessions (request_id, topic, context, status, opens_at, created_by)
        VALUES ($1, $2, $3, 'open', NOW(), $4)
        RETURNING id
        "#,
    )
    .bind(req_row.0)
    .bind(&topic)
    .bind(&payload.description)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Link vote session to anomaly
    sqlx::query("UPDATE anomaly_reports SET vote_session_id = $1 WHERE id = $2")
        .bind(session_row.0)
        .bind(row.0)
        .execute(&state.db_pool)
        .await?;

    write_audit_log(
        &state.db_pool,
        "anomaly_reports",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "severity": payload.severity, "direct_to_directors": true })),
    )
    .await?;

    // Cache vote state
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let cache_key = format!("vote_state:{}", session_row.0);
        let _: Result<(), _> = conn.set::<_, _, ()>(
            &cache_key,
            serde_json::json!({ "id": session_row.0, "status": "open", "total_yay": 0, "total_nay": 0, "total_abstain": 0 }).to_string(),
        ).await;
        let _: Result<(), _> = conn.del(format!("anomaly_reports:open:{}", settlement_id)).await;
    }

    // Notify all directors
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        SELECT u.id, 'vote:new', $2::jsonb
        FROM users u JOIN roles r ON r.id = u.role_id
        WHERE r.name IN ('GeneralDirector','TheDirector','TheAccountant','TheLibrarian',
            'TheNomad','TheArtificer','TheObserver','TheWanderer','TheTaskmaster',
            'TheGuardian','TheStatistician','TheCoordinator','TheOverseer','TheAnchorman','Administrator')
        AND u.deleted_at IS NULL AND u.is_active = TRUE
        "#,
    )
    .bind(session_row.0)
    .bind(serde_json::json!({ "session_id": session_row.0, "topic": topic }))
    .execute(&state.db_pool)
    .await?;

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SC-03: Request Settlement Abandonment
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_request_abandonment(
    state: State<'_, AppState>,
    payload: AbandonmentPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::SettlerCommander]);
    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    // Verify the anomaly report exists and belongs to this settlement
    let ar_check: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM anomaly_reports WHERE id = $1 AND settlement_id = $2 AND deleted_at IS NULL",
    )
    .bind(payload.anomaly_report_id)
    .bind(settlement_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if ar_check.is_none() {
        return Err(AppError::Internal("Anomaly report not found in your settlement.".into()));
    }

    // Create vote session
    let topic = format!("Settlement Abandonment Request — {}", payload.reason.chars().take(80).collect::<String>());

    let req_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO requests (type, status, requester_id, title, description, payload)
        VALUES ('settlement_abandonment', 'in_vote', $1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&topic)
    .bind(&payload.reason)
    .bind(serde_json::json!({
        "settlement_id": settlement_id,
        "anomaly_report_id": payload.anomaly_report_id,
    }))
    .fetch_one(&state.db_pool)
    .await?;

    let session_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO vote_sessions (request_id, topic, context, status, opens_at, created_by)
        VALUES ($1, $2, $3, 'open', NOW(), $4)
        RETURNING id
        "#,
    )
    .bind(req_row.0)
    .bind(&topic)
    .bind(&payload.reason)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Insert into abandonment_requests
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO abandonment_requests
            (settlement_id, commander_id, anomaly_report_id, reason, status, vote_session_id)
        VALUES ($1, $2, $3, $4, 'pending_vote', $5)
        RETURNING id
        "#,
    )
    .bind(settlement_id)
    .bind(user.id)
    .bind(payload.anomaly_report_id)
    .bind(&payload.reason)
    .bind(session_row.0)
    .fetch_one(&state.db_pool)
    .await?;

    // Mark anomaly as triggered abandonment
    sqlx::query("UPDATE anomaly_reports SET status = 'triggered_abandonment' WHERE id = $1")
        .bind(payload.anomaly_report_id)
        .execute(&state.db_pool)
        .await?;

    write_audit_log(
        &state.db_pool,
        "abandonment_requests",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "anomaly_report_id": payload.anomaly_report_id })),
    )
    .await?;

    // Cache + notify directors
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let cache_key = format!("vote_state:{}", session_row.0);
        let _: Result<(), _> = conn.set::<_, _, ()>(
            &cache_key,
            serde_json::json!({ "id": session_row.0, "status": "open", "total_yay": 0, "total_nay": 0, "total_abstain": 0 }).to_string(),
        ).await;
    }

    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        SELECT u.id, 'vote:new', $2::jsonb
        FROM users u JOIN roles r ON r.id = u.role_id
        WHERE r.name IN ('GeneralDirector','TheDirector','TheAccountant','TheLibrarian',
            'TheNomad','TheArtificer','TheObserver','TheWanderer','TheTaskmaster',
            'TheGuardian','TheStatistician','TheCoordinator','TheOverseer','TheAnchorman','Administrator')
        AND u.deleted_at IS NULL AND u.is_active = TRUE
        "#,
    )
    .bind(session_row.0)
    .bind(serde_json::json!({ "session_id": session_row.0, "topic": topic }))
    .execute(&state.db_pool)
    .await?;

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SC-04: Request Settler Repatriation
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_request_repatriation(
    state: State<'_, AppState>,
    payload: RepatriationPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::SettlerCommander]);
    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    // Verify complaint exists
    let comp_check: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM settler_complaints WHERE id = $1 AND settlement_id = $2 AND deleted_at IS NULL",
    )
    .bind(payload.complaint_id)
    .bind(settlement_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if comp_check.is_none() {
        return Err(AppError::Internal("Complaint not found in your settlement.".into()));
    }

    // Create vote session
    let topic = format!("Settler Repatriation Request — {}", payload.reason.chars().take(80).collect::<String>());

    let req_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO requests (type, status, requester_id, title, description, payload)
        VALUES ('settler_repatriation', 'in_vote', $1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&topic)
    .bind(&payload.reason)
    .bind(serde_json::json!({
        "settlement_id": settlement_id,
        "settler_id": payload.settler_id,
        "complaint_id": payload.complaint_id,
    }))
    .fetch_one(&state.db_pool)
    .await?;

    let session_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO vote_sessions (request_id, topic, context, status, opens_at, created_by)
        VALUES ($1, $2, $3, 'open', NOW(), $4)
        RETURNING id
        "#,
    )
    .bind(req_row.0)
    .bind(&topic)
    .bind(&payload.reason)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO repatriation_requests
            (settlement_id, commander_id, settler_id, complaint_id, reason, status, vote_session_id)
        VALUES ($1, $2, $3, $4, $5, 'pending_vote', $6)
        RETURNING id
        "#,
    )
    .bind(settlement_id)
    .bind(user.id)
    .bind(payload.settler_id)
    .bind(payload.complaint_id)
    .bind(&payload.reason)
    .bind(session_row.0)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "repatriation_requests",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "settler_id": payload.settler_id, "complaint_id": payload.complaint_id })),
    )
    .await?;

    // Cache + notify directors
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let cache_key = format!("vote_state:{}", session_row.0);
        let _: Result<(), _> = conn.set::<_, _, ()>(
            &cache_key,
            serde_json::json!({ "id": session_row.0, "status": "open", "total_yay": 0, "total_nay": 0, "total_abstain": 0 }).to_string(),
        ).await;
    }

    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        SELECT u.id, 'vote:new', $2::jsonb
        FROM users u JOIN roles r ON r.id = u.role_id
        WHERE r.name IN ('GeneralDirector','TheDirector','TheAccountant','TheLibrarian',
            'TheNomad','TheArtificer','TheObserver','TheWanderer','TheTaskmaster',
            'TheGuardian','TheStatistician','TheCoordinator','TheOverseer','TheAnchorman','Administrator')
        AND u.deleted_at IS NULL AND u.is_active = TRUE
        "#,
    )
    .bind(session_row.0)
    .bind(serde_json::json!({ "session_id": session_row.0, "topic": topic }))
    .execute(&state.db_pool)
    .await?;

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SC-05: Set House Arrest
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_set_house_arrest(
    state: State<'_, AppState>,
    payload: HouseArrestPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::SettlerCommander]);
    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    let result = sqlx::query(
        r#"
        UPDATE settler_assignments
        SET house_arrest = $1
        WHERE user_id = $2 AND settlement_id = $3 AND departed_at IS NULL
        "#,
    )
    .bind(payload.arrest)
    .bind(payload.settler_id)
    .bind(settlement_id)
    .execute(&state.db_pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Internal("Settler not found in your settlement.".into()));
    }

    write_audit_log(
        &state.db_pool,
        "settler_assignments",
        payload.settler_id,
        AuditOperation::Update,
        user.id,
        None,
        Some(serde_json::json!({ "house_arrest": payload.arrest })),
    )
    .await?;

    // Invalidate roster cache
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(format!("settler_roster:{}", settlement_id)).await;
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SC-06: Settlement Dashboard
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_get_dashboard(
    state: State<'_, AppState>,
) -> Result<DashboardData, AppError> {
    let user = crate::require_auth_any!(state, [Role::SettlerCommander]);
    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    // Settlement info
    let settlement: (String, String, String) = sqlx::query_as(
        "SELECT name, planet, status FROM settlements WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(settlement_id)
    .fetch_one(&state.db_pool)
    .await?;

    // Settlers — try Redis cache first, fall back to DB
    let cache_key = format!("settler_roster:{}", settlement_id);
    let settlers: Vec<DashboardSettler> = {
        let mut cached_result: Option<Vec<DashboardSettler>> = None;
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
                if let Ok(parsed) = serde_json::from_str::<Vec<DashboardSettler>>(&cached) {
                    cached_result = Some(parsed);
                }
            }
        }

        if let Some(settlers) = cached_result {
            settlers
        } else {
            let rows = sqlx::query_as::<_, DashboardSettler>(
                r#"
                SELECT sa.user_id, u.full_name, r.name AS role_name,
                       sa.type, sa.residence_unit, sa.house_arrest,
                       sa.arrived_at::text AS assigned_at
                FROM settler_assignments sa
                JOIN users u ON u.id = sa.user_id
                JOIN roles r ON r.id = u.role_id
                WHERE sa.settlement_id = $1 AND sa.departed_at IS NULL
                ORDER BY r.name, u.full_name
                "#,
            )
            .bind(settlement_id)
            .fetch_all(&state.db_pool)
            .await?;

            if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
                if let Ok(json) = serde_json::to_string(&rows) {
                    let _: Result<(), _> = conn.set_ex(&cache_key, &json, 1800).await;
                }
            }

            rows
        }
    };

    // Counts
    let open_anomalies: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM anomaly_reports WHERE settlement_id = $1 AND status IN ('submitted','under_review') AND deleted_at IS NULL",
    )
    .bind(settlement_id)
    .fetch_one(&state.db_pool)
    .await?;

    let pending_requests: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM supply_requests WHERE settlement_id = $1 AND status = 'pending_commander' AND deleted_at IS NULL",
    )
    .bind(settlement_id)
    .fetch_one(&state.db_pool)
    .await?;

    let house_arrests: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM settler_assignments WHERE settlement_id = $1 AND house_arrest = TRUE AND departed_at IS NULL",
    )
    .bind(settlement_id)
    .fetch_one(&state.db_pool)
    .await?;

    Ok(DashboardData {
        settlement_name: settlement.0,
        planet: settlement.1,
        status: settlement.2,
        settlers,
        open_anomalies: open_anomalies.0,
        pending_requests: pending_requests.0,
        house_arrests: house_arrests.0,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SC-07: Submit Commander Supply Request (direct to Directors)
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_submit_commander_supply(
    state: State<'_, AppState>,
    payload: CommanderSupplyPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::SettlerCommander]);
    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    // Commander supply requests go directly to directors (status: in_vote)
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO supply_requests
            (settlement_id, submitted_by, source_type, items, justification, status)
        VALUES ($1, $2, 'commander', $3, $4, 'pending_vote')
        RETURNING id
        "#,
    )
    .bind(settlement_id)
    .bind(user.id)
    .bind(&payload.items)
    .bind(&payload.justification)
    .fetch_one(&state.db_pool)
    .await?;

    // Create vote session
    let topic = format!("Commander Supply Request — {}", payload.justification.as_deref().unwrap_or("").chars().take(80).collect::<String>());

    let req_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO requests (type, status, requester_id, title, description, payload)
        VALUES ('settler_supply_request', 'in_vote', $1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&topic)
    .bind(payload.justification.as_deref().unwrap_or(""))
    .bind(serde_json::json!({
        "supply_request_id": row.0,
        "settlement_id": settlement_id,
    }))
    .fetch_one(&state.db_pool)
    .await?;

    let session_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO vote_sessions (request_id, topic, context, status, opens_at, created_by)
        VALUES ($1, $2, $3, 'open', NOW(), $4)
        RETURNING id
        "#,
    )
    .bind(req_row.0)
    .bind(&topic)
    .bind(payload.justification.as_deref().unwrap_or(""))
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Link vote session
    sqlx::query("UPDATE supply_requests SET vote_session_id = $1 WHERE id = $2")
        .bind(session_row.0)
        .bind(row.0)
        .execute(&state.db_pool)
        .await?;

    write_audit_log(
        &state.db_pool,
        "supply_requests",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "source_type": "commander", "direct_to_directors": true })),
    )
    .await?;

    // Cache + notify directors
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let cache_key = format!("vote_state:{}", session_row.0);
        let _: Result<(), _> = conn.set::<_, _, ()>(
            &cache_key,
            serde_json::json!({ "id": session_row.0, "status": "open", "total_yay": 0, "total_nay": 0, "total_abstain": 0 }).to_string(),
        ).await;
    }

    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        SELECT u.id, 'vote:new', $2::jsonb
        FROM users u JOIN roles r ON r.id = u.role_id
        WHERE r.name IN ('GeneralDirector','TheDirector','TheAccountant','TheLibrarian',
            'TheNomad','TheArtificer','TheObserver','TheWanderer','TheTaskmaster',
            'TheGuardian','TheStatistician','TheCoordinator','TheOverseer','TheAnchorman','Administrator')
        AND u.deleted_at IS NULL AND u.is_active = TRUE
        "#,
    )
    .bind(session_row.0)
    .bind(serde_json::json!({ "session_id": session_row.0, "topic": topic }))
    .execute(&state.db_pool)
    .await?;

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SC-08: Manage Settlement Inventory
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_manage_inventory(
    state: State<'_, AppState>,
    payload: ManageInventoryPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::SettlerCommander]);
    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    match payload.action.as_str() {
        "add" => {
            let item_name = payload.item_name.as_deref()
                .ok_or_else(|| AppError::Internal("item_name is required for add.".into()))?;
            let quantity = payload.quantity.unwrap_or(0);

            let min_thresh = payload.min_threshold.unwrap_or(0);

            sqlx::query(
                r#"
                INSERT INTO settlement_inventory (settlement_id, item_name, category, quantity, unit, min_threshold, last_updated_by)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(settlement_id)
            .bind(item_name)
            .bind(&payload.category)
            .bind(quantity)
            .bind(&payload.unit)
            .bind(min_thresh)
            .bind(user.id)
            .execute(&state.db_pool)
            .await?;
        }
        "update" => {
            let id = payload.id
                .ok_or_else(|| AppError::Internal("id is required for update.".into()))?;

            sqlx::query(
                r#"
                UPDATE settlement_inventory
                SET item_name = COALESCE($1, item_name),
                    category = COALESCE($2, category),
                    quantity = COALESCE($3, quantity),
                    unit = COALESCE($4, unit),
                    min_threshold = COALESCE($5, min_threshold),
                    last_updated_by = $6,
                    updated_at = NOW()
                WHERE id = $7 AND settlement_id = $8 AND deleted_at IS NULL
                "#,
            )
            .bind(&payload.item_name)
            .bind(&payload.category)
            .bind(payload.quantity)
            .bind(&payload.unit)
            .bind(payload.min_threshold)
            .bind(user.id)
            .bind(id)
            .bind(settlement_id)
            .execute(&state.db_pool)
            .await?;
        }
        "delete" => {
            let id = payload.id
                .ok_or_else(|| AppError::Internal("id is required for delete.".into()))?;

            sqlx::query(
                "UPDATE settlement_inventory SET deleted_at = NOW(), deleted_by = $1 WHERE id = $2 AND settlement_id = $3",
            )
            .bind(user.id)
            .bind(id)
            .bind(settlement_id)
            .execute(&state.db_pool)
            .await?;
        }
        _ => return Err(AppError::Internal("Invalid action. Use 'add', 'update', or 'delete'.".into())),
    }

    // Invalidate inventory cache
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(format!("settlement_inventory:{}", settlement_id)).await;
    }

    Ok(())
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  CIVIL ENGINEER COMMANDS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// ═══════════════════════════════════════════════════════════════════════════════
// UC-CE-01: Submit Construction Progress Report
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_submit_construction_report(
    state: State<'_, AppState>,
    payload: ConstructionReportPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::CivilEngineer]);
    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    // Verify task belongs to this user
    let task_check: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM settler_tasks WHERE id = $1 AND assigned_to = $2 AND deleted_at IS NULL",
    )
    .bind(payload.task_id)
    .bind(user.id)
    .fetch_optional(&state.db_pool)
    .await?;

    if task_check.is_none() {
        return Err(AppError::Internal("Task not found or not assigned to you.".into()));
    }

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO construction_progress_reports
            (task_id, settlement_id, submitted_by, week, rag_status, materials_used, construction_progress, issues)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(payload.task_id)
    .bind(settlement_id)
    .bind(user.id)
    .bind(&payload.week)
    .bind(&payload.rag_status)
    .bind(&payload.materials_used)
    .bind(&payload.construction_progress)
    .bind(&payload.issues)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "construction_progress_reports",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "task_id": payload.task_id })),
    )
    .await?;

    // Notify commander
    let commander_id: Option<(Uuid,)> = sqlx::query_as(
        "SELECT commander_id FROM settlements WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(settlement_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some((cid,)) = commander_id {
        let _ = sqlx::query(
            r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'report:received', $2::jsonb)"#,
        )
        .bind(cid)
        .bind(serde_json::json!({
            "report_type": "construction_progress",
            "task_id": payload.task_id,
        }))
        .execute(&state.db_pool)
        .await;
    }

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-CE-02: Request Materials (routed as supply request)
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_request_materials(
    state: State<'_, AppState>,
    payload: SupplyRequestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::CivilEngineer]);
    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO supply_requests
            (settlement_id, submitted_by, source_type, items, justification, status)
        VALUES ($1, $2, 'civil_engineer', $3, $4, 'pending_commander')
        RETURNING id
        "#,
    )
    .bind(settlement_id)
    .bind(user.id)
    .bind(&payload.items)
    .bind(&payload.justification)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "supply_requests",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "source_type": "civil_engineer" })),
    )
    .await?;

    // Notify commander
    let commander_id: Option<(Uuid,)> = sqlx::query_as(
        "SELECT commander_id FROM settlements WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(settlement_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some((cid,)) = commander_id {
        let _ = sqlx::query(
            r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'report:received', $2::jsonb)"#,
        )
        .bind(cid)
        .bind(serde_json::json!({ "report_type": "material_request", "request_id": row.0 }))
        .execute(&state.db_pool)
        .await;
    }

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-CE-03: Log Building Health Check
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_log_building_health(
    state: State<'_, AppState>,
    payload: BuildingHealthPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::CivilEngineer]);
    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    let check_date = NaiveDate::parse_from_str(&payload.check_date, "%Y-%m-%d")
        .map_err(|_| AppError::Internal("Invalid check_date format. Use YYYY-MM-DD.".into()))?;

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO building_health_logs
            (settlement_id, building_name, checked_by, check_date, findings, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(settlement_id)
    .bind(&payload.building_name)
    .bind(user.id)
    .bind(check_date)
    .bind(&payload.findings)
    .bind(&payload.status)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "building_health_logs",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "building_name": payload.building_name, "status": payload.status })),
    )
    .await?;

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-CE-04: View Assigned Residence
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_get_residence(
    state: State<'_, AppState>,
) -> Result<ResidenceInfo, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::CivilEngineer, Role::Farmer, Role::SettlerCommander, Role::TemporarySetter
    ]);

    let info = sqlx::query_as::<_, ResidenceInfo>(
        r#"
        SELECT s.name AS settlement_name, s.planet, sa.residence_unit, sa.arrived_at
        FROM settler_assignments sa
        JOIN settlements s ON s.id = sa.settlement_id
        WHERE sa.user_id = $1 AND sa.departed_at IS NULL AND s.deleted_at IS NULL
        "#,
    )
    .bind(user.id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("No active settlement assignment found.".into()))?;

    Ok(info)
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  FARMER COMMANDS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// ═══════════════════════════════════════════════════════════════════════════════
// UC-FA-01: Request Farming Supplies
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_request_farming_supplies(
    state: State<'_, AppState>,
    payload: SupplyRequestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::Farmer]);
    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO supply_requests
            (settlement_id, submitted_by, source_type, items, justification, status)
        VALUES ($1, $2, 'farmer', $3, $4, 'pending_commander')
        RETURNING id
        "#,
    )
    .bind(settlement_id)
    .bind(user.id)
    .bind(&payload.items)
    .bind(&payload.justification)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "supply_requests",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "source_type": "farmer" })),
    )
    .await?;

    // Notify commander
    let commander_id: Option<(Uuid,)> = sqlx::query_as(
        "SELECT commander_id FROM settlements WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(settlement_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some((cid,)) = commander_id {
        let _ = sqlx::query(
            r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'report:received', $2::jsonb)"#,
        )
        .bind(cid)
        .bind(serde_json::json!({ "report_type": "farming_supply_request", "request_id": row.0 }))
        .execute(&state.db_pool)
        .await;
    }

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-FA-02: Log Farm Health Check
// ═══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn stl_log_farm_health(
    state: State<'_, AppState>,
    payload: FarmHealthPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::Farmer]);
    let settlement_id = get_user_settlement(&state.db_pool, user.id).await?;

    let log_date = NaiveDate::parse_from_str(&payload.log_date, "%Y-%m-%d")
        .map_err(|_| AppError::Internal("Invalid log_date format. Use YYYY-MM-DD.".into()))?;

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO farm_health_logs
            (settlement_id, logged_by, log_date, subject_type, subject_name, condition, treatment, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(settlement_id)
    .bind(user.id)
    .bind(log_date)
    .bind(&payload.subject_type)
    .bind(&payload.subject_name)
    .bind(&payload.condition)
    .bind(&payload.treatment)
    .bind(&payload.notes)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "farm_health_logs",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "subject_type": payload.subject_type, "subject_name": payload.subject_name })),
    )
    .await?;

    Ok(row.0)
}
