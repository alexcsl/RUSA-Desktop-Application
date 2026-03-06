-- Seed: Security Teams test data
-- Creates GalacticSecurityStaff users and sample incident/daily reports
-- Requires: 20250101000014_security_teams.sql

-- ── GalacticSecurityStaff seed users ──────────────────────────────────────────
-- Get the GalacticSecurityStaff role ID
DO $$
DECLARE
  v_staff_role_id UUID;
  v_head_role_id UUID;
  v_head_user_id UUID;
  v_staff1_id UUID;
  v_staff2_id UUID;
  v_base_id UUID;
BEGIN
  SELECT id INTO v_staff_role_id FROM roles WHERE name = 'GalacticSecurityStaff';
  SELECT id INTO v_head_role_id FROM roles WHERE name = 'GalacticSecurityHead';

  -- Find the existing head
  SELECT u.id INTO v_head_user_id FROM users u JOIN roles r ON r.id = u.role_id WHERE r.name = 'GalacticSecurityHead' LIMIT 1;

  -- Get a base location (same as head's or first available)
  SELECT base_location_id INTO v_base_id FROM users WHERE id = v_head_user_id;
  IF v_base_id IS NULL THEN
    SELECT id INTO v_base_id FROM locations LIMIT 1;
  END IF;

  -- Insert GalacticSecurityStaff users if the role exists and we don't already have staff
  IF v_staff_role_id IS NOT NULL AND NOT EXISTS (
    SELECT 1 FROM users u JOIN roles r ON r.id = u.role_id WHERE r.name = 'GalacticSecurityStaff'
  ) THEN
    -- password: 'Test1234!' hashed with bcrypt cost 12
    INSERT INTO users (id, full_name, username, password_hash, role_id, base_location_id, is_active)
    VALUES (gen_random_uuid(), 'Sierra Guard', 'sierra.guard', '$2a$12$LJ3m4ys3LzH5v7W6xKQp4ejKf2FcYQvMkG8jqF7Z1x9h5n3v4OxoW', v_staff_role_id, v_base_id, true);

    INSERT INTO users (id, full_name, username, password_hash, role_id, base_location_id, is_active)
    VALUES (gen_random_uuid(), 'Sam Sentinel', 'sam.sentinel', '$2a$12$LJ3m4ys3LzH5v7W6xKQp4ejKf2FcYQvMkG8jqF7Z1x9h5n3v4OxoW', v_staff_role_id, v_base_id, true);

    SELECT id INTO v_staff1_id FROM users WHERE username = 'sierra.guard';
    SELECT id INTO v_staff2_id FROM users WHERE username = 'sam.sentinel';

    -- ── Sample incident reports ───────────────────────────────────────────────
    INSERT INTO incident_reports (reported_by, source, incident_type, location, occurred_at, description, severity, recommended_action, sector_or_base, incident_meta)
    VALUES
      (v_head_user_id, 'direct_observation', 'Unauthorized Access', 'Sector Alpha — Docking Bay 3', NOW() - INTERVAL '2 hours', 'Unidentified individual observed attempting to bypass airlock security protocols at Docking Bay 3. Individual was apprehended by on-duty staff.', 'high', 'Increase patrol frequency at all docking bays. Review and update access credentials.', 'Sector Alpha',
       '{"compromised_party": "Docking Bay 3 Access Control", "category": ["unauthorized_access", "physical_security"], "response_procedure": "Lockdown docking bay, apprehend individual, review access logs", "incident_start": "2025-01-15T10:30:00Z", "incident_end": "2025-01-15T11:15:00Z"}'::jsonb),
      (v_staff1_id, 'external_report', 'Equipment Malfunction', 'Sector Beta — Lab Wing 7', NOW() - INTERVAL '5 hours', 'Security camera array in Lab Wing 7 reported offline by maintenance crew. Possible tampering detected based on diagnostic logs.', 'medium', 'Deploy mobile surveillance unit. Forensic analysis of camera firmware.', 'Sector Beta',
       '{"compromised_party": "Surveillance System Lab Wing 7", "category": ["equipment_malfunction", "surveillance"], "response_procedure": "Deploy backup cameras, investigate firmware integrity"}'::jsonb),
      (v_staff2_id, 'assignment', 'Perimeter Breach', 'Outer Ring — Segment 14', NOW() - INTERVAL '1 day', 'Routine perimeter scan detected micro-fractures in hull segment 14 outer ring. Could be natural wear or deliberate sabotage.', 'critical', 'Immediate structural assessment. Quarantine segment 14. Deploy engineering team.', 'Outer Ring',
       '{"compromised_party": "Hull Segment 14", "category": ["perimeter_breach", "structural_integrity"], "response_procedure": "Quarantine segment, deploy repair team, investigate cause", "incident_start": "2025-01-14T08:00:00Z"}'::jsonb);

    -- ── Sample daily security reports ─────────────────────────────────────────
    INSERT INTO daily_security_reports (submitted_by, report_date, findings_summary, risk_notes, recommended_actions)
    VALUES
      (v_head_user_id, CURRENT_DATE - 1, 'Routine patrols completed across all sectors. One unauthorized access attempt at Docking Bay 3 (see incident IR-001). All other sectors report nominal status. Night shift changeover completed without issues.', 'Elevated risk at docking facilities due to repeated access attempts this week. Camera outage in Lab Wing 7 reduces surveillance coverage.', 'Increase docking bay patrols to 15-minute intervals. Expedite camera repair in Lab Wing 7. Schedule security briefing for all staff.'),
      (v_head_user_id, CURRENT_DATE, 'Morning sweep of all sectors complete. Perimeter breach investigation ongoing at Segment 14. All personnel accounted for. Communication systems operational.', 'Critical risk at Outer Ring Segment 14 — structural integrity assessment pending. Recommend elevated alert status until engineering report received.', 'Maintain quarantine on Segment 14. Continue monitoring hull integrity sensors. Brief Guardian on engineering assessment timeline.');
  END IF;
END $$;
