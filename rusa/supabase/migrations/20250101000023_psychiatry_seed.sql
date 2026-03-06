-- Seed data: Psychiatry Division (Sub-07)
-- Source of truth: 07_PSYCHIATRY.md
-- Creates: 1 Psychiatrist, 1 Assistant, 2 patients (using existing users as patients), schedule slots, recovery logs

-- ── Psychiatrist user ──────────────────────────────────────────────────────────
INSERT INTO users (id, full_name, username, email, password_hash, role_id, base_location_id)
VALUES (
    'a1b2c3d4-0001-4000-8000-000000000001',
    'Dr. Elara Voss',
    'elara.voss',
    'elara.voss@rusa.org',
    '$2b$12$LJ3m4ys3Lk8FnQpVfE0ZaeJdK1w3cY0sOmDtx5F4FvX5vR1gKjQ2C',  -- Test1234!
    '282e9319-fab7-42f1-ba0b-5566756dedf0',  -- Psychiatrist
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'    -- RUSA HQ (has_data_regulation = true)
)
ON CONFLICT (username) DO NOTHING;

-- ── Psychiatrist Assistant user ────────────────────────────────────────────────
INSERT INTO users (id, full_name, username, email, password_hash, role_id, base_location_id)
VALUES (
    'a1b2c3d4-0002-4000-8000-000000000002',
    'Nurse Kael Thorn',
    'kael.thorn',
    'kael.thorn@rusa.org',
    '$2b$12$LJ3m4ys3Lk8FnQpVfE0ZaeJdK1w3cY0sOmDtx5F4FvX5vR1gKjQ2C',  -- Test1234!
    '70594bcb-d263-4225-87e6-316dbd70dd93',  -- PsychiatristAssistant
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'    -- RUSA HQ
)
ON CONFLICT (username) DO NOTHING;

-- ── Second Psychiatrist (at non-GDPR location) ────────────────────────────────
INSERT INTO users (id, full_name, username, email, password_hash, role_id, base_location_id)
VALUES (
    'a1b2c3d4-0003-4000-8000-000000000003',
    'Dr. Mira Solaris',
    'mira.solaris',
    'mira.solaris@rusa.org',
    '$2b$12$LJ3m4ys3Lk8FnQpVfE0ZaeJdK1w3cY0sOmDtx5F4FvX5vR1gKjQ2C',  -- Test1234!
    '282e9319-fab7-42f1-ba0b-5566756dedf0',  -- Psychiatrist
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'    -- Kepler Outpost (has_data_regulation = false)
)
ON CONFLICT (username) DO NOTHING;

-- ── Patient records (using existing users as patients) ─────────────────────────
-- Patient 1: Nova Stardust (SpaceStationSettler) under Dr. Elara Voss
INSERT INTO psychiatric_patients (id, user_id, psychiatrist_id, patient_profile, initial_notes)
VALUES (
    'b1000001-0001-4000-8000-000000000001',
    (SELECT id FROM users WHERE username = 'nova.stardust' LIMIT 1),
    'a1b2c3d4-0001-4000-8000-000000000001',
    '{
        "full_name": "Nova Stardust",
        "demographics": {"dob": "2000-05-15", "gender": "Female", "nationality": "Terran"},
        "emergency_contact": {"name": "Rex Beacon", "relation": "Crew Partner", "phone": "+1-555-0101"},
        "allergies": ["None known"],
        "medications": ["Melatonin 3mg nightly"],
        "history_of_illnesses": "Mild insomnia post-deployment. No prior psychiatric history."
    }',
    'Referred for post-deployment adjustment counseling. Reports difficulty sleeping and increased anxiety in confined station environments.'
)
ON CONFLICT (user_id, psychiatrist_id) DO NOTHING;

-- Patient 2: Rex Beacon under Dr. Elara Voss
INSERT INTO psychiatric_patients (id, user_id, psychiatrist_id, patient_profile, initial_notes)
VALUES (
    'b1000001-0002-4000-8000-000000000002',
    (SELECT id FROM users WHERE username = 'rex.beacon' LIMIT 1),
    'a1b2c3d4-0001-4000-8000-000000000001',
    '{
        "full_name": "Rex Beacon",
        "demographics": {"dob": "1995-11-22", "gender": "Male", "nationality": "Terran"},
        "emergency_contact": {"name": "Nova Stardust", "relation": "Crew Partner", "phone": "+1-555-0102"},
        "allergies": ["Penicillin"],
        "medications": [],
        "history_of_illnesses": "Minor depressive episode 3 years ago, resolved with therapy."
    }',
    'Self-referred. Reports feelings of isolation and homesickness. No active medication.'
)
ON CONFLICT (user_id, psychiatrist_id) DO NOTHING;

-- ── Patient Index entries ──────────────────────────────────────────────────────
INSERT INTO patient_index (user_id, department, care_status, psychiatrist_id)
SELECT u.id, 'psychiatry', 'active', 'a1b2c3d4-0001-4000-8000-000000000001'
FROM users u WHERE u.username = 'nova.stardust'
ON CONFLICT (user_id) DO NOTHING;

INSERT INTO patient_index (user_id, department, care_status, psychiatrist_id)
SELECT u.id, 'psychiatry', 'active', 'a1b2c3d4-0001-4000-8000-000000000001'
FROM users u WHERE u.username = 'rex.beacon'
ON CONFLICT (user_id) DO NOTHING;

-- ── Appointment records ────────────────────────────────────────────────────────
INSERT INTO psychiatric_appointments (id, patient_id, psychiatrist_id, scheduled_at, status, booked_by, appointment_log)
VALUES
(
    'c1000001-0001-4000-8000-000000000001',
    'b1000001-0001-4000-8000-000000000001',
    'a1b2c3d4-0001-4000-8000-000000000001',
    '2026-02-20 10:00:00+00',
    'completed',
    'a1b2c3d4-0002-4000-8000-000000000002',  -- booked by Kael (assistant)
    '{
        "complaint": "Difficulty falling asleep. Persistent anxiety about station integrity.",
        "findings": "Patient exhibits moderate generalized anxiety. Sleep hygiene review needed.",
        "resolution": "Prescribed relaxation techniques and melatonin continuation. Follow-up in 2 weeks.",
        "wellbeing_assessment": "Moderate — stable but needs monitoring"
    }'
),
(
    'c1000001-0002-4000-8000-000000000002',
    'b1000001-0002-4000-8000-000000000002',
    'a1b2c3d4-0001-4000-8000-000000000001',
    '2026-02-22 14:00:00+00',
    'completed',
    'a1b2c3d4-0001-4000-8000-000000000001',  -- booked by doctor
    '{
        "complaint": "Feeling disconnected from colleagues. Low motivation.",
        "findings": "Mild subclinical depression. No suicidal ideation. Social integration deficit.",
        "resolution": "Recommended group activity participation. Scheduled follow-up in 1 week.",
        "wellbeing_assessment": "Mild — improving trajectory"
    }'
),
(
    'c1000001-0003-4000-8000-000000000003',
    'b1000001-0001-4000-8000-000000000001',
    'a1b2c3d4-0001-4000-8000-000000000001',
    '2026-03-05 10:00:00+00',
    'scheduled',
    'a1b2c3d4-0002-4000-8000-000000000002',
    NULL  -- upcoming, no log yet
);

-- ── Recovery logs ──────────────────────────────────────────────────────────────
INSERT INTO psychiatric_recovery_logs (id, patient_id, milestone, status, details, logged_by)
VALUES
(
    'd1000001-0001-4000-8000-000000000001',
    'b1000001-0001-4000-8000-000000000001',
    'Initial Assessment Complete',
    'achieved',
    'Baseline anxiety and sleep patterns documented. Treatment plan established with relaxation techniques and melatonin.',
    'a1b2c3d4-0001-4000-8000-000000000001'
),
(
    'd1000001-0002-4000-8000-000000000002',
    'b1000001-0001-4000-8000-000000000001',
    'Sleep Pattern Normalization',
    'in_progress',
    'Patient reports improved sleep 4 out of 7 nights. Continuing current regimen. Target: 6/7 nights within 2 weeks.',
    'a1b2c3d4-0001-4000-8000-000000000001'
),
(
    'd1000001-0003-4000-8000-000000000003',
    'b1000001-0002-4000-8000-000000000002',
    'Social Reintegration',
    'in_progress',
    'Patient participating in 2 group activities per week. Mood improving. Homesickness reducing.',
    'a1b2c3d4-0001-4000-8000-000000000001'
);

-- ── Psychiatrist schedule slots ────────────────────────────────────────────────
INSERT INTO psychiatrist_schedule (id, psychiatrist_id, slot_start, slot_end, is_available)
VALUES
('e1000001-0001-4000-8000-000000000001', 'a1b2c3d4-0001-4000-8000-000000000001', '2026-03-03 09:00:00+00', '2026-03-03 10:00:00+00', true),
('e1000001-0002-4000-8000-000000000002', 'a1b2c3d4-0001-4000-8000-000000000001', '2026-03-03 10:00:00+00', '2026-03-03 11:00:00+00', true),
('e1000001-0003-4000-8000-000000000003', 'a1b2c3d4-0001-4000-8000-000000000001', '2026-03-03 11:00:00+00', '2026-03-03 12:00:00+00', false),
('e1000001-0004-4000-8000-000000000004', 'a1b2c3d4-0001-4000-8000-000000000001', '2026-03-03 14:00:00+00', '2026-03-03 15:00:00+00', true),
('e1000001-0005-4000-8000-000000000005', 'a1b2c3d4-0001-4000-8000-000000000001', '2026-03-04 09:00:00+00', '2026-03-04 10:00:00+00', true),
('e1000001-0006-4000-8000-000000000006', 'a1b2c3d4-0001-4000-8000-000000000001', '2026-03-04 10:00:00+00', '2026-03-04 11:00:00+00', true);

-- ── Schedule access grant (kael.thorn has permission to see nova.stardust's schedule) ──
INSERT INTO patient_schedule_access (patient_user_id, assistant_id, granted)
SELECT u.id, 'a1b2c3d4-0002-4000-8000-000000000002', true
FROM users u WHERE u.username = 'nova.stardust'
ON CONFLICT (patient_user_id, assistant_id) DO NOTHING;
