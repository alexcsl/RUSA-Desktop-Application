-- Migration: Psychiatrist ↔ Assistant explicit assignment table
-- Source of truth: 07_PSYCHIATRY.md FR-7-09
-- "each psychiatrist can only have one assistant"
-- UNIQUE(psychiatrist_id) enforces the 1-to-1 constraint.

CREATE TABLE IF NOT EXISTS psychiatrist_assistant_assignments (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    psychiatrist_id  UUID NOT NULL REFERENCES users(id),
    assistant_id     UUID NOT NULL REFERENCES users(id),
    assigned_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at       TIMESTAMPTZ,
    deleted_by       UUID REFERENCES users(id),
    -- One ACTIVE assistant per psychiatrist (partial index allows soft-deleted rows)
    CONSTRAINT psychiatrist_one_active_assistant EXCLUDE (psychiatrist_id WITH =) WHERE (deleted_at IS NULL)
);

CREATE INDEX IF NOT EXISTS idx_psych_assignments_assistant
    ON psychiatrist_assistant_assignments(assistant_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_psych_assignments_psychiatrist
    ON psychiatrist_assistant_assignments(psychiatrist_id)
    WHERE deleted_at IS NULL;
