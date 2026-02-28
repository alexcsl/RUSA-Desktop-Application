-- ============================================================
-- 013_settler_test_seed.sql
-- Test data for the Exoplanet Settlers subsystem
-- Password: password123  →  $2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO
-- ============================================================

DO $$
DECLARE
  -- User IDs (looked up dynamically)
  v_commander   UUID;
  v_engineer    UUID;
  v_farmer      UUID;
  v_temp        UUID;
  v_nomad       UUID;

  -- Settlement
  v_settlement  UUID;

  -- Generated IDs
  v_task1       UUID;
  v_task2       UUID;
  v_task3       UUID;
  v_supply1     UUID;
  v_anomaly1    UUID;
  v_complaint1  UUID;
BEGIN

  -- ════════════════════════════════════════════════════════════════════════════
  -- SETTLER USER ACCOUNTS
  -- ════════════════════════════════════════════════════════════════════════════

  -- echo.settler already exists as SettlerCommander from seed 001.
  -- Add additional settler users.

  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'rina.build', 'rina@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Rina Build', r.id, 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
  FROM roles r WHERE r.name = 'CivilEngineer'
  ON CONFLICT (username) DO NOTHING;

  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'hank.harvest', 'hank@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Hank Harvest', r.id, 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
  FROM roles r WHERE r.name = 'Farmer'
  ON CONFLICT (username) DO NOTHING;

  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'zara.temp', 'zara@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Zara Temp', r.id, 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
  FROM roles r WHERE r.name = 'TemporarySetter'
  ON CONFLICT (username) DO NOTHING;

  -- Look up IDs
  SELECT id INTO v_commander FROM users WHERE username = 'echo.settler';
  SELECT id INTO v_engineer  FROM users WHERE username = 'rina.build';
  SELECT id INTO v_farmer    FROM users WHERE username = 'hank.harvest';
  SELECT id INTO v_temp      FROM users WHERE username = 'zara.temp';
  SELECT id INTO v_nomad     FROM users WHERE username = 'nate.drift';

  IF v_commander IS NULL THEN
    RAISE NOTICE 'Settler commander (echo.settler) not found -- skipping settler seed';
    RETURN;
  END IF;

  -- ════════════════════════════════════════════════════════════════════════════
  -- SETTLEMENT
  -- ════════════════════════════════════════════════════════════════════════════

  v_settlement := gen_random_uuid();

  INSERT INTO settlements (id, name, planet, commander_id, status)
  VALUES (v_settlement, 'New Kepler Colony', 'Kepler-22b', v_commander, 'active');

  -- ════════════════════════════════════════════════════════════════════════════
  -- SETTLER ASSIGNMENTS
  -- ════════════════════════════════════════════════════════════════════════════

  INSERT INTO settler_assignments (user_id, settlement_id, type, residence_unit, arrived_at)
  VALUES
    (v_commander, v_settlement, 'permanent', 'HQ-01', NOW() - INTERVAL '180 days'),
    (v_engineer,  v_settlement, 'permanent', 'B-12',  NOW() - INTERVAL '150 days'),
    (v_farmer,    v_settlement, 'permanent', 'F-03',  NOW() - INTERVAL '120 days'),
    (v_temp,      v_settlement, 'temporary', 'T-01',  NOW() - INTERVAL '10 days');

  -- ════════════════════════════════════════════════════════════════════════════
  -- SETTLEMENT INVENTORY
  -- ════════════════════════════════════════════════════════════════════════════

  INSERT INTO settlement_inventory (settlement_id, item_name, category, quantity, unit, last_updated_by)
  VALUES
    (v_settlement, 'Cement Bags',         'construction', 250, 'bags',  v_commander),
    (v_settlement, 'Steel Beams',         'construction', 80,  'units', v_commander),
    (v_settlement, 'Solar Panels',        'energy',       40,  'units', v_commander),
    (v_settlement, 'Water Purifiers',     'utilities',    12,  'units', v_commander),
    (v_settlement, 'Wheat Seeds',         'farming',      500, 'kg',    v_farmer),
    (v_settlement, 'Cattle Feed',         'farming',      300, 'kg',    v_farmer),
    (v_settlement, 'Medical Supplies Kit','medical',      25,  'kits',  v_commander),
    (v_settlement, 'Oxygen Canisters',    'life_support', 100, 'units', v_commander);

  -- ════════════════════════════════════════════════════════════════════════════
  -- SETTLER TASKS (Commander → Settlers)
  -- ════════════════════════════════════════════════════════════════════════════

  v_task1 := gen_random_uuid();
  v_task2 := gen_random_uuid();
  v_task3 := gen_random_uuid();

  INSERT INTO settler_tasks (id, settlement_id, assigned_by, assigned_to, title, description, scope, urgency, deadline, status)
  VALUES
    (v_task1, v_settlement, v_commander, v_engineer,
     'Construct Water Recycling Unit',
     'Build the water recycling facility in sector B. Refer to blueprint WR-2024.',
     'construction', 'high', (CURRENT_DATE + INTERVAL '30 days')::DATE, 'in_progress'),

    (v_task2, v_settlement, v_commander, v_farmer,
     'Prepare South Field for Planting Season',
     'Clear and prepare the 5-hectare south field. Soil analysis results are attached.',
     'farming', 'medium', (CURRENT_DATE + INTERVAL '14 days')::DATE, 'assigned'),

    (v_task3, v_settlement, v_commander, v_engineer,
     'Inspect Dome Structural Integrity',
     'Perform a full structural integrity assessment of the main habitat dome.',
     'inspection', 'critical', (CURRENT_DATE + INTERVAL '7 days')::DATE, 'assigned');

  -- ════════════════════════════════════════════════════════════════════════════
  -- PROGRESS REPORTS
  -- ════════════════════════════════════════════════════════════════════════════

  INSERT INTO settler_progress_reports (task_id, settlement_id, submitted_by, week, rag_status, progress_made, materials_equipment)
  VALUES
    (v_task1, v_settlement, v_engineer, 'Week 2', 'green',
     'Foundation laid. Plumbing connections started. On schedule.',
     'Used 40 cement bags, 12 steel beams, 6 pipe segments');

  -- ════════════════════════════════════════════════════════════════════════════
  -- ANOMALY REPORTS
  -- ════════════════════════════════════════════════════════════════════════════

  v_anomaly1 := gen_random_uuid();

  INSERT INTO anomaly_reports (id, settlement_id, reported_by, description, location_in_settlement, danger_level, severity, category, status)
  VALUES
    (v_anomaly1, v_settlement, v_engineer,
     'Micro-fractures detected in dome section C-7. Possible pressure leak risk.',
     'Dome Section C-7', 'high', 'high',
     '["structural"]'::jsonb, 'submitted');

  -- ════════════════════════════════════════════════════════════════════════════
  -- SETTLER COMPLAINTS
  -- ════════════════════════════════════════════════════════════════════════════

  v_complaint1 := gen_random_uuid();

  INSERT INTO settler_complaints (id, settlement_id, reported_by, subject_user_id, incident_description, status)
  VALUES
    (v_complaint1, v_settlement, v_farmer, v_temp,
     'Temporary settler Zara repeatedly ignored safety protocols during chemical handling in the lab.',
     'submitted');

  -- ════════════════════════════════════════════════════════════════════════════
  -- SUPPLY REQUESTS
  -- ════════════════════════════════════════════════════════════════════════════

  v_supply1 := gen_random_uuid();

  INSERT INTO supply_requests (id, settlement_id, submitted_by, source_type, items, justification, status)
  VALUES
    (v_supply1, v_settlement, v_farmer, 'farmer',
     '[{"item":"Wheat Seeds","spec":"Drought-resistant variety","quantity":200,"reason":"Current stock low for planting season"}]'::jsonb,
     'Planting season begins in 2 weeks. Current seed stock insufficient for planned 3-hectare expansion.',
     'pending_commander');

  -- Commander supply request (goes directly to directors)
  INSERT INTO supply_requests (settlement_id, submitted_by, source_type, items, justification, status)
  VALUES
    (v_settlement, v_commander, 'commander',
     '[{"item":"Emergency Habitat Modules","spec":"4-person pressurized","quantity":2,"reason":"Population growth exceeding capacity"},{"item":"Radiation Shielding Panels","spec":"Grade-A titanium","quantity":50,"reason":"Dome section C-7 repair"}]'::jsonb,
     'Critical infrastructure needs following population growth and dome integrity issue.',
     'pending_vote');

  -- ════════════════════════════════════════════════════════════════════════════
  -- BUILDING HEALTH LOGS
  -- ════════════════════════════════════════════════════════════════════════════

  INSERT INTO building_health_logs (settlement_id, building_name, checked_by, check_date, findings, status)
  VALUES
    (v_settlement, 'Main Habitat Dome', v_engineer, CURRENT_DATE - 7,
     'Overall structure sound. Minor wear on airlock seals. Recommended replacement within 30 days.', 'pass'),
    (v_settlement, 'Water Recycling Plant', v_engineer, CURRENT_DATE - 3,
     'Pipe junction in section 2 showing corrosion. Immediate repair required.', 'needs_repair');

  -- ════════════════════════════════════════════════════════════════════════════
  -- FARM HEALTH LOGS
  -- ════════════════════════════════════════════════════════════════════════════

  INSERT INTO farm_health_logs (settlement_id, logged_by, log_date, subject_type, subject_name, condition, treatment, notes)
  VALUES
    (v_settlement, v_farmer, CURRENT_DATE - 5, 'plant', 'Wheat Field A',
     'Healthy growth, on track for harvest', NULL, 'Expected yield: 2 tonnes per hectare'),
    (v_settlement, v_farmer, CURRENT_DATE - 2, 'livestock', 'Cattle Herd Batch 1',
     'Two animals showing reduced appetite', 'Vitamin supplement administered',
     'Monitoring for 48 hours. Isolate if symptoms persist.');

  -- ════════════════════════════════════════════════════════════════════════════
  -- CONSTRUCTION PROGRESS REPORTS
  -- ════════════════════════════════════════════════════════════════════════════

  INSERT INTO construction_progress_reports (task_id, settlement_id, submitted_by, week, rag_status, materials_used, construction_progress, issues)
  VALUES
    (v_task1, v_settlement, v_engineer, 'Week 1', 'green',
     '[{"material":"Cement","quantity":20,"unit":"bags"},{"material":"Steel Beams","quantity":6,"unit":"units"}]'::jsonb,
     'Foundation excavation complete. Concrete poured for base slab.', NULL);

  RAISE NOTICE 'Settler test seed complete. Settlement: % | Commander: %', v_settlement, v_commander;
END;
$$;
