// commands/psychiatry.rs — Psychiatry Division commands (Sub-07)
// Source of truth: 07_PSYCHIATRY.md
//
// Actors:
//   Psychiatrist  — full access to own patients
//   PsychiatristAssistant — schedule + recovery log overview (GDPR-checked)
//   Patient (any RUSA user under care) — grant/deny schedule access
//
// Privacy rules enforced at command level:
//   - Psychiatrist sees ONLY own patients
//   - Assistant sees restricted view (no clinical findings)
//   - GDPR flag checked on base_locations.has_data_regulation

use chrono::{DateTime, Utc};
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

// ── Payloads ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreatePatientPayload {
    pub user_id: Uuid,
    pub patient_profile: serde_json::Value,
    pub initial_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LogAppointmentPayload {
    pub patient_id: Uuid,
    pub scheduled_at: String,
    pub appointment_log: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecoveryLogPayload {
    pub patient_id: Uuid,
    pub milestone: String,
    pub status: String,
    pub details: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ManageSchedulePayload {
    pub slots: Vec<ScheduleSlotInput>,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleSlotInput {
    pub slot_start: String,
    pub slot_end: String,
    pub is_available: bool,
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleAppointmentPayload {
    pub patient_id: Uuid,
    pub psychiatrist_id: Uuid,
    pub scheduled_at: String,
}

#[derive(Debug, Deserialize)]
pub struct GrantAccessPayload {
    pub assistant_id: Uuid,
    pub grant: bool,
}

// ── Response types ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PatientSummary {
    pub id: Uuid,
    pub user_id: Uuid,
    pub patient_name: String,
    pub care_status: String,
    pub initial_notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PatientDetail {
    pub id: Uuid,
    pub user_id: Uuid,
    pub psychiatrist_id: Uuid,
    pub patient_profile: serde_json::Value,
    pub initial_notes: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AppointmentRecord {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub psychiatrist_id: Uuid,
    pub scheduled_at: DateTime<Utc>,
    pub status: String,
    pub booked_by: Option<Uuid>,
    pub appointment_log: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Restricted appointment view for assistants — no clinical data
#[derive(Debug, Serialize)]
pub struct AppointmentSummary {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub scheduled_at: DateTime<Utc>,
    pub status: String,
    pub booked_by: Option<Uuid>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RecoveryLogEntry {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub milestone: String,
    pub status: String,
    pub details: Option<String>,
    pub logged_at: Option<DateTime<Utc>>,
    pub logged_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ScheduleSlot {
    pub id: Uuid,
    pub psychiatrist_id: Uuid,
    pub slot_start: DateTime<Utc>,
    pub slot_end: DateTime<Utc>,
    pub is_available: bool,
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PatientIndexEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub patient_name: String,
    pub department: String,
    pub care_status: String,
    pub last_updated: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct PatientListEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub patient_name: String,
    pub care_status: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AccessSetting {
    pub assistant_id: Uuid,
    pub assistant_name: String,
    pub granted: bool,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct UserPickerEntry {
    pub id: Uuid,
    pub full_name: String,
    pub username: String,
    pub role_name: String,
}

#[derive(Debug, Serialize)]
pub struct CreatedId {
    pub id: Uuid,
}

// ── Helper: get psychiatrist's assigned assistant ──────────────────────────────

/// Check base_locations.has_data_regulation for the current user's base
async fn check_data_regulation(pool: &sqlx::PgPool, user_id: Uuid) -> Result<bool, AppError> {
    let row: Option<(bool,)> = sqlx::query_as(
        r#"SELECT bl.has_data_regulation
           FROM users u
           JOIN base_locations bl ON bl.id = u.base_location_id
           WHERE u.id = $1 AND u.deleted_at IS NULL AND bl.deleted_at IS NULL"#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.0).unwrap_or(true)) // default: regulated
}

// ══════════════════════════════════════════════════════════════════════════════
// PSYCHIATRIST COMMANDS
// ══════════════════════════════════════════════════════════════════════════════

/// UC-PSY-01: Create Patient Record
/// Creates a new patient linked to the current psychiatrist.
/// Patient picked from user directory (non-Director, non-Admin only).
#[tauri::command]
pub async fn psy_create_patient_record(
    state: State<'_, AppState>,
    payload: CreatePatientPayload,
) -> Result<CreatedId, AppError> {
    let user = crate::require_auth_any!(state, [Role::Psychiatrist, Role::Administrator]);
    let pool = &state.db_pool;

    // Validate target user exists and is not a Director/Admin
    let target: Option<(String,)> = sqlx::query_as(
        r#"SELECT r.name FROM users u JOIN roles r ON r.id = u.role_id
           WHERE u.id = $1 AND u.deleted_at IS NULL"#,
    )
    .bind(payload.user_id)
    .fetch_optional(pool)
    .await?;

    let target_role = target.ok_or(AppError::Internal("Not found.".into()))?.0;

    let blocked_roles = [
        "GeneralDirector", "TheDirector", "TheAccountant", "TheLibrarian",
        "TheNomad", "TheArtificer", "TheObserver", "TheWanderer",
        "TheTaskmaster", "TheGuardian", "TheStatistician", "TheCoordinator",
        "TheOverseer", "TheAnchorman", "Administrator",
    ];
    if blocked_roles.contains(&target_role.as_str()) {
        return Err(AppError::Forbidden);
    }

    let row: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO psychiatric_patients (user_id, psychiatrist_id, patient_profile, initial_notes)
           VALUES ($1, $2, $3, $4)
           RETURNING id"#,
    )
    .bind(payload.user_id)
    .bind(user.id)
    .bind(&payload.patient_profile)
    .bind(&payload.initial_notes)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to create patient: {e}")))?;

    // Upsert patient_index
    sqlx::query(
        r#"INSERT INTO patient_index (user_id, department, care_status, psychiatrist_id)
           VALUES ($1, 'psychiatry', 'active', $2)
           ON CONFLICT (user_id) DO UPDATE SET
             department = CASE WHEN patient_index.department = 'medical' THEN 'both' ELSE 'psychiatry' END,
             psychiatrist_id = $2,
             care_status = 'active',
             last_updated = NOW()"#,
    )
    .bind(payload.user_id)
    .bind(user.id)
    .execute(pool)
    .await?;

    write_audit_log(
        pool, "psychiatric_patients", row.0, AuditOperation::Create, user.id,
        None,
        Some(serde_json::json!({
            "user_id": payload.user_id,
            "psychiatrist_id": user.id,
        })),
    )
    .await?;

    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(format!("patient_list:{}", user.id)).await;
    }

    Ok(CreatedId { id: row.0 })
}

/// UC-PSY-01 helper: Get eligible users for patient picker (non-Director, non-Admin)
#[tauri::command]
pub async fn psy_get_user_directory(
    state: State<'_, AppState>,
) -> Result<Vec<UserPickerEntry>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::Psychiatrist, Role::Administrator]);
    let pool = &state.db_pool;

    let rows = sqlx::query_as::<_, UserPickerEntry>(
        r#"SELECT u.id, u.full_name, u.username, r.name AS role_name
           FROM users u JOIN roles r ON r.id = u.role_id
           WHERE u.deleted_at IS NULL
             AND r.name NOT IN (
               'GeneralDirector','TheDirector','TheAccountant','TheLibrarian',
               'TheNomad','TheArtificer','TheObserver','TheWanderer',
               'TheTaskmaster','TheGuardian','TheStatistician','TheCoordinator',
               'TheOverseer','TheAnchorman','Administrator'
             )
           ORDER BY u.full_name"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Get my patients — only patients assigned to current psychiatrist
#[tauri::command]
pub async fn psy_get_my_patients(
    state: State<'_, AppState>,
) -> Result<Vec<PatientSummary>, AppError> {
    let user = crate::require_auth_any!(state, [Role::Psychiatrist, Role::Administrator]);
    let pool = &state.db_pool;

    // Check Redis cache (15 min TTL)
    let cache_key = format!("patient_list:{}", user.id);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(items) = serde_json::from_str::<Vec<PatientSummary>>(&cached) {
                return Ok(items);
            }
        }
    }

    let rows = sqlx::query_as::<_, PatientSummary>(
        r#"SELECT pp.id, pp.user_id, u.full_name AS patient_name,
                  COALESCE(pi.care_status, 'active') AS care_status,
                  pp.initial_notes, pp.created_at
           FROM psychiatric_patients pp
           JOIN users u ON u.id = pp.user_id
           LEFT JOIN patient_index pi ON pi.user_id = pp.user_id
           WHERE pp.psychiatrist_id = $1 AND pp.deleted_at IS NULL
           ORDER BY pp.created_at DESC"#,
    )
    .bind(user.id)
    .fetch_all(pool)
    .await?;

    // Populate cache — 15 minutes
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(json) = serde_json::to_string(&rows) {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json, 900).await;
        }
    }

    Ok(rows)
}

/// Get patient detail — psychiatrist's own patient only
#[tauri::command]
pub async fn psy_get_patient_detail(
    state: State<'_, AppState>,
    patient_id: Uuid,
) -> Result<PatientDetail, AppError> {
    let user = crate::require_auth_any!(state, [Role::Psychiatrist, Role::Administrator]);
    let pool = &state.db_pool;

    let row = sqlx::query_as::<_, PatientDetail>(
        r#"SELECT id, user_id, psychiatrist_id, patient_profile, initial_notes, created_at, updated_at
           FROM psychiatric_patients
           WHERE id = $1 AND psychiatrist_id = $2 AND deleted_at IS NULL"#,
    )
    .bind(patient_id)
    .bind(user.id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Internal("Not found.".into()))?;

    Ok(row)
}

/// UC-PSY-02: Log Appointment Findings
#[tauri::command]
pub async fn psy_log_appointment(
    state: State<'_, AppState>,
    payload: LogAppointmentPayload,
) -> Result<CreatedId, AppError> {
    let user = crate::require_auth_any!(state, [Role::Psychiatrist, Role::Administrator]);
    let pool = &state.db_pool;

    // Verify this is the psychiatrist's own patient
    let _patient: (Uuid,) = sqlx::query_as(
        r#"SELECT id FROM psychiatric_patients
           WHERE id = $1 AND psychiatrist_id = $2 AND deleted_at IS NULL"#,
    )
    .bind(payload.patient_id)
    .bind(user.id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Internal("Not found.".into()))?;

    let scheduled_at = payload.scheduled_at.parse::<DateTime<Utc>>()
        .map_err(|e| AppError::Internal(format!("Invalid date: {e}")))?;

    let row: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO psychiatric_appointments (patient_id, psychiatrist_id, scheduled_at, status, booked_by, appointment_log)
           VALUES ($1, $2, $3, 'completed', $2, $4)
           RETURNING id"#,
    )
    .bind(payload.patient_id)
    .bind(user.id)
    .bind(scheduled_at)
    .bind(&payload.appointment_log)
    .fetch_one(pool)
    .await?;

    write_audit_log(
        pool, "psychiatric_appointments", row.0, AuditOperation::Create, user.id,
        None,
        Some(serde_json::json!({
            "patient_id": payload.patient_id,
            "scheduled_at": payload.scheduled_at,
        })),
    )
    .await?;

    Ok(CreatedId { id: row.0 })
}

/// Get appointments for a patient (psychiatrist's own)
#[tauri::command]
pub async fn psy_get_appointments(
    state: State<'_, AppState>,
    patient_id: Uuid,
) -> Result<Vec<AppointmentRecord>, AppError> {
    let user = crate::require_auth_any!(state, [Role::Psychiatrist, Role::Administrator]);
    let pool = &state.db_pool;

    // Ownership check
    let _: (Uuid,) = sqlx::query_as(
        r#"SELECT id FROM psychiatric_patients
           WHERE id = $1 AND psychiatrist_id = $2 AND deleted_at IS NULL"#,
    )
    .bind(patient_id)
    .bind(user.id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Internal("Not found.".into()))?;

    let rows = sqlx::query_as::<_, AppointmentRecord>(
        r#"SELECT id, patient_id, psychiatrist_id, scheduled_at, status,
                  booked_by, appointment_log, created_at
           FROM psychiatric_appointments
           WHERE patient_id = $1 AND deleted_at IS NULL
           ORDER BY scheduled_at DESC"#,
    )
    .bind(patient_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// UC-PSY-03: Update Patient Recovery Log (add entry)
#[tauri::command]
pub async fn psy_update_recovery_log(
    state: State<'_, AppState>,
    payload: UpdateRecoveryLogPayload,
) -> Result<CreatedId, AppError> {
    let user = crate::require_auth_any!(state, [Role::Psychiatrist, Role::Administrator]);
    let pool = &state.db_pool;

    // Ownership check
    let _: (Uuid,) = sqlx::query_as(
        r#"SELECT id FROM psychiatric_patients
           WHERE id = $1 AND psychiatrist_id = $2 AND deleted_at IS NULL"#,
    )
    .bind(payload.patient_id)
    .bind(user.id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Internal("Not found.".into()))?;

    // Validate status
    let valid_statuses = ["in_progress", "achieved", "regressed", "on_hold"];
    if !valid_statuses.contains(&payload.status.as_str()) {
        return Err(AppError::Internal(format!(
            "Invalid status. Must be one of: {}",
            valid_statuses.join(", ")
        )));
    }

    let row: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO psychiatric_recovery_logs (patient_id, milestone, status, details, logged_by)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id"#,
    )
    .bind(payload.patient_id)
    .bind(&payload.milestone)
    .bind(&payload.status)
    .bind(&payload.details)
    .bind(user.id)
    .fetch_one(pool)
    .await?;

    write_audit_log(
        pool, "psychiatric_recovery_logs", row.0, AuditOperation::Create, user.id,
        None,
        Some(serde_json::json!({
            "patient_id": payload.patient_id,
            "milestone": payload.milestone,
            "status": payload.status,
        })),
    )
    .await?;

    Ok(CreatedId { id: row.0 })
}

/// Get recovery logs for a patient (psychiatrist's own — full details)
#[tauri::command]
pub async fn psy_get_recovery_logs(
    state: State<'_, AppState>,
    patient_id: Uuid,
) -> Result<Vec<RecoveryLogEntry>, AppError> {
    let user = crate::require_auth_any!(state, [Role::Psychiatrist, Role::Administrator]);
    let pool = &state.db_pool;

    // Ownership check
    let _: (Uuid,) = sqlx::query_as(
        r#"SELECT id FROM psychiatric_patients
           WHERE id = $1 AND psychiatrist_id = $2 AND deleted_at IS NULL"#,
    )
    .bind(patient_id)
    .bind(user.id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Internal("Not found.".into()))?;

    let rows = sqlx::query_as::<_, RecoveryLogEntry>(
        r#"SELECT id, patient_id, milestone, status, details, logged_at, logged_by
           FROM psychiatric_recovery_logs
           WHERE patient_id = $1 AND deleted_at IS NULL
           ORDER BY logged_at DESC"#,
    )
    .bind(patient_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// UC-PSY-04: Manage own schedule (add/replace slots)
#[tauri::command]
pub async fn psy_manage_schedule(
    state: State<'_, AppState>,
    payload: ManageSchedulePayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::Psychiatrist, Role::Administrator]);
    let pool = &state.db_pool;

    for slot in &payload.slots {
        let start = slot.slot_start.parse::<DateTime<Utc>>()
            .map_err(|e| AppError::Internal(format!("Invalid slot_start: {e}")))?;
        let end = slot.slot_end.parse::<DateTime<Utc>>()
            .map_err(|e| AppError::Internal(format!("Invalid slot_end: {e}")))?;

        if end <= start {
            return Err(AppError::Internal("slot_end must be after slot_start".into()));
        }

        let row: (Uuid,) = sqlx::query_as(
            r#"INSERT INTO psychiatrist_schedule (psychiatrist_id, slot_start, slot_end, is_available, blocked_reason)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING id"#,
        )
        .bind(user.id)
        .bind(start)
        .bind(end)
        .bind(slot.is_available)
        .bind(&slot.blocked_reason)
        .fetch_one(pool)
        .await?;

        write_audit_log(
            pool, "psychiatrist_schedule", row.0, AuditOperation::Create, user.id,
            None,
            Some(serde_json::json!({
                "slot_start": slot.slot_start,
                "slot_end": slot.slot_end,
                "is_available": slot.is_available,
            })),
        )
        .await?;
    }

    // Invalidate cache
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(format!("psychiatrist_schedule:{}", user.id)).await;
    }

    Ok(())
}

/// Get psychiatrist's own schedule
#[tauri::command]
pub async fn psy_get_schedule(
    state: State<'_, AppState>,
) -> Result<Vec<ScheduleSlot>, AppError> {
    let user = crate::require_auth_any!(state, [Role::Psychiatrist, Role::Administrator]);
    let pool = &state.db_pool;

    // Check Redis cache (30 min TTL)
    let cache_key = format!("psychiatrist_schedule:{}", user.id);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(items) = serde_json::from_str::<Vec<ScheduleSlot>>(&cached) {
                return Ok(items);
            }
        }
    }

    let rows = sqlx::query_as::<_, ScheduleSlot>(
        r#"SELECT id, psychiatrist_id, slot_start, slot_end, is_available, blocked_reason
           FROM psychiatrist_schedule
           WHERE psychiatrist_id = $1 AND deleted_at IS NULL
           ORDER BY slot_start ASC"#,
    )
    .bind(user.id)
    .fetch_all(pool)
    .await?;

    // Populate cache — 30 minutes
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(json) = serde_json::to_string(&rows) {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json, 1800).await;
        }
    }

    Ok(rows)
}

/// Delete a schedule slot
#[tauri::command]
pub async fn psy_delete_schedule_slot(
    state: State<'_, AppState>,
    slot_id: Uuid,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::Psychiatrist, Role::Administrator]);
    let pool = &state.db_pool;

    let result = sqlx::query(
        r#"UPDATE psychiatrist_schedule SET deleted_at = NOW(), deleted_by = $1
           WHERE id = $2 AND psychiatrist_id = $1 AND deleted_at IS NULL"#,
    )
    .bind(user.id)
    .bind(slot_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Internal("Not found.".into()));
    }

    write_audit_log(
        pool, "psychiatrist_schedule", slot_id, AuditOperation::Delete, user.id,
        None, None,
    )
    .await?;

    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(format!("psychiatrist_schedule:{}", user.id)).await;
    }

    Ok(())
}

/// UC-PSY-05: View shared patient index (coordination only — no clinical data)
#[tauri::command]
pub async fn psy_get_patient_index(
    state: State<'_, AppState>,
) -> Result<Vec<PatientIndexEntry>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::Psychiatrist, Role::Administrator]);
    let pool = &state.db_pool;

    let rows = sqlx::query_as::<_, PatientIndexEntry>(
        r#"SELECT pi.id, pi.user_id, u.full_name AS patient_name,
                  pi.department, pi.care_status, pi.last_updated
           FROM patient_index pi
           JOIN users u ON u.id = pi.user_id
           WHERE pi.deleted_at IS NULL
           ORDER BY u.full_name"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

// ══════════════════════════════════════════════════════════════════════════════
// PSYCHIATRIST ASSISTANT COMMANDS
// ══════════════════════════════════════════════════════════════════════════════

/// Helper: get the psychiatrist that this assistant works with
/// (We assume an assistant is associated with a psychiatrist at the same base location)
/// In this implementation, assistant can see patients of ALL psychiatrists at same base.
/// For simplicity, we link via base_location_id.
async fn get_assistant_psychiatrists(
    pool: &sqlx::PgPool,
    assistant_id: Uuid,
) -> Result<Vec<Uuid>, AppError> {
    let rows: Vec<(Uuid,)> = sqlx::query_as(
        r#"SELECT p.id FROM users p
           JOIN roles r ON r.id = p.role_id
           JOIN users a ON a.base_location_id = p.base_location_id
           WHERE a.id = $1 AND r.name = 'Psychiatrist'
             AND p.deleted_at IS NULL AND a.deleted_at IS NULL"#,
    )
    .bind(assistant_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// UC-PA-01: Schedule patient appointment
#[tauri::command]
pub async fn psy_schedule_appointment(
    state: State<'_, AppState>,
    payload: ScheduleAppointmentPayload,
) -> Result<CreatedId, AppError> {
    let user = crate::require_auth_any!(state, [Role::PsychiatristAssistant, Role::Administrator]);
    let pool = &state.db_pool;

    // Verify the patient record exists
    let patient: Option<(Uuid, Uuid)> = sqlx::query_as(
        r#"SELECT id, psychiatrist_id FROM psychiatric_patients
           WHERE id = $1 AND deleted_at IS NULL"#,
    )
    .bind(payload.patient_id)
    .fetch_optional(pool)
    .await?;

    let (patient_id, psychiatrist_id) = patient.ok_or(AppError::Internal("Not found.".into()))?;

    // Verify the target psychiatrist matches
    if psychiatrist_id != payload.psychiatrist_id {
        return Err(AppError::Internal("Psychiatrist mismatch".into()));
    }

    let scheduled_at = payload.scheduled_at.parse::<DateTime<Utc>>()
        .map_err(|e| AppError::Internal(format!("Invalid date: {e}")))?;

    let row: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO psychiatric_appointments (patient_id, psychiatrist_id, scheduled_at, status, booked_by)
           VALUES ($1, $2, $3, 'scheduled', $4)
           RETURNING id"#,
    )
    .bind(patient_id)
    .bind(psychiatrist_id)
    .bind(scheduled_at)
    .bind(user.id)
    .fetch_one(pool)
    .await?;

    write_audit_log(
        pool, "psychiatric_appointments", row.0, AuditOperation::Create, user.id,
        None,
        Some(serde_json::json!({
            "patient_id": patient_id,
            "psychiatrist_id": psychiatrist_id,
            "scheduled_at": payload.scheduled_at,
            "booked_by": user.id,
        })),
    )
    .await?;

    // Notify psychiatrist and patient
    let patient_user_id: Option<(Uuid,)> = sqlx::query_as(
        r#"SELECT user_id FROM psychiatric_patients WHERE id = $1"#,
    )
    .bind(patient_id)
    .fetch_optional(pool)
    .await?;

    for target_id in [psychiatrist_id, patient_user_id.map(|r| r.0).unwrap_or(psychiatrist_id)] {
        sqlx::query(
            r#"INSERT INTO notifications (user_id, type, payload)
               VALUES ($1, 'appointment:booked', $2)"#,
        )
        .bind(target_id)
        .bind(serde_json::json!({
            "appointment_id": row.0,
            "scheduled_at": payload.scheduled_at,
            "booked_by": user.full_name,
        }))
        .execute(pool)
        .await?;
    }

    Ok(CreatedId { id: row.0 })
}

/// UC-PA-02: Request access to patient's schedule
#[tauri::command]
pub async fn psy_request_schedule_access(
    state: State<'_, AppState>,
    patient_user_id: Uuid,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::PsychiatristAssistant, Role::Administrator]);
    let pool = &state.db_pool;

    // Upsert access request (granted = false until patient approves)
    sqlx::query(
        r#"INSERT INTO patient_schedule_access (patient_user_id, assistant_id, granted, requested_at)
           VALUES ($1, $2, false, NOW())
           ON CONFLICT (patient_user_id, assistant_id) DO UPDATE SET
             requested_at = NOW(), updated_at = NOW()"#,
    )
    .bind(patient_user_id)
    .bind(user.id)
    .execute(pool)
    .await?;

    // Notify the patient
    sqlx::query(
        r#"INSERT INTO notifications (user_id, type, payload)
           VALUES ($1, 'schedule_access:requested', $2)"#,
    )
    .bind(patient_user_id)
    .bind(serde_json::json!({
        "assistant_id": user.id,
        "assistant_name": user.full_name,
    }))
    .execute(pool)
    .await?;

    Ok(())
}

/// UC-PA-03: View patient list (restricted — names and identifiers only)
#[tauri::command]
pub async fn psy_assistant_get_patients(
    state: State<'_, AppState>,
) -> Result<Vec<PatientListEntry>, AppError> {
    let user = crate::require_auth_any!(state, [Role::PsychiatristAssistant, Role::Administrator]);
    let pool = &state.db_pool;

    let psychiatrist_ids = get_assistant_psychiatrists(pool, user.id).await?;
    if psychiatrist_ids.is_empty() {
        return Ok(vec![]);
    }

    let rows = sqlx::query_as::<_, PatientListEntry>(
        r#"SELECT pp.id, pp.user_id, u.full_name AS patient_name,
                  COALESCE(pi.care_status, 'active') AS care_status
           FROM psychiatric_patients pp
           JOIN users u ON u.id = pp.user_id
           LEFT JOIN patient_index pi ON pi.user_id = pp.user_id
           WHERE pp.psychiatrist_id = ANY($1) AND pp.deleted_at IS NULL
           ORDER BY u.full_name"#,
    )
    .bind(&psychiatrist_ids)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// UC-PA-04: View patient recovery log (GDPR-checked)
/// If base has_data_regulation = true → details field stripped
/// If base has_data_regulation = false → details field included
#[tauri::command]
pub async fn psy_assistant_get_recovery_log(
    state: State<'_, AppState>,
    patient_id: Uuid,
) -> Result<Vec<RecoveryLogEntry>, AppError> {
    let user = crate::require_auth_any!(state, [Role::PsychiatristAssistant, Role::Administrator]);
    let pool = &state.db_pool;

    // Check GDPR flag
    let has_regulation = check_data_regulation(pool, user.id).await?;

    // Verify the patient belongs to a psychiatrist at the same base
    let psychiatrist_ids = get_assistant_psychiatrists(pool, user.id).await?;

    let _patient: Option<(Uuid,)> = sqlx::query_as(
        r#"SELECT id FROM psychiatric_patients
           WHERE id = $1 AND psychiatrist_id = ANY($2) AND deleted_at IS NULL"#,
    )
    .bind(patient_id)
    .bind(&psychiatrist_ids)
    .fetch_optional(pool)
    .await?;

    if _patient.is_none() {
        return Err(AppError::Internal("Not found.".into()));
    }

    let mut rows = sqlx::query_as::<_, RecoveryLogEntry>(
        r#"SELECT id, patient_id, milestone, status, details, logged_at, logged_by
           FROM psychiatric_recovery_logs
           WHERE patient_id = $1 AND deleted_at IS NULL
           ORDER BY logged_at DESC"#,
    )
    .bind(patient_id)
    .fetch_all(pool)
    .await?;

    // If GDPR applies, strip the details field
    if has_regulation {
        for entry in &mut rows {
            entry.details = None;
        }
    }

    Ok(rows)
}

/// Get schedule for a psychiatrist (assistant read-only)
#[tauri::command]
pub async fn psy_assistant_get_schedule(
    state: State<'_, AppState>,
    psychiatrist_id: Uuid,
) -> Result<Vec<ScheduleSlot>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::PsychiatristAssistant, Role::Administrator]);
    let pool = &state.db_pool;

    let rows = sqlx::query_as::<_, ScheduleSlot>(
        r#"SELECT id, psychiatrist_id, slot_start, slot_end, is_available, blocked_reason
           FROM psychiatrist_schedule
           WHERE psychiatrist_id = $1 AND deleted_at IS NULL
           ORDER BY slot_start ASC"#,
    )
    .bind(psychiatrist_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Get appointments for a patient (assistant restricted view — no clinical findings)
#[tauri::command]
pub async fn psy_assistant_get_appointments(
    state: State<'_, AppState>,
    patient_id: Uuid,
) -> Result<Vec<AppointmentSummary>, AppError> {
    let user = crate::require_auth_any!(state, [Role::PsychiatristAssistant, Role::Administrator]);
    let pool = &state.db_pool;

    let psychiatrist_ids = get_assistant_psychiatrists(pool, user.id).await?;

    let _exists: (Uuid,) = sqlx::query_as(
        r#"SELECT id FROM psychiatric_patients
           WHERE id = $1 AND psychiatrist_id = ANY($2) AND deleted_at IS NULL"#,
    )
    .bind(patient_id)
    .bind(&psychiatrist_ids)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Internal("Not found.".into()))?;

    let rows = sqlx::query_as::<_, AppointmentRecord>(
        r#"SELECT id, patient_id, psychiatrist_id, scheduled_at, status,
                  booked_by, appointment_log, created_at
           FROM psychiatric_appointments
           WHERE patient_id = $1 AND deleted_at IS NULL
           ORDER BY scheduled_at DESC"#,
    )
    .bind(patient_id)
    .fetch_all(pool)
    .await?;

    // Strip clinical data — return only summary
    let summaries: Vec<AppointmentSummary> = rows
        .into_iter()
        .map(|a| AppointmentSummary {
            id: a.id,
            patient_id: a.patient_id,
            scheduled_at: a.scheduled_at,
            status: a.status,
            booked_by: a.booked_by,
        })
        .collect();

    Ok(summaries)
}

// ══════════════════════════════════════════════════════════════════════════════
// PATIENT COMMANDS (any RUSA user under care)
// ══════════════════════════════════════════════════════════════════════════════

/// UC-PAT-01: Grant or deny assistant schedule access
#[tauri::command]
pub async fn psy_grant_schedule_access(
    state: State<'_, AppState>,
    payload: GrantAccessPayload,
) -> Result<(), AppError> {
    // Any authenticated user can manage their own access settings
    let user_lock = state.current_user.lock().await;
    let user = user_lock.as_ref().ok_or(AppError::Unauthenticated)?;
    let user_id = user.id;
    let user_full_name = user.full_name.clone();
    drop(user_lock);

    let pool = &state.db_pool;

    let result = sqlx::query(
        r#"UPDATE patient_schedule_access
           SET granted = $1, updated_at = NOW()
           WHERE patient_user_id = $2 AND assistant_id = $3"#,
    )
    .bind(payload.grant)
    .bind(user_id)
    .bind(payload.assistant_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Internal("Not found.".into()));
    }

    // Notify the assistant
    let event_type = if payload.grant { "schedule_access:granted" } else { "schedule_access:revoked" };
    sqlx::query(
        r#"INSERT INTO notifications (user_id, type, payload)
           VALUES ($1, $2, $3)"#,
    )
    .bind(payload.assistant_id)
    .bind(event_type)
    .bind(serde_json::json!({
        "patient_name": user_full_name,
        "granted": payload.grant,
    }))
    .execute(pool)
    .await?;

    // Invalidate cache
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(format!("schedule_access:{}", user_id)).await;
    }

    Ok(())
}

/// Get pending access requests and current settings for the patient
#[tauri::command]
pub async fn psy_get_access_settings(
    state: State<'_, AppState>,
) -> Result<Vec<AccessSetting>, AppError> {
    let user_lock = state.current_user.lock().await;
    let user = user_lock.as_ref().ok_or(AppError::Unauthenticated)?;
    let user_id = user.id;
    drop(user_lock);

    let pool = &state.db_pool;

    // Check Redis cache (1 hour TTL)
    let cache_key = format!("schedule_access:{}", user_id);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(items) = serde_json::from_str::<Vec<AccessSetting>>(&cached) {
                return Ok(items);
            }
        }
    }

    let rows = sqlx::query_as::<_, AccessSetting>(
        r#"SELECT psa.assistant_id, u.full_name AS assistant_name,
                  psa.granted, psa.updated_at
           FROM patient_schedule_access psa
           JOIN users u ON u.id = psa.assistant_id
           WHERE psa.patient_user_id = $1
           ORDER BY psa.updated_at DESC"#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    // Populate cache — 1 hour
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(json) = serde_json::to_string(&rows) {
            let _: Result<(), _> = conn.set_ex(&cache_key, &json, 3600).await;
        }
    }

    Ok(rows)
}
