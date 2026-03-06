// commands/directors.rs — Director-level commands (per-role)
// Source of truth: AUTH_GUIDE.md §10, 11_DIRECTORS.md
//
// This file contains:
// - Account creation / termination (TheDirector, TheAnchorman)
// - Meeting management (UC-DIR-03)
// - Personnel update (UC-TDIR-02)
// - Financial monitoring (UC-ACC-01, UC-ACC-02)
// - Personnel relocation (UC-NOM-01)
// - Task assignment (UC-ART-01, UC-OBS-01, UC-NOM-02/03, UC-TM-01)
// - Task progress tracking (UC-ART-02, UC-OBS-02, UC-TM-02)
// - Closure approval (UC-TM-03)
// - Help request/response proxy (UC-ART-03/04, UC-OBS-03/04)
// - Territory renaming (UC-WAN-04)
// - Broadcast management (UC-GUA-02/03, UC-ANC-00/01/02)
// - Data request gating (UC-STAT-01/02/03)
// - Event management (UC-CORD-01/02/03)
// - Termination records (UC-ANC-03)
// - Secure messaging (UC-GUA-04, UC-OVR-02)

use chrono::{DateTime, NaiveDate, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::State;
use uuid::Uuid;

use crate::{
    audit::{write_audit_log, AuditOperation},
    commands::auth::hash_password,
    error::AppError,
    state::{AppState, Role},
};

// ── Payloads & Responses ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateAccountPayload {
    pub full_name: String,
    pub username: String,
    pub email: Option<String>,
    pub initial_password: String,
    pub role: String,                    // role name string from frontend
    pub base_location_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct CreatedUser {
    pub id: Uuid,
    pub username: String,
    pub full_name: String,
    pub role: String,
}

/// Internal struct for the inserted row.
#[derive(Debug, FromRow)]
struct InsertedUser {
    id: Uuid,
}

// ── Create Personnel Account ──────────────────────────────────────────────────

/// Creates a new user account.
///
/// **Access:** TheDirector (non-Director accounts only) or Administrator (any account).
///
/// Rules (AUTH_GUIDE §10):
/// - TheDirector cannot create Director-level or Administrator accounts.
/// - Administrator can create any account, including Director-level.
/// - Password is hashed with bcrypt cost 12 before storage.
/// - The plaintext password is never logged, stored, or transmitted beyond this call.
/// - A CREATE audit entry is written to audit_log.
#[tauri::command]
pub async fn create_personnel_account(
    state: State<'_, AppState>,
    payload: CreateAccountPayload,
) -> Result<CreatedUser, AppError> {
    // 1. Auth guard: only TheDirector or Administrator
    let creator = crate::require_auth_any!(state, [Role::TheDirector, Role::Administrator]);

    // 2. Parse the target role
    let target_role = Role::from_str(&payload.role)
        .map_err(AppError::Internal)?;

    // 3. Rank restriction: TheDirector cannot create Director-level accounts
    if creator.role == Role::TheDirector && target_role.is_director() {
        return Err(AppError::Forbidden);
    }

    // 4. TheDirector also cannot create Administrator accounts
    //    (covered by is_director() returning true for Administrator, but explicit for clarity)
    if creator.role == Role::TheDirector && target_role == Role::Administrator {
        return Err(AppError::Forbidden);
    }

    // 5. Hash the initial password — bcrypt cost 12, never stored in plaintext
    let password_hash = hash_password(&payload.initial_password)?;

    // 6. Insert the user record
    let inserted = sqlx::query_as::<_, InsertedUser>(
        r#"
        INSERT INTO users (full_name, username, email, password_hash, role_id, base_location_id)
        SELECT $1, $2, $3, $4, r.id, $5
        FROM roles r WHERE r.name = $6
        RETURNING id
        "#,
    )
    .bind(&payload.full_name)
    .bind(&payload.username)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(payload.base_location_id)
    .bind(&payload.role)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        // Surface a clean message for unique constraint violations
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.constraint() == Some("users_username_key") {
                return AppError::Internal(format!(
                    "Username '{}' is already taken.",
                    payload.username
                ));
            }
            if db_err.constraint() == Some("users_email_key") {
                return AppError::Internal("That email address is already in use.".into());
            }
        }
        AppError::Database(e.to_string())
    })?;

    // 7. Write audit log entry
    let after_data = serde_json::json!({
        "id": inserted.id,
        "full_name": payload.full_name,
        "username": payload.username,
        "email": payload.email,
        "role": payload.role,
        "base_location_id": payload.base_location_id,
        // password_hash deliberately excluded from audit log
    });

    write_audit_log(
        &state.db_pool,
        "users",
        inserted.id,
        AuditOperation::Create,
        creator.id,
        None,            // no before_data for CREATE
        Some(after_data),
    )
    .await?;

    // 8. Return the created user info
    Ok(CreatedUser {
        id: inserted.id,
        username: payload.username,
        full_name: payload.full_name,
        role: payload.role,
    })
}

// ── Terminate Personnel Account ───────────────────────────────────────────────

/// Soft-deletes a user account and invalidates their Redis session.
///
/// **Access:** TheAnchorman or Administrator only.
///
/// Rules (AUTH_GUIDE §9, §11):
/// - Sets deleted_at and deleted_by on the user record (soft delete — never DELETE FROM).
/// - Immediately deletes session:{id} and role:{id} from Redis.
/// - Writes a DELETE audit entry.
#[tauri::command]
pub async fn terminate_personnel_account(
    state: State<'_, AppState>,
    target_user_id: Uuid,
) -> Result<(), AppError> {
    // 1. Auth guard: TheAnchorman or Administrator
    let terminator = crate::require_auth_any!(state, [Role::TheAnchorman, Role::Administrator]);

    // 2. Prevent self-termination
    if terminator.id == target_user_id {
        return Err(AppError::Internal(
            "You cannot terminate your own account.".into(),
        ));
    }

    // 3. Fetch current user data for audit before_data
    let before_row: Option<(Uuid, String, String, bool)> = sqlx::query_as(
        r#"
        SELECT id, username, full_name, is_active
        FROM users
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(target_user_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (_, username, full_name, is_active) = before_row
        .ok_or_else(|| AppError::Internal("User not found or already terminated.".into()))?;

    let before_data = serde_json::json!({
        "id": target_user_id,
        "username": username,
        "full_name": full_name,
        "is_active": is_active,
    });

    // 4. Soft-delete: set deleted_at + deleted_by, deactivate
    sqlx::query(
        r#"
        UPDATE users
        SET deleted_at = NOW(), deleted_by = $1, is_active = FALSE, updated_at = NOW()
        WHERE id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(terminator.id)
    .bind(target_user_id)
    .execute(&state.db_pool)
    .await?;

    // 5. Immediately invalidate Redis session for the terminated user
    let redis_result = state
        .redis_client
        .get_multiplexed_async_connection()
        .await;

    if let Ok(mut redis_conn) = redis_result {
        let _: Result<(), _> = redis_conn
            .del(&[
                format!("session:{}", target_user_id),
                format!("role:{}", target_user_id),
            ])
            .await;
    }

    // 6. If the terminated user is the currently logged-in user, clear AppState
    {
        let mut guard = state.current_user.lock().await;
        if let Some(ref current) = *guard {
            if current.id == target_user_id {
                *guard = None;
            }
        }
    }

    // 7. Audit log
    let after_data = serde_json::json!({
        "id": target_user_id,
        "deleted_at": "NOW()",
        "deleted_by": terminator.id,
        "is_active": false,
    });

    write_audit_log(
        &state.db_pool,
        "users",
        target_user_id,
        AuditOperation::Delete,
        terminator.id,
        Some(before_data),
        Some(after_data),
    )
    .await?;

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-TDIR-02: Update Non-Director Personnel Account
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct UpdateAccountPayload {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub base_location_id: Option<Uuid>,
}

/// Update an existing non-Director personnel account.
///
/// **Access:** TheDirector (non-Director targets) or Administrator (any target).
#[tauri::command]
pub async fn update_personnel_account(
    state: State<'_, AppState>,
    target_user_id: Uuid,
    payload: UpdateAccountPayload,
) -> Result<(), AppError> {
    let updater = crate::require_auth_any!(state, [Role::TheDirector, Role::Administrator]);

    // Fetch current state for audit
    let before: Option<(String, Option<String>, String, Option<Uuid>)> = sqlx::query_as(
        r#"
        SELECT u.full_name, u.email, r.name AS role_name, u.base_location_id
        FROM users u JOIN roles r ON r.id = u.role_id
        WHERE u.id = $1 AND u.deleted_at IS NULL
        "#,
    )
    .bind(target_user_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (old_name, old_email, old_role, old_loc) = before
        .ok_or_else(|| AppError::Internal("User not found.".into()))?;

    // TheDirector cannot update Director-level accounts
    let target_role = Role::from_str(&old_role).map_err(AppError::Internal)?;
    if updater.role == Role::TheDirector && target_role.is_director() {
        return Err(AppError::Forbidden);
    }

    // If changing role, validate new role
    if let Some(ref new_role) = payload.role {
        let parsed = Role::from_str(new_role).map_err(AppError::Internal)?;
        if updater.role == Role::TheDirector && parsed.is_director() {
            return Err(AppError::Forbidden);
        }
    }

    // Build dynamic UPDATE
    let full_name = payload.full_name.as_deref().unwrap_or(&old_name);
    let email = payload.email.as_deref().or(old_email.as_deref());
    let role_name = payload.role.as_deref().unwrap_or(&old_role);

    sqlx::query(
        r#"
        UPDATE users SET
          full_name = $2,
          email = $3,
          role_id = (SELECT id FROM roles WHERE name = $4),
          base_location_id = $5,
          updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(target_user_id)
    .bind(full_name)
    .bind(email)
    .bind(role_name)
    .bind(payload.base_location_id.or(old_loc))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "users",
        target_user_id,
        AuditOperation::Update,
        updater.id,
        Some(serde_json::json!({
            "full_name": old_name, "email": old_email,
            "role": old_role, "base_location_id": old_loc,
        })),
        Some(serde_json::json!({
            "full_name": full_name, "email": email,
            "role": role_name, "base_location_id": payload.base_location_id.or(old_loc),
        })),
    )
    .await?;

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-DIR-03: Create Meeting
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct CreateMeetingPayload {
    pub title: String,
    pub agenda: Option<String>,
    pub scheduled_at: DateTime<Utc>,
    pub invitee_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct MeetingCreated {
    pub id: Uuid,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MeetingSummary {
    pub id: Uuid,
    pub title: String,
    pub agenda: Option<String>,
    pub scheduled_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Create a meeting with invitees. All invitees are notified.
///
/// **Access:** Any Director or Administrator.
#[tauri::command]
pub async fn create_meeting(
    state: State<'_, AppState>,
    payload: CreateMeetingPayload,
) -> Result<MeetingCreated, AppError> {
    let user = crate::require_auth_director!(state);

    if payload.title.trim().is_empty() {
        return Err(AppError::Internal("Meeting title cannot be empty.".into()));
    }

    // Insert meeting
    let meeting_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO meetings (title, agenda, scheduled_at, created_by)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(&payload.title)
    .bind(&payload.agenda)
    .bind(payload.scheduled_at)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Insert invitees
    for invitee_id in &payload.invitee_ids {
        sqlx::query(
            "INSERT INTO meeting_invitees (meeting_id, user_id, notified_at) VALUES ($1, $2, NOW())",
        )
        .bind(meeting_id.0)
        .bind(invitee_id)
        .execute(&state.db_pool)
        .await?;

        // Create notification for each invitee
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'meeting:invited', $2::jsonb)
            "#,
        )
        .bind(invitee_id)
        .bind(serde_json::json!({
            "meeting_id": meeting_id.0,
            "title": payload.title,
            "scheduled_at": payload.scheduled_at,
            "invited_by": user.full_name,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    write_audit_log(
        &state.db_pool,
        "meetings",
        meeting_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "title": payload.title,
            "agenda": payload.agenda,
            "scheduled_at": payload.scheduled_at,
            "invitee_count": payload.invitee_ids.len(),
        })),
    )
    .await?;

    Ok(MeetingCreated { id: meeting_id.0 })
}

/// List meetings (created by the current user or where they are an invitee).
///
/// **Access:** Any Director or Administrator.
#[tauri::command]
pub async fn get_meetings(
    state: State<'_, AppState>,
) -> Result<Vec<MeetingSummary>, AppError> {
    let user = crate::require_auth_director!(state);

    let meetings = sqlx::query_as::<_, MeetingSummary>(
        r#"
        SELECT DISTINCT m.id, m.title, m.agenda, m.scheduled_at, m.created_by, m.created_at
        FROM meetings m
        LEFT JOIN meeting_invitees mi ON mi.meeting_id = m.id
        WHERE m.deleted_at IS NULL
          AND (m.created_by = $1 OR mi.user_id = $1)
        ORDER BY m.scheduled_at DESC
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(meetings)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-ACC-01/02: Financial Monitoring + Flagging
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, FromRow)]
pub struct FinancialDocument {
    pub id: Uuid,
    pub type_: String,
    pub title: String,
    pub status: String,
    pub requester_id: Uuid,
    pub requester_name: String,
    pub payload: serde_json::Value,
    pub invoice_storage_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Get all budget/financial requests for the Accountant's monitoring queue.
///
/// **Access:** TheAccountant or Administrator.
#[tauri::command]
pub async fn get_financial_queue(
    state: State<'_, AppState>,
) -> Result<Vec<FinancialDocument>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::TheAccountant, Role::Administrator]);

    let docs = sqlx::query_as::<_, FinancialDocument>(
        r#"
        SELECT r.id, r.type AS type_, r.title, r.status,
               r.requester_id, u.full_name AS requester_name,
               r.payload, r.invoice_storage_path, r.created_at
        FROM requests r
        JOIN users u ON u.id = r.requester_id
        WHERE r.type = 'budget'
          AND r.deleted_at IS NULL
        ORDER BY r.created_at DESC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(docs)
}

#[derive(Debug, Deserialize)]
pub struct FlagReportPayload {
    pub request_id: Uuid,
    pub reason: String,
}

/// Flag a budget report as suspicious. Triggers a disciplinary meeting request + voting matter.
///
/// **Access:** TheAccountant or Administrator.
#[tauri::command]
pub async fn flag_budget_report(
    state: State<'_, AppState>,
    payload: FlagReportPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::TheAccountant, Role::Administrator]);

    if payload.reason.trim().is_empty() {
        return Err(AppError::Internal("Flag reason cannot be empty.".into()));
    }

    // Mark request as flagged in payload
    sqlx::query(
        r#"
        UPDATE requests SET
          payload = payload || $2::jsonb,
          updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.request_id)
    .bind(serde_json::json!({
        "flagged": true,
        "flagged_by": user.id,
        "flag_reason": payload.reason,
        "flagged_at": Utc::now(),
    }))
    .execute(&state.db_pool)
    .await?;

    // Create a vote session for the disciplinary matter
    let vote_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO vote_sessions (request_id, topic, context, status, opens_at, created_by)
        VALUES ($1, $2, $3, 'open', NOW(), $4)
        RETURNING id
        "#,
    )
    .bind(payload.request_id)
    .bind(format!("Disciplinary: Suspicious Budget Report"))
    .bind(&payload.reason)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Update the request status to 'in_vote'
    sqlx::query(
        "UPDATE requests SET status = 'in_vote', updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.request_id)
    .execute(&state.db_pool)
    .await?;

    // Notify all directors
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
        "session_id": vote_id.0,
        "topic": "Disciplinary: Suspicious Budget Report",
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "requests",
        payload.request_id,
        AuditOperation::Update,
        user.id,
        None,
        Some(serde_json::json!({
            "flagged": true,
            "flag_reason": payload.reason,
            "vote_session_id": vote_id.0,
        })),
    )
    .await?;

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-NOM-01: Relocate / Reposition Personnel
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct RelocatePayload {
    pub target_user_id: Uuid,
    pub origin_location: String,
    pub destination: String,
    pub relocation_type: String, // 'temporary' or 'permanent'
    pub effective_date: NaiveDate,
}

/// Relocate a non-Director, non-Administrator staff member.
///
/// **Access:** TheNomad, TheOverseer, or Administrator.
#[tauri::command]
pub async fn relocate_personnel(
    state: State<'_, AppState>,
    payload: RelocatePayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheNomad, Role::TheOverseer, Role::Administrator
    ]);

    if payload.relocation_type != "temporary" && payload.relocation_type != "permanent" {
        return Err(AppError::Internal(
            "Relocation type must be 'temporary' or 'permanent'.".into(),
        ));
    }

    // Check target is not a director or admin
    let target_role: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT r.name FROM users u JOIN roles r ON r.id = u.role_id
        WHERE u.id = $1 AND u.deleted_at IS NULL
        "#,
    )
    .bind(payload.target_user_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (role_name,) = target_role
        .ok_or_else(|| AppError::Internal("Target user not found.".into()))?;

    let target_r = Role::from_str(&role_name).map_err(AppError::Internal)?;
    if target_r.is_director() || target_r == Role::Administrator {
        return Err(AppError::Internal(
            "Cannot relocate Directors or Administrators. Select a non-Director staff member.".into(),
        ));
    }

    let reloc_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO personnel_relocations
          (target_user_id, relocated_by, origin_location, destination, relocation_type, effective_date)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(payload.target_user_id)
    .bind(user.id)
    .bind(&payload.origin_location)
    .bind(&payload.destination)
    .bind(&payload.relocation_type)
    .bind(payload.effective_date)
    .fetch_one(&state.db_pool)
    .await?;

    // Notify the target user
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        VALUES ($1, 'relocation:issued', $2::jsonb)
        "#,
    )
    .bind(payload.target_user_id)
    .bind(serde_json::json!({
        "relocation_id": reloc_id.0,
        "destination": payload.destination,
        "type": payload.relocation_type,
        "effective_date": payload.effective_date,
        "issued_by": user.full_name,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "personnel_relocations",
        reloc_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "target_user_id": payload.target_user_id,
            "origin": payload.origin_location,
            "destination": payload.destination,
            "type": payload.relocation_type,
            "effective_date": payload.effective_date,
        })),
    )
    .await?;

    Ok(reloc_id.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// Task Assignment (UC-ART-01, UC-OBS-01, UC-NOM-02/03, UC-TM-01)
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct AssignTaskPayload {
    pub assigned_to: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub task_type: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct TaskSummary {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub assigned_to: Uuid,
    pub assignee_name: String,
    pub assigned_by: Uuid,
    pub assigner_name: String,
    pub due_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

/// Roles that Artificer can assign tasks to
const ARTIFICER_SUBORDINATES: &[&str] = &["Mathematician", "Physicist"];

/// Roles that Observer can assign tasks to
const OBSERVER_SUBORDINATES: &[&str] = &[
    "Biologist", "Chemist", "AgriculturalEngineer", "BiologicalEngineer",
];

/// Roles that Wanderer can assign to
const WANDERER_SUBORDINATES: &[&str] = &["Astronaut"];

/// All subordinates under Taskmaster scope
const TASKMASTER_SUBORDINATES: &[&str] = &[
    "Mathematician", "Physicist", "Biologist", "Chemist",
    "AgriculturalEngineer", "BiologicalEngineer", "Astronaut",
];

/// Assign a task to a subordinate. Validates scope based on Director role.
///
/// **Access:** TheArtificer, TheObserver, TheWanderer, TheTaskmaster, TheNomad, TheOverseer, or Administrator.
#[tauri::command]
pub async fn assign_task(
    state: State<'_, AppState>,
    payload: AssignTaskPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheArtificer, Role::TheObserver, Role::TheWanderer,
        Role::TheTaskmaster, Role::TheNomad, Role::TheOverseer,
        Role::TheDirector, Role::Administrator
    ]);

    if payload.title.trim().is_empty() {
        return Err(AppError::Internal("Task title cannot be empty.".into()));
    }

    // Validate scope: check the target user's role is within assigner's scope
    let target_role: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT r.name FROM users u JOIN roles r ON r.id = u.role_id
        WHERE u.id = $1 AND u.deleted_at IS NULL
        "#,
    )
    .bind(payload.assigned_to)
    .fetch_optional(&state.db_pool)
    .await?;

    let (target_role_name,) = target_role
        .ok_or_else(|| AppError::Internal("Target user not found.".into()))?;

    // Scope validation per assigner role (Admin bypasses)
    if user.role != Role::Administrator && user.role != Role::TheDirector {
        let allowed = match user.role {
            Role::TheArtificer => ARTIFICER_SUBORDINATES,
            Role::TheObserver => OBSERVER_SUBORDINATES,
            Role::TheWanderer => WANDERER_SUBORDINATES,
            Role::TheTaskmaster => TASKMASTER_SUBORDINATES,
            Role::TheNomad | Role::TheOverseer => &["SettlerCommander", "HeadOfSanitary"][..],
            _ => &[][..],
        };

        if !allowed.contains(&target_role_name.as_str()) {
            return Err(AppError::Forbidden);
        }
    }

    let task_type = payload.task_type.as_deref().unwrap_or("general");
    let task_payload = payload.payload.unwrap_or(serde_json::json!({}));

    let task_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO tasks (assigned_by, assigned_to, type, title, description, payload, due_date)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(payload.assigned_to)
    .bind(task_type)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&task_payload)
    .bind(payload.due_date)
    .fetch_one(&state.db_pool)
    .await?;

    // Notify assignee
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        VALUES ($1, 'task:assigned', $2::jsonb)
        "#,
    )
    .bind(payload.assigned_to)
    .bind(serde_json::json!({
        "task_id": task_id.0,
        "title": payload.title,
        "assigned_by": user.full_name,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "tasks",
        task_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "assigned_to": payload.assigned_to,
            "title": payload.title,
            "type": task_type,
        })),
    )
    .await?;

    Ok(task_id.0)
}

// ── Task Progress Tracking ────────────────────────────────────────────────────

/// Get subordinate task progress filtered by the Director's scope.
///
/// **Access:** TheArtificer, TheObserver, TheWanderer, TheTaskmaster, TheNomad, TheOverseer, or Administrator.
#[tauri::command]
pub async fn get_subordinate_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<TaskSummary>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheArtificer, Role::TheObserver, Role::TheWanderer,
        Role::TheTaskmaster, Role::TheNomad, Role::TheOverseer,
        Role::TheDirector, Role::Administrator
    ]);

    // Build role filter based on current user's scope
    let role_filter: Vec<&str> = if user.role == Role::Administrator || user.role == Role::TheDirector {
        vec![] // empty = all (handled in query)
    } else {
        match user.role {
            Role::TheArtificer => ARTIFICER_SUBORDINATES.to_vec(),
            Role::TheObserver => OBSERVER_SUBORDINATES.to_vec(),
            Role::TheWanderer => WANDERER_SUBORDINATES.to_vec(),
            Role::TheTaskmaster => TASKMASTER_SUBORDINATES.to_vec(),
            Role::TheNomad | Role::TheOverseer => vec!["SettlerCommander", "HeadOfSanitary"],
            _ => vec![],
        }
    };

    let tasks = if role_filter.is_empty() {
        // Admin/TheDirector: see all tasks assigned by self
        sqlx::query_as::<_, TaskSummary>(
            r#"
            SELECT t.id, t.title, t.description, t.status, t.assigned_to,
                   a.full_name AS assignee_name, t.assigned_by,
                   b.full_name AS assigner_name, t.due_date, t.created_at
            FROM tasks t
            JOIN users a ON a.id = t.assigned_to
            JOIN users b ON b.id = t.assigned_by
            WHERE t.deleted_at IS NULL
            ORDER BY t.created_at DESC
            "#,
        )
        .fetch_all(&state.db_pool)
        .await?
    } else {
        sqlx::query_as::<_, TaskSummary>(
            r#"
            SELECT t.id, t.title, t.description, t.status, t.assigned_to,
                   a.full_name AS assignee_name, t.assigned_by,
                   b.full_name AS assigner_name, t.due_date, t.created_at
            FROM tasks t
            JOIN users a ON a.id = t.assigned_to
            JOIN users b ON b.id = t.assigned_by
            JOIN roles r ON r.id = a.role_id
            WHERE t.deleted_at IS NULL
              AND r.name = ANY($1)
            ORDER BY t.created_at DESC
            "#,
        )
        .bind(&role_filter)
        .fetch_all(&state.db_pool)
        .await?
    };

    Ok(tasks)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-ART-05 / UC-OBS-05: Experiment Proposal Queue + Decision
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ExperimentProposalSummary {
    pub request_id: Uuid,
    pub experiment_id: Uuid,
    pub experiment_type: String,
    pub title: String,
    pub description: Option<String>,
    pub requester_id: Uuid,
    pub requester_name: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// List pending experiment proposals routed to the current director.
///
/// **Access:** TheArtificer (Physicist/Mathematician), TheObserver (Chemist/Biologist), or Administrator.
#[tauri::command]
pub async fn get_experiment_proposal_queue(
    state: State<'_, AppState>,
) -> Result<Vec<ExperimentProposalSummary>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheArtificer, Role::TheObserver, Role::Administrator
    ]);

    let authority_filter: Option<&str> = match user.role {
        Role::TheArtificer => Some("TheArtificer"),
        Role::TheObserver => Some("TheObserver"),
        Role::Administrator => None,
        _ => return Err(AppError::Forbidden),
    };

    let rows = if let Some(auth) = authority_filter {
        sqlx::query_as::<_, ExperimentProposalSummary>(
            r#"
            SELECT r.id AS request_id,
                   (r.payload->>'experiment_id')::uuid AS experiment_id,
                   COALESCE(r.payload->>'experiment_type', '') AS experiment_type,
                   r.title,
                   r.description,
                   r.requester_id,
                   u.full_name AS requester_name,
                   r.status,
                   r.created_at
            FROM requests r
            JOIN users u ON u.id = r.requester_id
            WHERE r.type = 'experiment_proposal'
              AND r.bypass_authority = $1
              AND r.deleted_at IS NULL
            ORDER BY r.created_at ASC
            "#,
        )
        .bind(auth)
        .fetch_all(&state.db_pool)
        .await?
    } else {
        sqlx::query_as::<_, ExperimentProposalSummary>(
            r#"
            SELECT r.id AS request_id,
                   (r.payload->>'experiment_id')::uuid AS experiment_id,
                   COALESCE(r.payload->>'experiment_type', '') AS experiment_type,
                   r.title,
                   r.description,
                   r.requester_id,
                   u.full_name AS requester_name,
                   r.status,
                   r.created_at
            FROM requests r
            JOIN users u ON u.id = r.requester_id
            WHERE r.type = 'experiment_proposal'
              AND r.deleted_at IS NULL
            ORDER BY r.created_at ASC
            "#,
        )
        .fetch_all(&state.db_pool)
        .await?
    };

    Ok(rows)
}

#[derive(Debug, Deserialize)]
pub struct ExperimentDecisionPayload {
    pub request_id: Uuid,
    pub decision: String,   // 'approved' or 'denied'
    pub reason: Option<String>,
}

/// Approve or reject an experiment proposal (bypass decision).
///
/// **Access:** TheArtificer, TheObserver, or Administrator.
#[tauri::command]
pub async fn decide_experiment_proposal(
    state: State<'_, AppState>,
    payload: ExperimentDecisionPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheArtificer, Role::TheObserver, Role::Administrator
    ]);

    if payload.decision != "approved" && payload.decision != "denied" {
        return Err(AppError::Internal(
            "Decision must be 'approved' or 'denied'.".into(),
        ));
    }

    // Get the request + experiment_id
    let req_row: Option<(String, Uuid, serde_json::Value)> = sqlx::query_as(
        r#"
        SELECT status, requester_id, payload
        FROM requests WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.request_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (old_status, requester_id, req_payload) = req_row
        .ok_or_else(|| AppError::Internal("Request not found.".into()))?;

    let experiment_id = req_payload
        .get("experiment_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| AppError::Internal("Missing experiment_id in request payload.".into()))?;

    let new_status = if payload.decision == "approved" { "approved" } else { "denied" };

    // Update the request
    sqlx::query(
        r#"
        UPDATE requests SET
          status = $2, decided_by = $3, decision_reason = $4, updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.request_id)
    .bind(new_status)
    .bind(user.id)
    .bind(&payload.reason)
    .execute(&state.db_pool)
    .await?;

    // Update the experiment record
    let exp_status = if payload.decision == "approved" { "approved" } else { "rejected" };
    sqlx::query(
        r#"
        UPDATE experiments SET
          status = $2, approved_by = $3, approved_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(experiment_id)
    .bind(exp_status)
    .bind(if payload.decision == "approved" { Some(user.id) } else { None })
    .execute(&state.db_pool)
    .await?;

    // Notify the requester
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        VALUES ($1, 'experiment:decided', $2::jsonb)
        "#,
    )
    .bind(requester_id)
    .bind(serde_json::json!({
        "request_id": payload.request_id,
        "experiment_id": experiment_id,
        "decision": new_status,
        "reason": payload.reason,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "requests",
        payload.request_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({ "status": old_status })),
        Some(serde_json::json!({
            "status": new_status,
            "experiment_id": experiment_id,
            "decided_by": user.id,
            "decision_reason": payload.reason,
        })),
    )
    .await?;

    Ok(())
}

/// Get math results for tasks assigned by the current director.
///
/// **Access:** TheArtificer or Administrator.
#[tauri::command]
pub async fn get_math_results_for_director(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, AppError> {
    let user = crate::require_auth_any!(state, [Role::TheArtificer, Role::Administrator]);

    let rows: Vec<(Uuid, Uuid, Uuid, String, serde_json::Value, Option<String>, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            r#"
            SELECT mr.id, mr.task_id, mr.submitted_by, u.full_name,
                   mr.content, mr.pdf_storage_path, mr.created_at
            FROM math_results mr
            JOIN users u ON u.id = mr.submitted_by
            JOIN tasks t ON t.id = mr.task_id
            WHERE t.assigned_by = $1 AND mr.deleted_at IS NULL
            ORDER BY mr.created_at DESC
            "#,
        )
        .bind(user.id)
        .fetch_all(&state.db_pool)
        .await?;

    let results: Vec<serde_json::Value> = rows.iter().map(|(id, task_id, submitted_by, name, content, pdf, created_at)| {
        serde_json::json!({
            "id": id,
            "task_id": task_id,
            "submitted_by": submitted_by,
            "submitted_by_name": name,
            "content": content,
            "pdf_storage_path": pdf,
            "created_at": created_at,
        })
    }).collect();

    Ok(results)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-ART-06 / UC-OBS-06: Test Proposal Review Queue + Decision
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TestProposalQueueItem {
    pub id: Uuid,
    pub proposed_by: Uuid,
    pub proposer_name: String,
    pub proposal_data: serde_json::Value,
    pub status: String,
    pub reviewer_note: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// List pending test proposals for the current director's domain.
///
/// - TheArtificer sees proposals from Physicist
/// - TheObserver sees proposals from Chemist / Biologist
///
/// **Access:** TheArtificer, TheObserver, or Administrator.
#[tauri::command]
pub async fn get_test_proposal_queue(
    state: State<'_, AppState>,
) -> Result<Vec<TestProposalQueueItem>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheArtificer, Role::TheObserver, Role::Administrator
    ]);

    let role_filter: Vec<&str> = match user.role {
        Role::TheArtificer => vec!["Physicist"],
        Role::TheObserver  => vec!["Chemist", "Biologist"],
        _                  => vec!["Physicist", "Chemist", "Biologist"],
    };

    let rows = sqlx::query_as::<_, TestProposalQueueItem>(
        r#"
        SELECT tp.id, tp.proposed_by, u.full_name AS proposer_name,
               tp.proposal_data, tp.status, tp.reviewer_note, tp.created_at
        FROM test_proposals tp
        JOIN users u ON u.id = tp.proposed_by
        JOIN roles r ON r.id = u.role_id
        WHERE tp.status = 'pending'
          AND r.name = ANY($1)
        ORDER BY tp.created_at ASC
        "#,
    )
    .bind(&role_filter)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(rows)
}

#[derive(Debug, Deserialize)]
pub struct TestProposalDecisionPayload {
    pub proposal_id: Uuid,
    pub decision: String,   // 'approved' or 'rejected'
    pub reason: Option<String>,
}

/// Approve or reject a test proposal.
///
/// On approval, a new row is created in `approved_tests`.
///
/// **Access:** TheArtificer, TheObserver, or Administrator.
#[tauri::command]
pub async fn decide_test_proposal(
    state: State<'_, AppState>,
    payload: TestProposalDecisionPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheArtificer, Role::TheObserver, Role::Administrator
    ]);

    if payload.decision != "approved" && payload.decision != "rejected" {
        return Err(AppError::Internal(
            "Decision must be 'approved' or 'rejected'.".into(),
        ));
    }

    // Fetch the proposal
    let row: Option<(serde_json::Value, String, Uuid)> = sqlx::query_as(
        r#"
        SELECT proposal_data, status, proposed_by
        FROM test_proposals WHERE id = $1
        "#,
    )
    .bind(payload.proposal_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (proposal_data, old_status, proposer_id) =
        row.ok_or_else(|| AppError::Internal("Test proposal not found.".into()))?;

    if old_status != "pending" {
        return Err(AppError::Internal("Proposal is no longer pending.".into()));
    }

    // Update status
    sqlx::query(
        r#"
        UPDATE test_proposals
        SET status = $2, reviewer_note = $3
        WHERE id = $1
        "#,
    )
    .bind(payload.proposal_id)
    .bind(&payload.decision)
    .bind(&payload.reason)
    .execute(&state.db_pool)
    .await?;

    // If approved → insert into approved_tests
    if payload.decision == "approved" {
        let name = proposal_data.get("name").and_then(|v| v.as_str()).unwrap_or("Unnamed");
        let procedure = proposal_data.get("procedure").and_then(|v| v.as_str()).unwrap_or("");
        let category = proposal_data.get("category").and_then(|v| v.as_str()).unwrap_or("physical");
        let scope = proposal_data.get("species_scope").and_then(|v| v.as_str()).unwrap_or("matter");

        sqlx::query(
            r#"
            INSERT INTO approved_tests (name, procedure, category, applicable_scope, accepted_at)
            VALUES ($1, $2, $3, $4, NOW())
            "#,
        )
        .bind(name)
        .bind(procedure)
        .bind(category)
        .bind(scope)
        .execute(&state.db_pool)
        .await?;
    }

    // Notify the proposer
    let decision_label = if payload.decision == "approved" { "approved" } else { "rejected" };
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        VALUES ($1, 'test_proposal:decided', $2::jsonb)
        "#,
    )
    .bind(proposer_id)
    .bind(serde_json::json!({
        "proposal_id": payload.proposal_id,
        "decision": decision_label,
        "reason": payload.reason,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "test_proposals",
        payload.proposal_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({ "status": old_status })),
        Some(serde_json::json!({
            "status": decision_label,
            "decided_by": user.id,
            "reason": payload.reason,
        })),
    )
    .await?;

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-ART-07 / UC-OBS-07: Final Document Review Queue + Decision
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FinalDocQueueItem {
    pub id: Uuid,
    pub experiment_id: Uuid,
    pub experiment_title: String,
    #[sqlx(rename = "type")]
    pub doc_type: String,
    pub document_data: serde_json::Value,
    pub status: String,
    pub submitted_by: Uuid,
    pub submitter_name: String,
    pub reviewer_note: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// List pending final-object documents for the current director's domain.
///
/// - TheArtificer sees type = 'matter' | 'physical_object'
/// - TheObserver sees type = 'species'
///
/// **Access:** TheArtificer, TheObserver, or Administrator.
#[tauri::command]
pub async fn get_final_document_queue(
    state: State<'_, AppState>,
) -> Result<Vec<FinalDocQueueItem>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheArtificer, Role::TheObserver, Role::Administrator
    ]);

    let type_filter: Vec<&str> = match user.role {
        Role::TheArtificer => vec!["matter", "physical_object"],
        Role::TheObserver  => vec!["species"],
        _                  => vec!["matter", "physical_object", "species"],
    };

    let rows = sqlx::query_as::<_, FinalDocQueueItem>(
        r#"
        SELECT fd.id, fd.experiment_id, e.title AS experiment_title,
               fd.type, fd.document_data, fd.status, fd.submitted_by,
               u.full_name AS submitter_name, fd.reviewer_note, fd.created_at
        FROM final_object_documents fd
        JOIN users u ON u.id = fd.submitted_by
        JOIN experiments e ON e.id = fd.experiment_id
        WHERE fd.status = 'pending_approval'
          AND fd.type = ANY($1)
          AND fd.deleted_at IS NULL
        ORDER BY fd.created_at ASC
        "#,
    )
    .bind(&type_filter)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(rows)
}

#[derive(Debug, Deserialize)]
pub struct FinalDocDecisionPayload {
    pub document_id: Uuid,
    pub decision: String,   // 'approved' or 'rejected'
    pub reason: Option<String>,
}

/// Approve or reject a final-object document.
///
/// On approval, a new row is created in `science_archive`.
///
/// **Access:** TheArtificer, TheObserver, or Administrator.
#[tauri::command]
pub async fn decide_final_document(
    state: State<'_, AppState>,
    payload: FinalDocDecisionPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheArtificer, Role::TheObserver, Role::Administrator
    ]);

    if payload.decision != "approved" && payload.decision != "rejected" {
        return Err(AppError::Internal(
            "Decision must be 'approved' or 'rejected'.".into(),
        ));
    }

    // Fetch the document
    let row: Option<(Uuid, String, serde_json::Value, String, Uuid)> = sqlx::query_as(
        r#"
        SELECT experiment_id, type, document_data, status, submitted_by
        FROM final_object_documents
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.document_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (experiment_id, doc_type, doc_data, old_status, submitter_id) =
        row.ok_or_else(|| AppError::Internal("Final document not found.".into()))?;

    if old_status != "pending_approval" {
        return Err(AppError::Internal("Document is no longer pending approval.".into()));
    }

    let new_status = if payload.decision == "approved" { "approved" } else { "rejected" };

    // Update the final_object_documents row
    sqlx::query(
        r#"
        UPDATE final_object_documents
        SET status = $2, reviewer_note = $3, approved_by = $4, approved_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.document_id)
    .bind(new_status)
    .bind(&payload.reason)
    .bind(if payload.decision == "approved" { Some(user.id) } else { None })
    .execute(&state.db_pool)
    .await?;

    // If approved → insert into science_archive
    if payload.decision == "approved" {
        let name = doc_data.get("name").and_then(|v| v.as_str()).unwrap_or("Unnamed");
        let classification = doc_data.get("classification").and_then(|v| v.as_str());

        sqlx::query(
            r#"
            INSERT INTO science_archive (type, name, classification, detail, experiment_id, submitted_by, approved_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(&doc_type)
        .bind(name)
        .bind(classification)
        .bind(&doc_data)
        .bind(experiment_id)
        .bind(submitter_id)
        .bind(user.id)
        .execute(&state.db_pool)
        .await?;
    }

    // Notify the submitter
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        VALUES ($1, 'final_document:decided', $2::jsonb)
        "#,
    )
    .bind(submitter_id)
    .bind(serde_json::json!({
        "document_id": payload.document_id,
        "experiment_id": experiment_id,
        "doc_type": doc_type,
        "decision": new_status,
        "reason": payload.reason,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "final_object_documents",
        payload.document_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({ "status": old_status })),
        Some(serde_json::json!({
            "status": new_status,
            "decided_by": user.id,
            "reason": payload.reason,
        })),
    )
    .await?;

    // Invalidate science_archive cache for this document type on approval
    if payload.decision == "approved" {
        if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
            let _: Result<(), _> = conn.del(format!("science_archive:{}", doc_type)).await;
            // Also invalidate species_archive caches used by engineers
            if doc_type == "species" {
                let _: Result<(), _> = conn.del("species_archive:plant").await;
                let _: Result<(), _> = conn.del("species_archive:all_species").await;
            }
        }
    }

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-TM-03: Approve or Reject Closure Request
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct ClosureDecisionPayload {
    pub request_id: Uuid,
    pub decision: String,   // 'approved' or 'denied'
    pub reason: Option<String>,
}

/// Approve or reject a closure request (mission completion, experiment conclusion).
///
/// **Access:** TheTaskmaster or Administrator.
#[tauri::command]
pub async fn approve_closure_request(
    state: State<'_, AppState>,
    payload: ClosureDecisionPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::TheTaskmaster, Role::Administrator]);

    if payload.decision != "approved" && payload.decision != "denied" {
        return Err(AppError::Internal(
            "Decision must be 'approved' or 'denied'.".into(),
        ));
    }

    let before: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.request_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (old_status,) = before
        .ok_or_else(|| AppError::Internal("Request not found.".into()))?;

    let new_status = if payload.decision == "approved" { "approved" } else { "denied" };

    sqlx::query(
        r#"
        UPDATE requests SET
          status = $2, decided_by = $3, decision_reason = $4, updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.request_id)
    .bind(new_status)
    .bind(user.id)
    .bind(&payload.reason)
    .execute(&state.db_pool)
    .await?;

    // Notify requester
    let requester_id: Option<(Uuid,)> = sqlx::query_as(
        "SELECT requester_id FROM requests WHERE id = $1",
    )
    .bind(payload.request_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some((req_id,)) = requester_id {
        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, 'request:decided', $2::jsonb)
            "#,
        )
        .bind(req_id)
        .bind(serde_json::json!({
            "request_id": payload.request_id,
            "decision": new_status,
            "reason": payload.reason,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    write_audit_log(
        &state.db_pool,
        "requests",
        payload.request_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({ "status": old_status })),
        Some(serde_json::json!({
            "status": new_status,
            "decided_by": user.id,
            "decision_reason": payload.reason,
        })),
    )
    .await?;

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-ART-03/04, UC-OBS-03/04: Help Request/Response Proxy
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct ProxyDecisionPayload {
    pub task_id: Uuid,
    pub decision: String,   // 'forward', 'reject', 'convert', 'withhold'
    pub reason: Option<String>,
}

/// Review and decide on an incoming help request (proxy queue).
/// Artificer decides for Math/Physics, Observer for Bio/Chem/AgEng/BioEng.
///
/// **Access:** TheArtificer, TheObserver, TheTaskmaster, or Administrator.
#[tauri::command]
pub async fn review_help_request(
    state: State<'_, AppState>,
    payload: ProxyDecisionPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheArtificer, Role::TheObserver, Role::TheTaskmaster, Role::Administrator
    ]);

    let valid_decisions = ["forward", "reject", "convert"];
    if !valid_decisions.contains(&payload.decision.as_str()) {
        return Err(AppError::Internal(
            "Decision must be 'forward', 'reject', or 'convert'.".into(),
        ));
    }

    let new_status = match payload.decision.as_str() {
        "forward" => "in_progress",
        "reject" => "rejected",
        "convert" => "pending", // converted to formal task
        _ => unreachable!(),
    };

    sqlx::query(
        r#"
        UPDATE tasks SET
          status = $2,
          payload = payload || $3::jsonb,
          updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.task_id)
    .bind(new_status)
    .bind(serde_json::json!({
        "proxy_decision": payload.decision,
        "proxy_reason": payload.reason,
        "proxy_decided_by": user.id,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "tasks",
        payload.task_id,
        AuditOperation::Update,
        user.id,
        None,
        Some(serde_json::json!({
            "proxy_decision": payload.decision,
            "new_status": new_status,
        })),
    )
    .await?;

    Ok(())
}

/// Review and decide on an outgoing help response (proxy queue).
///
/// **Access:** TheArtificer, TheObserver, TheTaskmaster, or Administrator.
#[tauri::command]
pub async fn review_help_response(
    state: State<'_, AppState>,
    payload: ProxyDecisionPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheArtificer, Role::TheObserver, Role::TheTaskmaster, Role::Administrator
    ]);

    let valid_decisions = ["forward", "withhold"];
    if !valid_decisions.contains(&payload.decision.as_str()) {
        return Err(AppError::Internal(
            "Decision must be 'forward' or 'withhold'.".into(),
        ));
    }

    let new_status = match payload.decision.as_str() {
        "forward" => "completed",
        "withhold" => "rejected",
        _ => unreachable!(),
    };

    sqlx::query(
        r#"
        UPDATE tasks SET
          status = $2,
          payload = payload || $3::jsonb,
          updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.task_id)
    .bind(new_status)
    .bind(serde_json::json!({
        "response_proxy_decision": payload.decision,
        "response_proxy_reason": payload.reason,
        "response_proxy_decided_by": user.id,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "tasks",
        payload.task_id,
        AuditOperation::Update,
        user.id,
        None,
        Some(serde_json::json!({
            "response_proxy_decision": payload.decision,
            "new_status": new_status,
        })),
    )
    .await?;

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-WAN-04: Rename Territory
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct RenameTerritoryPayload {
    pub territory_id: Uuid,
    pub new_name: String,
}

/// Rename a planet or sector territory.
///
/// **Access:** TheWanderer or Administrator.
#[tauri::command]
pub async fn rename_territory(
    state: State<'_, AppState>,
    payload: RenameTerritoryPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::TheWanderer, Role::Administrator]);

    if payload.new_name.trim().is_empty() {
        return Err(AppError::Internal("Territory name cannot be empty.".into()));
    }

    let old: Option<(String, String)> = sqlx::query_as(
        "SELECT name, territory_type FROM territory_names WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.territory_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (old_name, territory_type) = old
        .ok_or_else(|| AppError::Internal("Territory not found.".into()))?;

    sqlx::query(
        r#"
        UPDATE territory_names SET
          previous_name = name,
          name = $2,
          renamed_by = $3,
          renamed_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(payload.territory_id)
    .bind(&payload.new_name)
    .bind(user.id)
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "territory_names",
        payload.territory_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({
            "name": old_name,
            "territory_type": territory_type,
        })),
        Some(serde_json::json!({
            "name": payload.new_name,
            "previous_name": old_name,
        })),
    )
    .await?;

    Ok(())
}

/// Get all territories for the renaming interface.
///
/// **Access:** TheWanderer or Administrator.
#[derive(Debug, Serialize, FromRow)]
pub struct TerritorySummary {
    pub id: Uuid,
    pub territory_type: String,
    pub name: String,
    pub previous_name: Option<String>,
    pub renamed_at: DateTime<Utc>,
}

#[tauri::command]
pub async fn get_territories(
    state: State<'_, AppState>,
) -> Result<Vec<TerritorySummary>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::TheWanderer, Role::Administrator]);

    let territories = sqlx::query_as::<_, TerritorySummary>(
        r#"
        SELECT id, territory_type, name, previous_name, renamed_at
        FROM territory_names
        WHERE deleted_at IS NULL
        ORDER BY name ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(territories)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-GUA-02/03: Broadcast Approval + Emergency Broadcast
// UC-ANC-00/01/02: Broadcast Request + Informational Broadcast
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct BroadcastRequestPayload {
    pub type_: String,          // 'security' or 'informational'
    pub subject: String,
    pub content: String,
    pub target_scope: Option<String>,
    pub target_ids: Option<Vec<Uuid>>,
    pub urgency: Option<String>,
    pub rationale: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BroadcastDecisionPayload {
    pub request_id: Uuid,
    pub decision: String,       // 'approved' or 'rejected'
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct BroadcastRequestSummary {
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

/// Submit a broadcast request.
///
/// **Access:** Any authenticated user (non-security → informational → TheAnchorman;
///             security staff → security → TheGuardian).
#[tauri::command]
pub async fn submit_broadcast_request(
    state: State<'_, AppState>,
    payload: BroadcastRequestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    let req_type = &payload.type_;
    if req_type != "security" && req_type != "informational" {
        return Err(AppError::Internal(
            "Broadcast type must be 'security' or 'informational'.".into(),
        ));
    }

    let req_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO broadcast_requests
          (requester_id, type, subject, content, target_scope, target_ids, urgency, rationale)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(req_type)
    .bind(&payload.subject)
    .bind(&payload.content)
    .bind(payload.target_scope.as_deref().unwrap_or("company_wide"))
    .bind(&payload.target_ids)
    .bind(payload.urgency.as_deref().unwrap_or("normal"))
    .bind(&payload.rationale)
    .fetch_one(&state.db_pool)
    .await?;

    // Notify the appropriate decision maker
    let decider_role = if req_type == "security" { "TheGuardian" } else { "TheAnchorman" };
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        SELECT u.id, 'broadcast_request:new', $2::jsonb
        FROM users u JOIN roles r ON r.id = u.role_id
        WHERE r.name = $1 AND u.deleted_at IS NULL AND u.is_active = TRUE
        "#,
    )
    .bind(decider_role)
    .bind(serde_json::json!({
        "request_id": req_id.0,
        "subject": payload.subject,
        "type": req_type,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "broadcast_requests",
        req_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "type": req_type,
            "subject": payload.subject,
        })),
    )
    .await?;

    Ok(req_id.0)
}

/// Get broadcast requests for review.
///
/// **Access:** TheGuardian (security), TheAnchorman (informational), or Administrator (all).
#[tauri::command]
pub async fn get_broadcast_request_queue(
    state: State<'_, AppState>,
) -> Result<Vec<BroadcastRequestSummary>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheGuardian, Role::TheAnchorman, Role::Administrator
    ]);

    let type_filter = match user.role {
        Role::TheGuardian => Some("security"),
        Role::TheAnchorman => Some("informational"),
        Role::Administrator => None,
        _ => return Err(AppError::Forbidden),
    };

    let requests = if let Some(tf) = type_filter {
        sqlx::query_as::<_, BroadcastRequestSummary>(
            r#"
            SELECT br.id, br.requester_id, u.full_name AS requester_name,
                   br.type AS type_, br.subject, br.content, br.target_scope,
                   br.urgency, br.rationale, br.status, br.created_at
            FROM broadcast_requests br
            JOIN users u ON u.id = br.requester_id
            WHERE br.status = 'pending' AND br.type = $1 AND br.deleted_at IS NULL
            ORDER BY br.created_at ASC
            "#,
        )
        .bind(tf)
        .fetch_all(&state.db_pool)
        .await?
    } else {
        sqlx::query_as::<_, BroadcastRequestSummary>(
            r#"
            SELECT br.id, br.requester_id, u.full_name AS requester_name,
                   br.type AS type_, br.subject, br.content, br.target_scope,
                   br.urgency, br.rationale, br.status, br.created_at
            FROM broadcast_requests br
            JOIN users u ON u.id = br.requester_id
            WHERE br.status = 'pending' AND br.deleted_at IS NULL
            ORDER BY br.created_at ASC
            "#,
        )
        .fetch_all(&state.db_pool)
        .await?
    };

    Ok(requests)
}

/// Approve or reject a broadcast request (bypasses Directors voting).
///
/// **Access:** TheGuardian (security), TheAnchorman (informational), or Administrator.
#[tauri::command]
pub async fn decide_broadcast_request(
    state: State<'_, AppState>,
    payload: BroadcastDecisionPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheGuardian, Role::TheAnchorman, Role::Administrator
    ]);

    if payload.decision != "approved" && payload.decision != "rejected" {
        return Err(AppError::Internal(
            "Decision must be 'approved' or 'rejected'.".into(),
        ));
    }

    let new_status = if payload.decision == "approved" { "approved" } else { "rejected" };

    sqlx::query(
        r#"
        UPDATE broadcast_requests SET
          status = $2, decided_by = $3, decision_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.request_id)
    .bind(new_status)
    .bind(user.id)
    .execute(&state.db_pool)
    .await?;

    // If approved, mark as sent automatically
    if payload.decision == "approved" {
        sqlx::query(
            "UPDATE broadcast_requests SET status = 'sent', sent_at = NOW() WHERE id = $1",
        )
        .bind(payload.request_id)
        .execute(&state.db_pool)
        .await?;

        // Notify requester
        let requester_id: Option<(Uuid,)> = sqlx::query_as(
            "SELECT requester_id FROM broadcast_requests WHERE id = $1",
        )
        .bind(payload.request_id)
        .fetch_optional(&state.db_pool)
        .await?;

        if let Some((req_id,)) = requester_id {
            sqlx::query(
                r#"
                INSERT INTO notifications (user_id, type, payload)
                VALUES ($1, 'broadcast_request:decided', $2::jsonb)
                "#,
            )
            .bind(req_id)
            .bind(serde_json::json!({
                "request_id": payload.request_id,
                "decision": "approved",
            }))
            .execute(&state.db_pool)
            .await?;
        }
    }

    write_audit_log(
        &state.db_pool,
        "broadcast_requests",
        payload.request_id,
        AuditOperation::Update,
        user.id,
        None,
        Some(serde_json::json!({
            "status": new_status,
            "decided_by": user.id,
        })),
    )
    .await?;

    Ok(())
}

/// Send an emergency broadcast directly (no request needed).
///
/// **Access:** TheGuardian or Administrator.
#[tauri::command]
pub async fn send_emergency_broadcast(
    state: State<'_, AppState>,
    payload: BroadcastRequestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::TheGuardian, Role::Administrator]);

    let req_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO broadcast_requests
          (requester_id, type, subject, content, target_scope, target_ids, urgency, status, decided_by, decision_at, sent_at)
        VALUES ($1, 'security', $2, $3, $4, $5, 'critical', 'sent', $1, NOW(), NOW())
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.subject)
    .bind(&payload.content)
    .bind(payload.target_scope.as_deref().unwrap_or("company_wide"))
    .bind(&payload.target_ids)
    .fetch_one(&state.db_pool)
    .await?;

    // Also insert into the messaging system so it appears in the broadcast inbox
    let msg_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO messages (from_id, subject, body, channel)
        VALUES ($1, $2, $3, 'broadcast')
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.subject)
    .bind(&payload.content)
    .fetch_one(&state.db_pool)
    .await?;

    // Add all active users as 'to' recipients so the broadcast appears in everyone's inbox
    sqlx::query(
        r#"
        INSERT INTO message_recipients (message_id, user_id, type)
        SELECT $1, u.id, 'to'
        FROM users u
        WHERE u.deleted_at IS NULL AND u.is_active = TRUE
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(msg_row.0)
    .execute(&state.db_pool)
    .await?;

    // Notify all active users (broadcast)
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        SELECT u.id, 'broadcast:emergency', $1::jsonb
        FROM users u
        WHERE u.deleted_at IS NULL AND u.is_active = TRUE
        "#,
    )
    .bind(serde_json::json!({
        "broadcast_id": req_id.0,
        "message_id": msg_row.0,
        "subject": payload.subject,
        "content": payload.content,
        "urgency": "critical",
        "from": user.full_name,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "broadcast_requests",
        req_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "type": "emergency",
            "subject": payload.subject,
            "sent_immediately": true,
        })),
    )
    .await?;

    Ok(req_id.0)
}

/// Send an informational broadcast directly.
///
/// **Access:** TheAnchorman or Administrator.
#[tauri::command]
pub async fn send_informational_broadcast(
    state: State<'_, AppState>,
    payload: BroadcastRequestPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::TheAnchorman, Role::Administrator]);

    let req_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO broadcast_requests
          (requester_id, type, subject, content, target_scope, target_ids, urgency, status, decided_by, decision_at, sent_at)
        VALUES ($1, 'informational', $2, $3, $4, $5, $6, 'sent', $1, NOW(), NOW())
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.subject)
    .bind(&payload.content)
    .bind(payload.target_scope.as_deref().unwrap_or("company_wide"))
    .bind(&payload.target_ids)
    .bind(payload.urgency.as_deref().unwrap_or("normal"))
    .fetch_one(&state.db_pool)
    .await?;

    // Also insert into the messaging system so it appears in the broadcast inbox
    let msg_row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO messages (from_id, subject, body, channel)
        VALUES ($1, $2, $3, 'broadcast')
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.subject)
    .bind(&payload.content)
    .fetch_one(&state.db_pool)
    .await?;

    // Add all active users as 'to' recipients
    sqlx::query(
        r#"
        INSERT INTO message_recipients (message_id, user_id, type)
        SELECT $1, u.id, 'to'
        FROM users u
        WHERE u.deleted_at IS NULL AND u.is_active = TRUE
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(msg_row.0)
    .execute(&state.db_pool)
    .await?;

    // Notify targets
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        SELECT u.id, 'broadcast:informational', $1::jsonb
        FROM users u
        WHERE u.deleted_at IS NULL AND u.is_active = TRUE
        "#,
    )
    .bind(serde_json::json!({
        "broadcast_id": req_id.0,
        "message_id": msg_row.0,
        "subject": payload.subject,
        "content": payload.content,
        "from": user.full_name,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "broadcast_requests",
        req_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "type": "informational",
            "subject": payload.subject,
        })),
    )
    .await?;

    Ok(req_id.0)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-STAT-01/02/03: Data Request Gating (Statistician)
// Now queries the dedicated data_requests / data_responses tables
// (see 02_DATA_ANALYSTS.md + migration 20250101000006).
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DataRequestSummary {
    pub id: Uuid,
    pub dataset_description: String,
    pub scope: String,
    pub purpose: String,
    pub urgency: String,
    pub sensitivity_note: Option<String>,
    pub status: String,
    pub requester_id: Uuid,
    pub requester_name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct DataRequestDecisionPayload {
    pub request_id: Uuid,
    pub decision: String,    // 'approved' or 'rejected'
    pub reason: Option<String>,
}

/// Payload for outbound data response review (UC-STAT-03).
#[derive(Debug, Deserialize)]
pub struct OutboundDataReviewPayload {
    pub response_id: Uuid,
    pub decision: String,    // 'forward' or 'withhold'
    pub reason: Option<String>,
}

/// Summary of a data response awaiting outbound review.
#[derive(Debug, Serialize, FromRow)]
pub struct OutboundResponseSummary {
    pub id: Uuid,
    pub request_id: Uuid,
    pub analyst_name: String,
    pub requester_name: String,
    pub dataset_description: String,
    pub result_payload: Option<serde_json::Value>,
    pub spreadsheet_storage_path: Option<String>,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
}

/// Get data requests for The Statistician's review queue (UC-STAT-01).
/// Returns requests with status = 'pending_statistician'.
///
/// **Access:** TheStatistician or Administrator.
#[tauri::command]
pub async fn get_data_request_queue(
    state: State<'_, AppState>,
) -> Result<Vec<DataRequestSummary>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::TheStatistician, Role::Administrator]);

    // Check Redis cache
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(cached) = conn
            .get::<_, String>("data_requests:statistician_queue")
            .await
        {
            if let Ok(parsed) = serde_json::from_str::<Vec<DataRequestSummary>>(&cached) {
                return Ok(parsed);
            }
        }
    }

    let requests = sqlx::query_as::<_, DataRequestSummary>(
        r#"
        SELECT dr.id, dr.dataset_description, dr.scope, dr.purpose,
               dr.urgency, dr.sensitivity_note, dr.status,
               dr.requested_by AS requester_id,
               u.full_name AS requester_name,
               dr.created_at
        FROM data_requests dr
        JOIN users u ON u.id = dr.requested_by
        WHERE dr.status = 'pending_statistician'
          AND dr.deleted_at IS NULL
        ORDER BY
          CASE dr.urgency
            WHEN 'critical' THEN 0
            WHEN 'high' THEN 1
            WHEN 'medium' THEN 2
            WHEN 'low' THEN 3
          END,
          dr.created_at ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    // Cache for 5 minutes
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        if let Ok(serialized) = serde_json::to_string(&requests) {
            let _: Result<(), _> = conn
                .set_ex("data_requests:statistician_queue", &serialized, 300)
                .await;
        }
    }

    Ok(requests)
}

/// Approve or reject a data request (UC-STAT-02). Bypasses Directors voting.
/// On approval: status → 'approved', forwarded to Data Analyst team.
/// On rejection: status → 'rejected', requester notified.
///
/// **Access:** TheStatistician or Administrator.
#[tauri::command]
pub async fn decide_data_request(
    state: State<'_, AppState>,
    payload: DataRequestDecisionPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::TheStatistician, Role::Administrator]);

    if payload.decision != "approved" && payload.decision != "rejected" {
        return Err(AppError::Internal(
            "Decision must be 'approved' or 'rejected'.".into(),
        ));
    }

    // Fetch before-state for audit
    let before: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM data_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(payload.request_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if before.is_none() {
        return Err(AppError::Internal("Data request not found.".into()));
    }

    let new_status = &payload.decision; // 'approved' or 'rejected'

    sqlx::query(
        r#"
        UPDATE data_requests SET
          status = $2,
          statistician_decision_reason = $3,
          updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(payload.request_id)
    .bind(new_status)
    .bind(&payload.reason)
    .execute(&state.db_pool)
    .await?;

    // Notify requester of the decision
    let requester_id: Option<(Uuid,)> = sqlx::query_as(
        "SELECT requested_by FROM data_requests WHERE id = $1",
    )
    .bind(payload.request_id)
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some((req_id,)) = requester_id {
        let notif_type = if payload.decision == "approved" {
            "data_request:approved"
        } else {
            "data_request:rejected"
        };

        sqlx::query(
            r#"
            INSERT INTO notifications (user_id, type, payload)
            VALUES ($1, $2, $3::jsonb)
            "#,
        )
        .bind(req_id)
        .bind(notif_type)
        .bind(serde_json::json!({
            "request_id": payload.request_id,
            "decision": payload.decision,
            "reason": payload.reason,
        }))
        .execute(&state.db_pool)
        .await?;
    }

    // If approved, also notify Data Analyst team
    if payload.decision == "approved" {
        let analysts: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT u.id FROM users u
            JOIN roles r ON r.id = u.role_id
            WHERE r.name = 'DataAnalyst'
              AND u.deleted_at IS NULL
            "#,
        )
        .fetch_all(&state.db_pool)
        .await?;

        for (analyst_id,) in &analysts {
            sqlx::query(
                r#"
                INSERT INTO notifications (user_id, type, payload)
                VALUES ($1, 'data_request:forwarded', $2::jsonb)
                "#,
            )
            .bind(analyst_id)
            .bind(serde_json::json!({
                "request_id": payload.request_id,
                "message": "A new data request has been approved and is ready for processing.",
            }))
            .execute(&state.db_pool)
            .await?;
        }
    }

    // Invalidate caches
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del("data_requests:statistician_queue").await;
        let _: Result<(), _> = conn.del("data_requests:analyst_inbox").await;
        let _: Result<(), _> = conn
            .del(format!("data_request:status:{}", payload.request_id))
            .await;
    }

    write_audit_log(
        &state.db_pool,
        "data_requests",
        payload.request_id,
        AuditOperation::Update,
        user.id,
        Some(serde_json::json!({ "status": "pending_statistician" })),
        Some(serde_json::json!({
            "status": new_status,
            "statistician_decision_reason": payload.reason,
        })),
    )
    .await?;

    Ok(())
}

/// Get outbound data responses pending Statistician review (UC-STAT-03).
///
/// **Access:** TheStatistician or Administrator.
#[tauri::command]
pub async fn get_outbound_review_queue(
    state: State<'_, AppState>,
) -> Result<Vec<OutboundResponseSummary>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::TheStatistician, Role::Administrator]);

    let rows = sqlx::query_as::<_, OutboundResponseSummary>(
        r#"
        SELECT resp.id, resp.request_id,
               analyst.full_name AS analyst_name,
               requester.full_name AS requester_name,
               dr.dataset_description,
               resp.result_payload, resp.spreadsheet_storage_path,
               resp.status, resp.submitted_at
        FROM data_responses resp
        JOIN users analyst ON analyst.id = resp.prepared_by
        JOIN data_requests dr ON dr.id = resp.request_id
        JOIN users requester ON requester.id = dr.requested_by
        WHERE resp.status = 'pending_outbound_review'
        ORDER BY resp.submitted_at ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(rows)
}

/// Review outbound data response before delivery to requester (UC-STAT-03).
/// On 'forward': response cleared, delivered to requester.
/// On 'withhold': response returned to analyst with revision notes.
///
/// **Access:** TheStatistician or Administrator.
#[tauri::command]
pub async fn review_outbound_data_response(
    state: State<'_, AppState>,
    payload: OutboundDataReviewPayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::TheStatistician, Role::Administrator]);

    let valid_decisions = ["forward", "withhold"];
    if !valid_decisions.contains(&payload.decision.as_str()) {
        return Err(AppError::Internal(
            "Decision must be 'forward' or 'withhold'.".into(),
        ));
    }

    if payload.decision == "forward" {
        // Clear the response → deliver to requester
        sqlx::query(
            r#"
            UPDATE data_responses SET
              status = 'cleared',
              statistician_review_note = $2,
              cleared_at = NOW(),
              delivered_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(payload.response_id)
        .bind(&payload.reason)
        .execute(&state.db_pool)
        .await?;

        // Update the parent data_request to 'delivered'
        let request_id: Option<(Uuid,)> = sqlx::query_as(
            "SELECT request_id FROM data_responses WHERE id = $1",
        )
        .bind(payload.response_id)
        .fetch_optional(&state.db_pool)
        .await?;

        if let Some((req_id,)) = request_id {
            sqlx::query(
                "UPDATE data_requests SET status = 'delivered', updated_at = NOW() WHERE id = $1",
            )
            .bind(req_id)
            .execute(&state.db_pool)
            .await?;

            // Notify the original requester
            let requester: Option<(Uuid,)> = sqlx::query_as(
                "SELECT requested_by FROM data_requests WHERE id = $1",
            )
            .bind(req_id)
            .fetch_optional(&state.db_pool)
            .await?;

            if let Some((requester_id,)) = requester {
                sqlx::query(
                    r#"
                    INSERT INTO notifications (user_id, type, payload)
                    VALUES ($1, 'data_response:delivered', $2::jsonb)
                    "#,
                )
                .bind(requester_id)
                .bind(serde_json::json!({
                    "request_id": req_id,
                    "response_id": payload.response_id,
                    "message": "Your data request has been fulfilled and the response is ready.",
                }))
                .execute(&state.db_pool)
                .await?;
            }

            // Invalidate status cache
            if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
                let _: Result<(), _> = conn
                    .del(format!("data_request:status:{}", req_id))
                    .await;
            }
        }
    } else {
        // Withhold — return to analyst for revision
        sqlx::query(
            r#"
            UPDATE data_responses SET
              status = 'withheld',
              statistician_review_note = $2
            WHERE id = $1
            "#,
        )
        .bind(payload.response_id)
        .bind(&payload.reason)
        .execute(&state.db_pool)
        .await?;

        // Reset the parent data_request back to 'processing' so analyst can revise
        let request_id: Option<(Uuid,)> = sqlx::query_as(
            "SELECT request_id FROM data_responses WHERE id = $1",
        )
        .bind(payload.response_id)
        .fetch_optional(&state.db_pool)
        .await?;

        if let Some((req_id,)) = request_id {
            sqlx::query(
                "UPDATE data_requests SET status = 'processing', updated_at = NOW() WHERE id = $1",
            )
            .bind(req_id)
            .execute(&state.db_pool)
            .await?;

            // Notify the analyst who prepared the response
            let analyst_id: Option<(Uuid,)> = sqlx::query_as(
                "SELECT prepared_by FROM data_responses WHERE id = $1",
            )
            .bind(payload.response_id)
            .fetch_optional(&state.db_pool)
            .await?;

            if let Some((a_id,)) = analyst_id {
                sqlx::query(
                    r#"
                    INSERT INTO notifications (user_id, type, payload)
                    VALUES ($1, 'data_response:withheld', $2::jsonb)
                    "#,
                )
                .bind(a_id)
                .bind(serde_json::json!({
                    "request_id": req_id,
                    "response_id": payload.response_id,
                    "reason": payload.reason,
                    "message": "Your data response was withheld. Please revise and resubmit.",
                }))
                .execute(&state.db_pool)
                .await?;
            }

            // Invalidate caches
            if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
                let _: Result<(), _> = conn
                    .del(format!("data_request:status:{}", req_id))
                    .await;
                let _: Result<(), _> = conn.del("data_requests:analyst_inbox").await;
            }
        }
    }

    write_audit_log(
        &state.db_pool,
        "data_responses",
        payload.response_id,
        AuditOperation::Update,
        user.id,
        None,
        Some(serde_json::json!({
            "outbound_review_decision": payload.decision,
            "outbound_review_reason": payload.reason,
        })),
    )
    .await?;

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-CORD-01/02/03: Event Management (Coordinator)
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct CreateEventPayload {
    pub title: String,
    pub event_date: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub agenda: Option<String>,
    pub attendees: Vec<EventAttendeeInput>,
}

#[derive(Debug, Deserialize)]
pub struct EventAttendeeInput {
    pub name: String,
    pub is_external: bool,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EventSummary {
    pub id: Uuid,
    pub title: String,
    pub event_date: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub agenda: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct EventDocSummary {
    pub id: Uuid,
    pub event_id: Uuid,
    pub document_type: Option<String>,
    pub original_filename: String,
    pub content_type: String,
    pub storage_path: String,
    pub uploaded_at: DateTime<Utc>,
}

/// Create a company event or external meeting.
///
/// **Access:** TheCoordinator or Administrator.
#[tauri::command]
pub async fn create_event(
    state: State<'_, AppState>,
    payload: CreateEventPayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::TheCoordinator, Role::Administrator]);

    if payload.title.trim().is_empty() {
        return Err(AppError::Internal("Event title cannot be empty.".into()));
    }

    let event_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO events (title, event_date, location, agenda, created_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
    )
    .bind(&payload.title)
    .bind(payload.event_date)
    .bind(&payload.location)
    .bind(&payload.agenda)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Insert attendees
    for att in &payload.attendees {
        sqlx::query(
            r#"
            INSERT INTO event_attendees (event_id, name, is_external, user_id)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(event_id.0)
        .bind(&att.name)
        .bind(att.is_external)
        .bind(att.user_id)
        .execute(&state.db_pool)
        .await?;

        // Notify internal attendees
        if let Some(uid) = att.user_id {
            sqlx::query(
                r#"
                INSERT INTO notifications (user_id, type, payload)
                VALUES ($1, 'event:invited', $2::jsonb)
                "#,
            )
            .bind(uid)
            .bind(serde_json::json!({
                "event_id": event_id.0,
                "title": payload.title,
                "event_date": payload.event_date,
            }))
            .execute(&state.db_pool)
            .await?;
        }
    }

    write_audit_log(
        &state.db_pool,
        "events",
        event_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "title": payload.title,
            "event_date": payload.event_date,
            "attendee_count": payload.attendees.len(),
        })),
    )
    .await?;

    Ok(event_id.0)
}

/// Get all events for the archive view.
///
/// **Access:** TheCoordinator or Administrator.
#[tauri::command]
pub async fn get_events(
    state: State<'_, AppState>,
) -> Result<Vec<EventSummary>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::TheCoordinator, Role::Administrator]);

    let events = sqlx::query_as::<_, EventSummary>(
        r#"
        SELECT id, title, event_date, location, agenda, created_by, created_at
        FROM events
        WHERE deleted_at IS NULL
        ORDER BY event_date DESC NULLS LAST
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(events)
}

/// Upload an event document (UC-CORD-02). File is uploaded to Supabase Storage.
///
/// **Access:** TheCoordinator or Administrator.
#[tauri::command]
pub async fn upload_event_document(
    state: State<'_, AppState>,
    event_id: Uuid,
    file_bytes: Vec<u8>,
    filename: String,
    content_type: String,
    document_type: String,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [Role::TheCoordinator, Role::Administrator]);

    // Generate storage path
    let file_uuid = Uuid::new_v4();
    let storage_path = format!("events/{}/{}-{}", event_id, file_uuid, filename);

    // Upload to Supabase Storage (rusa-files bucket)
    // Ensure bucket exists (idempotent — Supabase returns 409 if already exists)
    let client = reqwest::Client::new();
    let _ = client
        .post(&format!("{}/bucket", state.supabase_storage_url))
        .header("Authorization", format!("Bearer {}", state.supabase_service_jwt))
        .header("Content-Type", "application/json")
        .body(r#"{"id":"rusa-files","name":"rusa-files","public":false}"#)
        .send()
        .await;

    let upload_url = format!(
        "{}/object/rusa-files/{}",
        state.supabase_storage_url, storage_path
    );

    let resp = client
        .post(&upload_url)
        .header("Authorization", format!("Bearer {}", state.supabase_service_jwt))
        .header("Content-Type", &content_type)
        .body(file_bytes)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Storage upload failed: {}", e)))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Internal(format!(
            "Storage upload returned {}: {}",
            status, body
        )));
    }

    // Insert record in DB
    let doc_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO event_documents
          (event_id, document_type, original_filename, content_type, storage_path, uploaded_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id
        "#,
    )
    .bind(event_id)
    .bind(&document_type)
    .bind(&filename)
    .bind(&content_type)
    .bind(&storage_path)
    .bind(user.id)
    .fetch_one(&state.db_pool)
    .await?;

    // Invalidate cached signed URLs
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn.del(format!("event_docs:signed_urls:{}", event_id)).await;
    }

    write_audit_log(
        &state.db_pool,
        "event_documents",
        doc_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "event_id": event_id,
            "document_type": document_type,
            "filename": filename,
            "storage_path": storage_path,
        })),
    )
    .await?;

    Ok(doc_id.0)
}

/// Get event documents with signed URLs (UC-CORD-03).
///
/// **Access:** TheCoordinator or Administrator.
#[tauri::command]
pub async fn get_event_documents(
    state: State<'_, AppState>,
    event_id: Uuid,
) -> Result<Vec<EventDocSummary>, AppError> {
    let _user = crate::require_auth_any!(state, [Role::TheCoordinator, Role::Administrator]);

    let docs = sqlx::query_as::<_, EventDocSummary>(
        r#"
        SELECT id, event_id, document_type, original_filename, content_type,
               storage_path, uploaded_at
        FROM event_documents
        WHERE event_id = $1 AND deleted_at IS NULL
        ORDER BY uploaded_at DESC
        "#,
    )
    .bind(event_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(docs)
}

// ════════════════════════════════════════════════════════════════════════════════
// UC-ANC-03: Terminate Personnel (with record)
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct TerminatePayload {
    pub target_user_id: Uuid,
    pub reason: String,
    pub effective_date: NaiveDate,
}

/// Terminate a non-Director, non-Administrator staff member.
/// Creates a termination record, soft-deletes the user, notifies relevant parties.
///
/// **Access:** TheAnchorman or Administrator.
#[tauri::command]
pub async fn terminate_personnel(
    state: State<'_, AppState>,
    payload: TerminatePayload,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [Role::TheAnchorman, Role::Administrator]);

    if user.id == payload.target_user_id {
        return Err(AppError::Internal("Cannot terminate yourself.".into()));
    }

    // Check target is not a director or admin
    let target_role: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT r.name FROM users u JOIN roles r ON r.id = u.role_id
        WHERE u.id = $1 AND u.deleted_at IS NULL
        "#,
    )
    .bind(payload.target_user_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (role_name,) = target_role
        .ok_or_else(|| AppError::Internal("User not found or already terminated.".into()))?;

    let target_r = Role::from_str(&role_name).map_err(AppError::Internal)?;
    if target_r.is_director() && user.role != Role::Administrator {
        return Err(AppError::Forbidden);
    }

    // Create termination record
    let term_id: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO termination_records (terminated_user_id, terminated_by, reason, effective_date)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(payload.target_user_id)
    .bind(user.id)
    .bind(&payload.reason)
    .bind(payload.effective_date)
    .fetch_one(&state.db_pool)
    .await?;

    // Soft-delete user
    sqlx::query(
        r#"
        UPDATE users SET
          deleted_at = NOW(), deleted_by = $1, is_active = FALSE, updated_at = NOW()
        WHERE id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(user.id)
    .bind(payload.target_user_id)
    .execute(&state.db_pool)
    .await?;

    // Invalidate Redis
    if let Ok(mut conn) = state.redis_client.get_multiplexed_async_connection().await {
        let _: Result<(), _> = conn
            .del(&[
                format!("session:{}", payload.target_user_id),
                format!("role:{}", payload.target_user_id),
            ])
            .await;
    }

    // Notify all directors about termination
    sqlx::query(
        r#"
        INSERT INTO notifications (user_id, type, payload)
        SELECT u.id, 'termination:issued', $1::jsonb
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
        "termination_id": term_id.0,
        "terminated_user": payload.target_user_id,
        "reason": payload.reason,
        "effective_date": payload.effective_date,
        "terminated_by": user.full_name,
    }))
    .execute(&state.db_pool)
    .await?;

    write_audit_log(
        &state.db_pool,
        "termination_records",
        term_id.0,
        AuditOperation::Create,
        user.id,
        None,
        Some(serde_json::json!({
            "terminated_user_id": payload.target_user_id,
            "reason": payload.reason,
            "effective_date": payload.effective_date,
        })),
    )
    .await?;

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// Notifications (shared — used by all directors)
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, FromRow)]
pub struct NotificationSummary {
    pub id: Uuid,
    pub type_: String,
    pub payload: serde_json::Value,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Get unread notifications for the current user.
///
/// **Access:** Any authenticated user.
#[tauri::command]
pub async fn get_notifications(
    state: State<'_, AppState>,
) -> Result<Vec<NotificationSummary>, AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    let notifs = sqlx::query_as::<_, NotificationSummary>(
        r#"
        SELECT id, type AS type_, payload, read_at, created_at
        FROM notifications
        WHERE user_id = $1 AND deleted_at IS NULL AND read_at IS NULL
        ORDER BY created_at DESC
        LIMIT 50
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(notifs)
}

/// Mark a notification as read.
///
/// **Access:** Any authenticated user (own notifications only).
#[tauri::command]
pub async fn mark_notification_read(
    state: State<'_, AppState>,
    notification_id: Uuid,
) -> Result<(), AppError> {
    let user = crate::auth::get_current_user(&state).await?;

    sqlx::query(
        "UPDATE notifications SET read_at = NOW() WHERE id = $1 AND user_id = $2",
    )
    .bind(notification_id)
    .bind(user.id)
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// Personnel Listing (for pickers in forms)
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, FromRow)]
pub struct PersonnelListItem {
    pub id: Uuid,
    pub full_name: String,
    pub username: String,
    pub role_name: String,
}

/// Get all active personnel for use in pickers (invitee, assignee, etc.).
///
/// **Access:** Any Director or Administrator.
#[tauri::command]
pub async fn get_personnel_list(
    state: State<'_, AppState>,
) -> Result<Vec<PersonnelListItem>, AppError> {
    let _user = crate::require_auth_director!(state);

    let personnel = sqlx::query_as::<_, PersonnelListItem>(
        r#"
        SELECT u.id, u.full_name, u.username, r.name AS role_name
        FROM users u
        JOIN roles r ON r.id = u.role_id
        WHERE u.deleted_at IS NULL AND u.is_active = TRUE
        ORDER BY u.full_name ASC
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(personnel)
}

// ════════════════════════════════════════════════════════════════════════════════
// Secure Messaging (UC-GUA-04, UC-OVR-02)
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct SendMessagePayload {
    pub subject: String,
    pub body: String,
    pub channel: String,                 // 'security'
    pub recipients_to: Vec<Uuid>,
    pub recipients_cc: Option<Vec<Uuid>>,
    pub recipients_bcc: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MessageSummary {
    pub id: Uuid,
    pub from_id: Uuid,
    pub from_name: String,
    pub subject: String,
    pub body: String,
    pub channel: String,
    pub recalled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct MessageRecipientRow {
    pub user_id: Uuid,
    pub full_name: String,
    #[sqlx(rename = "type")]
    pub type_: String,
    pub read_at: Option<DateTime<Utc>>,
}

/// Send a secure message on the security channel (UC-GUA-04).
///
/// **Access:** TheGuardian, TheOverseer, or Administrator.
/// (TheOverseer gets read-only in the UI, but the backend allows
///  Guardian and Administrator to compose.)
#[tauri::command]
pub async fn send_security_message(
    state: State<'_, AppState>,
    payload: SendMessagePayload,
) -> Result<Uuid, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheGuardian, Role::Administrator
    ]);

    if payload.channel != "security" {
        return Err(AppError::Internal("Only 'security' channel is supported here.".into()));
    }
    if payload.recipients_to.is_empty() {
        return Err(AppError::Internal("At least one 'To' recipient is required.".into()));
    }

    // Insert the message
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO messages (from_id, subject, body, channel)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(user.id)
    .bind(&payload.subject)
    .bind(&payload.body)
    .bind(&payload.channel)
    .fetch_one(&state.db_pool)
    .await?;

    let msg_id = row.0;

    // Insert recipients
    for uid in &payload.recipients_to {
        sqlx::query(
            "INSERT INTO message_recipients (message_id, user_id, type) VALUES ($1, $2, 'to') ON CONFLICT DO NOTHING"
        )
        .bind(msg_id).bind(uid)
        .execute(&state.db_pool).await?;
    }
    if let Some(cc) = &payload.recipients_cc {
        for uid in cc {
            sqlx::query(
                "INSERT INTO message_recipients (message_id, user_id, type) VALUES ($1, $2, 'cc') ON CONFLICT DO NOTHING"
            )
            .bind(msg_id).bind(uid)
            .execute(&state.db_pool).await?;
        }
    }
    if let Some(bcc) = &payload.recipients_bcc {
        for uid in bcc {
            sqlx::query(
                "INSERT INTO message_recipients (message_id, user_id, type) VALUES ($1, $2, 'bcc') ON CONFLICT DO NOTHING"
            )
            .bind(msg_id).bind(uid)
            .execute(&state.db_pool).await?;
        }
    }

    write_audit_log(
        &state.db_pool, "messages", msg_id, AuditOperation::Create, user.id,
        None, Some(serde_json::json!({ "subject": payload.subject, "channel": "security" })),
    ).await?;

    Ok(msg_id)
}

/// Get all security-channel messages visible to the current user.
///
/// **Access:** TheGuardian, TheOverseer, or Administrator.
/// Returns messages where the user is sender OR a recipient.
#[tauri::command]
pub async fn get_security_messages(
    state: State<'_, AppState>,
) -> Result<Vec<MessageSummary>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheGuardian, Role::TheOverseer, Role::Administrator
    ]);

    let messages = sqlx::query_as::<_, MessageSummary>(
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
        LIMIT 200
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(messages)
}

/// Get recipients for a specific message.
///
/// **Access:** TheGuardian, TheOverseer, or Administrator.
#[tauri::command]
pub async fn get_message_recipients(
    state: State<'_, AppState>,
    message_id: Uuid,
) -> Result<Vec<MessageRecipientRow>, AppError> {
    let _user = crate::require_auth_any!(state, [
        Role::TheGuardian, Role::TheOverseer, Role::Administrator
    ]);

    let rows = sqlx::query_as::<_, MessageRecipientRow>(
        r#"
        SELECT mr.user_id, u.full_name, mr.type, mr.read_at
        FROM message_recipients mr
        JOIN users u ON u.id = mr.user_id
        WHERE mr.message_id = $1
        ORDER BY mr.type ASC, u.full_name ASC
        "#,
    )
    .bind(message_id)
    .fetch_all(&state.db_pool)
    .await?;

    Ok(rows)
}

/// Recall (unsend) a message — only if no recipient has read it yet.
///
/// **Access:** TheGuardian or Administrator (only message sender can recall).
#[tauri::command]
pub async fn recall_message(
    state: State<'_, AppState>,
    message_id: Uuid,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheGuardian, Role::Administrator
    ]);

    // Verify sender owns this message
    let msg: Option<(Uuid,)> = sqlx::query_as(
        "SELECT from_id FROM messages WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(message_id)
    .fetch_optional(&state.db_pool)
    .await?;

    let (from_id,) = msg.ok_or(AppError::Internal("Message not found.".into()))?;
    if from_id != user.id {
        return Err(AppError::Internal("Only the sender can recall a message.".into()));
    }

    // Check if any recipient has read it
    let read_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM message_recipients WHERE message_id = $1 AND read_at IS NOT NULL",
    )
    .bind(message_id)
    .fetch_one(&state.db_pool)
    .await?;

    if read_count.0 > 0 {
        return Err(AppError::Internal("Cannot recall — at least one recipient has already read it.".into()));
    }

    sqlx::query("UPDATE messages SET recalled_at = NOW() WHERE id = $1")
        .bind(message_id)
        .execute(&state.db_pool)
        .await?;

    write_audit_log(
        &state.db_pool, "messages", message_id, AuditOperation::Update, user.id,
        None, Some(serde_json::json!({ "action": "recall" })),
    ).await?;

    Ok(())
}

/// Mark a message as read for the current user.
///
/// **Access:** TheGuardian, TheOverseer, or Administrator.
#[tauri::command]
pub async fn mark_message_read(
    state: State<'_, AppState>,
    message_id: Uuid,
) -> Result<(), AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheGuardian, Role::TheOverseer, Role::Administrator
    ]);

    sqlx::query(
        "UPDATE message_recipients SET read_at = NOW() WHERE message_id = $1 AND user_id = $2 AND read_at IS NULL",
    )
    .bind(message_id)
    .bind(user.id)
    .execute(&state.db_pool)
    .await?;

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// Daily Security Reports — Guardian read-only view (TODO_DIRECTORS.md item 2)
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, FromRow)]
pub struct DirDailySecurityReport {
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

/// Get all daily security reports (for Guardian review & oversight).
///
/// When TheGuardian fetches reports that have not yet been marked as
/// delivered, they are automatically stamped with `delivered_to_guardian_at`.
///
/// **Access:** TheGuardian, TheOverseer, or Administrator.
#[tauri::command]
pub async fn dir_get_daily_security_reports(
    state: State<'_, AppState>,
) -> Result<Vec<DirDailySecurityReport>, AppError> {
    let user = crate::require_auth_any!(state, [
        Role::TheGuardian, Role::TheOverseer, Role::Administrator
    ]);

    // Mark un-delivered reports as delivered when The Guardian reads them
    if user.role == Role::TheGuardian {
        sqlx::query(
            "UPDATE daily_security_reports SET delivered_to_guardian_at = NOW() WHERE delivered_to_guardian_at IS NULL AND deleted_at IS NULL",
        )
        .execute(&state.db_pool)
        .await?;
    }

    let reports = sqlx::query_as::<_, DirDailySecurityReport>(
        r#"
        SELECT d.id, d.submitted_by, u.full_name AS submitter_name,
               d.report_date, d.findings_summary, d.risk_notes,
               d.recommended_actions, d.delivered_to_guardian_at, d.created_at
        FROM daily_security_reports d
        JOIN users u ON u.id = d.submitted_by
        WHERE d.deleted_at IS NULL
        ORDER BY d.report_date DESC, d.created_at DESC
        LIMIT 200
        "#,
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(reports)
}
