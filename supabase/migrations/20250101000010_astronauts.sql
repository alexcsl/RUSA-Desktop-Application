-- Migration 010: Astronauts subsystem tables
-- Source of truth: 05_ASTRONAUTS.md, 00_MASTER_GUIDE.md
--
-- Tables created:
--   missions             — Mission documents assigned by Wanderer
--   mission_assignments  — Many-to-many: astronauts ↔ missions
--   mission_status_reports — UC-AS-03: periodic status reports
--   mission_completion_requests — UC-AS-04: completion with evidence files
--   mission_counters     — Official interstellar/terrain tallies per astronaut
--
-- Supabase Storage:
--   Bucket: rusa-files
--   Path:   missions/completion-evidence/{request_id}/{filename}
--   Column: mission_completion_requests.evidence_storage_paths TEXT[]

-- ── Missions ──────────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS missions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title           TEXT NOT NULL,
    type            TEXT NOT NULL CHECK (type IN ('interstellar', 'terrain')),
    danger_level    TEXT CHECK (danger_level IN ('low', 'medium', 'high', 'critical')),
    location        TEXT NOT NULL,
    mission_objective TEXT,
    procedures      TEXT,
    known_dangers   TEXT,
    assigned_by     UUID REFERENCES users(id),
    status          TEXT NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active', 'completion_requested', 'completed', 'rejected')),
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    deleted_by      UUID REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_missions_status ON missions(status) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_missions_assigned_by ON missions(assigned_by) WHERE deleted_at IS NULL;

-- ── Mission Assignments (astronaut ↔ mission) ────────────────────────────────

CREATE TABLE IF NOT EXISTS mission_assignments (
    mission_id   UUID REFERENCES missions(id),
    astronaut_id UUID REFERENCES users(id),
    assigned_at  TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (mission_id, astronaut_id)
);

CREATE INDEX IF NOT EXISTS idx_mission_assignments_astronaut ON mission_assignments(astronaut_id);

-- ── Mission Status Reports (UC-AS-03) ─────────────────────────────────────────

CREATE TABLE IF NOT EXISTS mission_status_reports (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mission_id                  UUID NOT NULL REFERENCES missions(id),
    submitted_by                UUID NOT NULL REFERENCES users(id),
    report_date                 TIMESTAMPTZ NOT NULL,
    month_tracker               TEXT,               -- e.g. "Month 3"
    rag_status                  TEXT CHECK (rag_status IN ('red', 'amber', 'green')),
    current_status              TEXT NOT NULL,
    issues_blockers             TEXT,
    collected_samples_last_month TEXT,
    progress_last_month         TEXT,
    plans_next_month            TEXT,
    created_at                  TIMESTAMPTZ DEFAULT NOW(),
    deleted_at                  TIMESTAMPTZ,
    deleted_by                  UUID REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_mission_status_reports_mission ON mission_status_reports(mission_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_mission_status_reports_by ON mission_status_reports(submitted_by) WHERE deleted_at IS NULL;

-- ── Mission Completion Requests (UC-AS-04) ────────────────────────────────────

CREATE TABLE IF NOT EXISTS mission_completion_requests (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mission_id              UUID NOT NULL REFERENCES missions(id),
    submitted_by            UUID NOT NULL REFERENCES users(id),
    findings_summary        TEXT NOT NULL,
    -- Evidence files uploaded to Supabase Storage (rusa-files bucket).
    -- Path pattern: missions/completion-evidence/{request_id}/{filename}
    -- JSONB schema: TEXT[] of storage_path strings
    evidence_storage_paths  TEXT[],
    status                  TEXT NOT NULL DEFAULT 'pending_wanderer'
                            CHECK (status IN ('pending_wanderer', 'pending_taskmaster', 'approved', 'rejected')),
    wanderer_note           TEXT,
    taskmaster_note         TEXT,
    created_at              TIMESTAMPTZ DEFAULT NOW(),
    updated_at              TIMESTAMPTZ DEFAULT NOW(),
    deleted_at              TIMESTAMPTZ,
    deleted_by              UUID REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_mission_completion_requests_mission ON mission_completion_requests(mission_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_mission_completion_requests_status ON mission_completion_requests(status) WHERE deleted_at IS NULL;

-- ── Mission Counters (official tally, per astronaut) ──────────────────────────

CREATE TABLE IF NOT EXISTS mission_counters (
    astronaut_id        UUID PRIMARY KEY REFERENCES users(id),
    interstellar_count  INTEGER NOT NULL DEFAULT 0,
    terrain_count       INTEGER NOT NULL DEFAULT 0,
    updated_at          TIMESTAMPTZ DEFAULT NOW()
);

-- ── Add Astronaut role if not already present ─────────────────────────────────

INSERT INTO roles (name, description)
VALUES ('Astronaut', 'Space exploration personnel — interstellar and terrain missions')
ON CONFLICT (name) DO NOTHING;
