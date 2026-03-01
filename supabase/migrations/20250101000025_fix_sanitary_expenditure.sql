-- Fix: Add missing foul_play columns to sanitary_expenditure_reports
-- These columns are referenced by the Rust backend (san_get_expenditure_reports, san_submit_expenditure_report)

ALTER TABLE sanitary_expenditure_reports
  ADD COLUMN IF NOT EXISTS foul_play_flag BOOLEAN NOT NULL DEFAULT FALSE,
  ADD COLUMN IF NOT EXISTS foul_play_note TEXT;
