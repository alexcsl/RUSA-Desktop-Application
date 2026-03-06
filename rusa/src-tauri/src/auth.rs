// auth.rs — Auth guard functions + convenience macros
// These are used at the top of every protected Tauri command.
// They eliminate boilerplate and ensure consistent enforcement.

use crate::{
    error::AppError,
    state::{AppState, AuthenticatedUser, Role},
};
use tauri::State;

/// Extracts the currently authenticated user from AppState.
/// Returns AppError::Unauthenticated if no user is logged in.
pub async fn get_current_user(
    state: &State<'_, AppState>,
) -> Result<AuthenticatedUser, AppError> {
    let guard = state.current_user.lock().await;
    guard.clone().ok_or(AppError::Unauthenticated)
}

/// Checks that the current user holds the required role, OR is Administrator.
/// Administrator bypasses all role checks (see 12_ADMINISTRATOR.md).
pub fn require_role_or_admin(user: &AuthenticatedUser, role: Role) -> Result<(), AppError> {
    if user.role == Role::Administrator || user.role == role {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

/// Checks that the current user holds one of several acceptable roles, OR is Administrator.
/// Used for commands accessible to multiple roles (e.g., any Director can cast a vote).
pub fn require_any_role_or_admin(
    user: &AuthenticatedUser,
    roles: &[Role],
) -> Result<(), AppError> {
    if user.role == Role::Administrator || roles.contains(&user.role) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

/// Checks that the current user is a Director (any Director title), OR is Administrator.
pub fn require_director_or_admin(user: &AuthenticatedUser) -> Result<(), AppError> {
    if user.role == Role::Administrator || user.role.is_director() {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

// ── Convenience macros ────────────────────────────────────────────────────────

/// Standard guard for a single-role command.
/// Usage: let user = require_auth!(state, Role::Astronaut);
#[macro_export]
macro_rules! require_auth {
    ($state:expr, $role:expr) => {{
        let user = $crate::auth::get_current_user(&$state).await?;
        $crate::auth::require_role_or_admin(&user, $role)?;
        user
    }};
}

/// Guard for commands requiring any one of multiple roles.
/// Usage: let user = require_auth_any!(state, [Role::Astronaut, Role::TheWanderer]);
#[macro_export]
macro_rules! require_auth_any {
    ($state:expr, [$($role:expr),+ $(,)?]) => {{
        let user = $crate::auth::get_current_user(&$state).await?;
        $crate::auth::require_any_role_or_admin(&user, &[$($role),+])?;
        user
    }};
}

/// Guard for Director-only commands (any Director title passes).
/// Usage: let user = require_auth_director!(state);
#[macro_export]
macro_rules! require_auth_director {
    ($state:expr) => {{
        let user = $crate::auth::get_current_user(&$state).await?;
        $crate::auth::require_director_or_admin(&user)?;
        user
    }};
}

/// Guard for Administrator-only commands.
/// Usage: let user = require_auth_admin!(state);
#[macro_export]
macro_rules! require_auth_admin {
    ($state:expr) => {{
        let user = $crate::auth::get_current_user(&$state).await?;
        if user.role != $crate::state::Role::Administrator {
            return Err($crate::error::AppError::Forbidden);
        }
        user
    }};
}
