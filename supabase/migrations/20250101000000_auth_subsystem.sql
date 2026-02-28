-- ============================================================
-- 001_auth_subsystem.sql
-- Auth foundation migration — run FIRST, before any subsystem.
-- Source of truth: AUTH_GUIDE.md
-- ============================================================

-- ============================================================
-- ROLES
-- ============================================================
CREATE TABLE roles (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name        TEXT NOT NULL UNIQUE,   -- matches Role enum constants exactly
  description TEXT,
  created_at  TIMESTAMPTZ DEFAULT NOW()
);

-- Seed all roles immediately after creation
INSERT INTO roles (name, description) VALUES
  ('AgriculturalEngineer',  'Plant-focused engineer'),
  ('BiologicalEngineer',    'All-species engineer'),
  ('DataAnalyst',           'Data processing and analytics. No medical table access.'),
  ('GalacticSecurityHead',  'Galactic Security Head (Earth excluded)'),
  ('GalacticSecurityStaff', 'Galactic Security Staff (Earth excluded)'),
  ('Mathematician',         'Calculation and results'),
  ('Physicist',             'Physical property experiments'),
  ('Chemist',               'Matter experiments'),
  ('Biologist',             'Species observation and documentation'),
  ('Astronaut',             'Mission workflows'),
  ('SettlerCommander',      'Settlement authority'),
  ('CivilEngineer',         'Construction-focused settler'),
  ('Farmer',                'Farming-focused settler'),
  ('TemporarySetter',       'Visiting deployed personnel'),
  ('SpaceStationSettler',   'Station operations and maintenance'),
  ('Psychiatrist',          'Patient access scoped to own patients only'),
  ('PsychiatristAssistant', 'Limited patient view: schedule + recovery log overview only'),
  ('MedicalStaff',          'Treatment and inventory. No admin powers.'),
  ('HeadOfMedicine',        'Administrative and practicing MD'),
  ('HeadOfSanitary',        'Full sanitary department authority'),
  ('InspectorCrew',         'QA/hygiene/safety inspection'),
  ('DisposalCrew',          'Hazardous material documentation'),
  ('WastewaterCrew',        'Wastewater treatment documentation'),
  ('CleanupCrew',           'Task execution only'),
  ('TransportCrew',         'Task execution only'),
  ('GeneralDirector',       'Abstract base: shared voting and meeting capabilities'),
  ('TheDirector',           'Hire and update non-director personnel'),
  ('TheAccountant',         'Financial monitoring and foul-play flagging'),
  ('TheLibrarian',          'Archive governance and redaction'),
  ('TheNomad',              'Personnel relocation (below Director level)'),
  ('TheArtificer',          'Math/Physics task assignment and proxy'),
  ('TheObserver',           'Bio/Chem/AgEng/BioEng task assignment and proxy'),
  ('TheWanderer',           'Astronaut mission management'),
  ('TheTaskmaster',         'Combined Artificer+Observer+Wanderer authority'),
  ('TheGuardian',           'Security oversight and broadcast approval'),
  ('TheStatistician',       'Data request gatekeeper'),
  ('TheCoordinator',        'Events and meeting organization'),
  ('TheOverseer',           'Combined Nomad+Guardian scope; no broadcast authority'),
  ('TheAnchorman',          'Company-wide broadcasts and personnel termination'),
  ('Administrator',         'Highest official rank. Single user. Final authority on everything.');

-- ============================================================
-- PERMISSIONS
-- ============================================================
CREATE TABLE permissions (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  role_id     UUID NOT NULL REFERENCES roles(id),
  permission  TEXT NOT NULL,  -- e.g. 'experiments:create', 'votes:cast'
  created_at  TIMESTAMPTZ DEFAULT NOW(),
  UNIQUE (role_id, permission)
);

-- ============================================================
-- BASE LOCATIONS
-- Needed early: auth checks base-level flags (e.g., GDPR)
-- ============================================================
CREATE TABLE base_locations (
  id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name                TEXT NOT NULL,
  planet              TEXT,
  sector              TEXT,
  has_data_regulation BOOLEAN NOT NULL DEFAULT TRUE,
  -- TRUE  = GDPR-compliant location (default; stricter privacy rules apply)
  -- FALSE = location without personal data regulation (some fields unlock)
  created_at          TIMESTAMPTZ DEFAULT NOW(),
  deleted_at          TIMESTAMPTZ,
  deleted_by          UUID  -- forward reference resolved after users table exists
);

-- ============================================================
-- USERS
-- ============================================================
CREATE TABLE users (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  full_name        TEXT NOT NULL,
  username         TEXT NOT NULL UNIQUE,
  email            TEXT UNIQUE,
  password_hash    TEXT NOT NULL,          -- bcrypt, cost factor >= 12
  role_id          UUID NOT NULL REFERENCES roles(id),
  base_location_id UUID REFERENCES base_locations(id),
  is_active        BOOLEAN NOT NULL DEFAULT TRUE,
  -- Soft delete fields (Anchorman / Administrator termination)
  deleted_at       TIMESTAMPTZ,
  deleted_by       UUID REFERENCES users(id),
  created_at       TIMESTAMPTZ DEFAULT NOW(),
  updated_at       TIMESTAMPTZ DEFAULT NOW()
);

-- Resolve forward reference on base_locations.deleted_by
ALTER TABLE base_locations
  ADD CONSTRAINT base_locations_deleted_by_fk
  FOREIGN KEY (deleted_by) REFERENCES users(id);

-- ============================================================
-- AUDIT LOG (must exist before any other write happens)
-- ============================================================
CREATE TABLE audit_log (
  id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  table_name   TEXT NOT NULL,
  record_id    UUID NOT NULL,
  operation    TEXT NOT NULL CHECK (operation IN ('CREATE','UPDATE','DELETE')),
  performed_by UUID REFERENCES users(id),
  performed_at TIMESTAMPTZ DEFAULT NOW(),
  before_data  JSONB,
  after_data   JSONB
);

-- ============================================================
-- INDEXES
-- ============================================================
CREATE INDEX idx_users_username   ON users(username)  WHERE deleted_at IS NULL;
CREATE INDEX idx_users_role_id    ON users(role_id)   WHERE deleted_at IS NULL;
CREATE INDEX idx_users_active     ON users(is_active) WHERE deleted_at IS NULL;
CREATE INDEX idx_audit_log_record ON audit_log(table_name, record_id);
CREATE INDEX idx_audit_log_by     ON audit_log(performed_by);