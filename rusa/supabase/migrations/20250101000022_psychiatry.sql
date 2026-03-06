-- Migration: Psychiatry Division (Sub-07)
-- Source of truth: 07_PSYCHIATRY.md
-- Tables: psychiatric_patients, psychiatric_appointments, psychiatric_recovery_logs,
--         psychiatrist_schedule, patient_schedule_access, patient_index

-- ── Core patient record (private per psychiatrist) ─────────────────────────────
CREATE TABLE IF NOT EXISTS psychiatric_patients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    psychiatrist_id UUID NOT NULL REFERENCES users(id),
    -- JSONB shape:
    -- {
    --   "full_name": "...",
    --   "demographics": { "dob": "...", "gender": "...", "nationality": "..." },
    --   "emergency_contact": { "name": "...", "relation": "...", "phone": "..." },
    --   "allergies": ["...", "..."],
    --   "medications": ["...", "..."],
    --   "history_of_illnesses": "..."
    -- }
    patient_profile JSONB NOT NULL DEFAULT '{}',
    initial_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    deleted_by UUID REFERENCES users(id),
    UNIQUE (user_id, psychiatrist_id)
);

-- ── Appointment records ────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS psychiatric_appointments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID NOT NULL REFERENCES psychiatric_patients(id),
    psychiatrist_id UUID NOT NULL REFERENCES users(id),
    scheduled_at TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'scheduled' CHECK (status IN ('scheduled','completed','cancelled')),
    booked_by UUID REFERENCES users(id),
    -- JSONB shape (psychiatrist-only; hidden from assistant):
    -- {
    --   "complaint": "...",
    --   "findings": "...",
    --   "resolution": "...",
    --   "wellbeing_assessment": "..."
    -- }
    appointment_log JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    deleted_by UUID REFERENCES users(id)
);

-- ── Recovery log ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS psychiatric_recovery_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_id UUID NOT NULL REFERENCES psychiatric_patients(id),
    milestone TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('in_progress','achieved','regressed','on_hold')),
    details TEXT,  -- psychiatrist-only unless base has_data_regulation = FALSE
    logged_at TIMESTAMPTZ DEFAULT NOW(),
    logged_by UUID REFERENCES users(id),
    deleted_at TIMESTAMPTZ,
    deleted_by UUID REFERENCES users(id)
);

-- ── Psychiatrist availability schedule ─────────────────────────────────────────
CREATE TABLE IF NOT EXISTS psychiatrist_schedule (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    psychiatrist_id UUID NOT NULL REFERENCES users(id),
    slot_start TIMESTAMPTZ NOT NULL,
    slot_end TIMESTAMPTZ NOT NULL,
    is_available BOOLEAN NOT NULL DEFAULT TRUE,
    blocked_reason TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    deleted_by UUID REFERENCES users(id)
);

-- ── Patient schedule access grants (assistant ↔ patient) ──────────────────────
CREATE TABLE IF NOT EXISTS patient_schedule_access (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    patient_user_id UUID NOT NULL REFERENCES users(id),
    assistant_id UUID NOT NULL REFERENCES users(id),
    granted BOOLEAN NOT NULL DEFAULT FALSE,
    requested_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE (patient_user_id, assistant_id)
);

-- ── Cross-department patient index ─────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS patient_index (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) UNIQUE,
    department TEXT NOT NULL CHECK (department IN ('psychiatry','medical','both')),
    care_status TEXT NOT NULL DEFAULT 'active' CHECK (care_status IN ('active','discharged','transferred')),
    psychiatrist_id UUID REFERENCES users(id),
    medical_staff_id UUID REFERENCES users(id),
    last_updated TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    deleted_by UUID REFERENCES users(id)
);

-- ── Indexes ────────────────────────────────────────────────────────────────────
CREATE INDEX IF NOT EXISTS idx_psych_patients_psychiatrist ON psychiatric_patients(psychiatrist_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_psych_patients_user ON psychiatric_patients(user_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_psych_appointments_patient ON psychiatric_appointments(patient_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_psych_appointments_psychiatrist ON psychiatric_appointments(psychiatrist_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_psych_appointments_scheduled ON psychiatric_appointments(scheduled_at) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_psych_recovery_patient ON psychiatric_recovery_logs(patient_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_psych_schedule_doctor ON psychiatrist_schedule(psychiatrist_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_patient_index_dept ON patient_index(department) WHERE deleted_at IS NULL;
