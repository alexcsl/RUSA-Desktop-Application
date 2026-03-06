-- ============================================================
-- 003_messaging_subsystem.sql
-- Messaging infrastructure enhancements — shared across all channels.
-- Source of truth: 00_MASTER_GUIDE.md §6 (Messaging System),
--   03_SECURITY_TEAMS.md (UC-SH-04, UC-SS-02),
--   09_MEDICAL.md (UC-HOM-04),
--   11_DIRECTORS.md (UC-GUA-04, UC-OVR-02, UC-ANC-02)
-- ============================================================

-- ────────────────────────────────────────────────────────────
-- 1. Expand channel CHECK to include 'broadcast'
--    (original migration only had: general, security, medical_heads)
-- ────────────────────────────────────────────────────────────
ALTER TABLE messages DROP CONSTRAINT IF EXISTS messages_channel_check;
ALTER TABLE messages ADD CONSTRAINT messages_channel_check
  CHECK (channel IN ('general', 'security', 'medical_heads', 'broadcast'));

-- ────────────────────────────────────────────────────────────
-- 2. Message attachments (binary files stored in Supabase Storage)
--    Bucket: rusa-files  •  Path pattern: messages/{message_id}/{uuid}-{filename}
-- ────────────────────────────────────────────────────────────
CREATE TABLE message_attachments (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  message_id        UUID NOT NULL REFERENCES messages(id),
  original_filename TEXT NOT NULL,
  content_type      TEXT NOT NULL,         -- MIME type, e.g. 'application/pdf'
  storage_path      TEXT NOT NULL,         -- bucket-relative path in rusa-files
  file_size_bytes   BIGINT,
  uploaded_by       UUID NOT NULL REFERENCES users(id),
  uploaded_at       TIMESTAMPTZ DEFAULT NOW(),
  deleted_at        TIMESTAMPTZ,
  deleted_by        UUID REFERENCES users(id)
);

CREATE INDEX idx_msg_attachments_msg ON message_attachments(message_id)
  WHERE deleted_at IS NULL;

-- ────────────────────────────────────────────────────────────
-- 3. Scheduled message delivery tracking
--    messages.scheduled_at already exists on the messages table.
--    This index helps the scheduler find pending scheduled messages.
-- ────────────────────────────────────────────────────────────
CREATE INDEX idx_messages_scheduled ON messages(scheduled_at)
  WHERE deleted_at IS NULL
    AND recalled_at IS NULL
    AND scheduled_at IS NOT NULL;

-- ────────────────────────────────────────────────────────────
-- 4. Additional indexes for inbox queries
-- ────────────────────────────────────────────────────────────
CREATE INDEX idx_messages_from_id ON messages(from_id)
  WHERE deleted_at IS NULL;

CREATE INDEX idx_msg_recipients_read ON message_recipients(user_id)
  WHERE read_at IS NULL;

-- ────────────────────────────────────────────────────────────
-- 5. Group soft-delete tracking on group_members
-- ────────────────────────────────────────────────────────────
ALTER TABLE group_members ADD COLUMN IF NOT EXISTS removed_at TIMESTAMPTZ;

-- ────────────────────────────────────────────────────────────
-- JSONB Schema Documentation (comment annotations)
-- ────────────────────────────────────────────────────────────

-- notifications.payload JSONB shape (messaging context):
-- {
--   "message_id": "uuid",            -- when type = 'message:new'
--   "channel": "general|security|medical_heads|broadcast",
--   "from_name": "...",
--   "subject": "...",
--   "preview": "first 100 chars of body"
-- }

-- messages table: No JSONB columns. All fields are direct columns.
-- message_recipients table: No JSONB columns.
-- message_attachments table: No JSONB columns.
