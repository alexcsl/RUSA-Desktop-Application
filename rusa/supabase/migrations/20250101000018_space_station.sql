-- Migration: Space Station Settlers subsystem
-- Source of truth: 08_SPACE_STATION_SETTLERS.md
--
-- Tables:
--   space_stations           — station registry
--   station_inventory        — supply tracking per station
--   station_personnel_log    — arrival/departure log
--   station_maps             — map metadata (image in Supabase Storage)
--   station_map_annotations  — click-to-view annotation points
--   station_private_archive  — JSONB document archive (internal findings)
--   station_supply_requests  — supply requests routed to Directors voting

-- ── space_stations ────────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS space_stations (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  sector TEXT NOT NULL,
  nearest_galactic_security_team_id UUID REFERENCES users(id),
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','abandoned')),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ,
  deleted_by UUID REFERENCES users(id)
);

-- ── station_inventory ─────────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS station_inventory (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  station_id UUID NOT NULL REFERENCES space_stations(id),
  item_name TEXT NOT NULL,
  category TEXT,
  quantity INTEGER NOT NULL DEFAULT 0,
  unit TEXT,
  last_updated_by UUID REFERENCES users(id),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ,
  deleted_by UUID REFERENCES users(id)
);

-- ── station_personnel_log ─────────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS station_personnel_log (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  station_id UUID NOT NULL REFERENCES space_stations(id),
  person_name TEXT NOT NULL,
  role TEXT,
  arrived_at TIMESTAMPTZ NOT NULL,
  departed_at TIMESTAMPTZ,
  logged_by UUID REFERENCES users(id),
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- ── station_maps ──────────────────────────────────────────────────────────────
-- Image files live in Supabase Storage bucket: rusa-maps
-- Path pattern: station-maps/{station_id}/{map_id}.png

CREATE TABLE IF NOT EXISTS station_maps (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  station_id UUID NOT NULL REFERENCES space_stations(id),
  created_by UUID REFERENCES users(id),
  image_storage_path TEXT NOT NULL,
  image_width FLOAT,
  image_height FLOAT,
  is_published BOOLEAN NOT NULL DEFAULT FALSE,
  published_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ,
  deleted_by UUID REFERENCES users(id)
);

-- ── station_map_annotations ───────────────────────────────────────────────────

CREATE TABLE IF NOT EXISTS station_map_annotations (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  map_id UUID NOT NULL REFERENCES station_maps(id),
  label TEXT NOT NULL,
  description TEXT,
  x_position FLOAT NOT NULL,
  y_position FLOAT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ,
  deleted_by UUID REFERENCES users(id)
);

-- ── station_private_archive ───────────────────────────────────────────────────
-- Each finding is a JSONB document. Replaces MongoDB station_private_archive.
-- JSONB shape:
-- {
--   "description": "...",
--   "severity": "low|medium|high|critical",
--   "is_reported_to_security": false,
--   "location_on_station": "...",
--   "response_procedure": "...",
--   "category": ["...", "..."]
-- }

CREATE TABLE IF NOT EXISTS station_private_archive (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  station_id UUID NOT NULL REFERENCES space_stations(id),
  logged_by UUID REFERENCES users(id),
  finding_type TEXT,
  archive_data JSONB NOT NULL DEFAULT '{}',
  created_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ,
  deleted_by UUID REFERENCES users(id)
);

-- ── station_supply_requests ───────────────────────────────────────────────────
-- Items are stored as JSONB: [{ "item": "...", "quantity": N, "justification": "..." }]

CREATE TABLE IF NOT EXISTS station_supply_requests (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  station_id UUID NOT NULL REFERENCES space_stations(id),
  submitted_by UUID REFERENCES users(id),
  items JSONB NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending_approval'
    CHECK (status IN ('pending_approval','approved','rejected')),
  vote_session_id UUID REFERENCES vote_sessions(id),
  created_at TIMESTAMPTZ DEFAULT NOW(),
  deleted_at TIMESTAMPTZ,
  deleted_by UUID REFERENCES users(id)
);

-- ── Indexes ───────────────────────────────────────────────────────────────────

CREATE INDEX IF NOT EXISTS idx_station_inventory_station ON station_inventory(station_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_station_personnel_station ON station_personnel_log(station_id);
CREATE INDEX IF NOT EXISTS idx_station_maps_station ON station_maps(station_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_station_annotations_map ON station_map_annotations(map_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_station_archive_station ON station_private_archive(station_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_station_supply_station ON station_supply_requests(station_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_station_maps_published ON station_maps(station_id, is_published) WHERE deleted_at IS NULL AND is_published = TRUE;
