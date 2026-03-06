-- Migration: Security Teams subsystem
-- Creates incident_reports, daily_security_reports
-- Adds optional columns to broadcast_requests for security features
-- Source of truth: 03_SECURITY_TEAMS.md, Internal_Docs.md

-- ── incident_reports ──────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS incident_reports (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  reported_by UUID REFERENCES users(id) NOT NULL,
  source TEXT NOT NULL CHECK (source IN ('direct_observation','external_report','assignment')),
  incident_type TEXT NOT NULL,
  location TEXT NOT NULL,
  occurred_at TIMESTAMPTZ,
  description TEXT NOT NULL,
  severity TEXT NOT NULL CHECK (severity IN ('low','medium','high','critical')),
  recommended_action TEXT,
  sector_or_base TEXT,
  assigned_to UUID REFERENCES users(id),
  related_incident_ids UUID[],
  -- Flexible per-incident metadata (compromised_party, category[], response_procedure, incident_start/end)
  incident_meta JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ,
  deleted_by UUID REFERENCES users(id)
);

-- ── daily_security_reports ────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS daily_security_reports (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  submitted_by UUID REFERENCES users(id) NOT NULL,
  report_date DATE NOT NULL,
  findings_summary TEXT NOT NULL,
  risk_notes TEXT,
  recommended_actions TEXT,
  delivered_to_guardian_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ,
  deleted_by UUID REFERENCES users(id)
);

-- ── Indexes ───────────────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_incident_reports_reported_by ON incident_reports(reported_by);
CREATE INDEX IF NOT EXISTS idx_incident_reports_assigned_to ON incident_reports(assigned_to);
CREATE INDEX IF NOT EXISTS idx_incident_reports_severity ON incident_reports(severity);
CREATE INDEX IF NOT EXISTS idx_daily_security_reports_submitted_by ON daily_security_reports(submitted_by);
CREATE INDEX IF NOT EXISTS idx_daily_security_reports_date ON daily_security_reports(report_date);
