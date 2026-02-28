-- ============================================================
-- 002_seed_data.sql
-- Test data for development. Not for production.
-- ============================================================

-- Insert base locations for testing
INSERT INTO base_locations (id, name, planet, sector, has_data_regulation) VALUES
  ('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'RUSA HQ',          'Earth',      'Sector 0', TRUE),
  ('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Kepler Outpost',   'Kepler-22b', 'Sector 7', FALSE),
  ('cccccccc-cccc-cccc-cccc-cccccccccccc', 'Alpha Station',     'Mars',       'Sector 1', TRUE)
ON CONFLICT DO NOTHING;

-- Insert test users
-- All passwords below are bcrypt hash of "password123" (cost 12)
-- $2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'admin',          'admin@rusa.internal',      '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO', 'System Administrator', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'Administrator'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'data.data',          'data@rusa.internal',      '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO', 'Bryan Data', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'DataAnalyst'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'alice.vanguard', 'director@rusa.internal',   '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO', 'Alice Vanguard',       r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'TheDirector'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'bob.metrics',    'stat@rusa.internal',       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO', 'Bob Metrics',          r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'TheStatistician'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'charlie.pulse',  'med@rusa.internal',        '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO', 'Charlie Pulse',        r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'MedicalStaff'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'delta.astro',    'astro@rusa.internal',      '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO', 'Delta Starborn',       r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'Astronaut'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'echo.settler',   'settler@rusa.internal',    '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO', 'Echo Nomad',           r.id, 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
FROM roles r WHERE r.name = 'SettlerCommander'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'foxtrot.psych',  'psych@rusa.internal',      '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO', 'Foxtrot Mindwell',     r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'Psychiatrist'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'golf.security',  'security@rusa.internal',   '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO', 'Golf Shield',          r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'GalacticSecurityHead'
ON CONFLICT (username) DO NOTHING;
