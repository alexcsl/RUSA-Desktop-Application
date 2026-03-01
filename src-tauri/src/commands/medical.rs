// commands/medical.rs — Medical Department subsystem commands
// Source of truth: 09_MEDICAL.md
//
// Use cases:
//   UC-MED-01  Log Patient Treatment         (MedicalStaff + HeadOfMedicine)
//   UC-MED-02  View Patient Record            (MedicalStaff + HeadOfMedicine)
//   UC-MED-03  Manage Medical Inventory       (MedicalStaff + HeadOfMedicine)
//   UC-MED-04  View Work Schedule             (MedicalStaff + HeadOfMedicine)
//   UC-HOM-01  Allocate Staff Shifts          (HeadOfMedicine only)
//   UC-HOM-02  Submit Budget Funding Request  (HeadOfMedicine only)
//   UC-HOM-03  Submit Budget Expenditure Rpt  (HeadOfMedicine only)
//   UC-HOM-04  Contact Other Heads            (HeadOfMedicine only — medical_heads channel)
//
// Data isolation: DataAnalyst role is NEVER allowed to call these commands.

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

// ── Shared medical role set ───────────────────────────────────────────────────

const MEDICAL_ROLES: [Role; 2] = [Role::MedicalStaff, Role::HeadOfMedicine];

// ── Payloads & Response Types ─────────────────────────────────────────────────

// -- Patients --

#[derive(Debug, Serialize, FromRow)]
pub struct PatientSummary {
    pub id: Uuid,
    pub user_id: Uuid,
    pub full_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct TreatmentLog {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub treated_by: Uuid,
    pub treater_name: String,
    pub treatment_date: DateTime<Utc>,
    pub diagnosis: String,
    pub treatment_provided: String,
    pub medications: Option<String>,
    pub follow_up_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LogTreatmentPayload {
    pub patient_id: Uuid,
    pub treatment_date: String,
    pub diagnosis: String,
    pub treatment_provided: String,
    pub medications: Option<String>,
    pub follow_up_notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterPatientPayload {
    pub user_id: Uuid,
}

// -- Inventory --

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct InventoryItem {
    pub id: Uuid,
    pub base_id: Uuid,
    pub item_name: String,
    pub item_type: String,
    pub quantity: i32,
    pub unit: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddInventoryPayload {
    pub item_name: String,
    pub item_type: String,
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

// -- Shifts --

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ShiftEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub staff_name: String,
    pub specialty: Option<String>,
    pub base_id: Uuid,
    pub shift_start: DateTime<Utc>,
    pub shift_end: DateTime<Utc>,
    pub allocated_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AllocateShiftPayload {
    pub user_id: Uuid,
    pub shift_start: String,
    pub shift_end: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteShiftPayload {
    pub shift_id: Uuid,
}

// -- Staff list for Head --

#[derive(Debug, Serialize, FromRow)]
pub struct MedicalStaffMember {
    pub id: Uuid,
    pub full_name: String,
    pub specialty: Option<String>,
    pub base_id: Option<Uuid>,
    pub base_name: Option<String>,
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

// -- Expenditure --

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

// ══════════════════════════════════════════════════════════════════════════════
// UC-MED-01: Log Patient Treatment
// ══════════════════════════════════════════════════════════════════════════════

/// Register a user as a medical patient (if not already registered).
///
/// **Access:** MedicalStaff, HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_register_patient(
    state: State<'_, AppState>,
    payload: RegisterPatientPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::MedicalStaff, Role::HeadOfMedicine]);

    // Check if already registered
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM medical_patients WHERE user_id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.user_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some((id,)) = existing {
        return Ok(id);
    }

    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO medical_patients (user_id) VALUES ($1) RETURNING id",
    )
    .bind(payload.user_id)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "medical_patients",
        id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({ "user_id": payload.user_id })),
    )
    .await?;

    Ok(id)
}

/// Log a patient treatment encounter.
///
/// **Access:** MedicalStaff, HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_log_treatment(
    state: State<'_, AppState>,
    payload: LogTreatmentPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::MedicalStaff, Role::HeadOfMedicine]);

    let treatment_date = DateTime::parse_from_rfc3339(&payload.treatment_date)
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let (id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO patient_treatment_logs
            (patient_id, treated_by, treatment_date, diagnosis, treatment_provided, medications, follow_up_notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(payload.patient_id)
    .bind(user.id)
    .bind(treatment_date)
    .bind(&payload.diagnosis)
    .bind(&payload.treatment_provided)
    .bind(&payload.medications)
    .bind(&payload.follow_up_notes)
    .fetch_one(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "patient_treatment_logs",
        id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "patient_id": payload.patient_id,
            "diagnosis": payload.diagnosis,
        })),
    )
    .await?;

    Ok(id)
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-MED-02: View Patient Record
// ══════════════════════════════════════════════════════════════════════════════

/// List all registered patients.
///
/// **Access:** MedicalStaff, HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_get_patients(
    state: State<'_, AppState>,
) -> Result<Vec<PatientSummary>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::MedicalStaff, Role::HeadOfMedicine]);

    let patients = sqlx::query_as::<_, PatientSummary>(
        r#"
        SELECT mp.id, mp.user_id, u.full_name, mp.created_at
        FROM medical_patients mp
        JOIN users u ON u.id = mp.user_id
        WHERE mp.deleted_at IS NULL
        ORDER BY u.full_name
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(patients)
}

/// Get treatment history for a specific patient.
///
/// **Access:** MedicalStaff, HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_get_patient_record(
    state: State<'_, AppState>,
    patient_id: Uuid,
) -> Result<Vec<TreatmentLog>, AppError> {
    let user = crate::require_auth_any!(state, [Role::MedicalStaff, Role::HeadOfMedicine]);

    let logs = sqlx::query_as::<_, TreatmentLog>(
        r#"
        SELECT tl.id, tl.patient_id, tl.treated_by,
               u.full_name AS treater_name,
               tl.treatment_date, tl.diagnosis, tl.treatment_provided,
               tl.medications, tl.follow_up_notes, tl.created_at
        FROM patient_treatment_logs tl
        JOIN users u ON u.id = tl.treated_by
        WHERE tl.patient_id = $1 AND tl.deleted_at IS NULL
        ORDER BY tl.treatment_date DESC
        "#,
    )
    .bind(patient_id)
    .fetch_all(&state.db_pool)
    .await?;

    // Log view access to audit_log (all access is logged per privacy requirement)
    write_audit_log(
        &state.db_pool,
        "patient_treatment_logs",
        patient_id, // using patient_id as record_id for view audit
        AuditOperation::Update, // "viewed" captured as update with descriptive payload
        user.id,
        None,
        Some(serde_json::json!({ "action": "view_patient_record", "patient_id": patient_id })),
    )
    .await?;

    Ok(logs)
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-MED-03: Manage Medical Inventory
// ══════════════════════════════════════════════════════════════════════════════

/// Get medical inventory for the user's base.
///
/// **Access:** MedicalStaff, HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_get_inventory(
    state: State<'_, AppState>,
) -> Result<Vec<InventoryItem>, AppError> {
    let user = crate::require_auth_any!(state, [Role::MedicalStaff, Role::HeadOfMedicine]);

    // HeadOfMedicine can see all bases, staff sees their own base
    let items = if user.role == Role::HeadOfMedicine || user.role == Role::Administrator {
        sqlx::query_as::<_, InventoryItem>(
            r#"
            SELECT id, base_id, item_name, item_type, quantity, unit, updated_at
            FROM medical_inventory
            WHERE deleted_at IS NULL
            ORDER BY item_name
            "#,
        )
        .fetch_all(&state.db_pool)
        .await?
    } else {
        let base_id = user.base_location_id.ok_or_else(|| {
            AppError::Internal("No base location assigned.".into())
        })?;

        // Try Redis cache
        let cache_key = format!("medical_inventory:{}", base_id);
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
                if let Ok(items) = serde_json::from_str::<Vec<InventoryItem>>(&cached) {
                    return Ok(items);
                }
            }
        }

        let items = sqlx::query_as::<_, InventoryItem>(
            r#"
            SELECT id, base_id, item_name, item_type, quantity, unit, updated_at
            FROM medical_inventory
            WHERE base_id = $1 AND deleted_at IS NULL
            ORDER BY item_name
            "#,
        )
        .bind(base_id)
        .fetch_all(&state.db_pool)
        .await?;

        // Write to cache
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            let _: Result<(), _> = conn
                .set_ex(&cache_key, serde_json::to_string(&items).unwrap_or_default(), 900)
                .await;
        }

        items
    };

    Ok(items)
}

/// Add a new item to medical inventory.
///
/// **Access:** MedicalStaff, HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_add_inventory(
    state: State<'_, AppState>,
    payload: AddInventoryPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::MedicalStaff, Role::HeadOfMedicine]);

    let base_id = user.base_location_id.ok_or_else(|| {
        AppError::Internal("No base location assigned.".into())
    })?;

    let valid_types = ["medicine", "equipment", "supply"];
    if !valid_types.contains(&payload.item_type.as_str()) {
        return Err(AppError::Internal(
            "item_type must be 'medicine', 'equipment', or 'supply'.".into(),
        ));
    }

    let (id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO medical_inventory (base_id, item_name, item_type, quantity, unit, last_updated_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(base_id)
    .bind(&payload.item_name)
    .bind(&payload.item_type)
    .bind(payload.quantity)
    .bind(&payload.unit)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Invalidate cache
    invalidate_inventory_cache(&state, base_id).await;

    write_audit_log(
        &state.db_pool,
        "medical_inventory",
        id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "item_name": payload.item_name,
            "item_type": payload.item_type,
            "quantity": payload.quantity,
        })),
    )
    .await?;

    Ok(id)
}

/// Update the quantity of an existing inventory item.
///
/// **Access:** MedicalStaff, HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_update_inventory(
    state: State<'_, AppState>,
    payload: UpdateInventoryPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::MedicalStaff, Role::HeadOfMedicine]);

    #[derive(FromRow)]
    struct InvRow {
        base_id: Uuid,
        quantity: i32,
    }

    let before = sqlx::query_as::<_, InvRow>(
        "SELECT base_id, quantity FROM medical_inventory WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.item_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Inventory item not found.".into()))?;

    sqlx::query(
        r#"
        UPDATE medical_inventory
        SET quantity = $2, last_updated_by = $3, updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.item_id)
    .bind(payload.quantity)
    .bind(user.id)
    .execute(&state.db_pool)
    .await?;

    invalidate_inventory_cache(&state, before.base_id).await;

    write_audit_log(
        &state.db_pool,
        "medical_inventory",
        payload.item_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({ "quantity": before.quantity })),
        Some(serde_json::json!({ "quantity": payload.quantity })),
    )
    .await?;

    Ok(())
}

/// Soft-delete an inventory item.
///
/// **Access:** MedicalStaff, HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_remove_inventory(
    state: State<'_, AppState>,
    payload: RemoveInventoryPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::MedicalStaff, Role::HeadOfMedicine]);

    #[derive(FromRow)]
    struct BaseRow {
        base_id: Uuid,
    }

    let row = sqlx::query_as::<_, BaseRow>(
        "SELECT base_id FROM medical_inventory WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.item_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Inventory item not found.".into()))?;

    sqlx::query(
        "UPDATE medical_inventory SET deleted_at = NOW(), deleted_by = $2 WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.item_id)
    .bind(user.id)
    .execute(&state.db_pool)
    .await?;

    invalidate_inventory_cache(&state, row.base_id).await;

    write_audit_log(
        &state.db_pool,
        "medical_inventory",
        payload.item_id,
        AuditOperation::Delete,
        user.id,
        None,
        None,
    )
    .await?;

    Ok(())
}

/// Invalidate Redis cache for a base's medical inventory.
async fn invalidate_inventory_cache(state: &State<'_, AppState>, base_id: Uuid) {
    let cache_key = format!("medical_inventory:{}", base_id);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(&cache_key).await;
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-MED-04: View Work Schedule
// ══════════════════════════════════════════════════════════════════════════════

/// Get the logged-in user's shift schedule.
///
/// **Access:** MedicalStaff, HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_get_my_shifts(
    state: State<'_, AppState>,
) -> Result<Vec<ShiftEntry>, AppError> {
    let user = crate::require_auth_any!(state, [Role::MedicalStaff, Role::HeadOfMedicine]);

    // Try Redis cache
    let cache_key = format!("medical_shifts:{}", user.id);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(shifts) = serde_json::from_str::<Vec<ShiftEntry>>(&cached) {
                return Ok(shifts);
            }
        }
    }

    let shifts = sqlx::query_as::<_, ShiftEntry>(
        r#"
        SELECT ss.id, ss.user_id, u.full_name AS staff_name,
               msp.specialty,
               ss.base_id, ss.shift_start, ss.shift_end,
               ss.allocated_by, ss.created_at
        FROM staff_shifts ss
        JOIN users u ON u.id = ss.user_id
        LEFT JOIN medical_staff_profiles msp ON msp.user_id = ss.user_id
        WHERE ss.user_id = $1 AND ss.deleted_at IS NULL
        ORDER BY ss.shift_start ASC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    // Write to cache (1 hour TTL)
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn
            .set_ex(&cache_key, serde_json::to_string(&shifts).unwrap_or_default(), 3600)
            .await;
    }

    Ok(shifts)
}

// ══════════════════════════════════════════════════════════════════════════════
// UC-HOM-01: Allocate Staff Shifts
// ══════════════════════════════════════════════════════════════════════════════

/// List all medical staff at the Head's base (with specialty).
///
/// **Access:** HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_get_staff_list(
    state: State<'_, AppState>,
) -> Result<Vec<MedicalStaffMember>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::HeadOfMedicine]);

    let staff = sqlx::query_as::<_, MedicalStaffMember>(
        r#"
        SELECT u.id, u.full_name, msp.specialty,
               u.base_location_id AS base_id,
               bl.name AS base_name
        FROM users u
        JOIN roles r ON r.id = u.role_id
        LEFT JOIN medical_staff_profiles msp ON msp.user_id = u.id
        LEFT JOIN base_locations bl ON bl.id = u.base_location_id
        WHERE r.name IN ('MedicalStaff', 'HeadOfMedicine')
          AND u.deleted_at IS NULL AND u.is_active = TRUE
        ORDER BY u.full_name
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(staff)
}

/// Get all shifts (Head's view — all staff, not just own).
///
/// **Access:** HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_get_all_shifts(
    state: State<'_, AppState>,
) -> Result<Vec<ShiftEntry>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::HeadOfMedicine]);

    let shifts = sqlx::query_as::<_, ShiftEntry>(
        r#"
        SELECT ss.id, ss.user_id, u.full_name AS staff_name,
               msp.specialty,
               ss.base_id, ss.shift_start, ss.shift_end,
               ss.allocated_by, ss.created_at
        FROM staff_shifts ss
        JOIN users u ON u.id = ss.user_id
        LEFT JOIN medical_staff_profiles msp ON msp.user_id = ss.user_id
        WHERE ss.deleted_at IS NULL
        ORDER BY ss.shift_start ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(shifts)
}

/// Allocate a shift for a staff member.
///
/// **Access:** HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_allocate_shift(
    state: State<'_, AppState>,
    payload: AllocateShiftPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfMedicine]);

    let shift_start = DateTime::parse_from_rfc3339(&payload.shift_start)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| AppError::Internal("Invalid shift_start date format.".into()))?;

    let shift_end = DateTime::parse_from_rfc3339(&payload.shift_end)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| AppError::Internal("Invalid shift_end date format.".into()))?;

    if shift_end <= shift_start {
        return Err(AppError::Internal("shift_end must be after shift_start.".into()));
    }

    let base_id = user.base_location_id.ok_or_else(|| {
        AppError::Internal("Head has no base location.".into())
    })?;

    let (id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO staff_shifts (user_id, base_id, shift_start, shift_end, allocated_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(payload.user_id)
    .bind(base_id)
    .bind(shift_start)
    .bind(shift_end)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Invalidate the staff member's shift cache
    let cache_key = format!("medical_shifts:{}", payload.user_id);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(&cache_key).await;
    }

    // Notify the staff member in real time
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
        "allocated_by": user.full_name,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "staff_shifts",
        id,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "user_id": payload.user_id,
            "shift_start": shift_start,
            "shift_end": shift_end,
        })),
    )
    .await?;

    Ok(id)
}

/// Soft-delete a shift.
///
/// **Access:** HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_delete_shift(
    state: State<'_, AppState>,
    payload: DeleteShiftPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfMedicine]);

    #[derive(FromRow)]
    struct ShiftUser {
        user_id: Uuid,
    }

    let row = sqlx::query_as::<_, ShiftUser>(
        "SELECT user_id FROM staff_shifts WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.shift_id)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or_else(|| AppError::Internal("Shift not found.".into()))?;

    sqlx::query(
        "UPDATE staff_shifts SET deleted_at = NOW(), deleted_by = $2 WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.shift_id)
    .bind(user.id)
    .execute(&state.db_pool)
    .await?;

    // Invalidate the staff member's shift cache
    let cache_key = format!("medical_shifts:{}", row.user_id);
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(&cache_key).await;
    }

    write_audit_log(
        &state.db_pool,
        "staff_shifts",
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
// UC-HOM-02: Submit Budget Funding Request
// ══════════════════════════════════════════════════════════════════════════════

/// Upload an invoice to Supabase Storage and submit a budget funding request.
/// The request enters the Directors' voting queue.
///
/// **Access:** HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_submit_budget_request(
    state: State<'_, AppState>,
    payload: SubmitBudgetRequestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfMedicine]);

    // 1. Generate the request ID early so we can use it for the storage path
    let request_id = Uuid::new_v4();
    let storage_path = format!(
        "medical/budget-invoices/{}/{}",
        request_id, payload.invoice_filename
    );

    // 2. Upload invoice to Supabase Storage
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

    // 3. Insert the budget request record
    let (id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO medical_budget_requests
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

    // 4. Create a generic request envelope for the voting pipeline
    let req_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO requests (type, requester_id, title, description, payload, invoice_storage_path)
        VALUES ('budget', $1, $2, $3, $4::jsonb, $5)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind("Medical Department Budget Request")
    .bind(&payload.justification)
    .bind(serde_json::json!({
        "department": "medical",
        "medical_budget_request_id": id,
        "line_items": payload.line_items,
        "total_amount": payload.total_amount,
    }))
    .bind(&storage_path)
    .fetch_one(&state.db_pool)
    .await?;

    // 5. Create a vote session for the Directors
    let (vote_id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO vote_sessions (request_id, topic, context, status, opens_at, created_by)
        VALUES ($1, $2, $3, 'open', NOW(), $4)
        RETURNING id
        "#,
    )
    .bind(req_id.0)
    .bind("Medical Budget Request")
    .bind(&payload.justification)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // 6. Link vote session to the budget request
    sqlx::query(
        "UPDATE medical_budget_requests SET vote_session_id = $2 WHERE id = $1",
    )
    .bind(id)
    .bind(vote_id)
    .execute(&state.db_pool)
    .await?;

    // 7. Update request status to in_vote
    sqlx::query(
        "UPDATE requests SET status = 'in_vote', updated_at = NOW() WHERE id = $1",
    )
    .bind(req_id.0)
    .execute(&state.db_pool)
    .await?;

    // 8. Notify all directors
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
        "topic": "Medical Budget Request",
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "medical_budget_requests",
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

/// Get all budget requests submitted by the logged-in Head.
///
/// **Access:** HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_get_budget_requests(
    state: State<'_, AppState>,
) -> Result<Vec<BudgetRequestSummary>, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfMedicine]);

    let reqs = sqlx::query_as::<_, BudgetRequestSummary>(
        r#"
        SELECT id, total_amount::float8 AS total_amount, justification, status, line_items, created_at
        FROM medical_budget_requests
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
// UC-HOM-03: Submit Budget Expenditure Report
// ══════════════════════════════════════════════════════════════════════════════

/// Submit an expenditure report. Routed to The Accountant.
///
/// **Access:** HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_submit_expenditure_report(
    state: State<'_, AppState>,
    payload: SubmitExpenditurePayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfMedicine]);

    let (id,): (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO medical_expenditure_reports
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

    // Also insert into generic requests so The Accountant sees it
    sqlx::query(
        r#"
        INSERT INTO requests (type, requester_id, title, description, payload, status)
        VALUES ('budget', $1, $2, 'Medical expenditure report', $3::jsonb, 'pending')
        "#,
    )
    .bind(user.id)
    .bind("Medical Expenditure Report")
    .bind(serde_json::json!({
        "department": "medical",
        "report_type": "expenditure",
        "medical_expenditure_report_id": id,
        "line_items": payload.line_items,
        "total_amount": payload.total_amount,
    }))
    .execute(&state.db_pool)
    .await?;

    // Notify The Accountant
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
        "department": "medical",
        "total_amount": payload.total_amount,
        "submitted_by": user.full_name,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "medical_expenditure_reports",
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

/// Get all expenditure reports submitted by the logged-in Head.
///
/// **Access:** HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_get_expenditure_reports(
    state: State<'_, AppState>,
) -> Result<Vec<ExpenditureReportSummary>, AppError> {
    let user = crate::require_auth_any!(state, [Role::HeadOfMedicine]);

    let reports = sqlx::query_as::<_, ExpenditureReportSummary>(
        r#"
        SELECT id, total_amount::float8 AS total_amount, line_items, foul_play_flag, foul_play_note, created_at
        FROM medical_expenditure_reports
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
// UC-HOM-04: Contact Other Heads of Medicine
// ══════════════════════════════════════════════════════════════════════════════
// Uses the shared messaging system with channel = 'medical_heads'.
// The messaging commands already exist in commands/messaging.rs.
// The frontend uses the existing messaging store with channel = 'medical_heads'.
//
// No new Rust command needed — the existing send_message / get_inbox handle
// channel scoping. The frontend layout wires a nav link:
//   /messaging/inbox?channel=medical_heads
//
// Role guard: The messaging backend enforces that only HeadOfMedicine or
// Administrator can read/send on the 'medical_heads' channel.
//
// See: commands/messaging.rs → send_message(), get_inbox()

// ══════════════════════════════════════════════════════════════════════════════
// Helper: get all users for patient registration lookup
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, FromRow)]
pub struct UserLookupItem {
    pub id: Uuid,
    pub full_name: String,
    pub role_name: String,
}

/// Get a list of all active users for patient registration lookup.
///
/// **Access:** MedicalStaff, HeadOfMedicine, Administrator.
#[tauri::command]
pub async fn med_get_user_lookup(
    state: State<'_, AppState>,
) -> Result<Vec<UserLookupItem>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::MedicalStaff, Role::HeadOfMedicine]);

    let users = sqlx::query_as::<_, UserLookupItem>(
        r#"
        SELECT u.id, u.full_name, r.name AS role_name
        FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE u.deleted_at IS NULL AND u.is_active = TRUE
        ORDER BY u.full_name
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(users)
}
