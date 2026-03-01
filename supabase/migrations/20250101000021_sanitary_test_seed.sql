-- ============================================================
-- 021_sanitary_test_seed.sql
-- Test data for Sanitary Department development.
-- Password: password123
-- ============================================================

DO $$
DECLARE
  v_head_id      UUID;
  v_inspector1   UUID;
  v_inspector2   UUID;
  v_disposal1    UUID;
  v_wastewater1  UUID;
  v_cleanup1     UUID;
  v_transport1   UUID;
  v_base_hq      UUID := 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';
  v_base_alpha   UUID := 'cccccccc-cccc-cccc-cccc-cccccccccccc';
  v_div_insp     UUID;
  v_div_disp     UUID;
  v_div_ww       UUID;
  v_div_clean    UUID;
  v_div_trans    UUID;
  v_task1        UUID;
  v_task2        UUID;
  v_inv1         UUID;
  v_inv2         UUID;
  v_inv3         UUID;
BEGIN

  -- ═══════════════════════════════════════════════════════════
  -- 1. HeadOfSanitary account
  -- ═══════════════════════════════════════════════════════════
  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'frank.sanitary', 'frank@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Frank Sanitary', r.id, v_base_hq
  FROM roles r WHERE r.name = 'HeadOfSanitary'
  ON CONFLICT (username) DO NOTHING;

  SELECT id INTO v_head_id FROM users WHERE username = 'frank.sanitary';

  -- ═══════════════════════════════════════════════════════════
  -- 2. Crew members (one per division + extras)
  -- ═══════════════════════════════════════════════════════════
  -- Inspector Crew
  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'iris.inspect', 'iris@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Iris Inspector', r.id, v_base_hq
  FROM roles r WHERE r.name = 'InspectorCrew'
  ON CONFLICT (username) DO NOTHING;
  SELECT id INTO v_inspector1 FROM users WHERE username = 'iris.inspect';

  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'ivan.inspect', 'ivan@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Ivan Inspector', r.id, v_base_hq
  FROM roles r WHERE r.name = 'InspectorCrew'
  ON CONFLICT (username) DO NOTHING;
  SELECT id INTO v_inspector2 FROM users WHERE username = 'ivan.inspect';

  -- Disposal Crew
  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'dana.disposal', 'dana@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Dana Disposal', r.id, v_base_hq
  FROM roles r WHERE r.name = 'DisposalCrew'
  ON CONFLICT (username) DO NOTHING;
  SELECT id INTO v_disposal1 FROM users WHERE username = 'dana.disposal';

  -- Wastewater Crew
  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'wendy.water', 'wendy@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Wendy Wastewater', r.id, v_base_hq
  FROM roles r WHERE r.name = 'WastewaterCrew'
  ON CONFLICT (username) DO NOTHING;
  SELECT id INTO v_wastewater1 FROM users WHERE username = 'wendy.water';

  -- Cleanup Crew
  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'carl.cleanup', 'carl@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Carl Cleanup', r.id, v_base_hq
  FROM roles r WHERE r.name = 'CleanupCrew'
  ON CONFLICT (username) DO NOTHING;
  SELECT id INTO v_cleanup1 FROM users WHERE username = 'carl.cleanup';

  -- Transport Crew
  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'tina.transport', 'tina@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Tina Transport', r.id, v_base_hq
  FROM roles r WHERE r.name = 'TransportCrew'
  ON CONFLICT (username) DO NOTHING;
  SELECT id INTO v_transport1 FROM users WHERE username = 'tina.transport';

  -- ═══════════════════════════════════════════════════════════
  -- 3. Division IDs
  -- ═══════════════════════════════════════════════════════════
  SELECT id INTO v_div_insp  FROM sanitary_divisions WHERE name = 'inspector';
  SELECT id INTO v_div_disp  FROM sanitary_divisions WHERE name = 'disposal';
  SELECT id INTO v_div_ww    FROM sanitary_divisions WHERE name = 'wastewater';
  SELECT id INTO v_div_clean FROM sanitary_divisions WHERE name = 'cleanup';
  SELECT id INTO v_div_trans FROM sanitary_divisions WHERE name = 'transport';

  -- ═══════════════════════════════════════════════════════════
  -- 4. Staff division assignments (current quarter)
  -- ═══════════════════════════════════════════════════════════
  INSERT INTO staff_division_assignments (user_id, division_id, quarter, assigned_by) VALUES
    (v_inspector1,  v_div_insp,  'Q1-2026', v_head_id),
    (v_inspector2,  v_div_insp,  'Q1-2026', v_head_id),
    (v_disposal1,   v_div_disp,  'Q1-2026', v_head_id),
    (v_wastewater1, v_div_ww,    'Q1-2026', v_head_id),
    (v_cleanup1,    v_div_clean, 'Q1-2026', v_head_id),
    (v_transport1,  v_div_trans, 'Q1-2026', v_head_id)
  ON CONFLICT DO NOTHING;

  -- Historical: previous quarter assignments
  INSERT INTO staff_division_assignments (user_id, division_id, quarter, assigned_by) VALUES
    (v_inspector1,  v_div_clean, 'Q4-2025', v_head_id),
    (v_cleanup1,    v_div_trans, 'Q4-2025', v_head_id)
  ON CONFLICT DO NOTHING;

  -- ═══════════════════════════════════════════════════════════
  -- 5. Sample sanitary inventory
  -- ═══════════════════════════════════════════════════════════
  INSERT INTO sanitary_inventory (item_name, category, quantity, unit, last_updated_by)
  VALUES
    ('Industrial Floor Cleaner', 'cleaning_product', 50, 'liters', v_head_id)
  RETURNING id INTO v_inv1;

  INSERT INTO sanitary_inventory (item_name, category, quantity, unit, last_updated_by)
  VALUES
    ('Heavy-Duty Mops', 'equipment', 25, 'units', v_head_id)
  RETURNING id INTO v_inv2;

  INSERT INTO sanitary_inventory (item_name, category, quantity, unit, last_updated_by)
  VALUES
    ('Hazmat Disposal Bags', 'supplies', 500, 'bags', v_head_id)
  RETURNING id INTO v_inv3;

  INSERT INTO sanitary_inventory (item_name, category, quantity, unit, last_updated_by) VALUES
    ('Antibacterial Soap', 'cleaning_product', 200, 'bottles', v_head_id),
    ('Pressure Washer',    'equipment',          3, 'units',   v_head_id),
    ('Latex Gloves (L)',   'supplies',         800, 'pairs',   v_head_id);

  -- ═══════════════════════════════════════════════════════════
  -- 6. Sample tasks
  -- ═══════════════════════════════════════════════════════════
  INSERT INTO sanitary_tasks (assigned_by, task_type, task_name, urgency, instructions, location, due_date, status)
  VALUES
    (v_head_id, 'cleaning', 'Deep clean Sector 7 corridors', 'high',
     'Full corridor scrub including ventilation grates. Report completion with photos.',
     'HQ — Sector 7', NOW() + INTERVAL '2 days', 'pending')
  RETURNING id INTO v_task1;

  INSERT INTO sanitary_task_assignments (task_id, user_id) VALUES
    (v_task1, v_cleanup1);

  INSERT INTO sanitary_tasks (assigned_by, task_type, task_name, urgency, instructions, location, due_date, status)
  VALUES
    (v_head_id, 'inspection', 'Quarterly hygiene audit — Cafeteria', 'medium',
     'Full audit of food preparation area. Check ventilation, surfaces, waste disposal.',
     'HQ — Main Cafeteria', NOW() + INTERVAL '5 days', 'pending')
  RETURNING id INTO v_task2;

  INSERT INTO sanitary_task_assignments (task_id, user_id) VALUES
    (v_task2, v_inspector1),
    (v_task2, v_inspector2);

  -- ═══════════════════════════════════════════════════════════
  -- 7. Sample inspection report
  -- ═══════════════════════════════════════════════════════════
  INSERT INTO inspection_reports
    (reported_by, report_date, location, area_or_machine, findings, severity, recommendations)
  VALUES
    (v_inspector1, CURRENT_DATE - 3,
     'HQ — Sector 4', 'Air Filtration Unit #12',
     'Filter clogged with particulate matter. Airflow reduced by ~40%. Potential health hazard.',
     'high',
     'Replace filter immediately. Schedule monthly inspection for all filtration units in Sector 4.');

  -- ═══════════════════════════════════════════════════════════
  -- 8. Sample disposal documentation
  -- ═══════════════════════════════════════════════════════════
  INSERT INTO disposal_handling_docs
    (waste_category, procedure, safety_requirements, compliance_notes, revision_history, authored_by)
  VALUES
    ('radioactive_waste',
     'Seal in lead-lined containment vessel. Transport to designated disposal bay via armored cart. Log serial number of each container.',
     'Full hazmat suit required. Two-person buddy system. Geiger counter reading before/after.',
     'Compliant with RUSA Regulation 14-B. Annual recertification required.',
     '[{"revision":1,"changed_by":"' || v_disposal1 || '","changed_at":"2025-12-01T10:00:00Z","summary":"Initial creation"}]'::jsonb,
     v_disposal1);

  -- ═══════════════════════════════════════════════════════════
  -- 9. Sample wastewater documentation
  -- ═══════════════════════════════════════════════════════════
  INSERT INTO wastewater_treatment_docs
    (treatment_type, steps, safety_requirements, compliance_notes, revision_history, authored_by)
  VALUES
    ('chemical_neutralization',
     '["Step 1: Test pH levels of incoming waste stream","Step 2: Add neutralizing agent per dosage chart","Step 3: Allow 30-minute reaction period with agitation","Step 4: Re-test pH — must be between 6.5 and 8.5","Step 5: Route to clean water holding tank"]'::jsonb,
     'Chemical-resistant gloves and goggles required. Emergency shower within 10m.',
     'Meets RUSA Environmental Standard 22-C.',
     '[{"revision":1,"changed_by":"' || v_wastewater1 || '","changed_at":"2025-11-15T14:00:00Z","summary":"Initial procedure draft"}]'::jsonb,
     v_wastewater1);

  -- ═══════════════════════════════════════════════════════════
  -- 10. Sample shifts (current quarter)
  -- ═══════════════════════════════════════════════════════════
  INSERT INTO sanitary_shifts (user_id, shift_start, shift_end, allocated_by, quarter) VALUES
    (v_inspector1, date_trunc('day', NOW()) + INTERVAL '6 hours',
                   date_trunc('day', NOW()) + INTERVAL '14 hours', v_head_id, 'Q1-2026'),
    (v_inspector1, date_trunc('day', NOW()) + INTERVAL '1 day 6 hours',
                   date_trunc('day', NOW()) + INTERVAL '1 day 14 hours', v_head_id, 'Q1-2026'),
    (v_cleanup1,   date_trunc('day', NOW()) + INTERVAL '14 hours',
                   date_trunc('day', NOW()) + INTERVAL '22 hours', v_head_id, 'Q1-2026'),
    (v_disposal1,  date_trunc('day', NOW()) + INTERVAL '8 hours',
                   date_trunc('day', NOW()) + INTERVAL '16 hours', v_head_id, 'Q1-2026');

  -- ═══════════════════════════════════════════════════════════
  -- 11. Sample inventory log entries
  -- ═══════════════════════════════════════════════════════════
  INSERT INTO sanitary_inventory_logs (item_id, logged_by, action, quantity, purchase_note) VALUES
    (v_inv1, v_cleanup1, 'remove', 5, NULL),
    (v_inv3, v_disposal1, 'remove', 20, NULL),
    (v_inv2, v_head_id, 'add', 10, 'Purchased 10 new mops — requisition #R-2026-044');

END $$;
