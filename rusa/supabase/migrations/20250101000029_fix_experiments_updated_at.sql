-- ============================================================
-- 029_fix_experiments_updated_at.sql
-- Add missing updated_at column to experiments table.
-- scientists.rs and engineers.rs both SET updated_at = NOW()
-- when transitioning experiment status, but the column was
-- omitted from the original 007_scientists.sql migration.
-- ============================================================

ALTER TABLE experiments
  ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW();
