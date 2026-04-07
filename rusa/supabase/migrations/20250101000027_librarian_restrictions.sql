-- Migration: Document access restrictions and redaction log
-- Source of truth: FR-11-15, UC-LIB-01, UC-LIB-02

CREATE TABLE IF NOT EXISTS document_restrictions (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id      UUID NOT NULL,
    restricted_by    UUID NOT NULL REFERENCES users(id),
    restriction_type TEXT NOT NULL CHECK (restriction_type IN ('full_access_control', 'field_redaction')),
    allowed_roles    TEXT[] NOT NULL DEFAULT '{}',
    notes            TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at       TIMESTAMPTZ,
    deleted_by       UUID REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS document_redactions (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id  UUID NOT NULL,
    field_name   TEXT NOT NULL,
    redacted_by  UUID NOT NULL REFERENCES users(id),
    redacted_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reason       TEXT,
    deleted_at   TIMESTAMPTZ,
    deleted_by   UUID REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_doc_restrictions_document
    ON document_restrictions(document_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_doc_redactions_document
    ON document_redactions(document_id)
    WHERE deleted_at IS NULL;
