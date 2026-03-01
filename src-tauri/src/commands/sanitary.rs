// commands/sanitary.rs — Sanitary Department subsystem commands
// Source of truth: 10_SANITARY.md
//
// Use cases:
//   UC-HS-01   Submit Budget Funding Request       (HeadOfSanitary)
//   UC-HS-02   Submit Budget Expenditure Report     (HeadOfSanitary)
//   UC-HS-03   Manage Department Inventory (Macro)  (HeadOfSanitary)
//   UC-HS-04   Allocate Staff Shifts (Quarterly)    (HeadOfSanitary)
//   UC-HS-05   Assign Task to Crew Member(s)        (HeadOfSanitary)
//   UC-HS-06   Review Division Transfer Request     (HeadOfSanitary)
//   UC-HS-07   Set Division Staff Quota             (HeadOfSanitary)
//   UC-HS-08   Assign New Recruit to Division       (HeadOfSanitary)
//   UC-STAS-01 Receive and Execute Task             (All Staff)
//   UC-STAS-02 View Work Schedule                   (All Staff)
//   UC-STAS-03 Submit Division Transfer Request     (All Staff)
//   UC-STAS-04 Log Inventory Usage / Addition       (All Staff)
//   UC-IC-01   Submit Inspection Report             (InspectorCrew)
//   UC-DC-01   Maintain Disposal Handling Docs      (DisposalCrew)
//   UC-WC-01   Maintain Wastewater Treatment Docs   (WastewaterCrew)
//
// Data isolation: DataAnalyst role is NEVER allowed to call these commands.

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

// ── Shared sanitary role sets ─────────────────────────────────────────────────

const SANITARY_ALL_ROLES: [Role; 6] = [
    Role::HeadOfSanitary,
    Role::InspectorCrew,
    Role::DisposalCrew,
    Role::WastewaterCrew,
    Role::CleanupCrew,
    Role::TransportCrew,
];

// ── Payloads & Response Types ─────────────────────────────────────────────────

// -- Division --

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DivisionRow {
    pub id: Uuid,
    pub name: String,
    pub quota: i32,
}

// -- Staff roster --

#[derive(Debug, Serialize, FromRow)]
pub struct StaffRosterEntry {
    pub user_id: Uuid,
    pub full_name: String,
    pub role_name: String,
    pub division_id: Option<Uuid>,
    pub division_name: Option<String>,
    pub quarter: Option<String>,
}

// -- Shifts --

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SanitaryShift {
    pub id: Uuid,
    pub user_id: Uuid,
    pub staff_name: String,
    pub shift_start: DateTime<Utc>,
    pub shift_end: DateTime<Utc>,
    pub allocated_by: Uuid,
    pub quarter: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AllocateShiftPayload {
    pub user_id: Uuid,
    pub shift_start: String,
    pub shift_end: String,
    pub quarter: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteShiftPayload {
    pub shift_id: Uuid,
}

// -- Tasks --

#[derive(Debug, Serialize, FromRow)]
pub struct SanitaryTask {
    pub id: Uuid,
    pub assigned_by: Uuid,
    pub assigner_name: String,
    pub task_type: String,
    pub task_name: Option<String>,
    pub urgency: Option<String>,
    pub instructions: Option<String>,
    pub location: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub status: String,
    pub source_report_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AssignTaskPayload {
    pub task_type: String,
    pub task_name: Option<String>,
    pub urgency: Option<String>,
    pub instructions: Option<String>,
    pub location: Option<String>,
    pub due_date: Option<String>,
    pub assigned_to: Vec<Uuid>,
    pub source_report_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskStatusPayload {
    pub task_id: Uuid,
    pub status: String,
}

// -- Inventory --

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SanitaryInventoryItem {
    pub id: Uuid,
    pub item_name: String,
    pub category: Option<String>,
    pub quantity: i32,
    pub unit: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddInventoryPayload {
    pub item_name: String,
    pub category: Option<String>,
    pub quantity: i32,
    pub unit: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInventoryPayload {
    pub item_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct RemoveInventoryPayload {
    pub item_id: Uuid,
}

#[derive(Debug, Serialize, FromRow)]
pub struct InventoryLogEntry {
    pub id: Uuid,
    pub item_id: Uuid,
    pub item_name: String,
    pub logged_by: Uuid,
    pub logger_name: String,
    pub action: String,
    pub quantity: i32,
    pub purchase_note: Option<String>,
    pub logged_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LogInventoryPayload {
    pub item_id: Uuid,
    pub action: String,
    pub quantity: i32,
    pub purchase_note: Option<String>,
}

// -- Transfer requests --

#[derive(Debug, Serialize, FromRow)]
pub struct TransferRequestRow {
    pub id: Uuid,
    pub requested_by: Uuid,
    pub requester_name: String,
    pub from_division_name: String,
    pub to_division_name: String,
    pub reason: Option<String>,
    pub status: String,
    pub decided_by: Option<Uuid>,
    pub decided_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitTransferPayload {
    pub to_division_id: Uuid,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewTransferPayload {
    pub request_id: Uuid,
    pub decision: String,
}

// -- Recruit --

#[derive(Debug, Deserialize)]
pub struct AssignRecruitPayload {
    pub user_id: Uuid,
    pub division_id: Uuid,
    pub quarter: String,
}

// -- Inspection reports --

#[derive(Debug, Serialize, FromRow)]
pub struct InspectionReport {
    pub id: Uuid,
    pub reported_by: Uuid,
    pub reporter_name: String,
    pub report_date: NaiveDate,
    pub location: String,
    pub area_or_machine: String,
    pub findings: String,
    pub severity: String,
    pub recommendations: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitInspectionPayload {
    pub report_date: String,
    pub location: String,
    pub area_or_machine: String,
    pub findings: String,
    pub severity: String,
    pub recommendations: Option<String>,
}

// -- Disposal docs --

#[derive(Debug, Serialize, FromRow)]
pub struct DisposalDoc {
    pub id: Uuid,
    pub waste_category: String,
    pub procedure: String,
    pub safety_requirements: Option<String>,
    pub compliance_notes: Option<String>,
    pub revision_history: serde_json::Value,
    pub authored_by: Uuid,
    pub author_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDisposalDocPayload {
    pub waste_category: String,
    pub procedure: String,
    pub safety_requirements: Option<String>,
    pub compliance_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDisposalDocPayload {
    pub doc_id: Uuid,
    pub procedure: Option<String>,
    pub safety_requirements: Option<String>,
    pub compliance_notes: Option<String>,
    pub change_summary: String,
}

// -- Wastewater docs --

#[derive(Debug, Serialize, FromRow)]
pub struct WastewaterDoc {
    pub id: Uuid,
    pub treatment_type: String,
    pub steps: serde_json::Value,
    pub safety_requirements: Option<String>,
    pub compliance_notes: Option<String>,
    pub revision_history: serde_json::Value,
    pub authored_by: Uuid,
    pub author_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWastewaterDocPayload {
    pub treatment_type: String,
    pub steps: serde_json::Value,
    pub safety_requirements: Option<String>,
    pub compliance_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWastewaterDocPayload {
    pub doc_id: Uuid,
    pub steps: Option<serde_json::Value>,
    pub safety_requirements: Option<String>,
    pub compliance_notes: Option<String>,
    pub change_summary: String,
}

// -- Budget --

#[derive(Debug, Deserialize)]
pub struct SubmitBudgetRequestPayload {
    pub line_items: serde_json::Value,
    pub total_amount: f64,
    pub justification: Option<String>,
    pub invoice_bytes: Vec<u8>,
    pub invoice_filename: String,
    pub invoice_content_type: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct BudgetRequestSummary {
    pub id: Uuid,
    pub total_amount: Option<f64>,
    pub justification: Option<String>,
    pub status: String,
    pub line_items: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitExpenditurePayload {
    pub line_items: serde_json::Value,
    pub total_amount: f64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct ExpenditureReportSummary {
    pub id: Uuid,
    pub total_amount: Option<f64>,
    pub line_items: serde_json::Value,
    pub foul_play_flag: bool,
    pub foul_play_note: Option<String>,
    pub created_at: DateTime<Utc>,
}

// -- Quota --

#[derive(Debug, Deserialize)]
pub struct SetQuotaPayload {
    pub division_id: Uuid,
    pub quota: i32,
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-HS-07: Get Divisions (quota display)
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_get_divisions(
    state: State<'_, AppState>,
) -> Result<Vec<DivisionRow>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::HeadOfSanitary, Role::InspectorCrew, Role::DisposalCrew, Role::WastewaterCrew, Role::CleanupCrew, Role::TransportCrew]);

    let cache_key = "sanitary_divisions:quota";
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(cache_key).await {
            if let Ok(rows) = serde_json::from_str::<Vec<DivisionRow>>(&cached) {
                return Ok(rows);
            }
        }
    }

    let rows = sqlx::query_as::<_, DivisionRow>(
        "SELECT id, name, quota FROM sanitary_divisions ORDER BY name",
    )
    .fetch_all(&state.db_pool)
    .await?;

    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn
            .set_ex(cache_key, serde_json::to_string(&rows).unwrap_or_default(), 3600)
            .await;
    }

    Ok(rows)
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-HS-07: Set Division Staff Quota
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_set_division_quota(
    state: State<'_, AppState>,
    payload: SetQuotaPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    #[derive(FromRow)]
    struct OldQuota { quota: i32 }

    let before = sqlx::query_as::<_, OldQuota>(
        "SELECT quota FROM sanitary_divisions WHERE id = $1",
    )
    .bind(payload.division_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Division not found.".into()))?;

    sqlx::query("UPDATE sanitary_divisions SET quota = $2 WHERE id = $1")
        .bind(payload.division_id)
        .bind(payload.quota)
        .execute(&state.db_pool)
        .await?;

    invalidate_divisions_cache(&state).await;

    write_audit_log(
        &state.db_pool,
        "sanitary_divisions",
        payload.division_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({ "quota": before.quota })),
        Some(serde_json::json!({ "quota": payload.quota })),
    )
    .await?;

    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-HS-04: Get Staff Roster (with division history)
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_get_staff_roster(
    state: State<'_, AppState>,
) -> Result<Vec<StaffRosterEntry>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    let rows = sqlx::query_as::<_, StaffRosterEntry>(
        r#"
        SELECT u.id AS user_id, u.full_name, r.name AS role_name,
               sda.division_id, sd.name AS division_name, sda.quarter
        FROM users u
        JOIN roles r ON r.id = u.role_id
        LEFT JOIN LATERAL (
            SELECT division_id, quarter
            FROM staff_division_assignments
            WHERE user_id = u.id
            ORDER BY assigned_at DESC
            LIMIT 1
        ) sda ON TRUE
        LEFT JOIN sanitary_divisions sd ON sd.id = sda.division_id
        WHERE r.name IN ('HeadOfSanitary','InspectorCrew','DisposalCrew','WastewaterCrew','CleanupCrew','TransportCrew')
          AND u.deleted_at IS NULL AND u.is_active = TRUE
        ORDER BY u.full_name
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(rows)
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-HS-04: Allocate Staff Shifts (Quarterly Scheduling)
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_get_all_shifts(
    state: State<'_, AppState>,
) -> Result<Vec<SanitaryShift>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    let shifts = sqlx::query_as::<_, SanitaryShift>(
        r#"
        SELECT ss.id, ss.user_id, u.full_name AS staff_name,
               ss.shift_start, ss.shift_end, ss.allocated_by,
               ss.quarter, ss.created_at
        FROM sanitary_shifts ss
        JOIN users u ON u.id = ss.user_id
        WHERE ss.deleted_at IS NULL
        ORDER BY ss.shift_start ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(shifts)
}

#[tauri::command]
pub async fn san_allocate_shift(
    state: State<'_, AppState>,
    payload: AllocateShiftPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    let shift_start = DateTime::parse_from_rfc3339(&payload.shift_start)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| AppError::Internal("Invalid shift_start date format.".into()))?;

    let shift_end = DateTime::parse_from_rfc3339(&payload.shift_end)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| AppError::Internal("Invalid shift_end date format.".into()))?;

    if shift_end <= shift_start {
        return Err(AppError::Internal("shift_end must be after shift_start.".into()));
    }

    let (id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO sanitary_shifts (user_id, shift_start, shift_end, allocated_by, quarter)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(payload.user_id)
    .bind(shift_start)
    .bind(shift_end)
    .bind(user.id)
    .bind(&payload.quarter)
    .fetch_one(&state.db_pool)
    .await?;

    // Invalidate schedule cache for affected staff
    invalidate_schedule_cache(&state, payload.user_id).await;

    // Notify assigned staff
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        VALUES ($1, 'shift:allocated', $2::jsonb)
        "#,
    )
    .bind(payload.user_id)
    .bind(serde_json::json!({
        "shift_start": shift_start.to_rfc3339(),
        "shift_end": shift_end.to_rfc3339(),
        "quarter": payload.quarter,
        "allocated_by": user.full_name,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "sanitary_shifts",
        id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "user_id": payload.user_id,
            "shift_start": shift_start,
            "shift_end": shift_end,
            "quarter": payload.quarter,
        })),
    )
    .await?;

    Ok(id)
}

#[tauri::command]
pub async fn san_delete_shift(
    state: State<'_, AppState>,
    payload: DeleteShiftPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    #[derive(FromRow)]
    struct ShiftUser { user_id: Uuid }

    let row = sqlx::query_as::<_, ShiftUser>(
        "SELECT user_id FROM sanitary_shifts WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.shift_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Shift not found.".into()))?;

    sqlx::query(
        "UPDATE sanitary_shifts SET deleted_at = NOW(), deleted_by = $2 WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.shift_id)
    .bind(user.id)
    .execute(&state.db_pool)
    .await?;

    invalidate_schedule_cache(&state, row.user_id).await;

    write_audit_log(
        &state.db_pool,
        "sanitary_shifts",
        payload.shift_id,
        AuditOperation::Delete,
        user.id,
        None,
        None,
    )
    .await?;

    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-HS-05: Assign Task to Crew Member(s)
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_assign_task(
    state: State<'_, AppState>,
    payload: AssignTaskPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    let valid_types = ["cleaning", "inspection", "disposal", "transport", "other"];
    if !valid_types.contains(&payload.task_type.as_str()) {
        return Err(AppError::Internal(
            "task_type must be one of: cleaning, inspection, disposal, transport, other.".into(),
        ));
    }

    let due_date = payload.due_date.as_deref().and_then(|d| {
        DateTime::parse_from_rfc3339(d)
            .map(|dt| dt.with_timezone(&Utc))
            .ok()
    });

    let (task_id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO sanitary_tasks
            (assigned_by, task_type, task_name, urgency, instructions, location, due_date, source_report_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.task_type)
    .bind(&payload.task_name)
    .bind(&payload.urgency)
    .bind(&payload.instructions)
    .bind(&payload.location)
    .bind(due_date)
    .bind(payload.source_report_id)
    .fetch_one(&state.db_pool)
    .await?;

    // Insert multi-assignments
    for uid in &payload.assigned_to {
        sqlx::query(
            "INSERT INTO sanitary_task_assignments (task_id, user_id) VALUES ($1, $2)",
        )
        .bind(task_id)
        .bind(uid)
        .execute(&state.db_pool)
        .await?;

        // Notify each assigned staff
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'task:assigned', $2::jsonb)
            "#,
        )
        .bind(uid)
        .bind(serde_json::json!({
            "task_id": task_id,
            "task_name": payload.task_name,
            "task_type": payload.task_type,
            "urgency": payload.urgency,
            "assigned_by": user.full_name,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    write_audit_log(
        &state.db_pool,
        "sanitary_tasks",
        task_id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "task_type": payload.task_type,
            "task_name": payload.task_name,
            "assigned_to": payload.assigned_to,
        })),
    )
    .await?;

    Ok(task_id)
}

#[tauri::command]
pub async fn san_get_all_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<SanitaryTask>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    let tasks = sqlx::query_as::<_, SanitaryTask>(
        r#"
        SELECT st.id, st.assigned_by, u.full_name AS assigner_name,
               st.task_type, st.task_name, st.urgency, st.instructions,
               st.location, st.due_date, st.status, st.source_report_id,
               st.created_at
        FROM sanitary_tasks st
        JOIN users u ON u.id = st.assigned_by
        WHERE st.deleted_at IS NULL
        ORDER BY st.created_at DESC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(tasks)
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-HS-06: Review Division Transfer Request
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_get_transfer_requests(
    state: State<'_, AppState>,
) -> Result<Vec<TransferRequestRow>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    let rows = sqlx::query_as::<_, TransferRequestRow>(
        r#"
        SELECT dtr.id, dtr.requested_by, u.full_name AS requester_name,
               fd.name AS from_division_name, td.name AS to_division_name,
               dtr.reason, dtr.status, dtr.decided_by, dtr.decided_at, dtr.created_at
        FROM division_transfer_requests dtr
        JOIN users u ON u.id = dtr.requested_by
        JOIN sanitary_divisions fd ON fd.id = dtr.from_division_id
        JOIN sanitary_divisions td ON td.id = dtr.to_division_id
        WHERE dtr.status = 'pending'
        ORDER BY dtr.created_at ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(rows)
}

#[tauri::command]
pub async fn san_review_transfer_request(
    state: State<'_, AppState>,
    payload: ReviewTransferPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    if payload.decision != "approved" && payload.decision != "rejected" {
        return Err(AppError::Internal("Decision must be 'approved' or 'rejected'.".into()));
    }

    #[derive(FromRow)]
    struct TransferInfo {
        requested_by: Uuid,
        to_division_id: Uuid,
        status: String,
    }

    let transfer = sqlx::query_as::<_, TransferInfo>(
        "SELECT requested_by, to_division_id, status FROM division_transfer_requests WHERE id = $1",
    )
    .bind(payload.request_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Transfer request not found.".into()))?;

    if transfer.status != "pending" {
        return Err(AppError::Internal("Transfer request already decided.".into()));
    }

    // If approving, check quota
    if payload.decision == "approved" {
        #[derive(FromRow)]
        struct QuotaCheck { quota: i32, current_count: i64 }

        let check = sqlx::query_as::<_, QuotaCheck>(
            r#"
            SELECT sd.quota,
                   (SELECT COUNT(*) FROM staff_division_assignments sda2
                    WHERE sda2.division_id = sd.id
                      AND sda2.quarter = (SELECT MAX(quarter) FROM staff_division_assignments)) AS current_count
            FROM sanitary_divisions sd
            WHERE sd.id = $1
            "#,
        )
        .bind(transfer.to_division_id)
        .fetch_one(&state.db_pool)
        .await?;

        if check.current_count >= check.quota as i64 {
            return Err(AppError::Internal(
                "Target division has reached its staff quota.".into(),
            ));
        }
    }

    sqlx::query(
        r#"
        UPDATE division_transfer_requests
        SET status = $2, decided_by = $3, decided_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(payload.request_id)
    .bind(&payload.decision)
    .bind(user.id)
    .execute(&state.db_pool)
    .await?;

    // Notify the requesting staff
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        VALUES ($1, 'transfer:decided', $2::jsonb)
        "#,
    )
    .bind(transfer.requested_by)
    .bind(serde_json::json!({
        "request_id": payload.request_id,
        "decision": payload.decision,
        "decided_by": user.full_name,
    }))
    .execute(&state.db_pool)
    .await?;

    // If approved, invalidate division cache for the user
    if payload.decision == "approved" {
        invalidate_staff_division_cache(&state, transfer.requested_by).await;
    }

    write_audit_log(
        &state.db_pool,
        "division_transfer_requests",
        payload.request_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({ "status": "pending" })),
        Some(serde_json::json!({ "status": payload.decision })),
    )
    .await?;

    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-HS-08: Assign New Recruit to Division
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_assign_recruit(
    state: State<'_, AppState>,
    payload: AssignRecruitPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    let (id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO staff_division_assignments (user_id, division_id, quarter, assigned_by)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(payload.user_id)
    .bind(payload.division_id)
    .bind(&payload.quarter)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    invalidate_staff_division_cache(&state, payload.user_id).await;

    write_audit_log(
        &state.db_pool,
        "staff_division_assignments",
        id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "user_id": payload.user_id,
            "division_id": payload.division_id,
            "quarter": payload.quarter,
        })),
    )
    .await?;

    Ok(id)
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-HS-03: Manage Department Inventory (Macro — Head only)
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_get_inventory(
    state: State<'_, AppState>,
) -> Result<Vec<SanitaryInventoryItem>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::HeadOfSanitary, Role::InspectorCrew, Role::DisposalCrew, Role::WastewaterCrew, Role::CleanupCrew, Role::TransportCrew]);

    let cache_key = "sanitary_inventory:list";
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(cache_key).await {
            if let Ok(items) = serde_json::from_str::<Vec<SanitaryInventoryItem>>(&cached) {
                return Ok(items);
            }
        }
    }

    let items = sqlx::query_as::<_, SanitaryInventoryItem>(
        r#"
        SELECT id, item_name, category, quantity, unit, updated_at
        FROM sanitary_inventory
        WHERE deleted_at IS NULL
        ORDER BY item_name
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn
            .set_ex(cache_key, serde_json::to_string(&items).unwrap_or_default(), 900)
            .await;
    }

    Ok(items)
}

#[tauri::command]
pub async fn san_add_inventory(
    state: State<'_, AppState>,
    payload: AddInventoryPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    let (id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO sanitary_inventory (item_name, category, quantity, unit, last_updated_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(&payload.item_name)
    .bind(&payload.category)
    .bind(payload.quantity)
    .bind(&payload.unit)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    invalidate_inventory_cache(&state).await;

    write_audit_log(
        &state.db_pool,
        "sanitary_inventory",
        id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "item_name": payload.item_name,
            "category": payload.category,
            "quantity": payload.quantity,
        })),
    )
    .await?;

    Ok(id)
}

#[tauri::command]
pub async fn san_update_inventory(
    state: State<'_, AppState>,
    payload: UpdateInventoryPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    #[derive(FromRow)]
    struct InvRow { quantity: i32 }

    let before = sqlx::query_as::<_, InvRow>(
        "SELECT quantity FROM sanitary_inventory WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.item_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Inventory item not found.".into()))?;

    sqlx::query(
        r#"
        UPDATE sanitary_inventory
        SET quantity = $2, last_updated_by = $3, updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.item_id)
    .bind(payload.quantity)
    .bind(user.id)
    .execute(&state.db_pool)
    .await?;

    invalidate_inventory_cache(&state).await;

    write_audit_log(
        &state.db_pool,
        "sanitary_inventory",
        payload.item_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({ "quantity": before.quantity })),
        Some(serde_json::json!({ "quantity": payload.quantity })),
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn san_remove_inventory(
    state: State<'_, AppState>,
    payload: RemoveInventoryPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    sqlx::query(
        "UPDATE sanitary_inventory SET deleted_at = NOW(), deleted_by = $2 WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.item_id)
    .bind(user.id)
    .execute(&state.db_pool)
    .await?;

    invalidate_inventory_cache(&state).await;

    write_audit_log(
        &state.db_pool,
        "sanitary_inventory",
        payload.item_id,
        AuditOperation::Delete,
        user.id,
        None,
        None,
    )
    .await?;

    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-STAS-01: Receive and Execute Task (All Staff)
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_get_my_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<SanitaryTask>, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary, Role::InspectorCrew, Role::DisposalCrew, Role::WastewaterCrew, Role::CleanupCrew, Role::TransportCrew]);

    let tasks = sqlx::query_as::<_, SanitaryTask>(
        r#"
        SELECT st.id, st.assigned_by, u.full_name AS assigner_name,
               st.task_type, st.task_name, st.urgency, st.instructions,
               st.location, st.due_date, st.status, st.source_report_id,
               st.created_at
        FROM sanitary_tasks st
        JOIN sanitary_task_assignments sta ON sta.task_id = st.id
        JOIN users u ON u.id = st.assigned_by
        WHERE sta.user_id = $1 AND st.deleted_at IS NULL
        ORDER BY st.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(tasks)
}

#[tauri::command]
pub async fn san_update_task_status(
    state: State<'_, AppState>,
    payload: UpdateTaskStatusPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary, Role::InspectorCrew, Role::DisposalCrew, Role::WastewaterCrew, Role::CleanupCrew, Role::TransportCrew]);

    let valid_statuses = ["pending", "in_progress", "completed"];
    if !valid_statuses.contains(&payload.status.as_str()) {
        return Err(AppError::Internal(
            "status must be one of: pending, in_progress, completed.".into(),
        ));
    }

    // Verify user is assigned to this task (or is HeadOfSanitary)
    if user.role != Role::HeadOfSanitary && user.role != Role::Administrator {
        let assigned: Option<(Uuid,)> = sqlx::query_as(
            "SELECT task_id FROM sanitary_task_assignments WHERE task_id = $1 AND user_id = $2",
        )
        .bind(payload.task_id)
        .bind(user.id)
        .fetch_optional(&state.db_pool)
        .await?;

        if assigned.is_none() {
            return Err(AppError::Forbidden);
        }
    }

    #[derive(FromRow)]
    struct OldStatus { status: String }

    let before = sqlx::query_as::<_, OldStatus>(
        "SELECT status FROM sanitary_tasks WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.task_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Task not found.".into()))?;

    sqlx::query(
        "UPDATE sanitary_tasks SET status = $2 WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.task_id)
    .bind(&payload.status)
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "sanitary_tasks",
        payload.task_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({ "status": before.status })),
        Some(serde_json::json!({ "status": payload.status })),
    )
    .await?;

    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-STAS-02: View Work Schedule (All Staff)
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_get_my_schedule(
    state: State<'_, AppState>,
) -> Result<Vec<SanitaryShift>, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary, Role::InspectorCrew, Role::DisposalCrew, Role::WastewaterCrew, Role::CleanupCrew, Role::TransportCrew]);

    let cache_key = format!("sanitary_schedule:{}", user.id);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(shifts) = serde_json::from_str::<Vec<SanitaryShift>>(&cached) {
                return Ok(shifts);
            }
        }
    }

    let shifts = sqlx::query_as::<_, SanitaryShift>(
        r#"
        SELECT ss.id, ss.user_id, u.full_name AS staff_name,
               ss.shift_start, ss.shift_end, ss.allocated_by,
               ss.quarter, ss.created_at
        FROM sanitary_shifts ss
        JOIN users u ON u.id = ss.user_id
        WHERE ss.user_id = $1 AND ss.deleted_at IS NULL
        ORDER BY ss.shift_start ASC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn
            .set_ex(&cache_key, serde_json::to_string(&shifts).unwrap_or_default(), 3600)
            .await;
    }

    Ok(shifts)
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-STAS-03: Submit Division Transfer Request (All Staff)
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_submit_transfer_request(
    state: State<'_, AppState>,
    payload: SubmitTransferPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary, Role::InspectorCrew, Role::DisposalCrew, Role::WastewaterCrew, Role::CleanupCrew, Role::TransportCrew]);

    // Find user's current division from the latest assignment
    #[derive(FromRow)]
    struct CurrentDiv { division_id: Uuid }

    let current = sqlx::query_as::<_, CurrentDiv>(
        r#"
        SELECT division_id FROM staff_division_assignments
        WHERE user_id = $1
        ORDER BY assigned_at DESC
        LIMIT 1
        "#,
    )
    .bind(user.id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("You have no division assignment on record.".into()))?;

    if current.division_id == payload.to_division_id {
        return Err(AppError::Internal("Cannot transfer to your current division.".into()));
    }

    let (id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO division_transfer_requests
            (requested_by, from_division_id, to_division_id, reason)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(current.division_id)
    .bind(payload.to_division_id)
    .bind(&payload.reason)
    .fetch_one(&state.db_pool)
    .await?;

    // Notify Head of Sanitary
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        SELECT u.id, 'transfer:requested', $1::jsonb
        FROM users u JOIN roles r ON r.id = u.role_id
        WHERE r.name = 'HeadOfSanitary'
          AND u.deleted_at IS NULL AND u.is_active = TRUE
        "#,
    )
    .bind(serde_json::json!({
        "request_id": id,
        "requester": user.full_name,
        "reason": payload.reason,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "division_transfer_requests",
        id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "from_division_id": current.division_id,
            "to_division_id": payload.to_division_id,
            "reason": payload.reason,
        })),
    )
    .await?;

    Ok(id)
}

#[tauri::command]
pub async fn san_get_my_transfers(
    state: State<'_, AppState>,
) -> Result<Vec<TransferRequestRow>, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary, Role::InspectorCrew, Role::DisposalCrew, Role::WastewaterCrew, Role::CleanupCrew, Role::TransportCrew]);

    let rows = sqlx::query_as::<_, TransferRequestRow>(
        r#"
        SELECT dtr.id, dtr.requested_by, u.full_name AS requester_name,
               fd.name AS from_division_name, td.name AS to_division_name,
               dtr.reason, dtr.status, dtr.decided_by, dtr.decided_at, dtr.created_at
        FROM division_transfer_requests dtr
        JOIN users u ON u.id = dtr.requested_by
        JOIN sanitary_divisions fd ON fd.id = dtr.from_division_id
        JOIN sanitary_divisions td ON td.id = dtr.to_division_id
        WHERE dtr.requested_by = $1
        ORDER BY dtr.created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(rows)
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-STAS-04: Log Inventory Usage / Addition (All Staff)
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_log_inventory_action(
    state: State<'_, AppState>,
    payload: LogInventoryPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary, Role::InspectorCrew, Role::DisposalCrew, Role::WastewaterCrew, Role::CleanupCrew, Role::TransportCrew]);

    if payload.action != "add" && payload.action != "remove" {
        return Err(AppError::Internal("action must be 'add' or 'remove'.".into()));
    }

    // Verify item exists
    #[derive(FromRow)]
    struct InvCheck { quantity: i32 }

    let item = sqlx::query_as::<_, InvCheck>(
        "SELECT quantity FROM sanitary_inventory WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.item_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Inventory item not found.".into()))?;

    // Update quantity
    let new_qty = if payload.action == "add" {
        item.quantity + payload.quantity
    } else {
        let q = item.quantity - payload.quantity;
        if q < 0 {
            return Err(AppError::Internal("Cannot remove more than current stock.".into()));
        }
        q
    };

    sqlx::query(
        "UPDATE sanitary_inventory SET quantity = $2, last_updated_by = $3, updated_at = NOW() WHERE id = $1",
    )
    .bind(payload.item_id)
    .bind(new_qty)
    .bind(user.id)
    .execute(&state.db_pool)
    .await?;

    let (log_id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO sanitary_inventory_logs (item_id, logged_by, action, quantity, purchase_note)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(payload.item_id)
    .bind(user.id)
    .bind(&payload.action)
    .bind(payload.quantity)
    .bind(&payload.purchase_note)
    .fetch_one(&state.db_pool)
    .await?;

    invalidate_inventory_cache(&state).await;

    write_audit_log(
        &state.db_pool,
        "sanitary_inventory_logs",
        log_id,
        AuditOperation::Create,
        user.id,
        Some(serde_json::json!({ "quantity": item.quantity })),
        Some(serde_json::json!({ "quantity": new_qty, "action": payload.action, "delta": payload.quantity })),
    )
    .await?;

    Ok(log_id)
}

#[tauri::command]
pub async fn san_get_inventory_logs(
    state: State<'_, AppState>,
) -> Result<Vec<InventoryLogEntry>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::HeadOfSanitary, Role::InspectorCrew, Role::DisposalCrew, Role::WastewaterCrew, Role::CleanupCrew, Role::TransportCrew]);

    let logs = sqlx::query_as::<_, InventoryLogEntry>(
        r#"
        SELECT sil.id, sil.item_id, si.item_name,
               sil.logged_by, u.full_name AS logger_name,
               sil.action, sil.quantity, sil.purchase_note, sil.logged_at
        FROM sanitary_inventory_logs sil
        JOIN sanitary_inventory si ON si.id = sil.item_id
        JOIN users u ON u.id = sil.logged_by
        ORDER BY sil.logged_at DESC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(logs)
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-IC-01: Submit Inspection Report (InspectorCrew)
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_submit_inspection_report(
    state: State<'_, AppState>,
    payload: SubmitInspectionPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::InspectorCrew, Role::HeadOfSanitary]);

    let valid_severity = ["low", "medium", "high", "critical"];
    if !valid_severity.contains(&payload.severity.as_str()) {
        return Err(AppError::Internal(
            "severity must be one of: low, medium, high, critical.".into(),
        ));
    }

    let report_date = NaiveDate::parse_from_str(&payload.report_date, "%Y-%m-%d")
        .map_err(|_| AppError::Internal("Invalid report_date format. Expected YYYY-MM-DD.".into()))?;

    let (id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO inspection_reports
            (reported_by, report_date, location, area_or_machine, findings, severity, recommendations)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(report_date)
    .bind(&payload.location)
    .bind(&payload.area_or_machine)
    .bind(&payload.findings)
    .bind(&payload.severity)
    .bind(&payload.recommendations)
    .fetch_one(&state.db_pool)
    .await?;

    // Notify Head of Sanitary
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        SELECT u.id, 'inspection_report:received', $1::jsonb
        FROM users u JOIN roles r ON r.id = u.role_id
        WHERE r.name = 'HeadOfSanitary'
          AND u.deleted_at IS NULL AND u.is_active = TRUE
        "#,
    )
    .bind(serde_json::json!({
        "report_id": id,
        "severity": payload.severity,
        "location": payload.location,
        "reported_by": user.full_name,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "inspection_reports",
        id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "severity": payload.severity,
            "location": payload.location,
        })),
    )
    .await?;

    Ok(id)
}

#[tauri::command]
pub async fn san_get_inspection_reports(
    state: State<'_, AppState>,
) -> Result<Vec<InspectionReport>, AppError> {
    let user = crate::require_auth_any!(state, [Role::InspectorCrew, Role::HeadOfSanitary]);

    let reports = if user.role == Role::HeadOfSanitary || user.role == Role::Administrator {
        sqlx::query_as::<_, InspectionReport>(
            r#"
            SELECT ir.id, ir.reported_by, u.full_name AS reporter_name,
                   ir.report_date, ir.location, ir.area_or_machine,
                   ir.findings, ir.severity, ir.recommendations, ir.created_at
            FROM inspection_reports ir
            JOIN users u ON u.id = ir.reported_by
            WHERE ir.deleted_at IS NULL
            ORDER BY ir.created_at DESC
            "#,
        )
        .fetch_all(&state.db_pool)
        .await?
    } else {
        sqlx::query_as::<_, InspectionReport>(
            r#"
            SELECT ir.id, ir.reported_by, u.full_name AS reporter_name,
                   ir.report_date, ir.location, ir.area_or_machine,
                   ir.findings, ir.severity, ir.recommendations, ir.created_at
            FROM inspection_reports ir
            JOIN users u ON u.id = ir.reported_by
            WHERE ir.reported_by = $1 AND ir.deleted_at IS NULL
            ORDER BY ir.created_at DESC
            "#,
        )
        .bind(user.id)
        .fetch_all(&state.db_pool)
        .await?
    };

    Ok(reports)
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-DC-01: Maintain Disposal Handling Documentation (DisposalCrew)
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_create_disposal_doc(
    state: State<'_, AppState>,
    payload: CreateDisposalDocPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::DisposalCrew, Role::HeadOfSanitary]);

    let initial_revision = serde_json::json!([{
        "revision": 1,
        "changed_by": user.id.to_string(),
        "changed_at": Utc::now().to_rfc3339(),
        "summary": "Initial creation"
    }]);

    let (id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO disposal_handling_docs
            (waste_category, procedure, safety_requirements, compliance_notes, revision_history, authored_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(&payload.waste_category)
    .bind(&payload.procedure)
    .bind(&payload.safety_requirements)
    .bind(&payload.compliance_notes)
    .bind(&initial_revision)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "disposal_handling_docs",
        id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "waste_category": payload.waste_category })),
    )
    .await?;

    Ok(id)
}

#[tauri::command]
pub async fn san_update_disposal_doc(
    state: State<'_, AppState>,
    payload: UpdateDisposalDocPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::DisposalCrew, Role::HeadOfSanitary]);

    // Get current revision count
    #[derive(FromRow)]
    struct RevInfo { revision_history: serde_json::Value }

    let info = sqlx::query_as::<_, RevInfo>(
        "SELECT revision_history FROM disposal_handling_docs WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.doc_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Disposal doc not found.".into()))?;

    let rev_count = info.revision_history.as_array().map(|a| a.len()).unwrap_or(0);
    let new_revision = serde_json::json!([{
        "revision": rev_count + 1,
        "changed_by": user.id.to_string(),
        "changed_at": Utc::now().to_rfc3339(),
        "summary": payload.change_summary
    }]);

    // Build dynamic SET clause
    let mut sets = vec!["revision_history = revision_history || $1::jsonb"];
    let mut param_idx = 2u32; // next bind index
    let mut bind_procedure = false;
    let mut bind_safety = false;
    let mut bind_compliance = false;

    if payload.procedure.is_some() {
        sets.push("procedure = $procedure_placeholder");
        bind_procedure = true;
    }
    if payload.safety_requirements.is_some() {
        sets.push("safety_requirements = $safety_placeholder");
        bind_safety = true;
    }
    if payload.compliance_notes.is_some() {
        sets.push("compliance_notes = $compliance_placeholder");
        bind_compliance = true;
    }

    // Use positional approach: build query with numbered params
    let mut query_str = String::from("UPDATE disposal_handling_docs SET revision_history = revision_history || $1::jsonb");
    if bind_procedure {
        param_idx += 0; // we'll number them sequentially
        query_str.push_str(&format!(", procedure = ${}", param_idx));
        param_idx += 1;
    }
    if bind_safety {
        query_str.push_str(&format!(", safety_requirements = ${}", param_idx));
        param_idx += 1;
    }
    if bind_compliance {
        query_str.push_str(&format!(", compliance_notes = ${}", param_idx));
        param_idx += 1;
    }
    query_str.push_str(&format!(" WHERE id = ${} AND deleted_at IS NULL", param_idx));

    let mut q = sqlx::query(&query_str).bind(&new_revision);
    if let Some(ref p) = payload.procedure {
        q = q.bind(p);
    }
    if let Some(ref s) = payload.safety_requirements {
        q = q.bind(s);
    }
    if let Some(ref c) = payload.compliance_notes {
        q = q.bind(c);
    }
    q = q.bind(payload.doc_id);

    q.execute(&state.db_pool).await?;

    write_audit_log(
        &state.db_pool,
        "disposal_handling_docs",
        payload.doc_id,
        AuditOperation::Update,
        user.id,
        None,
        Some(serde_json::json!({
            "revision": rev_count + 1,
            "change_summary": payload.change_summary,
        })),
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn san_get_disposal_docs(
    state: State<'_, AppState>,
    filter: Option<String>,
) -> Result<Vec<DisposalDoc>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::DisposalCrew, Role::HeadOfSanitary]);

    let docs = if let Some(ref cat) = filter {
        sqlx::query_as::<_, DisposalDoc>(
            r#"
            SELECT d.id, d.waste_category, d.procedure, d.safety_requirements,
                   d.compliance_notes, d.revision_history, d.authored_by,
                   u.full_name AS author_name, d.created_at
            FROM disposal_handling_docs d
            JOIN users u ON u.id = d.authored_by
            WHERE d.deleted_at IS NULL AND d.waste_category ILIKE $1
            ORDER BY d.created_at DESC
            "#,
        )
        .bind(format!("%{}%", cat))
        .fetch_all(&state.db_pool)
        .await?
    } else {
        sqlx::query_as::<_, DisposalDoc>(
            r#"
            SELECT d.id, d.waste_category, d.procedure, d.safety_requirements,
                   d.compliance_notes, d.revision_history, d.authored_by,
                   u.full_name AS author_name, d.created_at
            FROM disposal_handling_docs d
            JOIN users u ON u.id = d.authored_by
            WHERE d.deleted_at IS NULL
            ORDER BY d.created_at DESC
            "#,
        )
        .fetch_all(&state.db_pool)
        .await?
    };

    Ok(docs)
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-WC-01: Maintain Wastewater Treatment Documentation (WastewaterCrew)
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_create_wastewater_doc(
    state: State<'_, AppState>,
    payload: CreateWastewaterDocPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::WastewaterCrew, Role::HeadOfSanitary]);

    let initial_revision = serde_json::json!([{
        "revision": 1,
        "changed_by": user.id.to_string(),
        "changed_at": Utc::now().to_rfc3339(),
        "summary": "Initial creation"
    }]);

    let (id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO wastewater_treatment_docs
            (treatment_type, steps, safety_requirements, compliance_notes, revision_history, authored_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(&payload.treatment_type)
    .bind(&payload.steps)
    .bind(&payload.safety_requirements)
    .bind(&payload.compliance_notes)
    .bind(&initial_revision)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "wastewater_treatment_docs",
        id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "treatment_type": payload.treatment_type })),
    )
    .await?;

    Ok(id)
}

#[tauri::command]
pub async fn san_update_wastewater_doc(
    state: State<'_, AppState>,
    payload: UpdateWastewaterDocPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::WastewaterCrew, Role::HeadOfSanitary]);

    #[derive(FromRow)]
    struct RevInfo { revision_history: serde_json::Value }

    let info = sqlx::query_as::<_, RevInfo>(
        "SELECT revision_history FROM wastewater_treatment_docs WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.doc_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Wastewater doc not found.".into()))?;

    let rev_count = info.revision_history.as_array().map(|a| a.len()).unwrap_or(0);
    let new_revision = serde_json::json!([{
        "revision": rev_count + 1,
        "changed_by": user.id.to_string(),
        "changed_at": Utc::now().to_rfc3339(),
        "summary": payload.change_summary
    }]);

    let mut param_idx = 2u32;
    let mut query_str = String::from("UPDATE wastewater_treatment_docs SET revision_history = revision_history || $1::jsonb");

    let bind_steps = payload.steps.is_some();
    let bind_safety = payload.safety_requirements.is_some();
    let bind_compliance = payload.compliance_notes.is_some();

    if bind_steps {
        query_str.push_str(&format!(", steps = ${}", param_idx));
        param_idx += 1;
    }
    if bind_safety {
        query_str.push_str(&format!(", safety_requirements = ${}", param_idx));
        param_idx += 1;
    }
    if bind_compliance {
        query_str.push_str(&format!(", compliance_notes = ${}", param_idx));
        param_idx += 1;
    }
    query_str.push_str(&format!(" WHERE id = ${} AND deleted_at IS NULL", param_idx));

    let mut q = sqlx::query(&query_str).bind(&new_revision);
    if let Some(ref s) = payload.steps {
        q = q.bind(s);
    }
    if let Some(ref s) = payload.safety_requirements {
        q = q.bind(s);
    }
    if let Some(ref c) = payload.compliance_notes {
        q = q.bind(c);
    }
    q = q.bind(payload.doc_id);

    q.execute(&state.db_pool).await?;

    write_audit_log(
        &state.db_pool,
        "wastewater_treatment_docs",
        payload.doc_id,
        AuditOperation::Update,
        user.id,
        None,
        Some(serde_json::json!({
            "revision": rev_count + 1,
            "change_summary": payload.change_summary,
        })),
    )
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn san_get_wastewater_docs(
    state: State<'_, AppState>,
    filter: Option<String>,
) -> Result<Vec<WastewaterDoc>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::WastewaterCrew, Role::HeadOfSanitary]);

    let docs = if let Some(ref t) = filter {
        sqlx::query_as::<_, WastewaterDoc>(
            r#"
            SELECT w.id, w.treatment_type, w.steps, w.safety_requirements,
                   w.compliance_notes, w.revision_history, w.authored_by,
                   u.full_name AS author_name, w.created_at
            FROM wastewater_treatment_docs w
            JOIN users u ON u.id = w.authored_by
            WHERE w.deleted_at IS NULL AND w.treatment_type ILIKE $1
            ORDER BY w.created_at DESC
            "#,
        )
        .bind(format!("%{}%", t))
        .fetch_all(&state.db_pool)
        .await?
    } else {
        sqlx::query_as::<_, WastewaterDoc>(
            r#"
            SELECT w.id, w.treatment_type, w.steps, w.safety_requirements,
                   w.compliance_notes, w.revision_history, w.authored_by,
                   u.full_name AS author_name, w.created_at
            FROM wastewater_treatment_docs w
            JOIN users u ON u.id = w.authored_by
            WHERE w.deleted_at IS NULL
            ORDER BY w.created_at DESC
            "#,
        )
        .fetch_all(&state.db_pool)
        .await?
    };

    Ok(docs)
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-HS-01: Submit Budget Funding Request (HeadOfSanitary)
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_submit_budget_request(
    state: State<'_, AppState>,
    payload: SubmitBudgetRequestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    let request_id = Uuid::new_v4();
    let storage_path = format!(
        "sanitary/budget-invoices/{}/{}",
        request_id, payload.invoice_filename
    );

    // Upload invoice to Supabase Storage
    let upload_url = format!(
        "{}/object/rusa-files/{}",
        state.supabase_storage_url, storage_path
    );

    let client = reqwest::Client::new();
    let resp = client
        .post(&upload_url)
        .header("Authorization", format!("Bearer {}", state.supabase_service_jwt))
        .header("Content-Type", &payload.invoice_content_type)
        .body(payload.invoice_bytes)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Storage upload failed: {}", e)))?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Internal(format!(
            "Storage upload returned {}: {}",
            body.len(),
            body
        )));
    }

    // Insert into sanitary_budget_requests
    let (id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO sanitary_budget_requests
            (id, submitted_by, line_items, total_amount, invoice_storage_path, justification)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(request_id)
    .bind(user.id)
    .bind(&payload.line_items)
    .bind(payload.total_amount)
    .bind(&storage_path)
    .bind(&payload.justification)
    .fetch_one(&state.db_pool)
    .await?;

    // Create requests envelope row
    let req_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO requests (type, requester_id, title, description, payload, invoice_storage_path)
        VALUES ('budget', $1, $2, $3, $4::jsonb, $5)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind("Sanitary Department Budget Request")
    .bind(&payload.justification)
    .bind(serde_json::json!({
        "department": "sanitary",
        "sanitary_budget_request_id": id,
        "line_items": payload.line_items,
        "total_amount": payload.total_amount,
    }))
    .bind(&storage_path)
    .fetch_one(&state.db_pool)
    .await?;

    // Create vote session
    let (vote_id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO vote_sessions (request_id, topic, context, status, opens_at, created_by)
        VALUES ($1, $2, $3, 'open', NOW(), $4)
        RETURNING id
        "#,
    )
    .bind(req_id.0)
    .bind("Sanitary Budget Request")
    .bind(&payload.justification)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Link vote session back
    sqlx::query(
        "UPDATE sanitary_budget_requests SET vote_session_id = $2 WHERE id = $1",
    )
    .bind(id)
    .bind(vote_id)
    .execute(&state.db_pool)
    .await?;

    sqlx::query(
        "UPDATE requests SET status = 'in_vote', updated_at = NOW() WHERE id = $1",
    )
    .bind(req_id.0)
    .execute(&state.db_pool)
    .await?;

    // Notify all directors for voting
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        SELECT u.id, 'vote:new', $1::jsonb
        FROM users u JOIN roles r ON r.id = u.role_id
        WHERE r.name IN (
            'GeneralDirector','TheDirector','TheAccountant','TheLibrarian',
            'TheNomad','TheArtificer','TheObserver','TheWanderer',
            'TheTaskmaster','TheGuardian','TheStatistician','TheCoordinator',
            'TheOverseer','TheAnchorman','Administrator'
        ) AND u.deleted_at IS NULL AND u.is_active = TRUE
        "#,
    )
    .bind(serde_json::json!({
        "session_id": vote_id,
        "topic": "Sanitary Budget Request",
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "sanitary_budget_requests",
        id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "total_amount": payload.total_amount,
            "vote_session_id": vote_id,
        })),
    )
    .await?;

    Ok(id)
}

#[tauri::command]
pub async fn san_get_budget_requests(
    state: State<'_, AppState>,
) -> Result<Vec<BudgetRequestSummary>, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    let reqs = sqlx::query_as::<_, BudgetRequestSummary>(
        r#"
        SELECT id, total_amount::float8 AS total_amount, justification, status, line_items, created_at
        FROM sanitary_budget_requests
        WHERE submitted_by = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(reqs)
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-HS-02: Submit Budget Expenditure Report (HeadOfSanitary)
// ══════════════════════════════════════════════════════════════════════════════

#[tauri::command]
pub async fn san_submit_expenditure_report(
    state: State<'_, AppState>,
    payload: SubmitExpenditurePayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    let (id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO sanitary_expenditure_reports
            (submitted_by, line_items, total_amount, submitted_to_accountant_at)
        VALUES ($1, $2, $3, NOW())
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.line_items)
    .bind(payload.total_amount)
    .fetch_one(&state.db_pool)
    .await?;

    // Create requests envelope
    sqlx::query(
        r#"
        INSERT INTO requests (type, requester_id, title, description, payload, status)
        VALUES ('budget', $1, $2, 'Sanitary expenditure report', $3::jsonb, 'pending')
        "#,
    )
    .bind(user.id)
    .bind("Sanitary Expenditure Report")
    .bind(serde_json::json!({
        "department": "sanitary",
        "report_type": "expenditure",
        "sanitary_expenditure_report_id": id,
        "line_items": payload.line_items,
        "total_amount": payload.total_amount,
    }))
    .execute(&state.db_pool)
    .await?;

    // Notify TheAccountant
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        SELECT u.id, 'budget:expenditure_report', $1::jsonb
        FROM users u JOIN roles r ON r.id = u.role_id
        WHERE r.name = 'TheAccountant'
          AND u.deleted_at IS NULL AND u.is_active = TRUE
        "#,
    )
    .bind(serde_json::json!({
        "report_id": id,
        "department": "sanitary",
        "total_amount": payload.total_amount,
        "submitted_by": user.full_name,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "sanitary_expenditure_reports",
        id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "total_amount": payload.total_amount,
        })),
    )
    .await?;

    Ok(id)
}

#[tauri::command]
pub async fn san_get_expenditure_reports(
    state: State<'_, AppState>,
) -> Result<Vec<ExpenditureReportSummary>, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfSanitary]);

    let reports = sqlx::query_as::<_, ExpenditureReportSummary>(
        r#"
        SELECT id, total_amount::float8 AS total_amount, line_items, foul_play_flag, foul_play_note, created_at
        FROM sanitary_expenditure_reports
        WHERE submitted_by = $1 AND deleted_at IS NULL
        ORDER BY created_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(reports)
}

// ══════════════════════════════════════════════════════════════════════════════
// Cache invalidation helpers
// ══════════════════════════════════════════════════════════════════════════════

async fn invalidate_inventory_cache(state: &State<'_, AppState>) {
    let cache_key = "sanitary_inventory:list";
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(cache_key).await;
    }
}

async fn invalidate_divisions_cache(state: &State<'_, AppState>) {
    let cache_key = "sanitary_divisions:quota";
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(cache_key).await;
    }
}

async fn invalidate_schedule_cache(state: &State<'_, AppState>, user_id: Uuid) {
    let cache_key = format!("sanitary_schedule:{}", user_id);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(&cache_key).await;
    }
}

async fn invalidate_staff_division_cache(state: &State<'_, AppState>, user_id: Uuid) {
    let cache_key = format!("staff_division:{}", user_id);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(&cache_key).await;
    }
}
