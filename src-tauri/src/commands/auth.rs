// commands/auth.rs — Login, logout, session management, password hashing
// Source of truth: AUTH_GUIDE.md §4.4, §4.5

use bcrypt::verify;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tauri::State;
use uuid::Uuid;

use crate::{
    error::AppError,
    state::{AppState, AuthenticatedUser, Role},
};

/// Named constants — never use magic numbers.
const SESSION_TTL_SECONDS: u64 = 60 * 60 * 8; // 8 hours
const ROLE_TTL_SECONDS: u64 = 60 * 60;         // 1 hour
const BCRYPT_COST: u32 = 12;

// ── Payloads & Responses ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

/// A safe subset of AuthenticatedUser returned to the Svelte frontend.
/// Never includes password_hash or any secret.
#[derive(Debug, Clone, Serialize)]
pub struct LoginResponse {
    pub id: Uuid,
    pub username: String,
    pub full_name: String,
    pub role: String, // role name string for frontend routing logic
    pub base_location_id: Option<Uuid>,
    pub base_has_data_regulation: bool,
}

/// Internal struct for the login DB query result.
#[derive(Debug, FromRow)]
struct LoginRecord {
    id: Uuid,
    username: String,
    full_name: String,
    password_hash: String,
    is_active: bool,
    base_location_id: Option<Uuid>,
    role_name: String,
    has_data_regulation: bool,
}

// ── Login ─────────────────────────────────────────────────────────────────────

/// Login command. Called from the Svelte login screen.
/// On success: populates AppState.current_user and caches session + role in Redis.
/// On failure: returns a generic invalid-credentials error (never reveal which field was wrong).
#[tauri::command]
pub async fn login(
    state: State<'_, AppState>,
    payload: LoginPayload,
) -> Result<LoginResponse, AppError> {
    // 1. Fetch user record from DB (username match, not deleted)
    let record = sqlx::query_as::<_, LoginRecord>(
        r#"
        SELECT
            u.id,
            u.username,
            u.full_name,
            u.password_hash,
            u.is_active,
            u.base_location_id,
            r.name AS role_name,
            COALESCE(bl.has_data_regulation, TRUE) AS has_data_regulation
        FROM users u
        JOIN roles r ON r.id = u.role_id
        LEFT JOIN base_locations bl ON bl.id = u.base_location_id
        WHERE u.username = $1
          AND u.deleted_at IS NULL
        "#,
    )
    .bind(&payload.username)
    .fetch_optional(&state.db_pool)
    .await?
    .ok_or(AppError::InvalidCredentials)?; // username not found → same error as wrong password

    // 2. Check account is active
    if !record.is_active {
        return Err(AppError::AccountDeactivated);
    }

    // 3. Verify password against bcrypt hash
    let password_matches = verify(&payload.password, &record.password_hash)
        .map_err(|_| AppError::Internal("Password verification failed".into()))?;
    if !password_matches {
        return Err(AppError::InvalidCredentials);
    }

    // 4. Parse role
    let role = Role::from_str(&record.role_name)
        .map_err(AppError::Internal)?;

    // 5. Build the in-memory user object
    let auth_user = AuthenticatedUser {
        id: record.id,
        username: record.username.clone(),
        full_name: record.full_name.clone(),
        role: role.clone(),
        base_location_id: record.base_location_id,
        base_has_data_regulation: record.has_data_regulation,
    };

    // 6. Store in AppState singleton (replaces any prior session)
    {
        let mut guard = state.current_user.lock().await;
        *guard = Some(auth_user.clone());
    }

    // 7. Cache session and role in Redis
    let mut redis_conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| AppError::Cache(e.to_string()))?;

    let session_key = format!("session:{}", record.id);
    let role_key = format!("role:{}", record.id);

    // Session value: the user's ID as a string (proof of active session)
    redis_conn
        .set_ex::<_, _, ()>(&session_key, record.id.to_string(), SESSION_TTL_SECONDS)
        .await
        .map_err(|e| AppError::Cache(e.to_string()))?;

    // Role value: role name string (avoids DB hit on every command)
    redis_conn
        .set_ex::<_, _, ()>(&role_key, record.role_name.clone(), ROLE_TTL_SECONDS)
        .await
        .map_err(|e| AppError::Cache(e.to_string()))?;

    // 8. Return safe subset to frontend
    Ok(LoginResponse {
        id: record.id,
        username: record.username,
        full_name: record.full_name,
        role: record.role_name,
        base_location_id: record.base_location_id,
        base_has_data_regulation: record.has_data_regulation,
    })
}

// ── Logout ────────────────────────────────────────────────────────────────────

/// Logout command. Clears AppState.current_user and deletes Redis keys.
#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), AppError> {
    // 1. Get user ID before clearing (needed for Redis key deletion)
    let user_id = {
        let guard = state.current_user.lock().await;
        guard.as_ref().map(|u| u.id)
    };

    // 2. Clear AppState
    {
        let mut guard = state.current_user.lock().await;
        *guard = None;
    }

    // 3. Delete Redis keys
    if let Some(id) = user_id {
        let mut redis_conn = state
            .redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::Cache(e.to_string()))?;

        let _: () = redis_conn
            .del(&[format!("session:{}", id), format!("role:{}", id)])
            .await
            .map_err(|e| AppError::Cache(e.to_string()))?;
    }

    Ok(())
}

// ── Session Restore ───────────────────────────────────────────────────────────

/// Returns the current session user for frontend re-hydration on app restart.
/// Called once on Svelte app mount to restore the logged-in state.
#[tauri::command]
pub async fn get_current_session(
    state: State<'_, AppState>,
) -> Result<Option<LoginResponse>, AppError> {
    let guard = state.current_user.lock().await;
    Ok(guard.as_ref().map(|u| LoginResponse {
        id: u.id,
        username: u.username.clone(),
        full_name: u.full_name.clone(),
        role: u.role.as_str().to_string(),
        base_location_id: u.base_location_id,
        base_has_data_regulation: u.base_has_data_regulation,
    }))
}

// ── Password Hashing Utility ──────────────────────────────────────────────────

/// Hash a plaintext password for storage. Used during account creation.
/// Never called during login — login uses bcrypt::verify() against the stored hash.
pub fn hash_password(plaintext: &str) -> Result<String, AppError> {
    bcrypt::hash(plaintext, BCRYPT_COST)
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))
}

