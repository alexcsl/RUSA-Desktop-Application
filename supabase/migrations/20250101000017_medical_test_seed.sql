-- ============================================================
-- 017_medical_test_seed.sql
-- Test data for Medical Department development.
-- Password: password123
-- ============================================================

DO $$
DECLARE
  v_hom_id      UUID;
  v_staff1_id   UUID;   -- charlie.pulse already seeded as MedicalStaff
  v_staff2_id   UUID;
  v_base_hq     UUID := 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa';
  v_base_alpha  UUID := 'cccccccc-cccc-cccc-cccc-cccccccccccc';
  v_patient1_id UUID;
  v_patient2_id UUID;
  v_rand_user   UUID;
BEGIN

  -- ═══════════════════════════════════════════════════════════
  -- 1. HeadOfMedicine account (one at HQ, one at Alpha Station)
  -- ═══════════════════════════════════════════════════════════
  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'helen.medic', 'helen@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Dr. Helen Medic', r.id, v_base_hq
  FROM roles r WHERE r.name = 'HeadOfMedicine'
  ON CONFLICT (username) DO NOTHING;

  SELECT id INTO v_hom_id FROM users WHERE username = 'helen.medic';

  -- Second Head at Alpha Station
  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'ray.surgeon', 'ray@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Dr. Ray Surgeon', r.id, v_base_alpha
  FROM roles r WHERE r.name = 'HeadOfMedicine'
  ON CONFLICT (username) DO NOTHING;

  -- ═══════════════════════════════════════════════════════════
  -- 2. MedicalStaff accounts
  -- ═══════════════════════════════════════════════════════════
  -- charlie.pulse already exists from 002_seed_data.sql
  SELECT id INTO v_staff1_id FROM users WHERE username = 'charlie.pulse';

  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'nadia.care', 'nadia@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Nadia Care', r.id, v_base_hq
  FROM roles r WHERE r.name = 'MedicalStaff'
  ON CONFLICT (username) DO NOTHING;

  SELECT id INTO v_staff2_id FROM users WHERE username = 'nadia.care';

  INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
  SELECT 'marco.healer', 'marco@rusa.internal',
         '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
         'Marco Healer', r.id, v_base_alpha
  FROM roles r WHERE r.name = 'MedicalStaff'
  ON CONFLICT (username) DO NOTHING;

  -- ═══════════════════════════════════════════════════════════
  -- 3. Medical staff profiles (specialty)
  -- ═══════════════════════════════════════════════════════════
  INSERT INTO medical_staff_profiles (user_id, specialty, base_id) VALUES
    (v_hom_id,    'general',    v_base_hq),
    (v_staff1_id, 'emergency',  v_base_hq),
    (v_staff2_id, 'cardiology', v_base_hq)
  ON CONFLICT (user_id) DO NOTHING;

  INSERT INTO medical_staff_profiles (user_id, specialty, base_id)
  SELECT u.id, 'surgery', v_base_alpha
  FROM users u WHERE u.username = 'ray.surgeon'
  ON CONFLICT (user_id) DO NOTHING;

  INSERT INTO medical_staff_profiles (user_id, specialty, base_id)
  SELECT u.id, 'neurology', v_base_alpha
  FROM users u WHERE u.username = 'marco.healer'
  ON CONFLICT (user_id) DO NOTHING;

  -- ═══════════════════════════════════════════════════════════
  -- 4. Register some patients (use random existing users)
  -- ═══════════════════════════════════════════════════════════
  -- Use delta.astro (Astronaut) as a patient
  SELECT id INTO v_rand_user FROM users WHERE username = 'delta.astro';
  INSERT INTO medical_patients (user_id) VALUES (v_rand_user)
  ON CONFLICT (user_id) DO NOTHING;
  SELECT id INTO v_patient1_id FROM medical_patients WHERE user_id = v_rand_user;

  -- Use echo.settler as another patient
  SELECT id INTO v_rand_user FROM users WHERE username = 'echo.settler';
  INSERT INTO medical_patients (user_id) VALUES (v_rand_user)
  ON CONFLICT (user_id) DO NOTHING;
  SELECT id INTO v_patient2_id FROM medical_patients WHERE user_id = v_rand_user;

  -- ═══════════════════════════════════════════════════════════
  -- 5. Sample treatment logs
  -- ═══════════════════════════════════════════════════════════
  INSERT INTO patient_treatment_logs
    (patient_id, treated_by, treatment_date, diagnosis, treatment_provided, medications, follow_up_notes)
  VALUES
    (v_patient1_id, v_staff1_id, NOW() - INTERVAL '10 days',
     'Mild radiation exposure during EVA',
     'IV fluid replenishment, chelation therapy session 1',
     'Chelation agent DMSA 500mg BID',
     'Schedule session 2 in 5 days. Monitor blood lead levels.'),
    (v_patient1_id, v_hom_id, NOW() - INTERVAL '5 days',
     'Follow-up: radiation exposure',
     'Chelation therapy session 2. Blood panel reviewed — levels normalizing.',
     'DMSA 250mg QD tapering dose',
     'Final clearance pending next blood draw.'),
    (v_patient2_id, v_staff2_id, NOW() - INTERVAL '3 days',
     'Hypertension detected during routine physical',
     'Lifestyle counseling, BP monitoring device issued',
     'Amlodipine 5mg QD',
     'Recheck in 2 weeks. Log daily BP readings.');

  -- ═══════════════════════════════════════════════════════════
  -- 6. Sample medical inventory
  -- ═══════════════════════════════════════════════════════════
  INSERT INTO medical_inventory (base_id, item_name, item_type, quantity, unit, last_updated_by) VALUES
    (v_base_hq, 'Amlodipine 5mg tablets',       'medicine',   500, 'tablets',   v_hom_id),
    (v_base_hq, 'DMSA 500mg capsules',           'medicine',   120, 'capsules',  v_hom_id),
    (v_base_hq, 'Portable Ultrasound',           'equipment',    3, 'units',     v_hom_id),
    (v_base_hq, 'Saline IV Bags 500ml',          'supply',     200, 'bags',      v_staff1_id),
    (v_base_hq, 'Sterile Gloves (Large)',        'supply',    1000, 'pairs',     v_staff1_id),
    (v_base_alpha, 'Surgical Kit Standard',      'equipment',    5, 'kits',      v_hom_id),
    (v_base_alpha, 'Ibuprofen 400mg tablets',    'medicine',   800, 'tablets',   v_hom_id);

  -- ═══════════════════════════════════════════════════════════
  -- 7. Sample staff shifts
  -- ═══════════════════════════════════════════════════════════
  INSERT INTO staff_shifts (user_id, base_id, shift_start, shift_end, allocated_by) VALUES
    (v_staff1_id, v_base_hq,
     date_trunc('day', NOW()) + INTERVAL '8 hours',
     date_trunc('day', NOW()) + INTERVAL '16 hours',
     v_hom_id),
    (v_staff1_id, v_base_hq,
     date_trunc('day', NOW()) + INTERVAL '1 day 8 hours',
     date_trunc('day', NOW()) + INTERVAL '1 day 16 hours',
     v_hom_id),
    (v_staff2_id, v_base_hq,
     date_trunc('day', NOW()) + INTERVAL '16 hours',
     date_trunc('day', NOW()) + INTERVAL '24 hours',
     v_hom_id),
    (v_staff2_id, v_base_hq,
     date_trunc('day', NOW()) + INTERVAL '1 day 16 hours',
     date_trunc('day', NOW()) + INTERVAL '2 days',
     v_hom_id);

END $$;
