// error.rs — Unified error type for all Tauri commands
// Tauri v2: Serialize as string for clean frontend error messages.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Authentication required. Please log in.")]
    Unauthenticated,

    #[error("You do not have permission to perform this action.")]
    Forbidden,

    #[error("Invalid username or password.")]
    InvalidCredentials,

    #[error("Your account has been deactivated.")]
    AccountDeactivated,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

// Tauri v2 pattern: serialize error as its Display string so frontend
// receives a clean, user-facing message — never a raw struct.
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

// Allow sqlx errors to convert automatically
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

impl From<redis::RedisError> for AppError {
    fn from(e: redis::RedisError) -> Self {
        AppError::Cache(e.to_string())
    }
}
