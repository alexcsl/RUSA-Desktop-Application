// commands/security.rs — Galactic Security Teams subsystem commands
// Source of truth: 03_SECURITY_TEAMS.md, 00_MASTER_GUIDE.md
//
// Use cases implemented:
//   General Security Head (Galactic):
//     UC-SH-01: Submit Broadcast Announcement Request  (sec_submit_broadcast_request)
//     UC-SH-02: Create Incident Report                  (sec_create_incident_report)
//     UC-SH-03: Assign Staff to Incident Handling       (sec_assign_staff_to_incident)
//     UC-SH-04: Security Inter-Team Communication       (sec_send_security_message, sec_get_security_messages)
//     UC-SH-05: Submit Daily Security Findings Report   (sec_submit_daily_report)
//
//   General Security Staff (Galactic):
//     UC-SS-01: Create Incident Report                  (sec_create_incident_report — same command)
//     UC-SS-02: Security Inter-Team Communication       (sec_send_security_message, sec_get_security_messages)

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

// ── Cache TTL constants ────────────────────────────────────────────────────────
#[allow(dead_code)]
const SECURITY_INCIDENTS_TTL: u64 = 600;  // 10 minutes
#[allow(dead_code)]
const SECURITY_PERSONNEL_TTL: u64 = 3600; // 1 hour
#[allow(dead_code)]
const SECURITY_DAILY_TTL: u64 = 300;      // 5 minutes
#[allow(dead_code)]
const SECURITY_BROADCASTS_TTL: u64 = 300; // 5 minutes
#[allow(dead_code)]
const SECURITY_MESSAGES_TTL: u64 = 120;   // 2 minutes

/// Invalidates a Redis cache key, ignoring errors (cache is best-effort).
async fn invalidate_cache(state: &State<'_, AppState>, key: &str) {
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(key).await;
    }
}

// ── Role sets ─────────────────────────────────────────────────────────────────

#[allow(dead_code)]
const ALL_SECURITY_ROLES: [Role; 2] = [
    Role::GalacticSecurityHead,
    Role::GalacticSecurityStaff,
];

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  STRUCTS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// ── Incident report ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateIncidentReportPayload {
    pub source: String,
    pub incident_type: String,
    pub location: String,
    pub occurred_at: Option<String>,
    pub description: String,
    pub severity: String,
    pub recommended_action: Option<String>,
    pub sector_or_base: Option<String>,
    pub related_incident_ids: Option<Vec<Uuid>>,
    pub incident_meta: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct IncidentReportSummary {
    pub id: Uuid,
    pub reported_by: Uuid,
    pub reporter_name: String,
    pub source: String,
    pub incident_type: String,
    pub location: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub description: String,
    pub severity: String,
    pub recommended_action: Option<String>,
    pub sector_or_base: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub related_incident_ids: Option<Vec<Uuid>>,
    pub incident_meta: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// ── Daily security report ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DailyReportPayload {
    pub report_date: String,
    pub findings_summary: String,
    pub risk_notes: Option<String>,
    pub recommended_actions: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DailySecurityReportSummary {
    pub id: Uuid,
    pub submitted_by: Uuid,
    pub submitter_name: String,
    pub report_date: NaiveDate,
    pub findings_summary: String,
    pub risk_notes: Option<String>,
    pub recommended_actions: Option<String>,
    pub delivered_to_guardian_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ── Broadcast request ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SecBroadcastRequestPayload {
    pub subject: String,
    pub content: String,
    pub urgency: Option<String>,
    pub rationale: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SecBroadcastRequestSummary {
    pub id: Uuid,
    pub requester_id: Uuid,
    pub requester_name: String,
    pub type_: String,
    pub subject: String,
    pub content: String,
    pub target_scope: String,
    pub urgency: Option<String>,
    pub rationale: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

// ── Messaging ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SecSendMessagePayload {
    pub subject: String,
    pub body: String,
    pub recipients_to: Vec<Uuid>,
    pub recipients_cc: Option<Vec<Uuid>>,
    pub recipients_bcc: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SecMessageSummary {
    pub id: Uuid,
    pub from_id: Uuid,
    pub from_name: String,
    pub subject: String,
    pub body: String,
    pub channel: String,
    pub recalled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ── Staff assignment ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AssignStaffPayload {
    pub incident_id: Uuid,
    pub user_id: Uuid,
}

// ── Security staff list item ──────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SecurityPersonnelItem {
    pub id: Uuid,
    pub full_name: String,
    pub role_name: String,
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//  COMMANDS
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SH-02 / UC-SS-01: Create Incident Report
// ═══════════════════════════════════════════════════════════════════════════════

/// Creates an incident report.
///
/// **Access:** GalacticSecurityHead, GalacticSecurityStaff
#[tauri::command]
pub async fn sec_create_incident_report(
    state: State<'_, AppState>,
    payload: CreateIncidentReportPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::GalacticSecurityHead, Role::GalacticSecurityStaff
    ]);

    // Validate source
    if !["direct_observation", "external_report", "assignment"].contains(&payload.source.as_str()) {
        return Err(AppError::Internal("Invalid source. Must be direct_observation, external_report, or assignment.".into()));
    }

    // Validate severity
    if !["low", "medium", "high", "critical"].contains(&payload.severity.as_str()) {
        return Err(AppError::Internal("Invalid severity. Must be low, medium, high, or critical.".into()));
    }

    // Parse occurred_at if provided
    let occurred_at: Option<DateTime<Utc>> = match &payload.occurred_at {
        Some(s) if !s.is_empty() => Some(
            s.parse::<DateTime<Utc>>()
                .map_err(|_| AppError::Internal("Invalid occurred_at datetime format.".into()))?,
        ),
        _ => None,
    };

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO incident_reports
            (reported_by, source, incident_type, location, occurred_at, description,
             severity, recommended_action, sector_or_base, related_incident_ids, incident_meta)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.source)
    .bind(&payload.incident_type)
    .bind(&payload.location)
    .bind(occurred_at)
    .bind(&payload.description)
    .bind(&payload.severity)
    .bind(&payload.recommended_action)
    .bind(&payload.sector_or_base)
    .bind(&payload.related_incident_ids)
    .bind(&payload.incident_meta)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "incident_reports",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "incident_type": payload.incident_type,
            "severity": payload.severity,
        })),
    )
    .await?;

    // Invalidate incident archive cache so next read gets fresh data
    invalidate_cache(&state, "security_incidents").await;

    // If staff created the report, notify the Security Head
    if user.role == Role::GalacticSecurityStaff {
        let heads: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT u.id FROM users u JOIN roles r ON r.id = u.role_id WHERE r.name = 'GalacticSecurityHead' AND u.is_active = true AND u.deleted_at IS NULL",
        )
        .fetch_all(&state.db_pool)
        .await?;

        for (head_id,) in heads {
            let _ = sqlx::query(
                r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'report:received', $2::jsonb)"#,
            )
            .bind(head_id)
            .bind(serde_json::json!({
                "report_type": "incident_report",
                "incident_id": row.0,
                "severity": payload.severity,
                "submitted_by": user.id,
            }))
            .execute(&state.db_pool)
            .await;
        }
    }

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Get Incident Archive
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns all incident reports visible to the current security user.
/// Cached in Redis for 5 minutes; invalidated on every create/assign mutation.
///
/// **Access:** GalacticSecurityHead, GalacticSecurityStaff
#[tauri::command]
pub async fn sec_get_incident_archive(
    state: State<'_, AppState>,
) -> Result<Vec<IncidentReportSummary>, AppError> {
    let _user = crate::require_auth_any!(state, [
        Role::GalacticSecurityHead, Role::GalacticSecurityStaff
    ]);

    const CACHE_KEY: &str = "security_incidents";

    // Try Redis cache first
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(CACHE_KEY).await {
            if let Ok(parsed) = serde_json::from_str::<Vec<IncidentReportSummary>>(&cached) {
                return Ok(parsed);
            }
        }
    }

    let rows = sqlx::query_as::<_, IncidentReportSummary>(
        r#"
        SELECT ir.id, ir.reported_by, u.full_name AS reporter_name,
               ir.source, ir.incident_type, ir.location, ir.occurred_at,
               ir.description, ir.severity, ir.recommended_action,
               ir.sector_or_base, ir.assigned_to,
               au.full_name AS assigned_to_name,
               ir.related_incident_ids, ir.incident_meta, ir.created_at
        FROM incident_reports ir
        JOIN users u ON u.id = ir.reported_by
        LEFT JOIN users au ON au.id = ir.assigned_to
        WHERE ir.deleted_at IS NULL
        ORDER BY ir.created_at DESC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    // Populate cache
    if let Ok(json) = serde_json::to_string(&rows) {
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            let _: Result<(), _> = conn.set_ex(CACHE_KEY, &json, SECURITY_INCIDENTS_TTL).await;
        }
    }

    Ok(rows)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SH-03: Assign Staff to Incident Handling
// ═══════════════════════════════════════════════════════════════════════════════

/// Assigns a security staff member to handle an incident.
///
/// **Access:** GalacticSecurityHead only
#[tauri::command]
pub async fn sec_assign_staff_to_incident(
    state: State<'_, AppState>,
    payload: AssignStaffPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::GalacticSecurityHead]);

    // Verify incident exists
    let incident_check: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM incident_reports WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.incident_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if incident_check.is_none() {
        return Err(AppError::Internal("Incident report not found.".into()));
    }

    // Verify target user is a GalacticSecurityStaff
    let staff_check: Option<(Uuid,)> = sqlx::query_as(
        "SELECT u.id FROM users u JOIN roles r ON r.id = u.role_id WHERE u.id = $1 AND r.name = 'GalacticSecurityStaff' AND u.is_active = true AND u.deleted_at IS NULL",
    )
    .bind(payload.user_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if staff_check.is_none() {
        return Err(AppError::Internal("Target user is not an active GalacticSecurityStaff.".into()));
    }

    // Update the incident
    sqlx::query(
        "UPDATE incident_reports SET assigned_to = $1 WHERE id = $2",
    )
    .bind(payload.user_id)
    .bind(payload.incident_id)
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "incident_reports",
        payload.incident_id,
        AuditOperation::Update,
        user.id,
        None,
        Some(serde_json::json!({
            "action": "assign_staff",
            "assigned_to": payload.user_id,
        })),
    )
    .await?;

    // Invalidate incident archive cache
    invalidate_cache(&state, "security_incidents").await;

    // Notify the assigned staff
    let _ = sqlx::query(
        r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'task:assigned', $2::jsonb)"#,
    )
    .bind(payload.user_id)
    .bind(serde_json::json!({
        "title": "Incident Assignment",
        "incident_id": payload.incident_id,
        "assigned_by": user.id,
    }))
    .execute(&state.db_pool)
    .await;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SH-05: Submit Daily Security Findings Report
// ═══════════════════════════════════════════════════════════════════════════════

/// Submits a daily security findings report. Delivered to The Guardian.
///
/// **Access:** GalacticSecurityHead only
#[tauri::command]
pub async fn sec_submit_daily_report(
    state: State<'_, AppState>,
    payload: DailyReportPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::GalacticSecurityHead]);

    let report_date = payload
        .report_date
        .parse::<NaiveDate>()
        .map_err(|_| AppError::Internal("Invalid report_date format. Use YYYY-MM-DD.".into()))?;

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO daily_security_reports
            (submitted_by, report_date, findings_summary, risk_notes, recommended_actions)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(report_date)
    .bind(&payload.findings_summary)
    .bind(&payload.risk_notes)
    .bind(&payload.recommended_actions)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "daily_security_reports",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "report_date": payload.report_date })),
    )
    .await?;

    // Notify The Guardian
    let guardians: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT u.id FROM users u JOIN roles r ON r.id = u.role_id WHERE r.name = 'TheGuardian' AND u.is_active = true AND u.deleted_at IS NULL",
    )
    .fetch_all(&state.db_pool)
    .await?;

    for (gid,) in &guardians {
        let _ = sqlx::query(
            r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'report:received', $2::jsonb)"#,
        )
        .bind(gid)
        .bind(serde_json::json!({
            "report_type": "daily_security_report",
            "report_id": row.0,
            "report_date": payload.report_date,
            "submitted_by": user.id,
        }))
        .execute(&state.db_pool)
        .await;
    }

    // Mark as delivered to guardian
    if !guardians.is_empty() {
        let _ = sqlx::query(
            "UPDATE daily_security_reports SET delivered_to_guardian_at = NOW() WHERE id = $1",
        )
        .bind(row.0)
        .execute(&state.db_pool)
        .await;
    }

    // Invalidate this user's daily-report cache
    invalidate_cache(&state, &format!("security_daily_reports:{}", user.id)).await;

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SH-01: Submit Broadcast Announcement Request
// ═══════════════════════════════════════════════════════════════════════════════

/// Submits a security broadcast request for Guardian review.
/// Reuses the existing broadcast_requests table with type='security'.
///
/// **Access:** GalacticSecurityHead only
#[tauri::command]
pub async fn sec_submit_broadcast_request(
    state: State<'_, AppState>,
    payload: SecBroadcastRequestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::GalacticSecurityHead]);

    let urgency = payload.urgency.as_deref().unwrap_or("high");
    if !["low", "normal", "high", "critical"].contains(&urgency) {
        return Err(AppError::Internal("Invalid urgency. Must be low, normal, high, or critical.".into()));
    }

    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO broadcast_requests
            (requester_id, type, subject, content, target_scope, urgency, rationale, status)
        VALUES ($1, 'security', $2, $3, 'company_wide', $4, $5, 'pending')
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.subject)
    .bind(&payload.content)
    .bind(urgency)
    .bind(&payload.rationale)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "broadcast_requests",
        row.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "type": "security",
            "subject": payload.subject,
        })),
    )
    .await?;

    // Notify The Guardian about the pending request
    let guardians: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT u.id FROM users u JOIN roles r ON r.id = u.role_id WHERE r.name = 'TheGuardian' AND u.is_active = true AND u.deleted_at IS NULL",
    )
    .fetch_all(&state.db_pool)
    .await?;

    for (gid,) in guardians {
        let _ = sqlx::query(
            r#"INSERT INTO notifications (user_id, type, payload) VALUES ($1, 'report:received', $2::jsonb)"#,
        )
        .bind(gid)
        .bind(serde_json::json!({
            "report_type": "broadcast_request",
            "request_id": row.0,
            "subject": payload.subject,
            "submitted_by": user.id,
        }))
        .execute(&state.db_pool)
        .await;
    }

    // Invalidate this user's broadcast-request cache
    invalidate_cache(&state, &format!("security_broadcasts:{}", user.id)).await;

    Ok(row.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Get my broadcast requests (security team)
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns broadcast requests submitted by the current user (security type).
/// Cached in Redis for 5 minutes per user; invalidated on new submission.
///
/// **Access:** GalacticSecurityHead, GalacticSecurityStaff
#[tauri::command]
pub async fn sec_get_my_broadcast_requests(
    state: State<'_, AppState>,
) -> Result<Vec<SecBroadcastRequestSummary>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::GalacticSecurityHead, Role::GalacticSecurityStaff
    ]);

    let cache_key = format!("security_broadcasts:{}", user.id);

    // Try Redis cache first
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(parsed) = serde_json::from_str::<Vec<SecBroadcastRequestSummary>>(&cached) {
                return Ok(parsed);
            }
        }
    }

    let rows = sqlx::query_as::<_, SecBroadcastRequestSummary>(
        r#"
        SELECT br.id, br.requester_id, u.full_name AS requester_name,
               br.type AS type_, br.subject, br.content, br.target_scope,
               br.urgency, br.rationale, br.status, br.created_at
        FROM broadcast_requests br
        JOIN users u ON u.id = br.requester_id
        WHERE br.requester_id = $1
          AND br.type = 'security'
          AND br.deleted_at IS NULL
        ORDER BY br.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    // Populate cache
    if let Ok(json) = serde_json::to_string(&rows) {
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json, SECURITY_BROADCASTS_TTL).await;
        }
    }

    Ok(rows)
}

// ═══════════════════════════════════════════════════════════════════════════════
// UC-SH-04 / UC-SS-02: Security Inter-Team Communication Line
// ═══════════════════════════════════════════════════════════════════════════════

/// Sends a message on the security communication line.
/// Uses the shared messages table with channel='security'.
///
/// **Access:** GalacticSecurityHead, GalacticSecurityStaff
#[tauri::command]
pub async fn sec_send_security_message(
    state: State<'_, AppState>,
    payload: SecSendMessagePayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::GalacticSecurityHead, Role::GalacticSecurityStaff
    ]);

    if payload.recipients_to.is_empty() {
        return Err(AppError::Internal("At least one 'To' recipient is required.".into()));
    }

    // Insert the message
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO messages (from_id, subject, body, channel)
        VALUES ($1, $2, $3, 'security')
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.subject)
    .bind(&payload.body)
    .fetch_one(&state.db_pool)
    .await?;

    let msg_id = row.0;

    // Insert recipients
    for uid in &payload.recipients_to {
        let _ = sqlx::query(
            "INSERT INTO message_recipients (message_id, user_id, type) VALUES ($1, $2, 'to') ON CONFLICT DO NOTHING",
        )
        .bind(msg_id)
        .bind(uid)
        .execute(&state.db_pool)
        .await;
    }
    if let Some(cc) = &payload.recipients_cc {
        for uid in cc {
            let _ = sqlx::query(
                "INSERT INTO message_recipients (message_id, user_id, type) VALUES ($1, $2, 'cc') ON CONFLICT DO NOTHING",
            )
            .bind(msg_id)
            .bind(uid)
            .execute(&state.db_pool)
            .await;
        }
    }
    if let Some(bcc) = &payload.recipients_bcc {
        for uid in bcc {
            let _ = sqlx::query(
                "INSERT INTO message_recipients (message_id, user_id, type) VALUES ($1, $2, 'bcc') ON CONFLICT DO NOTHING",
            )
            .bind(msg_id)
            .bind(uid)
            .execute(&state.db_pool)
            .await;
        }
    }

    write_audit_log(
        &state.db_pool,
        "messages",
        msg_id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "subject": payload.subject, "channel": "security" })),
    )
    .await?;

    // Invalidate security messages cache for all involved parties
    invalidate_cache(&state, &format!("sec_messages:{}", user.id)).await;
    for uid in &payload.recipients_to {
        invalidate_cache(&state, &format!("sec_messages:{}", uid)).await;
    }

    Ok(msg_id)
}

/// Get all security-channel messages visible to the current security user.
/// Cached in Redis for 2 minutes per user; invalidated on send.
///
/// **Access:** GalacticSecurityHead, GalacticSecurityStaff
#[tauri::command]
pub async fn sec_get_security_messages(
    state: State<'_, AppState>,
) -> Result<Vec<SecMessageSummary>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::GalacticSecurityHead, Role::GalacticSecurityStaff
    ]);

    let cache_key = format!("sec_messages:{}", user.id);

    // Try Redis cache first
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(parsed) = serde_json::from_str::<Vec<SecMessageSummary>>(&cached) {
                return Ok(parsed);
            }
        }
    }

    let messages = sqlx::query_as::<_, SecMessageSummary>(
        r#"
        SELECT DISTINCT m.id, m.from_id, u.full_name AS from_name,
               m.subject, m.body, m.channel, m.recalled_at, m.created_at
        FROM messages m
        JOIN users u ON u.id = m.from_id
        LEFT JOIN message_recipients mr ON mr.message_id = m.id
        WHERE m.channel = 'security'
          AND m.deleted_at IS NULL
          AND (m.from_id = $1 OR mr.user_id = $1)
        ORDER BY m.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    // Populate cache
    if let Ok(json) = serde_json::to_string(&messages) {
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json, SECURITY_MESSAGES_TTL).await;
        }
    }

    Ok(messages)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Get Security Personnel
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns all active security personnel (both Head and Staff) + The Guardian.
/// Used for messaging recipient picker and staff assignment.
/// Cached in Redis for 1 hour; invalidated when user accounts are created/terminated.
///
/// **Access:** GalacticSecurityHead, GalacticSecurityStaff
#[tauri::command]
pub async fn sec_get_security_personnel(
    state: State<'_, AppState>,
) -> Result<Vec<SecurityPersonnelItem>, AppError> {
    let _user = crate::require_auth_any!(state, [
        Role::GalacticSecurityHead, Role::GalacticSecurityStaff
    ]);

    const CACHE_KEY: &str = "security_personnel";

    // Try Redis cache first
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(CACHE_KEY).await {
            if let Ok(parsed) = serde_json::from_str::<Vec<SecurityPersonnelItem>>(&cached) {
                return Ok(parsed);
            }
        }
    }

    let rows = sqlx::query_as::<_, SecurityPersonnelItem>(
        r#"
        SELECT u.id, u.full_name, r.name AS role_name
        FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE r.name IN ('GalacticSecurityHead', 'GalacticSecurityStaff', 'TheGuardian')
          AND u.is_active = true
          AND u.deleted_at IS NULL
        ORDER BY r.name, u.full_name
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    // Populate cache
    if let Ok(json) = serde_json::to_string(&rows) {
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            let _: Result<(), _> = conn.set_ex(CACHE_KEY, &json, SECURITY_PERSONNEL_TTL).await;
        }
    }

    Ok(rows)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Get Daily Security Reports (for Security Head self-view)
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns daily security reports submitted by the current user.
/// Cached in Redis for 5 minutes per user; invalidated on new submission.
///
/// **Access:** GalacticSecurityHead
#[tauri::command]
pub async fn sec_get_my_daily_reports(
    state: State<'_, AppState>,
) -> Result<Vec<DailySecurityReportSummary>, AppError> {
    let user = crate::require_auth_any!(state, [Role::GalacticSecurityHead]);

    let cache_key = format!("security_daily_reports:{}", user.id);

    // Try Redis cache first
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(parsed) = serde_json::from_str::<Vec<DailySecurityReportSummary>>(&cached) {
                return Ok(parsed);
            }
        }
    }

    let rows = sqlx::query_as::<_, DailySecurityReportSummary>(
        r#"
        SELECT dsr.id, dsr.submitted_by, u.full_name AS submitter_name,
               dsr.report_date, dsr.findings_summary, dsr.risk_notes,
               dsr.recommended_actions, dsr.delivered_to_guardian_at, dsr.created_at
        FROM daily_security_reports dsr
        JOIN users u ON u.id = dsr.submitted_by
        WHERE dsr.submitted_by = $1
          AND dsr.deleted_at IS NULL
        ORDER BY dsr.report_date DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    // Populate cache
    if let Ok(json) = serde_json::to_string(&rows) {
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json, SECURITY_DAILY_TTL).await;
        }
    }

    Ok(rows)
}
