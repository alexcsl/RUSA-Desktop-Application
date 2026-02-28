-- ============================================================
-- Migration 012: Exoplanet Settlers subsystem
-- Source of truth: 06_EXOPLANET_SETTLERS.md + 00_MASTER_GUIDE.md
-- ============================================================

-- ── Settlements ───────────────────────────────────────────────
CREATE TABLE settlements (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name          TEXT NOT NULL,
  planet        TEXT NOT NULL,
  commander_id  UUID REFERENCES users(id),
  status        TEXT NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active','abandoned')),
  created_at    TIMESTAMPTZ DEFAULT NOW(),
  deleted_at    TIMESTAMPTZ,
  deleted_by    UUID REFERENCES users(id)
);

CREATE INDEX idx_settlements_commander ON settlements(commander_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_settlements_status    ON settlements(status)       WHERE deleted_at IS NULL;

-- ── Settler Assignments (user ↔ settlement) ───────────────────
CREATE TABLE settler_assignments (
  user_id         UUID REFERENCES users(id),
  settlement_id   UUID REFERENCES settlements(id),
  type            TEXT NOT NULL CHECK (type IN ('permanent','temporary')),
  residence_unit  TEXT,
  house_arrest    BOOLEAN NOT NULL DEFAULT FALSE,
  arrived_at      TIMESTAMPTZ DEFAULT NOW(),
  departed_at     TIMESTAMPTZ,
  PRIMARY KEY (user_id, settlement_id)
);

CREATE INDEX idx_settler_assign_settlement ON settler_assignments(settlement_id) WHERE departed_at IS NULL;
CREATE INDEX idx_settler_assign_arrest     ON settler_assignments(house_arrest)   WHERE house_arrest = TRUE AND departed_at IS NULL;

-- ── Settler Tasks (Commander → Settler assignments) ───────────
-- Separate from the generic `tasks` table (Director → Commander).
CREATE TABLE settler_tasks (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  settlement_id   UUID NOT NULL REFERENCES settlements(id),
  assigned_by     UUID NOT NULL REFERENCES users(id),
  assigned_to     UUID NOT NULL REFERENCES users(id),
  title           TEXT NOT NULL,
  description     TEXT,
  scope           TEXT,
  urgency         TEXT CHECK (urgency IN ('low','medium','high','critical')),
  deadline        DATE,
  status          TEXT NOT NULL DEFAULT 'assigned'
                      CHECK (status IN ('assigned','in_progress','completed','cancelled')),
  created_at      TIMESTAMPTZ DEFAULT NOW(),
  updated_at      TIMESTAMPTZ DEFAULT NOW(),
  deleted_at      TIMESTAMPTZ,
  deleted_by      UUID REFERENCES users(id)
);

CREATE INDEX idx_settler_tasks_assigned_to  ON settler_tasks(assigned_to)   WHERE deleted_at IS NULL;
CREATE INDEX idx_settler_tasks_settlement   ON settler_tasks(settlement_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_settler_tasks_status       ON settler_tasks(status)        WHERE deleted_at IS NULL;

-- ── Settler Progress Reports (UC-PS-02) ───────────────────────
CREATE TABLE settler_progress_reports (
  id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  task_id                 UUID NOT NULL REFERENCES settler_tasks(id),
  settlement_id           UUID NOT NULL REFERENCES settlements(id),
  submitted_by            UUID NOT NULL REFERENCES users(id),
  week                    TEXT,
  rag_status              TEXT CHECK (rag_status IN ('red','amber','green')),
  progress_made           TEXT NOT NULL,
  materials_equipment     TEXT,
  created_at              TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_settler_progress_task ON settler_progress_reports(task_id);

-- ── Settlement Inventory ──────────────────────────────────────
CREATE TABLE settlement_inventory (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  settlement_id     UUID NOT NULL REFERENCES settlements(id),
  item_name         TEXT NOT NULL,
  category          TEXT,
  quantity          INTEGER NOT NULL DEFAULT 0,
  unit              TEXT,
  min_threshold     INTEGER NOT NULL DEFAULT 0,
  last_updated_by   UUID REFERENCES users(id),
  updated_at        TIMESTAMPTZ DEFAULT NOW(),
  deleted_at        TIMESTAMPTZ,
  deleted_by        UUID REFERENCES users(id)
);

CREATE INDEX idx_settlement_inv_settlement ON settlement_inventory(settlement_id) WHERE deleted_at IS NULL;

-- ── Anomaly Reports (UC-PS-03 / UC-TS-01 / UC-SC-02A) ────────
CREATE TABLE anomaly_reports (
  id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  settlement_id           UUID NOT NULL REFERENCES settlements(id),
  reported_by             UUID NOT NULL REFERENCES users(id),
  description             TEXT NOT NULL,
  location_in_settlement  TEXT,
  danger_level            TEXT CHECK (danger_level IN ('low','medium','high','critical')),
  severity                TEXT NOT NULL CHECK (severity IN ('low','medium','high','critical')),
  -- Category checkboxes stored as JSONB array: ["structural","biological","chemical","other"]
  category                JSONB DEFAULT '[]'::jsonb,
  status                  TEXT NOT NULL DEFAULT 'submitted'
                              CHECK (status IN (
                                'submitted','under_review','forwarded_to_directors',
                                'resolved','triggered_abandonment'
                              )),
  commander_note          TEXT,
  vote_session_id         UUID REFERENCES vote_sessions(id),
  created_at              TIMESTAMPTZ DEFAULT NOW(),
  deleted_at              TIMESTAMPTZ,
  deleted_by              UUID REFERENCES users(id)
);

COMMENT ON COLUMN anomaly_reports.category IS
  'JSONB array of category strings, e.g. ["structural","biological","chemical","other"]';

CREATE INDEX idx_anomaly_settlement ON anomaly_reports(settlement_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_anomaly_status     ON anomaly_reports(status)        WHERE deleted_at IS NULL;

-- ── Settler Complaints (UC-PS-04 / UC-TS-02) ─────────────────
CREATE TABLE settler_complaints (
  id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  settlement_id       UUID NOT NULL REFERENCES settlements(id),
  reported_by         UUID NOT NULL REFERENCES users(id),
  subject_user_id     UUID NOT NULL REFERENCES users(id),
  incident_description TEXT NOT NULL,
  status              TEXT NOT NULL DEFAULT 'submitted'
                          CHECK (status IN ('submitted','under_review','forwarded_to_directors','resolved','rejected')),
  commander_note      TEXT,
  created_at          TIMESTAMPTZ DEFAULT NOW(),
  deleted_at          TIMESTAMPTZ,
  deleted_by          UUID REFERENCES users(id)
);

CREATE INDEX idx_complaint_settlement ON settler_complaints(settlement_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_complaint_status     ON settler_complaints(status)        WHERE deleted_at IS NULL;

-- ── Supply Requests (UC-PS-05 / UC-TS-03 / UC-SC-07 / UC-CE-02 / UC-FA-01) ──
CREATE TABLE supply_requests (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  settlement_id     UUID NOT NULL REFERENCES settlements(id),
  submitted_by      UUID NOT NULL REFERENCES users(id),
  source_type       TEXT NOT NULL CHECK (source_type IN ('settler','commander','civil_engineer','farmer')),
  -- Items as JSONB array: [{"item":"...","spec":"...","quantity":N,"reason":"..."}]
  items             JSONB NOT NULL DEFAULT '[]'::jsonb,
  justification     TEXT,
  status            TEXT NOT NULL DEFAULT 'pending_commander'
                        CHECK (status IN ('pending_commander','rejected_by_commander','pending_vote','approved','rejected')),
  commander_note    TEXT,
  vote_session_id   UUID REFERENCES vote_sessions(id),
  created_at        TIMESTAMPTZ DEFAULT NOW(),
  deleted_at        TIMESTAMPTZ,
  deleted_by        UUID REFERENCES users(id)
);

COMMENT ON COLUMN supply_requests.items IS
  'JSONB array: [{"item":"Cement","spec":"50kg bags","quantity":100,"reason":"Foundation work"}]';

CREATE INDEX idx_supply_settlement ON supply_requests(settlement_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_supply_status     ON supply_requests(status)        WHERE deleted_at IS NULL;

-- ── Building Health Logs (UC-CE-03) ───────────────────────────
CREATE TABLE building_health_logs (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  settlement_id   UUID NOT NULL REFERENCES settlements(id),
  building_name   TEXT NOT NULL,
  checked_by      UUID NOT NULL REFERENCES users(id),
  check_date      DATE NOT NULL,
  findings        TEXT,
  status          TEXT NOT NULL CHECK (status IN ('pass','fail','needs_repair')),
  created_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_bhealth_settlement ON building_health_logs(settlement_id);

-- ── Farm Health Logs (UC-FA-02) ───────────────────────────────
CREATE TABLE farm_health_logs (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  settlement_id   UUID NOT NULL REFERENCES settlements(id),
  logged_by       UUID NOT NULL REFERENCES users(id),
  log_date        DATE NOT NULL,
  subject_type    TEXT NOT NULL CHECK (subject_type IN ('plant','livestock')),
  subject_name    TEXT NOT NULL,
  condition       TEXT NOT NULL,
  treatment       TEXT,
  notes           TEXT,
  created_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_fhealth_settlement ON farm_health_logs(settlement_id);

-- ── Construction Progress Reports (UC-CE-01) ──────────────────
CREATE TABLE construction_progress_reports (
  id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  task_id               UUID NOT NULL REFERENCES settler_tasks(id),
  settlement_id         UUID NOT NULL REFERENCES settlements(id),
  submitted_by          UUID NOT NULL REFERENCES users(id),
  week                  TEXT,
  rag_status            TEXT CHECK (rag_status IN ('red','amber','green')),
  -- Materials used as JSONB: [{"material":"Cement","quantity":50,"unit":"bags"}]
  materials_used        JSONB DEFAULT '[]'::jsonb,
  construction_progress TEXT NOT NULL,
  issues                TEXT,
  created_at            TIMESTAMPTZ DEFAULT NOW()
);

COMMENT ON COLUMN construction_progress_reports.materials_used IS
  'JSONB array: [{"material":"Cement","quantity":50,"unit":"bags"}]';

CREATE INDEX idx_constr_progress_task ON construction_progress_reports(task_id);

-- ── Abandonment Requests (UC-SC-03) ───────────────────────────
CREATE TABLE abandonment_requests (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  settlement_id     UUID NOT NULL REFERENCES settlements(id),
  commander_id      UUID NOT NULL REFERENCES users(id),
  anomaly_report_id UUID NOT NULL REFERENCES anomaly_reports(id),
  reason            TEXT NOT NULL,
  status            TEXT NOT NULL DEFAULT 'pending_vote'
                        CHECK (status IN ('pending_vote','approved','rejected')),
  vote_session_id   UUID REFERENCES vote_sessions(id),
  created_at        TIMESTAMPTZ DEFAULT NOW(),
  deleted_at        TIMESTAMPTZ,
  deleted_by        UUID REFERENCES users(id)
);

CREATE INDEX idx_abandon_settlement ON abandonment_requests(settlement_id) WHERE deleted_at IS NULL;

-- ── Repatriation Requests (UC-SC-04) ──────────────────────────
CREATE TABLE repatriation_requests (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  settlement_id     UUID NOT NULL REFERENCES settlements(id),
  commander_id      UUID NOT NULL REFERENCES users(id),
  settler_id        UUID NOT NULL REFERENCES users(id),
  complaint_id      UUID NOT NULL REFERENCES settler_complaints(id),
  reason            TEXT NOT NULL,
  status            TEXT NOT NULL DEFAULT 'pending_vote'
                        CHECK (status IN ('pending_vote','approved','rejected')),
  vote_session_id   UUID REFERENCES vote_sessions(id),
  created_at        TIMESTAMPTZ DEFAULT NOW(),
  deleted_at        TIMESTAMPTZ,
  deleted_by        UUID REFERENCES users(id)
);

CREATE INDEX idx_repatriation_settlement ON repatriation_requests(settlement_id) WHERE deleted_at IS NULL;

-- ── Row-Level Security ────────────────────────────────────────
ALTER TABLE settlements                   ENABLE ROW LEVEL SECURITY;
ALTER TABLE settler_assignments           ENABLE ROW LEVEL SECURITY;
ALTER TABLE settler_tasks                 ENABLE ROW LEVEL SECURITY;
ALTER TABLE settler_progress_reports      ENABLE ROW LEVEL SECURITY;
ALTER TABLE settlement_inventory          ENABLE ROW LEVEL SECURITY;
ALTER TABLE anomaly_reports               ENABLE ROW LEVEL SECURITY;
ALTER TABLE settler_complaints            ENABLE ROW LEVEL SECURITY;
ALTER TABLE supply_requests               ENABLE ROW LEVEL SECURITY;
ALTER TABLE building_health_logs          ENABLE ROW LEVEL SECURITY;
ALTER TABLE farm_health_logs              ENABLE ROW LEVEL SECURITY;
ALTER TABLE construction_progress_reports ENABLE ROW LEVEL SECURITY;
ALTER TABLE abandonment_requests          ENABLE ROW LEVEL SECURITY;
ALTER TABLE repatriation_requests         ENABLE ROW LEVEL SECURITY;

-- Service role bypasses all RLS
CREATE POLICY "service_role_all" ON settlements                   FOR ALL TO service_role USING (true);
CREATE POLICY "service_role_all" ON settler_assignments           FOR ALL TO service_role USING (true);
CREATE POLICY "service_role_all" ON settler_tasks                 FOR ALL TO service_role USING (true);
CREATE POLICY "service_role_all" ON settler_progress_reports      FOR ALL TO service_role USING (true);
CREATE POLICY "service_role_all" ON settlement_inventory          FOR ALL TO service_role USING (true);
CREATE POLICY "service_role_all" ON anomaly_reports               FOR ALL TO service_role USING (true);
CREATE POLICY "service_role_all" ON settler_complaints            FOR ALL TO service_role USING (true);
CREATE POLICY "service_role_all" ON supply_requests               FOR ALL TO service_role USING (true);
CREATE POLICY "service_role_all" ON building_health_logs          FOR ALL TO service_role USING (true);
CREATE POLICY "service_role_all" ON farm_health_logs              FOR ALL TO service_role USING (true);
CREATE POLICY "service_role_all" ON construction_progress_reports FOR ALL TO service_role USING (true);
CREATE POLICY "service_role_all" ON abandonment_requests          FOR ALL TO service_role USING (true);
CREATE POLICY "service_role_all" ON repatriation_requests         FOR ALL TO service_role USING (true);
