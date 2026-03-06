-- ============================================================
-- 011_astronaut_test_seed.sql
-- Test data for the Astronauts subsystem (UC-AS-01 through UC-AS-06)
-- Password: password123  →  $2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO
-- ============================================================

DO $$
DECLARE
  -- User IDs (looked up dynamically)
  v_astro1    UUID;
  v_astro2    UUID;
  v_astro3    UUID;
  v_wanderer  UUID; -- TheWanderer director
  v_taskmaster UUID; -- TheTaskmaster director

  -- Mission IDs
  v_m1  UUID; -- active interstellar
  v_m2  UUID; -- active terrain
  v_m3  UUID; -- completion_requested interstellar
  v_m4  UUID; -- completed terrain (for journal)
  v_m5  UUID; -- active terrain (multi-astronaut)

  -- Report / Request IDs
  v_sr1  UUID;
  v_sr2  UUID;
  v_sr3  UUID;
  v_cr1  UUID; -- pending_wanderer
  v_cr2  UUID; -- approved (for completed mission)
BEGIN

  -- ════════════════════════════════════════════════════════════════════════════
  -- ASTRONAUT USER ACCOUNTS
  -- ════════════════════════════════════════════════════════════════════════════

  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'nova.trek', 'nova@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Nova Trek', r.id, 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
  FROM roles r WHERE r.name = 'Astronaut'
  ON CONFLICT (username) DO NOTHING;

  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'orion.blaze', 'orion@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Orion Blaze', r.id, 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
  FROM roles r WHERE r.name = 'Astronaut'
  ON CONFLICT (username) DO NOTHING;

  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'stella.comet', 'stella@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Stella Comet', r.id, 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
  FROM roles r WHERE r.name = 'Astronaut'
  ON CONFLICT (username) DO NOTHING;

  -- Look up IDs
  SELECT id INTO v_astro1 FROM users WHERE username = 'nova.trek';
  SELECT id INTO v_astro2 FROM users WHERE username = 'orion.blaze';
  SELECT id INTO v_astro3 FROM users WHERE username = 'stella.comet';
  SELECT id INTO v_wanderer FROM users WHERE username = 'walt.trail';
  SELECT id INTO v_taskmaster FROM users WHERE username = 'task.iron';

  -- Skip seed if lookup failed (e.g. directors not seeded yet)
  IF v_wanderer IS NULL OR v_taskmaster IS NULL THEN
    RAISE NOTICE 'Wanderer or Taskmaster user not found -- skipping astronaut seed';
    RETURN;
  END IF;

  -- ════════════════════════════════════════════════════════════════════════════
  -- MISSIONS
  -- ════════════════════════════════════════════════════════════════════════════

  v_m1 := gen_random_uuid();
  v_m2 := gen_random_uuid();
  v_m3 := gen_random_uuid();
  v_m4 := gen_random_uuid();
  v_m5 := gen_random_uuid();

  INSERT INTO missions (id, title, type, danger_level, location, mission_objective, procedures, known_dangers, assigned_by, status, created_at)
  VALUES
    (v_m1, 'Deep Space Survey Alpha-7', 'interstellar', 'high',
     'Proxima Centauri Sector Alpha-7',
     'Conduct a full spectral survey of the Alpha-7 system. Map all planetary bodies and identify resource-rich asteroids.',
     '1. Enter orbit around primary star\n2. Deploy survey satellites\n3. Collect samples from largest asteroid\n4. Return to base station',
     'Radiation storms detected periodically. Magnetic interference may disrupt comms.',
     v_wanderer, 'active', NOW() - INTERVAL '30 days'),

    (v_m2, 'Terrain Mapping - Kepler Basin', 'terrain', 'medium',
     'Kepler-442b Surface Basin',
     'Map the geological features of Kepler Basin. Identify water sources and potential settlement areas.',
     '1. Land at designated coordinates\n2. Deploy ground-penetrating radar\n3. Collect soil and rock samples\n4. Document flora/fauna',
     'Unstable ground in eastern quadrant. Possible seismic activity.',
     v_wanderer, 'active', NOW() - INTERVAL '20 days'),

    (v_m3, 'Nebula Sampling - Orion Arm', 'interstellar', 'critical',
     'Orion Arm Nebula NGC-2024',
     'Collect gas and dust samples from the Orion Arm nebula for spectral analysis.',
     '1. Approach nebula at safe distance\n2. Deploy sample collectors\n3. Run in-situ spectrometer\n4. Package samples for return',
     'CRITICAL: Extreme radiation levels. EVA limited to 2-hour windows.',
     v_wanderer, 'completion_requested', NOW() - INTERVAL '60 days'),

    (v_m4, 'Relay Station Maintenance - Sector 9', 'terrain', 'low',
     'Relay Station RS-009',
     'Perform routine maintenance on communications relay station in Sector 9.',
     '1. Inspect antenna array\n2. Replace power cells\n3. Update firmware\n4. Run diagnostics',
     'Low-gravity environment. Use tethers.',
     v_wanderer, 'completed', NOW() - INTERVAL '90 days'),

    (v_m5, 'Joint Terrain Survey - Trappist Delta', 'terrain', 'high',
     'TRAPPIST-1e Delta Region',
     'Multi-crew terrain survey of the Delta region. Focus on mineralogy and atmospheric composition.',
     '1. Establish base camp\n2. Split into survey teams\n3. Collect 50+ samples\n4. Rendezvous and debrief',
     'Toxic atmosphere -- full EVA suits required at all times.',
     v_wanderer, 'active', NOW() - INTERVAL '15 days');

  -- ════════════════════════════════════════════════════════════════════════════
  -- MISSION ASSIGNMENTS
  -- ════════════════════════════════════════════════════════════════════════════

  INSERT INTO mission_assignments (mission_id, astronaut_id) VALUES
    (v_m1, v_astro1),
    (v_m1, v_astro2),
    (v_m2, v_astro1),
    (v_m3, v_astro2),
    (v_m3, v_astro3),
    (v_m4, v_astro1),
    (v_m4, v_astro3),
    (v_m5, v_astro1),
    (v_m5, v_astro2),
    (v_m5, v_astro3);

  -- ════════════════════════════════════════════════════════════════════════════
  -- MISSION COUNTERS
  -- ════════════════════════════════════════════════════════════════════════════

  INSERT INTO mission_counters (astronaut_id, interstellar_count, terrain_count) VALUES
    (v_astro1, 0, 1),  -- completed RS-009 (terrain)
    (v_astro3, 0, 1)   -- completed RS-009 (terrain)
  ON CONFLICT (astronaut_id) DO UPDATE
    SET interstellar_count = EXCLUDED.interstellar_count,
        terrain_count = EXCLUDED.terrain_count;

  -- ════════════════════════════════════════════════════════════════════════════
  -- MISSION STATUS REPORTS
  -- ════════════════════════════════════════════════════════════════════════════

  v_sr1 := gen_random_uuid();
  v_sr2 := gen_random_uuid();
  v_sr3 := gen_random_uuid();

  INSERT INTO mission_status_reports (id, mission_id, submitted_by, report_date, month_tracker, rag_status, current_status, issues_blockers, collected_samples_last_month, progress_last_month, plans_next_month) VALUES
    (v_sr1, v_m1, v_astro1, NOW() - INTERVAL '15 days', 'Month 1 of 6', 'green',
     'Successfully entered Alpha-7 orbit. Survey satellites deployed. Initial scans complete.',
     NULL, '12 asteroid fragments catalogued', 'Orbital insertion, satellite deployment, initial spectral scans', 'Begin targeted sampling of largest asteroid'),

    (v_sr2, v_m1, v_astro2, NOW() - INTERVAL '7 days', 'Month 2 of 6', 'amber',
     'Radiation storm disrupted comms for 48 hours. All crew safe. Resuming operations.',
     'Comms disruption from unexpected radiation burst. Equipment recalibration needed.',
     '3 deep-core samples from Asteroid A7-14', 'Recovered from storm. Deep-core drilling on A7-14', 'Complete A7-14 sampling, move to A7-15'),

    (v_sr3, v_m3, v_astro2, NOW() - INTERVAL '5 days', 'Month 3 of 3', 'red',
     'Final sample collection complete. EVA time exhausted. Requesting mission completion.',
     'Radiation levels exceeded projections by 40%. Crew approaching maximum safe exposure.',
     '200+ nebula gas/dust samples collected', 'All planned samples collected ahead of schedule', 'Return to base. Submit completion request.');

  -- ════════════════════════════════════════════════════════════════════════════
  -- COMPLETION REQUESTS
  -- ════════════════════════════════════════════════════════════════════════════

  -- M3 has a pending_wanderer completion request
  v_cr1 := gen_random_uuid();
  INSERT INTO mission_completion_requests (id, mission_id, submitted_by, findings_summary, evidence_storage_paths, status) VALUES
    (v_cr1, v_m3, v_astro2,
     'Nebula sampling mission complete. Collected 200+ gas and dust samples across 12 collection points within NGC-2024. Spectral analysis confirms presence of rare isotopes (He-3, Deuterium). Radiation exposure managed within safety margins despite 40% higher-than-projected levels. All crew members in good health. Samples packaged and ready for laboratory analysis.',
     ARRAY['missions/completion-evidence/sample/report_final.pdf', 'missions/completion-evidence/sample/spectra_data.csv'],
     'pending_wanderer');

  -- M4 was already approved (completed mission)
  v_cr2 := gen_random_uuid();
  INSERT INTO mission_completion_requests (id, mission_id, submitted_by, findings_summary, evidence_storage_paths, status, wanderer_note, taskmaster_note) VALUES
    (v_cr2, v_m4, v_astro1,
     'Relay station RS-009 maintenance complete. Replaced 4 power cells, updated firmware to v4.2.1, replaced damaged antenna element. All diagnostics pass. Signal strength improved by 18%.',
     NULL,
     'approved',
     'Good work. Forwarding for final approval.',
     'Approved. Counters updated.');

  -- ════════════════════════════════════════════════════════════════════════════
  -- NOTIFICATIONS -- demonstrate what astronauts/directors see
  -- ════════════════════════════════════════════════════════════════════════════

  INSERT INTO notifications (user_id, type, payload) VALUES
    -- Nova assigned to M5
    (v_astro1, 'mission:assigned', jsonb_build_object(
      'mission_id', v_m5::TEXT, 'title', 'Joint Terrain Survey - Trappist Delta',
      'type', 'terrain', 'danger_level', 'high', 'location', 'TRAPPIST-1e Delta Region'
    )),
    -- Orion: status report flagged (amber)
    (v_wanderer, 'mission:status_report', jsonb_build_object(
      'mission_id', v_m1::TEXT, 'submitter', 'Orion Blaze', 'rag_status', 'amber'
    )),
    -- Wanderer: completion request submitted
    (v_wanderer, 'mission:completion_submitted', jsonb_build_object(
      'mission_id', v_m3::TEXT, 'mission_title', 'Nebula Sampling - Orion Arm',
      'submitter', 'Orion Blaze'
    ));

  RAISE NOTICE 'Astronaut seed data inserted successfully';
END $$;
