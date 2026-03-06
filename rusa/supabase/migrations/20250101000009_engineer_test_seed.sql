-- ============================================================
-- 009_engineer_test_seed.sql
-- Test data for the Engineers subsystem (Agricultural + Biological)
-- Password: password123  →  $2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO
-- ============================================================

-- ════════════════════════════════════════════════════════════════════════════════
-- ENGINEER USER ACCOUNTS
-- ════════════════════════════════════════════════════════════════════════════════

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'flora.green', 'flora@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Flora Green', r.id, 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
FROM roles r WHERE r.name = 'AgriculturalEngineer'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'rex.strand', 'rex@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Rex Strand', r.id, 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
FROM roles r WHERE r.name = 'BiologicalEngineer'
ON CONFLICT (username) DO NOTHING;

-- ════════════════════════════════════════════════════════════════════════════════
-- SEED DATA — Tasks, Experiments, Tests, Species, Proposals, Notifications
-- ════════════════════════════════════════════════════════════════════════════════

DO $$
DECLARE
  -- User IDs
  v_flora_id     UUID;
  v_rex_id       UUID;
  v_olga_id      UUID;   -- TheObserver (proxy director for engineers)
  v_task_iron_id UUID;   -- TheTaskmaster
  v_bio_darwin_id UUID;  -- Biologist (attendee for logs)

  -- Task IDs
  v_task1 UUID;
  v_task2 UUID;
  v_task3 UUID;
  v_task4 UUID;
  v_task5 UUID;

  -- Approved Test IDs
  v_test_plants_1 UUID;
  v_test_plants_2 UUID;
  v_test_plants_3 UUID;
  v_test_all_1    UUID;
  v_test_all_2    UUID;
  v_test_all_3    UUID;

  -- Experiment IDs
  v_exp_agri_1   UUID;  -- approved (ready for logging)
  v_exp_agri_2   UUID;  -- active (has some logs)
  v_exp_agri_3   UUID;  -- conclusion_requested
  v_exp_bio_1    UUID;  -- approved
  v_exp_bio_2    UUID;  -- active
  v_exp_bio_3    UUID;  -- completed

  -- Science Archive (species) IDs
  v_species_1 UUID;
  v_species_2 UUID;
  v_species_3 UUID;
  v_species_4 UUID;
  v_species_5 UUID;

  -- Daily Log IDs
  v_log1 UUID;
  v_log2 UUID;
  v_log3 UUID;
  v_log4 UUID;

  -- Proposal IDs
  v_proposal1 UUID;
  v_proposal2 UUID;
  v_proposal3 UUID;

BEGIN
  -- ── Resolve User IDs ──────────────────────────────────────────────────────
  SELECT id INTO v_flora_id     FROM users WHERE username = 'flora.green';
  SELECT id INTO v_rex_id       FROM users WHERE username = 'rex.strand';
  SELECT id INTO v_olga_id      FROM users WHERE username = 'olga.scope';
  SELECT id INTO v_task_iron_id FROM users WHERE username = 'task.iron';
  SELECT id INTO v_bio_darwin_id FROM users WHERE username = 'bio.darwin';

  -- ═══════════════════════════════════════════════════════════════════════════
  -- TASKS assigned to engineers by The Observer (UC-GE-01)
  -- ═══════════════════════════════════════════════════════════════════════════

  -- Flora (AgriculturalEngineer) — 3 tasks
  INSERT INTO tasks (id, assigned_by, assigned_to, type, title, description, payload, status, due_date)
  VALUES (
    gen_random_uuid(), v_olga_id, v_flora_id,
    'experiment',
    'Set Up Hydroponic Bay 7 for Wheat Trial',
    'Configure hydroponic nutrient mix, lighting schedule, and sensors for winter wheat cultivar W-22 in Bay 7.',
    '{"bay": "7", "cultivar": "W-22", "priority": "high"}'::jsonb,
    'in_progress',
    CURRENT_DATE + INTERVAL '14 days'
  ) RETURNING id INTO v_task1;

  INSERT INTO tasks (id, assigned_by, assigned_to, type, title, description, payload, status, due_date)
  VALUES (
    gen_random_uuid(), v_olga_id, v_flora_id,
    'general',
    'Monthly Soil Composition Report',
    'Compile soil composition data from all agricultural bays for monthly review.',
    '{"report_type": "soil_composition", "month": "2026-01"}'::jsonb,
    'pending',
    CURRENT_DATE + INTERVAL '7 days'
  ) RETURNING id INTO v_task2;

  INSERT INTO tasks (id, assigned_by, assigned_to, type, title, description, payload, status, due_date)
  VALUES (
    gen_random_uuid(), v_olga_id, v_flora_id,
    'general',
    'Calibrate Irrigation Sensors — Block A',
    'Recalibrate all moisture and pH sensors across Block A greenhouses after firmware update.',
    '{"block": "A", "sensor_type": ["moisture", "pH"]}'::jsonb,
    'completed',
    CURRENT_DATE - INTERVAL '3 days'
  ) RETURNING id INTO v_task3;

  -- Rex (BiologicalEngineer) — 2 tasks
  INSERT INTO tasks (id, assigned_by, assigned_to, type, title, description, payload, status, due_date)
  VALUES (
    gen_random_uuid(), v_olga_id, v_rex_id,
    'experiment',
    'Begin Bacteriophage Resistance Study on E. xenobi',
    'Initiate study on bacteriophage resistance mechanisms in E. xenobi cultures sourced from Kepler Outpost.',
    '{"species": "E. xenobi", "source": "Kepler Outpost", "biosafety_level": 2}'::jsonb,
    'in_progress',
    CURRENT_DATE + INTERVAL '30 days'
  ) RETURNING id INTO v_task4;

  INSERT INTO tasks (id, assigned_by, assigned_to, type, title, description, payload, status, due_date)
  VALUES (
    gen_random_uuid(), v_olga_id, v_rex_id,
    'general',
    'Update Bio-Lab Sterilisation SOP',
    'Revise the standard operating procedures for bio-lab sterilisation cycles to include the new UV-C modules.',
    '{"document": "SOP-BL-004", "revision": 3}'::jsonb,
    'pending',
    CURRENT_DATE + INTERVAL '10 days'
  ) RETURNING id INTO v_task5;

  -- ═══════════════════════════════════════════════════════════════════════════
  -- APPROVED TESTS (UC-AGE-01 / UC-BE-01)
  -- ═══════════════════════════════════════════════════════════════════════════

  -- Plants scope (for AgriculturalEngineer)
  INSERT INTO approved_tests (id, name, procedure, category, applicable_scope, accepted_at)
  VALUES (
    gen_random_uuid(),
    'Chlorophyll Fluorescence Assay',
    '1. Dark-adapt leaf samples for 30 min.  2. Apply modulated light pulse.  3. Record Fv/Fm ratio via PAM fluorometer.  4. Log results per cultivar.',
    'photosynthesis',
    'plants',
    NOW() - INTERVAL '60 days'
  ) RETURNING id INTO v_test_plants_1;

  INSERT INTO approved_tests (id, name, procedure, category, applicable_scope, accepted_at)
  VALUES (
    gen_random_uuid(),
    'Soil Microbiome Diversity Index',
    '1. Collect 10 g rhizosphere soil.  2. Extract DNA via bead-beating.  3. Run 16S rRNA amplicon sequencing.  4. Calculate Shannon index.',
    'microbiology',
    'plants',
    NOW() - INTERVAL '45 days'
  ) RETURNING id INTO v_test_plants_2;

  INSERT INTO approved_tests (id, name, procedure, category, applicable_scope, accepted_at)
  VALUES (
    gen_random_uuid(),
    'Root Growth Kinetics Measurement',
    '1. Place seedlings on transparent agar gel.  2. Photograph every 6 hours for 7 days.  3. Digitise root length via ImageJ.  4. Fit logistic growth curve.',
    'growth',
    'plants',
    NOW() - INTERVAL '30 days'
  ) RETURNING id INTO v_test_plants_3;

  -- All-species scope (for BiologicalEngineer)
  INSERT INTO approved_tests (id, name, procedure, category, applicable_scope, accepted_at)
  VALUES (
    gen_random_uuid(),
    'CRISPR Gene-Editing Efficiency Assay',
    '1. Design sgRNA targeting locus X.  2. Transfect cell cultures via electroporation.  3. Sequence target locus 72 h post-transfection.  4. Calculate indel frequency.',
    'genetics',
    'all_species',
    NOW() - INTERVAL '90 days'
  ) RETURNING id INTO v_test_all_1;

  INSERT INTO approved_tests (id, name, procedure, category, applicable_scope, accepted_at)
  VALUES (
    gen_random_uuid(),
    'Antimicrobial Susceptibility Test (MIC)',
    '1. Prepare serial dilutions of antimicrobial agent.  2. Inoculate each dilution with standardised inoculum (0.5 McFarland).  3. Incubate 18–24 h at 35 °C.  4. Record MIC.',
    'microbiology',
    'all_species',
    NOW() - INTERVAL '75 days'
  ) RETURNING id INTO v_test_all_2;

  INSERT INTO approved_tests (id, name, procedure, category, applicable_scope, accepted_at)
  VALUES (
    gen_random_uuid(),
    'Cell Viability (MTT) Assay',
    '1. Seed cells in 96-well plate.  2. Apply treatment for 48 h.  3. Add MTT reagent and incubate 4 h.  4. Solubilise precipitate and read absorbance at 570 nm.',
    'cell_biology',
    'all_species',
    NOW() - INTERVAL '40 days'
  ) RETURNING id INTO v_test_all_3;

  -- ═══════════════════════════════════════════════════════════════════════════
  -- SCIENCE ARCHIVE — SPECIES entries (UC-AGE-05 / UC-BE-01)
  -- ═══════════════════════════════════════════════════════════════════════════

  INSERT INTO science_archive (id, type, name, classification, detail, submitted_by)
  VALUES (
    gen_random_uuid(), 'species',
    'Triticum novastella',
    'Plantae',
    '{
      "taxonomy": {"kingdom": "Plantae", "family": "Poaceae", "genus": "Triticum", "species": "novastella"},
      "habitat": "Kepler hydroponic bays",
      "description": "Space-adapted winter wheat cultivar developed for low-gravity grain production.",
      "discovery_date": "2025-03-14"
    }'::jsonb,
    v_flora_id
  ) RETURNING id INTO v_species_1;

  INSERT INTO science_archive (id, type, name, classification, detail, submitted_by)
  VALUES (
    gen_random_uuid(), 'species',
    'Solanum keplerensis',
    'Plantae',
    '{
      "taxonomy": {"kingdom": "Plantae", "family": "Solanaceae", "genus": "Solanum", "species": "keplerensis"},
      "habitat": "Pressurised greenhouse modules",
      "description": "Radiation-tolerant potato variant engineered for Kepler Outpost agriculture.",
      "discovery_date": "2025-06-01"
    }'::jsonb,
    v_flora_id
  ) RETURNING id INTO v_species_2;

  INSERT INTO science_archive (id, type, name, classification, detail, submitted_by)
  VALUES (
    gen_random_uuid(), 'species',
    'Chlorella stellaris',
    'Plantae',
    '{
      "taxonomy": {"kingdom": "Plantae", "family": "Chlorellaceae", "genus": "Chlorella", "species": "stellaris"},
      "habitat": "Bioreactor cultures",
      "description": "High-lipid microalgae optimised for life-support CO2 scrubbing and nutrition supplements.",
      "discovery_date": "2025-01-20"
    }'::jsonb,
    v_flora_id
  ) RETURNING id INTO v_species_3;

  INSERT INTO science_archive (id, type, name, classification, detail, submitted_by)
  VALUES (
    gen_random_uuid(), 'species',
    'Escherichia xenobi',
    'Bacteria',
    '{
      "taxonomy": {"kingdom": "Bacteria", "family": "Enterobacteriaceae", "genus": "Escherichia", "species": "xenobi"},
      "habitat": "Kepler Outpost soil samples",
      "description": "Novel extremophile bacterium showing natural CRISPR-Cas immunity. Potential bio-engineering chassis.",
      "discovery_date": "2025-09-10"
    }'::jsonb,
    v_rex_id
  ) RETURNING id INTO v_species_4;

  INSERT INTO science_archive (id, type, name, classification, detail, submitted_by)
  VALUES (
    gen_random_uuid(), 'species',
    'Tardigrada cosmonauta',
    'Animalia',
    '{
      "taxonomy": {"kingdom": "Animalia", "phylum": "Tardigrada", "genus": "Milnesium", "species": "cosmonauta"},
      "habitat": "Exterior hull sampling programme",
      "description": "Micro-animal recovered from Alpha Station hull, displaying enhanced radiation tolerance.",
      "discovery_date": "2025-11-02"
    }'::jsonb,
    v_rex_id
  ) RETURNING id INTO v_species_5;

  -- ═══════════════════════════════════════════════════════════════════════════
  -- EXPERIMENTS (UC-AGE-03/04/06, UC-BE-02/04/05)
  -- ═══════════════════════════════════════════════════════════════════════════

  -- Agricultural experiments — for Flora
  INSERT INTO experiments (id, title, type, metadata, proposed_by, status, approved_by, approved_at)
  VALUES (
    gen_random_uuid(),
    'Hydroponic Wheat Yield Optimisation — Bay 7',
    'agricultural',
    '{"cultivar": "W-22", "bay": 7, "objective": "Maximise grain yield under reduced-gravity nutrient delivery"}'::jsonb,
    v_flora_id,
    'approved',
    v_olga_id,
    NOW() - INTERVAL '10 days'
  ) RETURNING id INTO v_exp_agri_1;

  INSERT INTO experiments (id, title, type, metadata, proposed_by, status, approved_by, approved_at)
  VALUES (
    gen_random_uuid(),
    'Microalgae Lipid Accumulation Under UV-B Stress',
    'agricultural',
    '{"species": "Chlorella stellaris", "stress_factor": "UV-B", "duration_days": 21}'::jsonb,
    v_flora_id,
    'active',
    v_olga_id,
    NOW() - INTERVAL '15 days'
  ) RETURNING id INTO v_exp_agri_2;

  INSERT INTO experiments (id, title, type, metadata, proposed_by, status, approved_by, approved_at)
  VALUES (
    gen_random_uuid(),
    'Potato Root Architecture in Martian Regolith Simulant',
    'agricultural',
    '{"species": "Solanum keplerensis", "substrate": "Mars regolith simulant MGS-1", "duration_days": 60}'::jsonb,
    v_flora_id,
    'conclusion_requested',
    v_olga_id,
    NOW() - INTERVAL '65 days'
  ) RETURNING id INTO v_exp_agri_3;

  -- Biological engineering experiments — for Rex
  INSERT INTO experiments (id, title, type, metadata, proposed_by, status, approved_by, approved_at)
  VALUES (
    gen_random_uuid(),
    'E. xenobi CRISPR-Cas Characterisation',
    'biological_engineering',
    '{"target_gene": "cas9_homolog", "method": "whole-genome sequencing + functional assay", "biosafety": 2}'::jsonb,
    v_rex_id,
    'approved',
    v_olga_id,
    NOW() - INTERVAL '5 days'
  ) RETURNING id INTO v_exp_bio_1;

  INSERT INTO experiments (id, title, type, metadata, proposed_by, status, approved_by, approved_at)
  VALUES (
    gen_random_uuid(),
    'Tardigrade Radiation Tolerance — Gene Expression Profiling',
    'biological_engineering',
    '{"species": "Tardigrada cosmonauta", "radiation_dose_Gy": [0, 1, 5, 10], "timepoints_h": [0, 6, 24, 72]}'::jsonb,
    v_rex_id,
    'active',
    v_olga_id,
    NOW() - INTERVAL '20 days'
  ) RETURNING id INTO v_exp_bio_2;

  INSERT INTO experiments (id, title, type, metadata, proposed_by, status, approved_by, approved_at, closed_at)
  VALUES (
    gen_random_uuid(),
    'Antimicrobial Peptide Screening — Kepler Soil Isolates',
    'biological_engineering',
    '{"isolates_screened": 48, "hits": 7, "method": "agar-diffusion + MIC"}'::jsonb,
    v_rex_id,
    'closed',
    v_olga_id,
    NOW() - INTERVAL '90 days',
    NOW() - INTERVAL '10 days'
  ) RETURNING id INTO v_exp_bio_3;

  -- ═══════════════════════════════════════════════════════════════════════════
  -- EXPERIMENT DAILY LOGS (UC-AGE-03 / UC-BE-04)
  -- ═══════════════════════════════════════════════════════════════════════════

  -- 2 Logs for the active agricultural experiment (v_exp_agri_2)
  INSERT INTO experiment_daily_logs (id, experiment_id, log_date, rag_status, completed_actions, pending_actions, collected_data, created_by)
  VALUES (
    gen_random_uuid(), v_exp_agri_2,
    CURRENT_DATE - INTERVAL '5 days', 'green',
    'Inoculated 6 bioreactor flasks with C. stellaris seed culture.',
    'Begin UV-B exposure on Day 2.',
    '{"cell_density_OD680": 0.12, "temperature_C": 25, "pH": 7.2}'::jsonb,
    v_flora_id
  ) RETURNING id INTO v_log1;

  INSERT INTO experiment_daily_logs (id, experiment_id, log_date, rag_status, completed_actions, pending_actions, collected_data, created_by)
  VALUES (
    gen_random_uuid(), v_exp_agri_2,
    CURRENT_DATE - INTERVAL '4 days', 'amber',
    'Started UV-B at 3.5 W/m² for 4 h/day. Flask #3 showed slower growth — possible contamination.',
    'Microscopy check on Flask #3 tomorrow. Continue UV-B for remaining flasks.',
    '{"cell_density_OD680": [0.18, 0.20, 0.09, 0.19, 0.21, 0.17], "notes": "Flask 3 flagged"}'::jsonb,
    v_flora_id
  ) RETURNING id INTO v_log2;

  -- 2 Logs for the active biological experiment (v_exp_bio_2)
  INSERT INTO experiment_daily_logs (id, experiment_id, log_date, rag_status, completed_actions, pending_actions, collected_data, created_by)
  VALUES (
    gen_random_uuid(), v_exp_bio_2,
    CURRENT_DATE - INTERVAL '3 days', 'green',
    'Irradiated sample groups at 0, 1, 5, 10 Gy.  Harvested 0 h time-point for RNA extraction.',
    'Process 6 h time-point samples tomorrow.',
    '{"irradiation_completed": true, "samples_harvested": 12, "rna_quality_RIN": [9.2, 8.8, 9.0, 8.5]}'::jsonb,
    v_rex_id
  ) RETURNING id INTO v_log3;

  INSERT INTO experiment_daily_logs (id, experiment_id, log_date, rag_status, completed_actions, pending_actions, collected_data, created_by)
  VALUES (
    gen_random_uuid(), v_exp_bio_2,
    CURRENT_DATE - INTERVAL '2 days', 'green',
    'Harvested 6 h and 24 h time-points. Began cDNA library preparation for sequencing.',
    'Submit libraries to sequencer queue. Await 72 h sample harvest.',
    '{"samples_harvested": 24, "cDNA_libraries_prepared": 12}'::jsonb,
    v_rex_id
  ) RETURNING id INTO v_log4;

  -- ── Log attendees ──
  INSERT INTO experiment_log_attendees (log_id, user_id) VALUES
    (v_log1, v_flora_id),
    (v_log2, v_flora_id),
    (v_log3, v_rex_id),
    (v_log3, v_bio_darwin_id),
    (v_log4, v_rex_id),
    (v_log4, v_bio_darwin_id)
  ON CONFLICT DO NOTHING;

  -- ── Log tests performed ──
  INSERT INTO experiment_log_tests (log_id, test_id) VALUES
    (v_log1, v_test_plants_1),
    (v_log2, v_test_plants_2),
    (v_log3, v_test_all_1),
    (v_log4, v_test_all_3)
  ON CONFLICT DO NOTHING;

  -- ── Log species worked on ──
  INSERT INTO experiment_log_species (log_id, species_archive_id) VALUES
    (v_log1, v_species_3),    -- Chlorella stellaris
    (v_log2, v_species_3),
    (v_log3, v_species_5),    -- Tardigrada cosmonauta
    (v_log4, v_species_5)
  ON CONFLICT DO NOTHING;

  -- ═══════════════════════════════════════════════════════════════════════════
  -- PROGRESS REPORTS (UC-GE-02)
  -- Sample reports for completed & in-progress tasks
  -- ═══════════════════════════════════════════════════════════════════════════

  INSERT INTO progress_reports (task_id, submitted_by, report_date, current_status, work_completed, problems_encountered, plans_next)
  VALUES
    (v_task3, v_flora_id, CURRENT_DATE - INTERVAL '4 days',
     'completed',
     'All 24 moisture sensors and 12 pH sensors in Block A recalibrated. Firmware v2.4.1 verified.',
     NULL,
     NULL),
    (v_task1, v_flora_id, CURRENT_DATE - INTERVAL '1 day',
     'in_progress',
     'Nutrient reservoir mixed (N-P-K 5-3-4). Lighting rig installed with 18/6 photoperiod.',
     'Bay 7 exhaust fan intermittent — maintenance ticket submitted.',
     'Connect drip lines and seed trays by end of week.');

  -- Rex progress report on his in-progress task
  INSERT INTO progress_reports (task_id, submitted_by, report_date, current_status, work_completed, problems_encountered, plans_next)
  VALUES
    (v_task4, v_rex_id, CURRENT_DATE - INTERVAL '2 days',
     'in_progress',
     'Obtained E. xenobi cultures from Kepler cold-store. Set up BSL-2 containment rack. Initial phage challenge plates prepared.',
     'Incubator #2 temperature drift — swapped to Incubator #5.',
     'Read phage plates at 24 h and 48 h. Begin genomic DNA extraction for sequencing.');

  -- ═══════════════════════════════════════════════════════════════════════════
  -- TEST PROPOSALS (UC-AGE-02 / UC-BE-03)
  -- ═══════════════════════════════════════════════════════════════════════════

  INSERT INTO test_proposals (id, proposed_by, proposal_data, status)
  VALUES (
    gen_random_uuid(), v_flora_id,
    '{
      "name": "Leaf Water Potential Measurement (Pressure Chamber)",
      "goal": "Quantify plant water stress in hydroponic vs regolith substrates.",
      "procedure": "1. Predawn harvest of leaf. 2. Seal in Scholander pressure chamber. 3. Record balancing pressure.",
      "species_scope": "plants",
      "category": ["physiology", "water_stress"],
      "apparatuses": "Scholander Pressure Chamber Model 600",
      "required_data": "Balancing pressure (MPa) per cultivar per substrate",
      "justification": "Current test database lacks a direct water-stress assay for hydroponic environments."
    }'::jsonb,
    'pending'
  ) RETURNING id INTO v_proposal1;

  INSERT INTO test_proposals (id, proposed_by, proposal_data, status, reviewer_note)
  VALUES (
    gen_random_uuid(), v_rex_id,
    '{
      "name": "Phage Plaque Assay for E. xenobi",
      "goal": "Determine bacteriophage lytic activity against E. xenobi isolates.",
      "procedure": "1. Prepare double-agar overlay with E. xenobi lawn. 2. Spot serial dilutions of phage lysate. 3. Incubate 18 h. 4. Count plaques.",
      "species_scope": "all_species",
      "category": ["microbiology", "virology"],
      "apparatuses": "Standard microbiology bench equipment",
      "required_data": "PFU/mL per phage isolate",
      "justification": "No existing assay for bacteriophage quantification in updated test database."
    }'::jsonb,
    'approved',
    'Excellent proposal — approved by TheObserver. Please also capture plaque morphology images.'
  ) RETURNING id INTO v_proposal2;

  INSERT INTO test_proposals (id, proposed_by, proposal_data, status, reviewer_note)
  VALUES (
    gen_random_uuid(), v_flora_id,
    '{
      "name": "Seed Germination Rate Under Simulated Cosmic Radiation",
      "goal": "Assess germination percentage of irradiated vs control seeds.",
      "procedure": "1. Expose 100 seeds to cobalt-60 gamma source at 0.5 Gy. 2. Place on moist filter paper. 3. Count germinated seeds at 72 h, 120 h, 168 h.",
      "species_scope": "plants",
      "category": ["radiation_biology", "germination"],
      "apparatuses": "Cobalt-60 irradiation facility, growth chamber",
      "required_data": "Germination % at each time-point",
      "justification": "Needed for seed viability certification of Kepler-bound crop shipments."
    }'::jsonb,
    'rejected',
    'Radiation facility is under maintenance until Q2 2026. Please resubmit after March.'
  ) RETURNING id INTO v_proposal3;

  -- ═══════════════════════════════════════════════════════════════════════════
  -- HELP REQUESTS (UC-GE-03) — existing tasks created by engineers
  -- These are tasks where assigned_by = engineer, assigned_to = observer
  -- ═══════════════════════════════════════════════════════════════════════════

  INSERT INTO tasks (assigned_by, assigned_to, type, title, description, payload, status)
  VALUES (
    v_flora_id, v_olga_id,
    'help_request',
    'Request: Chemical Analysis Support — Bay 7 Nutrient Solution',
    'Need the Chemistry department to run ICP-OES on our nutrient reservoir. Trace-element levels seem off based on plant symptoms.',
    '{"target_department": "Chemistry", "urgency": "high", "requester_role": "AgriculturalEngineer"}'::jsonb,
    'pending'
  );

  INSERT INTO tasks (assigned_by, assigned_to, type, title, description, payload, status)
  VALUES (
    v_rex_id, v_olga_id,
    'help_request',
    'Request: Sequencing Queue Priority — E. xenobi Genome',
    'Requesting priority slot on the Illumina sequencer for whole-genome sequencing of E. xenobi — critical path item for CRISPR study.',
    '{"target_department": "Data Analysis", "urgency": "normal", "requester_role": "BiologicalEngineer"}'::jsonb,
    'in_progress'
  );

  -- ═══════════════════════════════════════════════════════════════════════════
  -- EXPERIMENT CONCLUSION REQUEST (already exists for v_exp_agri_3)
  -- ═══════════════════════════════════════════════════════════════════════════

  INSERT INTO requests (type, requester_id, title, description, payload, bypass_authority, status)
  VALUES (
    'experiment_closure', v_flora_id,
    'Conclusion: Potato Root Architecture in Martian Regolith Simulant',
    'After 60 days of growth, Solanum keplerensis shows 23% shorter primary roots in regolith vs hydroponic controls, but compensatory lateral branching maintained total root surface area within 5%.',
    jsonb_build_object(
      'experiment_id', v_exp_agri_3,
      'experiment_type', 'agricultural',
      'final_outcomes', 'Root biomass comparable. Recommend regolith-hydroponic hybrid substrate for Kepler deployment.'
    ),
    'TheTaskmaster',
    'pending'
  );

  -- ═══════════════════════════════════════════════════════════════════════════
  -- BROADCAST REQUEST (UC-ANC-00 — shared with directors module)
  -- ═══════════════════════════════════════════════════════════════════════════

  INSERT INTO broadcast_requests (requester_id, type, subject, content, target_scope, urgency, status)
  VALUES (
    v_flora_id,
    'informational',
    'Bay 7 Restricted Access — Wheat Trial Setup in Progress',
    'Hydroponic Bay 7 will be under restricted access for the next 48 hours while sensitive nutrient calibration and planting procedures are underway. Please avoid entry without clearance from the Agricultural Engineering team.',
    'company_wide',
    'normal',
    'pending'
  );

  -- ═══════════════════════════════════════════════════════════════════════════
  -- NOTIFICATIONS (for bell badge to have data)
  -- ═══════════════════════════════════════════════════════════════════════════

  INSERT INTO notifications (user_id, type, payload) VALUES
    (v_flora_id, 'task:assigned',
     jsonb_build_object('task_id', v_task1, 'title', 'Set Up Hydroponic Bay 7 for Wheat Trial', 'assigned_by', 'Olga Scope')),
    (v_flora_id, 'task:assigned',
     jsonb_build_object('task_id', v_task2, 'title', 'Monthly Soil Composition Report', 'assigned_by', 'Olga Scope')),
    (v_rex_id, 'task:assigned',
     jsonb_build_object('task_id', v_task4, 'title', 'Begin Bacteriophage Resistance Study on E. xenobi', 'assigned_by', 'Olga Scope')),
    (v_rex_id, 'test_proposal:approved',
     jsonb_build_object('proposal_id', v_proposal2, 'test_name', 'Phage Plaque Assay for E. xenobi', 'message', 'Your test proposal has been approved.')),
    (v_flora_id, 'test_proposal:rejected',
     jsonb_build_object('proposal_id', v_proposal3, 'test_name', 'Seed Germination Rate Under Simulated Cosmic Radiation', 'message', 'Your test proposal was rejected. See reviewer note.'));

  -- Notification for The Observer about progress reports / help requests
  INSERT INTO notifications (user_id, type, payload) VALUES
    (v_olga_id, 'progress_report:submitted',
     jsonb_build_object('task_id', v_task1, 'task_title', 'Set Up Hydroponic Bay 7', 'submitted_by', 'Flora Green', 'status', 'in_progress')),
    (v_olga_id, 'help_request:received',
     '{"title": "Request: Chemical Analysis Support", "from": "Flora Green", "requester_role": "AgriculturalEngineer"}'::jsonb),
    (v_olga_id, 'help_request:received',
     '{"title": "Request: Sequencing Queue Priority", "from": "Rex Strand", "requester_role": "BiologicalEngineer"}'::jsonb);

END $$;
