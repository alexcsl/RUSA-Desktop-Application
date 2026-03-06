-- Migration: Data Analysts + Statistician data-request flow subsystem
-- Source of truth: 02_DATA_ANALYSTS.md
--
-- Creates dedicated data_requests and data_responses tables.
-- The existing Statistician commands in directors.rs queried the generic `requests` table;
-- those will be updated to query these dedicated tables instead.

-- ── data_requests ──────────────────────────────────────────────────────────────
-- Stores every cross-department data request. Any authenticated user can submit.
-- The Statistician reviews (bypass authority — no Directors vote).
-- Approved requests are forwarded to the Data Analyst team.

CREATE TABLE IF NOT EXISTS data_requests (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  requested_by UUID REFERENCES users(id) NOT NULL,
  dataset_description TEXT NOT NULL,
  scope TEXT NOT NULL,
  purpose TEXT NOT NULL,
  urgency TEXT NOT NULL CHECK (urgency IN ('low','medium','high','critical')),
  sensitivity_note TEXT,
  status TEXT NOT NULL DEFAULT 'pending_statistician'
    CHECK (status IN (
      'pending_statistician','approved','rejected',
      'processing','pending_outbound_review','delivered','withheld'
    )),
  statistician_decision_reason TEXT,
  assigned_analyst_id UUID REFERENCES users(id),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ,
  deleted_by UUID REFERENCES users(id)
);

-- Index for Statistician queue (pending requests)
CREATE INDEX IF NOT EXISTS idx_data_requests_pending
  ON data_requests (status, created_at)
  WHERE deleted_at IS NULL AND status = 'pending_statistician';

-- Index for Analyst inbox (approved requests)
CREATE INDEX IF NOT EXISTS idx_data_requests_analyst
  ON data_requests (status, assigned_analyst_id, created_at)
  WHERE deleted_at IS NULL AND status IN ('approved','processing');

-- Index for requester tracking own requests
CREATE INDEX IF NOT EXISTS idx_data_requests_requester
  ON data_requests (requested_by, created_at)
  WHERE deleted_at IS NULL;


-- ── data_responses ─────────────────────────────────────────────────────────────
-- Stores the analyst's packaged response to a data request.
-- Routed to Statistician for outbound sensitivity review before delivery.
--
-- result_payload JSONB shape:
-- {
--   "result_table": [...],       -- array of row objects
--   "charts": [
--     { "type": "bar|line|pie", "data": {...}, "config": {...} }
--   ],
--   "analyst_notes": "...",
--   "rejected_explanations": "..."
-- }
-- Spreadsheet file uploads go to Supabase Storage; path stored in spreadsheet_storage_path.

CREATE TABLE IF NOT EXISTS data_responses (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  request_id UUID REFERENCES data_requests(id) NOT NULL,
  prepared_by UUID REFERENCES users(id) NOT NULL,
  result_payload JSONB,
  spreadsheet_storage_path TEXT,
  status TEXT NOT NULL DEFAULT 'pending_outbound_review'
    CHECK (status IN ('pending_outbound_review','cleared','withheld')),
  statistician_review_note TEXT,
  submitted_at TIMESTAMPTZ DEFAULT NOW(),
  cleared_at TIMESTAMPTZ,
  delivered_at TIMESTAMPTZ
);

-- Index for Statistician outbound review queue
CREATE INDEX IF NOT EXISTS idx_data_responses_outbound
  ON data_responses (status, submitted_at)
  WHERE status = 'pending_outbound_review';

-- Index for finding response by request
CREATE INDEX IF NOT EXISTS idx_data_responses_request
  ON data_responses (request_id);
