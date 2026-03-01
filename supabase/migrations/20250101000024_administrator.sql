-- ============================================================================
-- Migration 20250101000024 — Administrator Subsystem (Sub-12)
-- ============================================================================
-- The Administrator does NOT own new tables.  All data interactions use tables
-- from other subsystems.  This migration adds:
--   1. Performance indexes on audit_log for admin-level queries.
--   2. Any missing columns on shared tables (vote_sessions already has all
--      admin-override columns from 20250101000002).
-- ============================================================================

-- ── Indexes on audit_log for admin dashboard / audit viewer ───────────────────
CREATE INDEX IF NOT EXISTS idx_audit_log_performed_at
  ON audit_log (performed_at DESC);

CREATE INDEX IF NOT EXISTS idx_audit_log_table_name
  ON audit_log (table_name);

CREATE INDEX IF NOT EXISTS idx_audit_log_performed_by
  ON audit_log (performed_by);

CREATE INDEX IF NOT EXISTS idx_audit_log_operation
  ON audit_log (operation);

-- ── Composite index for the paginated audit viewer (table + time range) ───────
CREATE INDEX IF NOT EXISTS idx_audit_log_table_performed_at
  ON audit_log (table_name, performed_at DESC);

-- ── Index on users.is_active for quick status filtering ───────────────────────
CREATE INDEX IF NOT EXISTS idx_users_is_active
  ON users (is_active) WHERE deleted_at IS NULL;

-- ── Index on users.deleted_at for soft-delete filtering ───────────────────────
CREATE INDEX IF NOT EXISTS idx_users_deleted_at
  ON users (deleted_at) WHERE deleted_at IS NOT NULL;
