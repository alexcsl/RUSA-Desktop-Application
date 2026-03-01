-- ============================================================
-- 020_sanitary_department.sql
-- Sanitary Department subsystem tables.
-- Source of truth: 10_SANITARY.md
-- ============================================================

-- ════════════════════════════════════════════════════════════
-- SANITARY DIVISIONS (Inspector, Disposal, Wastewater, Cleanup, Transport)
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS sanitary_divisions (
  id    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name  TEXT NOT NULL UNIQUE CHECK (name IN (
    'inspector','disposal','wastewater','cleanup','transport'
  )),
  quota INTEGER NOT NULL DEFAULT 0
);

-- Seed the five divisions
INSERT INTO sanitary_divisions (name, quota) VALUES
  ('inspector',  10),
  ('disposal',   10),
  ('wastewater', 10),
  ('cleanup',    15),
  ('transport',  10)
ON CONFLICT (name) DO NOTHING;

-- ════════════════════════════════════════════════════════════
-- STAFF DIVISION ASSIGNMENTS (Historical — never deleted)
-- One row per quarter per staff member.
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS staff_division_assignments (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id     UUID NOT NULL REFERENCES users(id),
  division_id UUID NOT NULL REFERENCES sanitary_divisions(id),
  quarter     TEXT NOT NULL,          -- e.g. 'Q1-2026'
  assigned_by UUID REFERENCES users(id),
  assigned_at TIMESTAMPTZ DEFAULT NOW()
  -- Historical: never deleted — new quarter creates a new row
);

CREATE INDEX idx_sda_user ON staff_division_assignments(user_id);
CREATE INDEX idx_sda_quarter ON staff_division_assignments(quarter);

-- ════════════════════════════════════════════════════════════
-- DIVISION TRANSFER REQUESTS (UC-STAS-03 / UC-HS-06)
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS division_transfer_requests (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  requested_by     UUID NOT NULL REFERENCES users(id),
  from_division_id UUID NOT NULL REFERENCES sanitary_divisions(id),
  to_division_id   UUID NOT NULL REFERENCES sanitary_divisions(id),
  reason           TEXT,
  status           TEXT NOT NULL DEFAULT 'pending'
                       CHECK (status IN ('pending','approved','rejected')),
  decided_by       UUID REFERENCES users(id),
  decided_at       TIMESTAMPTZ,
  created_at       TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_dtr_status ON division_transfer_requests(status)
  WHERE status = 'pending';

-- ════════════════════════════════════════════════════════════
-- SANITARY TASKS (UC-HS-05 / UC-STAS-01)
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS sanitary_tasks (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  assigned_by      UUID NOT NULL REFERENCES users(id),
  task_type        TEXT NOT NULL CHECK (task_type IN (
    'cleaning','inspection','disposal','transport','other'
  )),
  task_name        TEXT,
  urgency          TEXT CHECK (urgency IN ('low','medium','high','critical')),
  instructions     TEXT,
  location         TEXT,
  due_date         TIMESTAMPTZ,
  status           TEXT NOT NULL DEFAULT 'pending'
                       CHECK (status IN ('pending','in_progress','completed')),
  source_report_id UUID,           -- if triggered by an inspector report
  created_at       TIMESTAMPTZ DEFAULT NOW(),
  deleted_at       TIMESTAMPTZ,
  deleted_by       UUID REFERENCES users(id)
);

CREATE INDEX idx_san_tasks_status ON sanitary_tasks(status)
  WHERE deleted_at IS NULL;

-- ════════════════════════════════════════════════════════════
-- SANITARY TASK ASSIGNMENTS (many-to-many: tasks ↔ staff)
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS sanitary_task_assignments (
  task_id UUID NOT NULL REFERENCES sanitary_tasks(id),
  user_id UUID NOT NULL REFERENCES users(id),
  PRIMARY KEY (task_id, user_id)
);

-- ════════════════════════════════════════════════════════════
-- SANITARY INVENTORY (UC-HS-03 macro / UC-STAS-04 micro)
-- Department-level inventory of cleaning supplies and equipment.
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS sanitary_inventory (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  item_name       TEXT NOT NULL,
  category        TEXT,              -- e.g. 'cleaning_product', 'equipment', 'chemicals'
  quantity        INTEGER NOT NULL DEFAULT 0,
  unit            TEXT,
  last_updated_by UUID REFERENCES users(id),
  updated_at      TIMESTAMPTZ DEFAULT NOW(),
  deleted_at      TIMESTAMPTZ,
  deleted_by      UUID REFERENCES users(id)
);

CREATE INDEX idx_san_inv ON sanitary_inventory(id)
  WHERE deleted_at IS NULL;

-- ════════════════════════════════════════════════════════════
-- SANITARY INVENTORY LOGS (UC-STAS-04 micro)
-- Staff records usage (remove) or purchase (add).
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS sanitary_inventory_logs (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  item_id       UUID NOT NULL REFERENCES sanitary_inventory(id),
  logged_by     UUID NOT NULL REFERENCES users(id),
  action        TEXT NOT NULL CHECK (action IN ('add','remove')),
  quantity      INTEGER NOT NULL,
  purchase_note TEXT,
  logged_at     TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_san_inv_logs_item ON sanitary_inventory_logs(item_id);

-- ════════════════════════════════════════════════════════════
-- INSPECTION REPORTS (UC-IC-01)
-- Inspector Crew submits to Head of Sanitary.
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS inspection_reports (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  reported_by     UUID NOT NULL REFERENCES users(id),
  report_date     DATE NOT NULL,
  location        TEXT NOT NULL,
  area_or_machine TEXT NOT NULL,
  findings        TEXT NOT NULL,
  severity        TEXT NOT NULL CHECK (severity IN ('low','medium','high','critical')),
  recommendations TEXT,
  created_at      TIMESTAMPTZ DEFAULT NOW(),
  deleted_at      TIMESTAMPTZ,
  deleted_by      UUID REFERENCES users(id)
);

CREATE INDEX idx_inspection_reports ON inspection_reports(reported_by)
  WHERE deleted_at IS NULL;

-- ════════════════════════════════════════════════════════════
-- DISPOSAL HANDLING DOCUMENTATION (UC-DC-01)
-- Replaces MongoDB disposal_handling_docs.
-- revision_history: append-only JSONB array.
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS disposal_handling_docs (
  id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  waste_category      TEXT NOT NULL,
  procedure           TEXT NOT NULL,
  safety_requirements TEXT,
  compliance_notes    TEXT,
  -- revision_history: append-only JSONB array
  -- Shape: [
  --   {
  --     "revision": 1,
  --     "changed_by": "uuid",
  --     "changed_at": "ISO datetime",
  --     "summary": "what changed"
  --   }
  -- ]
  revision_history    JSONB NOT NULL DEFAULT '[]',
  authored_by         UUID REFERENCES users(id),
  created_at          TIMESTAMPTZ DEFAULT NOW(),
  deleted_at          TIMESTAMPTZ,
  deleted_by          UUID REFERENCES users(id)
);

CREATE INDEX idx_disposal_docs_category ON disposal_handling_docs(waste_category)
  WHERE deleted_at IS NULL;

-- ════════════════════════════════════════════════════════════
-- WASTEWATER TREATMENT DOCUMENTATION (UC-WC-01)
-- Replaces MongoDB wastewater_treatment_docs.
-- steps: ordered JSONB array of strings.
-- revision_history: same append pattern as disposal docs.
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS wastewater_treatment_docs (
  id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  treatment_type      TEXT NOT NULL,
  -- steps: ordered JSONB array
  -- Shape: ["Step 1: ...", "Step 2: ...", ...]
  steps               JSONB NOT NULL DEFAULT '[]',
  safety_requirements TEXT,
  compliance_notes    TEXT,
  -- revision_history: same append-only JSONB pattern as disposal docs
  revision_history    JSONB NOT NULL DEFAULT '[]',
  authored_by         UUID REFERENCES users(id),
  created_at          TIMESTAMPTZ DEFAULT NOW(),
  deleted_at          TIMESTAMPTZ,
  deleted_by          UUID REFERENCES users(id)
);

CREATE INDEX idx_ww_docs_type ON wastewater_treatment_docs(treatment_type)
  WHERE deleted_at IS NULL;

-- ════════════════════════════════════════════════════════════
-- SANITARY BUDGET REQUESTS (UC-HS-01)
-- Same pattern as medical budget requests.
-- line_items JSONB shape:
-- [{ "item_name": "...", "quantity": N, "unit_cost": 0.00, "total": 0.00 }]
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS sanitary_budget_requests (
  id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  submitted_by         UUID NOT NULL REFERENCES users(id),
  line_items           JSONB NOT NULL,
  total_amount         NUMERIC,
  invoice_storage_path TEXT NOT NULL,    -- rusa-files: sanitary/budget-invoices/{id}/...
  justification        TEXT,
  status               TEXT NOT NULL DEFAULT 'pending_vote'
                           CHECK (status IN ('pending_vote','approved','denied')),
  vote_session_id      UUID REFERENCES vote_sessions(id),
  created_at           TIMESTAMPTZ DEFAULT NOW(),
  deleted_at           TIMESTAMPTZ,
  deleted_by           UUID REFERENCES users(id)
);

CREATE INDEX idx_san_budget_status ON sanitary_budget_requests(status)
  WHERE deleted_at IS NULL;

-- ════════════════════════════════════════════════════════════
-- SANITARY EXPENDITURE REPORTS (UC-HS-02)
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS sanitary_expenditure_reports (
  id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  submitted_by              UUID NOT NULL REFERENCES users(id),
  line_items                JSONB NOT NULL,
  total_amount              NUMERIC,
  submitted_to_accountant_at TIMESTAMPTZ,
  foul_play_flag            BOOLEAN NOT NULL DEFAULT FALSE,
  foul_play_note            TEXT,
  created_at                TIMESTAMPTZ DEFAULT NOW(),
  deleted_at                TIMESTAMPTZ,
  deleted_by                UUID REFERENCES users(id)
);

CREATE INDEX idx_san_exp_reports ON sanitary_expenditure_reports(submitted_by)
  WHERE deleted_at IS NULL;

-- ════════════════════════════════════════════════════════════
-- SANITARY STAFF SHIFTS (UC-HS-04 / UC-STAS-02)
-- Quarterly shift allocations made by HeadOfSanitary.
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS sanitary_shifts (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id      UUID NOT NULL REFERENCES users(id),
  shift_start  TIMESTAMPTZ NOT NULL,
  shift_end    TIMESTAMPTZ NOT NULL,
  allocated_by UUID NOT NULL REFERENCES users(id),
  quarter      TEXT,                   -- e.g. 'Q1-2026'
  created_at   TIMESTAMPTZ DEFAULT NOW(),
  deleted_at   TIMESTAMPTZ,
  deleted_by   UUID REFERENCES users(id)
);

CREATE INDEX idx_san_shifts_user ON sanitary_shifts(user_id)
  WHERE deleted_at IS NULL;
