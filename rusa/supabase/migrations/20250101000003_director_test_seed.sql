-- ============================================================
-- 003b_director_test_seed.sql
-- Seed the remaining 11 director accounts (alice.vanguard = TheDirector
-- and bob.metrics = TheStatistician already exist) plus test vote sessions
-- and sample data for every Director page.
-- Password: password123  →  $2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO
-- ============================================================

-- ════════════════════════════════════════════════════════════════════════════════
-- DIRECTOR ACCOUNTS (11 remaining roles)
-- ════════════════════════════════════════════════════════════════════════════════

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'gen.cosmos', 'gencosmos@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'General Cosmos', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'GeneralDirector'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'tina.ledger', 'ledger@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Tina Ledger', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'TheAccountant'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'lina.scroll', 'scroll@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Lina Scroll', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'TheLibrarian'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'nate.drift', 'drift@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Nate Drift', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'TheNomad'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'art.forge', 'forge@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Art Forge', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'TheArtificer'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'olga.scope', 'scope@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Olga Scope', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'TheObserver'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'walt.trail', 'trail@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Walt Trail', r.id, 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
FROM roles r WHERE r.name = 'TheWanderer'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'task.iron', 'iron@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Task Iron', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'TheTaskmaster'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'gina.shield', 'shield@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Gina Shield', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'TheGuardian'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'carl.events', 'events@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Carl Events', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'TheCoordinator'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'otto.watch', 'watch@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Otto Watch', r.id, 'cccccccc-cccc-cccc-cccc-cccccccccccc'
FROM roles r WHERE r.name = 'TheOverseer'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'andy.horn', 'horn@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Andy Horn', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'TheAnchorman'
ON CONFLICT (username) DO NOTHING;

-- ════════════════════════════════════════════════════════════════════════════════
-- Also seed a few subordinate personnel for testing task assignment / relocation
-- ════════════════════════════════════════════════════════════════════════════════

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'math.euler', 'euler@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Math Euler', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'Mathematician'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'phys.newton', 'newton@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Isaac Newton Jr', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'Physicist'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'bio.darwin', 'darwin@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Bio Darwin', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'Biologist'
ON CONFLICT (username) DO NOTHING;

INSERT INTO users (username, email, password_hash, full_name, role_id, base_location_id)
SELECT 'chem.curie', 'curie@rusa.internal',
       '$2a$12$.KjQz/uVDGYqKlG0pREaO.KbovOrUA1mjsJ1lN8Jc2hMzQnZjRROO',
       'Chem Curie', r.id, 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
FROM roles r WHERE r.name = 'Chemist'
ON CONFLICT (username) DO NOTHING;

-- ════════════════════════════════════════════════════════════════════════════════
-- TEST VOTE SESSIONS  (so voting pages have data to display)
-- ════════════════════════════════════════════════════════════════════════════════

-- We'll use alice.vanguard (TheDirector) as the creator for these test sessions.
-- First, get her user ID dynamically.

DO $$
DECLARE
  v_alice_id UUID;
  v_bob_id   UUID;
  v_gen_id   UUID;
  v_session1 UUID;
  v_session2 UUID;
  v_session3 UUID;
BEGIN
  SELECT id INTO v_alice_id FROM users WHERE username = 'alice.vanguard';
  SELECT id INTO v_bob_id   FROM users WHERE username = 'bob.metrics';
  SELECT id INTO v_gen_id   FROM users WHERE username = 'gen.cosmos';

  -- Session 1: Currently open vote
  INSERT INTO vote_sessions (id, topic, context, status, opens_at, created_by)
  VALUES (
    gen_random_uuid(), 
    'Approve Budget Increase for Kepler Outpost',
    'Kepler Outpost has requested a 15% budget increase for Q3 to cover expanded research operations and additional supply runs.',
    'open',
    NOW() - INTERVAL '2 hours',
    v_alice_id
  ) RETURNING id INTO v_session1;

  -- Session 2: Another open vote
  INSERT INTO vote_sessions (id, topic, context, status, opens_at, created_by)
  VALUES (
    gen_random_uuid(),
    'Authorize Emergency Medical Supply Transfer to Alpha Station',
    'Alpha Station reports critical shortage of surgical supplies. Requires vote to divert supplies from Earth reserves.',
    'open',
    NOW() - INTERVAL '1 hour',
    v_alice_id
  ) RETURNING id INTO v_session2;

  -- Session 3: A concluded vote (decided)
  INSERT INTO vote_sessions (id, topic, context, status, opens_at, closes_at, result, total_yay, total_nay, total_abstain, created_by)
  VALUES (
    gen_random_uuid(),
    'Rename Sector 7 Territory to "Nova Haven"',
    'TheWanderer proposed renaming the territory following successful colonization milestone.',
    'decided',
    NOW() - INTERVAL '3 days',
    NOW() - INTERVAL '2 days',
    'approved',
    10, 2, 1,
    v_alice_id
  ) RETURNING id INTO v_session3;

  -- Add some vote records for session 3 (the concluded one)
  INSERT INTO vote_records (session_id, director_id, vote, reason) VALUES
    (v_session3, v_alice_id, 'yay', 'Supports settler morale and establishes identity for the colony.'),
    (v_session3, v_bob_id, 'yay', 'Data shows positive correlation between territory naming and productivity.'),
    (v_session3, v_gen_id, 'nay', 'Prefer waiting until permanent structures are complete.');

  -- Add 2 sample vote records for session 1 (the open one)
  INSERT INTO vote_records (session_id, director_id, vote, reason) VALUES
    (v_session1, v_bob_id, 'yay', 'Statistical models project positive ROI within 2 quarters.');

  -- Update tallies for session 1
  UPDATE vote_sessions SET total_yay = 1 WHERE id = v_session1;

END $$;

-- ════════════════════════════════════════════════════════════════════════════════
-- TEST TERRITORY NAMES  (for TheWanderer page)
-- ════════════════════════════════════════════════════════════════════════════════

INSERT INTO territory_names (territory_type, name)
VALUES
  ('planet', 'Kepler-22b'),
  ('sector', 'Sector 7 - Nova Haven'),
  ('sector', 'Alpha Station Perimeter'),
  ('planet', 'Mars Dome Echo')
ON CONFLICT DO NOTHING;

-- ════════════════════════════════════════════════════════════════════════════════
-- TEST REQUESTS (for financial queue / data requests)
-- ════════════════════════════════════════════════════════════════════════════════

DO $$
DECLARE
  v_alice_id UUID;
  v_bob_id   UUID;
  v_charlie_id UUID;
BEGIN
  SELECT id INTO v_alice_id   FROM users WHERE username = 'alice.vanguard';
  SELECT id INTO v_bob_id     FROM users WHERE username = 'bob.metrics';
  SELECT id INTO v_charlie_id FROM users WHERE username = 'charlie.pulse';

  -- Financial request (for TheAccountant queue)
  INSERT INTO requests (requester_id, type, title, description, status, payload)
  VALUES (
    v_charlie_id,
    'budget',
    'Medical Equipment Purchase Order #2026-003',
    'Request for 5x portable ultrasound devices for field medical teams.',
    'pending',
    '{"amount": 125000, "currency": "RUSA Credits", "category": "medical_equipment", "priority": "high"}'::jsonb
  );

  INSERT INTO requests (requester_id, type, title, description, status, payload)
  VALUES (
    v_bob_id,
    'budget',
    'Quarterly Data Center Maintenance Budget',
    'Routine maintenance allocation for Q3 data infrastructure.',
    'pending',
    '{"amount": 45000, "currency": "RUSA Credits", "category": "infrastructure", "priority": "medium"}'::jsonb
  );

  -- Data request (for TheStatistician queue)
  INSERT INTO requests (requester_id, type, title, description, status, payload, bypass_authority)
  VALUES (
    v_charlie_id,
    'data',
    'Patient Recovery Statistics - Last 6 Months',
    'Need aggregate recovery rates by treatment type for medical review board.',
    'pending',
    '{"scope": "medical", "period": "6_months", "format": "csv"}'::jsonb,
    'TheStatistician'
  );

  INSERT INTO requests (requester_id, type, title, description, status, payload, bypass_authority)
  VALUES (
    v_alice_id,
    'data',
    'Personnel Distribution by Base Location',
    'Required for upcoming Directors meeting on resource allocation.',
    'pending',
    '{"scope": "personnel", "group_by": "base_location", "format": "json"}'::jsonb,
    'TheStatistician'
  );

END $$;

-- ════════════════════════════════════════════════════════════════════════════════
-- TEST BROADCAST REQUESTS (for TheAnchorman / TheGuardian pages)
-- ════════════════════════════════════════════════════════════════════════════════

DO $$
DECLARE
  v_gina_id  UUID;
  v_alice_id UUID;
BEGIN
  SELECT id INTO v_gina_id  FROM users WHERE username = 'gina.shield';
  SELECT id INTO v_alice_id FROM users WHERE username = 'alice.vanguard';

  INSERT INTO broadcast_requests (requester_id, type, subject, content, target_scope, urgency, status)
  VALUES (
    v_gina_id,
    'security',
    'Perimeter Breach Alert - Sector 7',
    'Unidentified vessel detected approaching Sector 7 at 0300 hours. All security personnel to standby positions.',
    'company_wide',
    'critical',
    'pending'
  );

  INSERT INTO broadcast_requests (requester_id, type, subject, content, target_scope, urgency, status)
  VALUES (
    v_alice_id,
    'informational',
    'Annual Safety Drill Schedule Published',
    'The 2026 safety drill calendar is now available on the internal board. All departments please review and confirm participation.',
    'company_wide',
    'low',
    'pending'
  );

END $$;

-- ════════════════════════════════════════════════════════════════════════════════
-- TEST NOTIFICATIONS (so notification bell has data)
-- ════════════════════════════════════════════════════════════════════════════════

DO $$
DECLARE
  v_alice_id UUID;
  v_bob_id   UUID;
BEGIN
  SELECT id INTO v_alice_id FROM users WHERE username = 'alice.vanguard';
  SELECT id INTO v_bob_id   FROM users WHERE username = 'bob.metrics';

  INSERT INTO notifications (user_id, type, payload) VALUES
    (v_alice_id, 'vote:new', '{"topic": "Budget Increase for Kepler Outpost", "message": "A new vote session requires your attention."}'::jsonb),
    (v_alice_id, 'broadcast:pending', '{"subject": "Perimeter Breach Alert", "message": "A broadcast request is awaiting review."}'::jsonb),
    (v_bob_id, 'data_request:new', '{"title": "Patient Recovery Statistics", "message": "A new data request has been submitted."}'::jsonb),
    (v_bob_id, 'vote:new', '{"topic": "Emergency Medical Supply Transfer", "message": "A new vote session requires your attention."}'::jsonb);

END $$;
