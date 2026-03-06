-- ============================================================
-- 016_medical_department.sql
-- Medical Department subsystem tables.
-- Source of truth: 09_MEDICAL.md
-- ============================================================

-- ════════════════════════════════════════════════════════════
-- MEDICAL STAFF PROFILES (specialty metadata for shift allocation)
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS medical_staff_profiles (
  user_id    UUID PRIMARY KEY REFERENCES users(id),
  specialty  TEXT NOT NULL,            -- e.g. 'cardiology', 'surgery', 'general', 'emergency', 'neurology'
  base_id    UUID REFERENCES base_locations(id),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- ════════════════════════════════════════════════════════════
-- MEDICAL PATIENTS (cross-department patient index)
-- Links to users table for identity.  Clinical data stays in
-- patient_treatment_logs, never exposed outside department.
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS medical_patients (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id    UUID REFERENCES users(id) UNIQUE NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ,
  deleted_by UUID REFERENCES users(id)
);

CREATE INDEX idx_medical_patients_user ON medical_patients(user_id)
  WHERE deleted_at IS NULL;

-- ════════════════════════════════════════════════════════════
-- PATIENT TREATMENT LOGS (UC-MED-01 / UC-MED-02)
-- One row per treatment encounter.
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS patient_treatment_logs (
  id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  patient_id         UUID NOT NULL REFERENCES medical_patients(id),
  treated_by         UUID NOT NULL REFERENCES users(id),
  treatment_date     TIMESTAMPTZ NOT NULL,
  diagnosis          TEXT NOT NULL,
  treatment_provided TEXT NOT NULL,
  medications        TEXT,
  follow_up_notes    TEXT,
  created_at         TIMESTAMPTZ DEFAULT NOW(),
  deleted_at         TIMESTAMPTZ,
  deleted_by         UUID REFERENCES users(id)
);

CREATE INDEX idx_treatment_logs_patient ON patient_treatment_logs(patient_id)
  WHERE deleted_at IS NULL;
CREATE INDEX idx_treatment_logs_treater ON patient_treatment_logs(treated_by)
  WHERE deleted_at IS NULL;

-- ════════════════════════════════════════════════════════════
-- MEDICAL INVENTORY (UC-MED-03)
-- Per-base inventory of medicines, equipment and supplies.
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS medical_inventory (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  base_id         UUID NOT NULL REFERENCES base_locations(id),
  item_name       TEXT NOT NULL,
  item_type       TEXT NOT NULL CHECK (item_type IN ('medicine','equipment','supply')),
  quantity        INTEGER NOT NULL DEFAULT 0,
  unit            TEXT,
  last_updated_by UUID REFERENCES users(id),
  updated_at      TIMESTAMPTZ DEFAULT NOW(),
  deleted_at      TIMESTAMPTZ,
  deleted_by      UUID REFERENCES users(id)
);

CREATE INDEX idx_medical_inv_base ON medical_inventory(base_id)
  WHERE deleted_at IS NULL;

-- ════════════════════════════════════════════════════════════
-- STAFF SHIFTS (UC-MED-04 / UC-HOM-01)
-- Shift allocations created by HeadOfMedicine, viewed by staff.
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS staff_shifts (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id      UUID NOT NULL REFERENCES users(id),
  base_id      UUID NOT NULL REFERENCES base_locations(id),
  shift_start  TIMESTAMPTZ NOT NULL,
  shift_end    TIMESTAMPTZ NOT NULL,
  allocated_by UUID NOT NULL REFERENCES users(id),
  created_at   TIMESTAMPTZ DEFAULT NOW(),
  deleted_at   TIMESTAMPTZ,
  deleted_by   UUID REFERENCES users(id)
);

CREATE INDEX idx_shifts_user ON staff_shifts(user_id)
  WHERE deleted_at IS NULL;
CREATE INDEX idx_shifts_base ON staff_shifts(base_id)
  WHERE deleted_at IS NULL;

-- ════════════════════════════════════════════════════════════
-- MEDICAL BUDGET REQUESTS (UC-HOM-02)
-- Itemised budget requests with mandatory invoice upload.
-- Routes to Directors voting queue.
--
-- line_items JSONB shape:
-- [
--   { "item_name": "Portable Ultrasound", "quantity": 5, "unit_cost": 25000.00, "total": 125000.00 }
-- ]
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS medical_budget_requests (
  id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  submitted_by         UUID NOT NULL REFERENCES users(id),
  line_items           JSONB NOT NULL,
  total_amount         NUMERIC,
  invoice_storage_path TEXT NOT NULL,     -- rusa-files bucket: medical/budget-invoices/{id}/invoice.pdf
  justification        TEXT,
  status               TEXT NOT NULL DEFAULT 'pending_vote'
                           CHECK (status IN ('pending_vote','approved','denied')),
  vote_session_id      UUID REFERENCES vote_sessions(id),
  created_at           TIMESTAMPTZ DEFAULT NOW(),
  deleted_at           TIMESTAMPTZ,
  deleted_by           UUID REFERENCES users(id)
);

CREATE INDEX idx_med_budget_status ON medical_budget_requests(status)
  WHERE deleted_at IS NULL;

-- ════════════════════════════════════════════════════════════
-- MEDICAL EXPENDITURE REPORTS (UC-HOM-03)
-- Expense reports submitted to The Accountant.
--
-- line_items JSONB shape: same as budget requests.
-- ════════════════════════════════════════════════════════════
CREATE TABLE IF NOT EXISTS medical_expenditure_reports (
  id                        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  submitted_by              UUID NOT NULL REFERENCES users(id),
  line_items                JSONB NOT NULL,
  total_amount              NUMERIC,
  foul_play_flag            BOOLEAN NOT NULL DEFAULT FALSE,
  foul_play_note            TEXT,
  submitted_to_accountant_at TIMESTAMPTZ,
  created_at                TIMESTAMPTZ DEFAULT NOW(),
  deleted_at                TIMESTAMPTZ,
  deleted_by                UUID REFERENCES users(id)
);

CREATE INDEX idx_med_exp_reports ON medical_expenditure_reports(submitted_by)
  WHERE deleted_at IS NULL;
