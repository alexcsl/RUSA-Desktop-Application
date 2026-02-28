-- ============================================================
-- 003_shared_and_directors.sql
-- Shared infrastructure (notifications, messaging, requests,
-- tasks, voting) + Director-specific tables.
-- Source of truth: 00_MASTER_GUIDE.md §6-§7, 11_DIRECTORS.md
-- ============================================================

-- ============================================================
-- NOTIFICATIONS  (real-time push queue — all subsystems)
-- ============================================================
CREATE TABLE notifications (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id    UUID NOT NULL REFERENCES users(id),
  type       TEXT NOT NULL,             -- e.g. 'vote:new', 'task:assigned', 'meeting:invited'
  payload    JSONB NOT NULL DEFAULT '{}',
  read_at    TIMESTAMPTZ,               -- null = unread
  created_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ,
  deleted_by UUID REFERENCES users(id)
);

CREATE INDEX idx_notifications_user   ON notifications(user_id) WHERE deleted_at IS NULL AND read_at IS NULL;
CREATE INDEX idx_notifications_type   ON notifications(user_id, type) WHERE deleted_at IS NULL;

-- ============================================================
-- MESSAGING (email-esque — Master Guide §6)
-- ============================================================
CREATE TABLE groups (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name        TEXT NOT NULL,
  description TEXT,
  created_by  UUID REFERENCES users(id),
  created_at  TIMESTAMPTZ DEFAULT NOW(),
  deleted_at  TIMESTAMPTZ,
  deleted_by  UUID REFERENCES users(id)
);

CREATE TABLE group_members (
  group_id UUID NOT NULL REFERENCES groups(id),
  user_id  UUID NOT NULL REFERENCES users(id),
  added_at TIMESTAMPTZ DEFAULT NOW(),
  PRIMARY KEY (group_id, user_id)
);

CREATE TABLE messages (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  from_id      UUID NOT NULL REFERENCES users(id),
  subject      TEXT NOT NULL,
  body         TEXT NOT NULL,
  channel      TEXT NOT NULL DEFAULT 'general'
                    CHECK (channel IN ('general','security','medical_heads')),
  scheduled_at TIMESTAMPTZ,            -- null = immediate
  recalled_at  TIMESTAMPTZ,            -- set when sender recalls (if unread)
  created_at   TIMESTAMPTZ DEFAULT NOW(),
  deleted_at   TIMESTAMPTZ,
  deleted_by   UUID REFERENCES users(id)
);

CREATE TABLE message_recipients (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  message_id UUID NOT NULL REFERENCES messages(id),
  user_id    UUID NOT NULL REFERENCES users(id),
  type       TEXT NOT NULL CHECK (type IN ('to','cc','bcc')),
  read_at    TIMESTAMPTZ,              -- null = unread
  UNIQUE (message_id, user_id, type)
);

CREATE INDEX idx_messages_channel ON messages(channel) WHERE deleted_at IS NULL;
CREATE INDEX idx_msg_recipients   ON message_recipients(user_id, message_id);

-- ============================================================
-- REQUESTS (generic envelope — all votable and bypassable items)
-- ============================================================
CREATE TABLE requests (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  type             TEXT NOT NULL,       -- e.g. 'budget','data','broadcast','ad_hoc_vote','experiment_closure','mission_completion'
  status           TEXT NOT NULL DEFAULT 'pending'
                       CHECK (status IN ('pending','in_vote','approved','denied','bypassed','cancelled')),
  requester_id     UUID NOT NULL REFERENCES users(id),
  title            TEXT NOT NULL,
  description      TEXT,
  payload          JSONB NOT NULL DEFAULT '{}',  -- flexible per-type data
  bypass_authority TEXT,                -- null = goes to Directors vote; 'TheStatistician', 'TheGuardian', 'TheTaskmaster'
  decided_by       UUID REFERENCES users(id),    -- populated when bypass authority or admin decides
  decision_reason  TEXT,
  invoice_storage_path TEXT,            -- for budget requests (rusa-files bucket)
  created_at       TIMESTAMPTZ DEFAULT NOW(),
  updated_at       TIMESTAMPTZ DEFAULT NOW(),
  deleted_at       TIMESTAMPTZ,
  deleted_by       UUID REFERENCES users(id)
);

CREATE INDEX idx_requests_status    ON requests(status)       WHERE deleted_at IS NULL;
CREATE INDEX idx_requests_type      ON requests(type)         WHERE deleted_at IS NULL;
CREATE INDEX idx_requests_requester ON requests(requester_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_requests_bypass    ON requests(bypass_authority) WHERE deleted_at IS NULL AND bypass_authority IS NOT NULL;

-- ============================================================
-- VOTE SESSIONS  (one per votable request — Master Guide §6)
-- ============================================================
CREATE TABLE vote_sessions (
  id                       UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  request_id               UUID REFERENCES requests(id),  -- null for ad-hoc votes
  topic                    TEXT NOT NULL,                  -- display title for the vote
  context                  TEXT,                           -- supporting information
  status                   TEXT NOT NULL DEFAULT 'open'
                               CHECK (status IN ('open','quorum_pending','closed','decided','overridden','terminated')),
  opens_at                 TIMESTAMPTZ NOT NULL,
  closes_at                TIMESTAMPTZ,                    -- null until quorum met or admin action
  result                   TEXT CHECK (result IN ('approved','denied')),
  total_yay                INT NOT NULL DEFAULT 0,
  total_nay                INT NOT NULL DEFAULT 0,
  total_abstain            INT NOT NULL DEFAULT 0,
  -- Administrator override columns
  admin_overridden         BOOLEAN NOT NULL DEFAULT FALSE,
  admin_override_decision  TEXT CHECK (admin_override_decision IN ('approved','denied')),
  admin_override_reason    TEXT,
  admin_terminated         BOOLEAN NOT NULL DEFAULT FALSE,
  created_by               UUID NOT NULL REFERENCES users(id),
  created_at               TIMESTAMPTZ DEFAULT NOW(),
  updated_at               TIMESTAMPTZ DEFAULT NOW(),
  deleted_at               TIMESTAMPTZ,
  deleted_by               UUID REFERENCES users(id)
);

CREATE INDEX idx_vote_sessions_status ON vote_sessions(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_vote_sessions_opens  ON vote_sessions(opens_at) WHERE deleted_at IS NULL;

-- ============================================================
-- VOTE RECORDS  (individual director votes per session)
-- ============================================================
CREATE TABLE vote_records (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  session_id UUID NOT NULL REFERENCES vote_sessions(id),
  director_id UUID NOT NULL REFERENCES users(id),
  vote       TEXT NOT NULL CHECK (vote IN ('yay','nay','abstain')),
  reason     TEXT NOT NULL,
  changed_at TIMESTAMPTZ,              -- set when vote is changed (null on first cast)
  created_at TIMESTAMPTZ DEFAULT NOW(),
  UNIQUE (session_id, director_id)     -- one vote per director per session
);

CREATE INDEX idx_vote_records_session ON vote_records(session_id);

-- ============================================================
-- TASKS  (generic task assignments — all subsystems)
-- ============================================================
CREATE TABLE tasks (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  assigned_by UUID NOT NULL REFERENCES users(id),
  assigned_to UUID NOT NULL REFERENCES users(id),
  type        TEXT NOT NULL DEFAULT 'general',   -- 'general','construction','sanitary','mission','experiment','help_request','help_response'
  title       TEXT NOT NULL,
  description TEXT,
  payload     JSONB NOT NULL DEFAULT '{}',
  status      TEXT NOT NULL DEFAULT 'pending'
                  CHECK (status IN ('pending','in_progress','completed','rejected','cancelled')),
  due_date    DATE,
  completed_at TIMESTAMPTZ,
  created_at  TIMESTAMPTZ DEFAULT NOW(),
  updated_at  TIMESTAMPTZ DEFAULT NOW(),
  deleted_at  TIMESTAMPTZ,
  deleted_by  UUID REFERENCES users(id)
);

CREATE INDEX idx_tasks_assigned_to ON tasks(assigned_to)  WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_assigned_by ON tasks(assigned_by)  WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_status      ON tasks(status)       WHERE deleted_at IS NULL;

-- ============================================================
-- MEETINGS  (Director-created — 11_DIRECTORS.md UC-DIR-03)
-- ============================================================
CREATE TABLE meetings (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  title        TEXT NOT NULL,
  agenda       TEXT,
  scheduled_at TIMESTAMPTZ NOT NULL,
  created_by   UUID NOT NULL REFERENCES users(id),
  created_at   TIMESTAMPTZ DEFAULT NOW(),
  updated_at   TIMESTAMPTZ DEFAULT NOW(),
  deleted_at   TIMESTAMPTZ,
  deleted_by   UUID REFERENCES users(id)
);

CREATE TABLE meeting_invitees (
  meeting_id  UUID NOT NULL REFERENCES meetings(id),
  user_id     UUID NOT NULL REFERENCES users(id),
  notified_at TIMESTAMPTZ,
  PRIMARY KEY (meeting_id, user_id)
);

CREATE INDEX idx_meetings_scheduled ON meetings(scheduled_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_meetings_creator   ON meetings(created_by)   WHERE deleted_at IS NULL;

-- ============================================================
-- TERRITORY NAMES  (11_DIRECTORS.md UC-WAN-04)
-- ============================================================
CREATE TABLE territory_names (
  id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  territory_type TEXT NOT NULL CHECK (territory_type IN ('planet','sector')),
  name           TEXT NOT NULL,
  renamed_by     UUID REFERENCES users(id),
  renamed_at     TIMESTAMPTZ DEFAULT NOW(),
  previous_name  TEXT,
  deleted_at     TIMESTAMPTZ,
  deleted_by     UUID REFERENCES users(id)
);

-- ============================================================
-- EVENTS  (11_DIRECTORS.md UC-CORD-01/02/03)
-- ============================================================
CREATE TABLE events (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  title      TEXT NOT NULL,
  event_date TIMESTAMPTZ,
  location   TEXT,
  agenda     TEXT,
  created_by UUID NOT NULL REFERENCES users(id),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ,
  deleted_by UUID REFERENCES users(id)
);

CREATE TABLE event_attendees (
  event_id    UUID NOT NULL REFERENCES events(id),
  name        TEXT NOT NULL,
  is_external BOOLEAN NOT NULL DEFAULT FALSE,
  user_id     UUID REFERENCES users(id),  -- null for external attendees
  PRIMARY KEY (event_id, name)
);

-- Event documents (binary in Supabase Storage, metadata here)
CREATE TABLE event_documents (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id          UUID NOT NULL REFERENCES events(id),
  document_type     TEXT CHECK (document_type IN ('ticket','invoice','contract','other')),
  original_filename TEXT NOT NULL,
  content_type      TEXT NOT NULL,
  storage_path      TEXT NOT NULL,   -- bucket: rusa-files, pattern: events/{event_id}/{uuid}-{filename}
  uploaded_by       UUID NOT NULL REFERENCES users(id),
  uploaded_at       TIMESTAMPTZ DEFAULT NOW(),
  deleted_at        TIMESTAMPTZ,
  deleted_by        UUID REFERENCES users(id)
);

CREATE INDEX idx_events_date       ON events(event_date) WHERE deleted_at IS NULL;
CREATE INDEX idx_event_docs_event  ON event_documents(event_id) WHERE deleted_at IS NULL;

-- ============================================================
-- TERMINATION RECORDS  (11_DIRECTORS.md UC-ANC-03)
-- ============================================================
CREATE TABLE termination_records (
  id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  terminated_user_id UUID NOT NULL REFERENCES users(id),
  terminated_by      UUID NOT NULL REFERENCES users(id),
  reason             TEXT NOT NULL,
  effective_date     DATE NOT NULL,
  created_at         TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- PERSONNEL RELOCATIONS  (11_DIRECTORS.md UC-NOM-01)
-- ============================================================
CREATE TABLE personnel_relocations (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  target_user_id    UUID NOT NULL REFERENCES users(id),
  relocated_by      UUID NOT NULL REFERENCES users(id),
  origin_location   TEXT NOT NULL,
  destination       TEXT NOT NULL,
  relocation_type   TEXT NOT NULL CHECK (relocation_type IN ('temporary','permanent')),
  effective_date    DATE NOT NULL,
  created_at        TIMESTAMPTZ DEFAULT NOW(),
  deleted_at        TIMESTAMPTZ,
  deleted_by        UUID REFERENCES users(id)
);

CREATE INDEX idx_relocations_target ON personnel_relocations(target_user_id) WHERE deleted_at IS NULL;

-- ============================================================
-- BROADCAST REQUESTS  (UC-GUA-02, UC-ANC-00/01)
-- ============================================================
CREATE TABLE broadcast_requests (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  requester_id  UUID NOT NULL REFERENCES users(id),
  type          TEXT NOT NULL CHECK (type IN ('security','informational')),
  subject       TEXT NOT NULL,
  content       TEXT NOT NULL,
  target_scope  TEXT NOT NULL DEFAULT 'company_wide',  -- 'company_wide','group','region','individual'
  target_ids    UUID[],                                -- group/user IDs depending on scope
  urgency       TEXT DEFAULT 'normal' CHECK (urgency IN ('low','normal','high','critical')),
  rationale     TEXT,
  status        TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending','approved','rejected','sent')),
  decided_by    UUID REFERENCES users(id),
  decision_at   TIMESTAMPTZ,
  sent_at       TIMESTAMPTZ,
  created_at    TIMESTAMPTZ DEFAULT NOW(),
  deleted_at    TIMESTAMPTZ,
  deleted_by    UUID REFERENCES users(id)
);

CREATE INDEX idx_broadcast_status ON broadcast_requests(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_broadcast_type   ON broadcast_requests(type)   WHERE deleted_at IS NULL;
