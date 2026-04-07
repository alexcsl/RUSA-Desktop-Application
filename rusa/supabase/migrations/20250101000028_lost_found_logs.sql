-- Migration: Lost and Found Logs
-- Required for Guardian UC-GUA-05 (Lost n Found Logs view)
-- Source of truth: Interview transcript 3_TheGuardian

CREATE TABLE IF NOT EXISTS lost_found_logs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  reported_by UUID REFERENCES users(id) NOT NULL,
  item_description TEXT NOT NULL,
  location_found TEXT NOT NULL,
  found_at TIMESTAMPTZ,
  status TEXT NOT NULL DEFAULT 'found' CHECK (status IN ('found', 'claimed', 'unclaimed')),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ,
  deleted_by UUID REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_lost_found_logs_reported_by ON lost_found_logs(reported_by);
CREATE INDEX IF NOT EXISTS idx_lost_found_logs_status ON lost_found_logs(status);
CREATE INDEX IF NOT EXISTS idx_lost_found_logs_created_at ON lost_found_logs(created_at);
