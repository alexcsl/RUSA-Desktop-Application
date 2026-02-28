// audit.rs — Reusable audit logging for all CUD operations
// Source of truth: 00_MASTER_GUIDE.md §5 (Audit Logging)
//
// Every Create, Update, Delete (soft) operation MUST write a row to audit_log.
// This module provides a single helper used across all subsystem commands.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

/// The three valid audit operations, matching the CHECK constraint on audit_log.operation.
pub enum AuditOperation {
    Create,
    Update,
    Delete,
}

impl AuditOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditOperation::Create => "CREATE",
            AuditOperation::Update => "UPDATE",
            AuditOperation::Delete => "DELETE",
        }
    }
}

/// Writes a row to the `audit_log` table.
///
/// # Arguments
/// * `pool` — The database connection pool.
/// * `table_name` — The table that was modified (e.g., `"users"`).
/// * `record_id` — The UUID of the affected row.
/// * `operation` — CREATE, UPDATE, or DELETE.
/// * `performed_by` — The user ID of who performed the action.
/// * `before_data` — JSON snapshot of the row before the change (None for CREATE).
/// * `after_data` — JSON snapshot of the row after the change (None for DELETE).
pub async fn write_audit_log(
    pool: &PgPool,
    table_name: &str,
    record_id: Uuid,
    operation: AuditOperation,
    performed_by: Uuid,
    before_data: Option<serde_json::Value>,
    after_data: Option<serde_json::Value>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO audit_log (table_name, record_id, operation, performed_by, before_data, after_data)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(table_name)
    .bind(record_id)
    .bind(operation.as_str())
    .bind(performed_by)
    .bind(before_data)
    .bind(after_data)
    .execute(pool)
    .await?;

    Ok(())
}
