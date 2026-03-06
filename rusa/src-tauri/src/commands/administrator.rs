// commands/administrator.rs — Administrator-exclusive commands (Sub-12)
// Source of truth: 12_ADMINISTRATOR.md, 00_MASTER_GUIDE.md
//
// The Administrator inherits access to ALL other subsystem commands via
// the require_role_or_admin / require_auth_any! / require_auth_admin! macros
// already defined in auth.rs.  This module adds commands that ONLY the
// Administrator can execute:
//
//   1. System-wide dashboard statistics
//   2. Paginated audit log viewer
//   3. Full user directory with extended fields (status, location, email)
//   4. Toggle user account activation status
//   5. Reset any user's password
//   6. Recent system-wide activity feed

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
    state::AppState,
};

// ── Responses ─────────────────────────────────────────────────────────────────

/// System overview stats for the admin dashboard.
#[derive(Debug, Serialize)]
pub struct SystemStats {
    pub total_users: i64,
    pub active_users: i64,
    pub deactivated_users: i64,
    pub terminated_users: i64,
    pub total_roles: i64,
    pub open_vote_sessions: i64,
    pub decided_vote_sessions: i64,
    pub overridden_vote_sessions: i64,
    pub terminated_vote_sessions: i64,
    pub total_audit_entries: i64,
    pub recent_audit_entries_24h: i64,
    pub pending_requests: i64,
    pub active_meetings: i64,
    pub total_notifications_unread: i64,
    pub total_base_locations: i64,
}

/// A row from the audit_log table for the admin viewer.
#[derive(Debug, Serialize, FromRow)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub table_name: String,
    pub record_id: Uuid,
    pub operation: String,
    pub performed_by: Option<Uuid>,
    pub performer_name: Option<String>,
    pub performed_at: DateTime<Utc>,
    pub before_data: Option<serde_json::Value>,
    pub after_data: Option<serde_json::Value>,
}

/// Paginated result wrapper for audit log.
#[derive(Debug, Serialize)]
pub struct AuditLogPage {
    pub entries: Vec<AuditLogEntry>,
    pub total_count: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Extended user info for the admin user directory.
#[derive(Debug, Serialize, FromRow)]
pub struct AdminUserEntry {
    pub id: Uuid,
    pub username: String,
    pub full_name: String,
    pub email: Option<String>,
    pub role_name: String,
    pub base_location_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Activity feed entry for dashboard.
#[derive(Debug, Serialize, FromRow)]
pub struct RecentActivity {
    pub id: Uuid,
    pub table_name: String,
    pub operation: String,
    pub performer_name: Option<String>,
    pub performed_at: DateTime<Utc>,
    pub summary: Option<String>,
}

// ── Payloads ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AuditLogFilter {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub table_name: Option<String>,
    pub operation: Option<String>,
    pub performer_id: Option<Uuid>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordPayload {
    pub target_user_id: Uuid,
    pub new_password: String,
}

// ════════════════════════════════════════════════════════════════════════════════
// 1. System Dashboard Stats
// ════════════════════════════════════════════════════════════════════════════════

/// Returns aggregate system statistics for the admin dashboard.
///
/// **Access:** Administrator only.
#[tauri::command]
pub async fn admin_get_system_stats(
    state: State<'_, AppState>,
) -> Result<SystemStats, AppError> {
    let _user = crate::require_auth_admin!(state);
    let pool = &state.db_pool;

    // All counts in a single query for efficiency
    let stats = sqlx::query_as::<_, (
        i64, i64, i64, i64, i64,
        i64, i64, i64, i64, i64,
        i64, i64, i64, i64, i64,
    )>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM users WHERE deleted_at IS NULL) AS total_users,
            (SELECT COUNT(*) FROM users WHERE deleted_at IS NULL AND is_active = TRUE) AS active_users,
            (SELECT COUNT(*) FROM users WHERE deleted_at IS NULL AND is_active = FALSE) AS deactivated_users,
            (SELECT COUNT(*) FROM users WHERE deleted_at IS NOT NULL) AS terminated_users,
            (SELECT COUNT(*) FROM roles) AS total_roles,
            (SELECT COUNT(*) FROM vote_sessions WHERE status IN ('open','quorum_pending')) AS open_votes,
            (SELECT COUNT(*) FROM vote_sessions WHERE status = 'decided') AS decided_votes,
            (SELECT COUNT(*) FROM vote_sessions WHERE admin_overridden = TRUE) AS overridden_votes,
            (SELECT COUNT(*) FROM vote_sessions WHERE admin_terminated = TRUE) AS terminated_votes,
            (SELECT COUNT(*) FROM audit_log) AS total_audit,
            (SELECT COUNT(*) FROM audit_log WHERE performed_at > NOW() - INTERVAL '24 hours') AS recent_audit,
            (SELECT COUNT(*) FROM requests WHERE status = 'pending' AND deleted_at IS NULL) AS pending_requests,
            (SELECT COUNT(*) FROM meetings WHERE deleted_at IS NULL AND scheduled_at > NOW()) AS active_meetings,
            (SELECT COUNT(*) FROM notifications WHERE read_at IS NULL) AS unread_notifs,
            (SELECT COUNT(*) FROM base_locations WHERE deleted_at IS NULL) AS base_locations
        "#,
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(SystemStats {
        total_users: stats.0,
        active_users: stats.1,
        deactivated_users: stats.2,
        terminated_users: stats.3,
        total_roles: stats.4,
        open_vote_sessions: stats.5,
        decided_vote_sessions: stats.6,
        overridden_vote_sessions: stats.7,
        terminated_vote_sessions: stats.8,
        total_audit_entries: stats.9,
        recent_audit_entries_24h: stats.10,
        pending_requests: stats.11,
        active_meetings: stats.12,
        total_notifications_unread: stats.13,
        total_base_locations: stats.14,
    })
}

// ════════════════════════════════════════════════════════════════════════════════
// 2. Paginated Audit Log Viewer
// ════════════════════════════════════════════════════════════════════════════════

/// Returns a paginated, filterable audit log for the administrator.
///
/// **Access:** Administrator only.
#[tauri::command]
pub async fn admin_get_audit_log(
    state: State<'_, AppState>,
    filter: AuditLogFilter,
) -> Result<AuditLogPage, AppError> {
    let _user = crate::require_auth_admin!(state);
    let pool = &state.db_pool;

    let page = filter.page.unwrap_or(1).max(1);
    let page_size = filter.page_size.unwrap_or(50).clamp(10, 200);
    let offset = (page - 1) * page_size;

    // Count query (with same filters)
    let total_count: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM audit_log a
        WHERE ($1::TEXT IS NULL OR a.table_name = $1)
          AND ($2::TEXT IS NULL OR a.operation = $2)
          AND ($3::UUID IS NULL OR a.performed_by = $3)
          AND ($4::DATE IS NULL OR a.performed_at >= $4::DATE::TIMESTAMPTZ)
          AND ($5::DATE IS NULL OR a.performed_at < ($5::DATE + INTERVAL '1 day')::TIMESTAMPTZ)
        "#
    )
    .bind(&filter.table_name)
    .bind(&filter.operation)
    .bind(filter.performer_id)
    .bind(filter.date_from)
    .bind(filter.date_to)
    .fetch_one(pool)
    .await?;

    // Data query
    let entries = sqlx::query_as::<_, AuditLogEntry>(
        r#"
        SELECT
            a.id,
            a.table_name,
            a.record_id,
            a.operation,
            a.performed_by,
            u.full_name AS performer_name,
            a.performed_at,
            a.before_data,
            a.after_data
        FROM audit_log a
        LEFT JOIN users u ON u.id = a.performed_by
        WHERE ($1::TEXT IS NULL OR a.table_name = $1)
          AND ($2::TEXT IS NULL OR a.operation = $2)
          AND ($3::UUID IS NULL OR a.performed_by = $3)
          AND ($4::DATE IS NULL OR a.performed_at >= $4::DATE::TIMESTAMPTZ)
          AND ($5::DATE IS NULL OR a.performed_at < ($5::DATE + INTERVAL '1 day')::TIMESTAMPTZ)
        ORDER BY a.performed_at DESC
        LIMIT $6 OFFSET $7
        "#,
    )
    .bind(&filter.table_name)
    .bind(&filter.operation)
    .bind(filter.performer_id)
    .bind(filter.date_from)
    .bind(filter.date_to)
    .bind(page_size)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(AuditLogPage {
        entries,
        total_count: total_count.0,
        page,
        page_size,
    })
}

// ════════════════════════════════════════════════════════════════════════════════
// 3. Full User Directory (including deactivated/terminated)
// ════════════════════════════════════════════════════════════════════════════════

/// Returns ALL users (including deactivated and soft-deleted), with extended info.
///
/// **Access:** Administrator only.
#[tauri::command]
pub async fn admin_get_all_users(
    state: State<'_, AppState>,
    include_terminated: Option<bool>,
) -> Result<Vec<AdminUserEntry>, AppError> {
    let _user = crate::require_auth_admin!(state);
    let pool = &state.db_pool;

    let include = include_terminated.unwrap_or(false);

    let users = sqlx::query_as::<_, AdminUserEntry>(
        r#"
        SELECT
            u.id,
            u.username,
            u.full_name,
            u.email,
            r.name AS role_name,
            bl.name AS base_location_name,
            u.is_active,
            u.created_at,
            u.updated_at,
            u.deleted_at
        FROM users u
        JOIN roles r ON r.id = u.role_id
        LEFT JOIN base_locations bl ON bl.id = u.base_location_id
        WHERE ($1 = TRUE OR u.deleted_at IS NULL)
        ORDER BY u.full_name ASC
        "#,
    )
    .bind(include)
    .fetch_all(pool)
    .await?;

    Ok(users)
}

// ════════════════════════════════════════════════════════════════════════════════
// 4. Toggle User Account Activation
// ════════════════════════════════════════════════════════════════════════════════

/// Activate or deactivate a user account (sets `is_active`).
/// This does NOT delete the user — only toggles their login capability.
///
/// **Access:** Administrator only.
#[tauri::command]
pub async fn admin_toggle_user_status(
    state: State<'_, AppState>,
    target_user_id: Uuid,
    is_active: bool,
) -> Result<(), AppError> {
    let admin = crate::require_auth_admin!(state);
    let pool = &state.db_pool;

    // Cannot deactivate yourself
    if target_user_id == admin.id {
        return Err(AppError::Internal("Cannot change your own account status.".into()));
    }

    // Fetch before state for audit
    let before: (bool,) = sqlx::query_as(
        "SELECT is_active FROM users WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(target_user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Internal("User not found.".into()))?;

    sqlx::query(
        "UPDATE users SET is_active = $2, updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(target_user_id)
    .bind(is_active)
    .execute(pool)
    .await?;

    // Invalidate role cache
    let mut redis_conn = state.redis_client.get_multiplexed_async_connection().await
        .map_err(|e| AppError::Cache(e.to_string()))?;
    let _: () = redis_conn.del(format!("role:{}", target_user_id)).await
        .unwrap_or(());
    let _: () = redis_conn.del(format!("session:{}", target_user_id)).await
        .unwrap_or(());

    write_audit_log(
        pool,
        "users",
        target_user_id,
        AuditOperation::Update,
        admin.id,
        Some(serde_json::json!({ "is_active": before.0 })),
        Some(serde_json::json!({ "is_active": is_active })),
    )
    .await?;

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// 5. Reset User Password
// ════════════════════════════════════════════════════════════════════════════════

/// Administrator resets any user's password (bcrypt hashed).
///
/// **Access:** Administrator only.
#[tauri::command]
pub async fn admin_reset_password(
    state: State<'_, AppState>,
    payload: ResetPasswordPayload,
) -> Result<(), AppError> {
    let admin = crate::require_auth_admin!(state);
    let pool = &state.db_pool;

    if payload.new_password.len() < 8 {
        return Err(AppError::Internal("Password must be at least 8 characters.".into()));
    }

    // Verify target exists
    let exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM users WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(payload.target_user_id)
    .fetch_optional(pool)
    .await?;

    if exists.is_none() {
        return Err(AppError::Internal("User not found.".into()));
    }

    let new_hash = hash_password(&payload.new_password)?;

    sqlx::query(
        "UPDATE users SET password_hash = $2, updated_at = NOW() WHERE id = $1"
    )
    .bind(payload.target_user_id)
    .bind(&new_hash)
    .execute(pool)
    .await?;

    // Invalidate session — forces re-login
    let mut redis_conn = state.redis_client.get_multiplexed_async_connection().await
        .map_err(|e| AppError::Cache(e.to_string()))?;
    let _: () = redis_conn.del(format!("session:{}", payload.target_user_id)).await
        .unwrap_or(());

    write_audit_log(
        pool,
        "users",
        payload.target_user_id,
        AuditOperation::Update,
        admin.id,
        None,
        Some(serde_json::json!({ "password_reset": true })),
    )
    .await?;

    Ok(())
}

// ════════════════════════════════════════════════════════════════════════════════
// 6. Recent Activity Feed (Dashboard)
// ════════════════════════════════════════════════════════════════════════════════

/// Returns the most recent audit log entries for the admin dashboard feed.
///
/// **Access:** Administrator only.
#[tauri::command]
pub async fn admin_get_recent_activity(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<RecentActivity>, AppError> {
    let _user = crate::require_auth_admin!(state);
    let pool = &state.db_pool;

    let lim = limit.unwrap_or(25).clamp(5, 100);

    let activity = sqlx::query_as::<_, RecentActivity>(
        r#"
        SELECT
            a.id,
            a.table_name,
            a.operation,
            u.full_name AS performer_name,
            a.performed_at,
            CASE
                WHEN a.operation = 'CREATE' THEN 'Created new ' || a.table_name || ' record'
                WHEN a.operation = 'UPDATE' THEN 'Updated ' || a.table_name || ' record'
                WHEN a.operation = 'DELETE' THEN 'Deleted ' || a.table_name || ' record'
                ELSE a.operation || ' on ' || a.table_name
            END AS summary
        FROM audit_log a
        LEFT JOIN users u ON u.id = a.performed_by
        ORDER BY a.performed_at DESC
        LIMIT $1
        "#,
    )
    .bind(lim)
    .fetch_all(pool)
    .await?;

    Ok(activity)
}

// ════════════════════════════════════════════════════════════════════════════════
// 7. Get Available Roles (for forms)
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, FromRow)]
pub struct RoleEntry {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

/// Returns all roles from the roles table for admin picker forms.
///
/// **Access:** Administrator only.
#[tauri::command]
pub async fn admin_get_roles(
    state: State<'_, AppState>,
) -> Result<Vec<RoleEntry>, AppError> {
    let _user = crate::require_auth_admin!(state);

    let roles = sqlx::query_as::<_, RoleEntry>(
        "SELECT id, name, description FROM roles ORDER BY name ASC"
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(roles)
}

// ════════════════════════════════════════════════════════════════════════════════
// 8. Get Base Locations (for forms)
// ════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Serialize, FromRow)]
pub struct BaseLocationEntry {
    pub id: Uuid,
    pub name: String,
    pub has_data_regulation: bool,
}

/// Returns all base locations for admin picker forms.
///
/// **Access:** Administrator only.
#[tauri::command]
pub async fn admin_get_base_locations(
    state: State<'_, AppState>,
) -> Result<Vec<BaseLocationEntry>, AppError> {
    let _user = crate::require_auth_admin!(state);

    let locations = sqlx::query_as::<_, BaseLocationEntry>(
        "SELECT id, name, has_data_regulation FROM base_locations WHERE deleted_at IS NULL ORDER BY name ASC"
    )
    .fetch_all(&state.db_pool)
    .await?;

    Ok(locations)
}
