-- Seed: Space Station Settlers test data
-- Creates SpaceStationSettler users, a station, inventory, personnel, archive entries
-- Requires: 20250101000018_space_station.sql

DO $$
DECLARE
  v_settler_role_id UUID;
  v_sec_head_role_id UUID;
  v_sec_head_id UUID;
  v_station_id UUID;
  v_settler1_id UUID;
  v_settler2_id UUID;
  v_base_id UUID;
BEGIN
  SELECT id INTO v_settler_role_id FROM roles WHERE name = 'SpaceStationSettler';
  SELECT id INTO v_sec_head_role_id FROM roles WHERE name = 'GalacticSecurityHead';

  -- Find the nearest security head user for the station
  SELECT u.id INTO v_sec_head_id
  FROM users u JOIN roles r ON r.id = u.role_id
  WHERE r.name = 'GalacticSecurityHead' AND u.deleted_at IS NULL
  LIMIT 1;

  -- Use Alpha Station base location
  SELECT id INTO v_base_id FROM base_locations WHERE name = 'Alpha Station' LIMIT 1;
  IF v_base_id IS NULL THEN
    SELECT id INTO v_base_id FROM base_locations LIMIT 1;
  END IF;

  -- Only seed if SpaceStationSettler role exists and no settlers yet
  IF v_settler_role_id IS NOT NULL AND NOT EXISTS (
    SELECT 1 FROM users u JOIN roles r ON r.id = u.role_id
    WHERE r.name = 'SpaceStationSettler' AND u.deleted_at IS NULL
  ) THEN

    -- ── Create SpaceStationSettler users ────────────────────────────────────
    -- password: 'Test1234!' hashed with bcrypt cost 12
    INSERT INTO users (id, full_name, username, password_hash, role_id, base_location_id, is_active)
    VALUES (gen_random_uuid(), 'Nova Stardust', 'nova.stardust',
      '$2a$12$LJ3m4ys3LzH5v7W6xKQp4ejKf2FcYQvMkG8jqF7Z1x9h5n3v4OxoW',
      v_settler_role_id, v_base_id, true);

    INSERT INTO users (id, full_name, username, password_hash, role_id, base_location_id, is_active)
    VALUES (gen_random_uuid(), 'Rex Beacon', 'rex.beacon',
      '$2a$12$LJ3m4ys3LzH5v7W6xKQp4ejKf2FcYQvMkG8jqF7Z1x9h5n3v4OxoW',
      v_settler_role_id, v_base_id, true);

    SELECT id INTO v_settler1_id FROM users WHERE username = 'nova.stardust';
    SELECT id INTO v_settler2_id FROM users WHERE username = 'rex.beacon';

    -- ── Create a space station ──────────────────────────────────────────────
    INSERT INTO space_stations (id, name, sector, nearest_galactic_security_team_id, status)
    VALUES (gen_random_uuid(), 'Orion Waypoint', 'Sector 1', v_sec_head_id, 'active');

    SELECT id INTO v_station_id FROM space_stations WHERE name = 'Orion Waypoint';

    -- ── Station inventory ───────────────────────────────────────────────────
    INSERT INTO station_inventory (station_id, item_name, category, quantity, unit, last_updated_by)
    VALUES
      (v_station_id, 'Emergency Ration Packs', 'Food', 200, 'packs', v_settler1_id),
      (v_station_id, 'Water Purification Tablets', 'Supplies', 500, 'tablets', v_settler1_id),
      (v_station_id, 'Medical Kits', 'Medical', 30, 'kits', v_settler2_id),
      (v_station_id, 'Bedsheet Sets', 'Bedding', 50, 'sets', v_settler2_id),
      (v_station_id, 'Oxygen Canisters', 'Life Support', 120, 'canisters', v_settler1_id);

    -- ── Personnel log ───────────────────────────────────────────────────────
    INSERT INTO station_personnel_log (station_id, person_name, role, arrived_at, logged_by)
    VALUES
      (v_station_id, 'Nova Stardust', 'SpaceStationSettler', NOW() - INTERVAL '30 days', v_settler1_id),
      (v_station_id, 'Rex Beacon', 'SpaceStationSettler', NOW() - INTERVAL '25 days', v_settler1_id),
      (v_station_id, 'Dr. Transit', 'Visiting Researcher', NOW() - INTERVAL '3 days', v_settler2_id);

    -- Add a departed entry
    INSERT INTO station_personnel_log (station_id, person_name, role, arrived_at, departed_at, logged_by)
    VALUES
      (v_station_id, 'Cargo Pilot Jensen', 'Delivery', NOW() - INTERVAL '7 days', NOW() - INTERVAL '5 days', v_settler1_id);

    -- ── Private archive findings ────────────────────────────────────────────
    INSERT INTO station_private_archive (station_id, logged_by, finding_type, archive_data)
    VALUES
      (v_station_id, v_settler1_id, 'Anomaly',
       '{"description": "Unusual electromagnetic readings detected in corridor B7. Readings intermittent, no equipment damage found.", "severity": "medium", "is_reported_to_security": false, "location_on_station": "Corridor B7 — Deck 3", "response_procedure": "Continue monitoring with portable scanners", "category": ["electromagnetic", "anomaly"]}'::jsonb),
      (v_station_id, v_settler2_id, 'Maintenance',
       '{"description": "Hydroponics bay humidity control unit showing wear. Scheduled replacement parts on next resupply.", "severity": "low", "is_reported_to_security": false, "location_on_station": "Hydroponics Bay — Deck 5", "response_procedure": "Temporary manual humidity adjustment", "category": ["maintenance", "life_support"]}'::jsonb),
      (v_station_id, v_settler1_id, 'Security Concern',
       '{"description": "Unauthorized access attempt detected on storage room D4. Keypad showed 3 failed attempts. No forced entry.", "severity": "high", "is_reported_to_security": true, "location_on_station": "Storage Room D4 — Deck 2", "response_procedure": "Change access codes, review security footage", "category": ["unauthorized_access", "security"]}'::jsonb);

    -- ── Supply request ──────────────────────────────────────────────────────
    INSERT INTO station_supply_requests (station_id, submitted_by, items, status)
    VALUES
      (v_station_id, v_settler1_id,
       '[{"item": "Emergency Ration Packs", "quantity": 100, "justification": "Current stock depleting faster than expected due to visiting crew"}, {"item": "Water Purification Tablets", "quantity": 200, "justification": "Approaching minimum safe reserves"}]'::jsonb,
       'pending_approval');
  END IF;
END $$;
