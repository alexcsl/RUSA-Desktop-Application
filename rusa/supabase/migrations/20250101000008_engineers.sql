-- ============================================================
-- 008_engineers.sql
-- Subsystem 06 — Engineers (Agricultural + Biological)
-- Source of truth: 01_ENGINEERS.md + 00_MASTER_GUIDE.md
-- ============================================================
--
-- Most tables are shared with Scientists (created in 007_scientists.sql):
--   approved_tests, approved_test_species, experiments,
--   experiment_daily_logs, experiment_log_attendees, experiment_log_tests,
--   test_proposals, science_archive
--
-- This migration adds only the tables specific to the Engineers subsystem.
-- ============================================================

-- ============================================================
-- PROGRESS REPORTS  (UC-GE-02)
-- Engineers submit progress reports linked to active tasks.
-- ============================================================
CREATE TABLE progress_reports (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  task_id       UUID NOT NULL REFERENCES tasks(id),
  submitted_by  UUID NOT NULL REFERENCES users(id),
  report_date   DATE NOT NULL DEFAULT CURRENT_DATE,
  current_status TEXT NOT NULL,
  work_completed TEXT NOT NULL,
  problems_encountered TEXT,
  plans_next     TEXT,
  created_at    TIMESTAMPTZ DEFAULT NOW(),
  deleted_at    TIMESTAMPTZ,
  deleted_by    UUID REFERENCES users(id)
);

CREATE INDEX idx_progress_reports_task ON progress_reports(task_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_progress_reports_user ON progress_reports(submitted_by) WHERE deleted_at IS NULL;

-- ============================================================
-- EXPERIMENT LOG SPECIES (junction)
-- Engineers log which species they worked on per daily log entry.
-- This supplements experiment_log_tests (which tests were performed).
-- ============================================================
CREATE TABLE experiment_log_species (
  log_id             UUID REFERENCES experiment_daily_logs(id),
  species_archive_id UUID REFERENCES science_archive(id),
  PRIMARY KEY (log_id, species_archive_id)
);

-- ============================================================
-- SEED: Add scope values for engineer-typed approved tests
-- Existing scopes: 'plants', 'all_species', 'matter', 'physical'
-- Agricultural Engineers use 'plants'
-- Biological Engineers use 'all_species'
-- ============================================================

-- ============================================================
-- NOTES
-- ============================================================
-- Engineers reuse the shared experiments table with types:
--   'agricultural'              → AgriculturalEngineer
--   'biological_engineering'    → BiologicalEngineer
--
-- Species archive: Engineers use science_archive with type='species'
--   scope is determined by experiment type:
--   - Agricultural experiments link to scope='plant' species
--   - Biological experiments link to scope='all_species' species
--
-- Supabase Storage:
--   No binary uploads required for core engineer use cases.
--   Optional: experiment log attachments → rusa-files/engineers/logs/{log_id}/{filename}
--
-- Redis Cache Keys:
--   approved_tests:plants         → filtered approved tests for AgEng  (TTL 1h)
--   approved_tests:all_species    → filtered approved tests for BioEng (TTL 1h)
--   species_archive:plant         → plant species list   (TTL 1h)
--   species_archive:all_species   → all species list     (TTL 1h)
--   experiments:user:{user_id}    → engineer's experiments (TTL 30m)
--   engineer_tasks:{user_id}      → engineer's task inbox  (TTL 15m)
--
-- Invalidation:
--   Any test approved/deleted      → DEL approved_tests:plants, approved_tests:all_species
--   Any species added/updated      → DEL species_archive:plant, species_archive:all_species
--   Experiment status change        → DEL experiments:user:{user_id}
--   Task assigned/updated           → DEL engineer_tasks:{user_id}
--   Progress report submitted       → DEL engineer_tasks:{user_id}
