// stores/security.ts — Frontend API for Galactic Security Teams subsystem
// Source of truth: 03_SECURITY_TEAMS.md
// All business logic lives in Rust — this file only mirrors the Tauri command interface.

import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────────────────────

export interface IncidentReportSummary {
  id: string;
  reported_by: string;
  reporter_name: string;
  source: string;
  incident_type: string;
  location: string;
  occurred_at: string | null;
  description: string;
  severity: string;
  recommended_action: string | null;
  sector_or_base: string | null;
  assigned_to: string | null;
  assigned_to_name: string | null;
  related_incident_ids: string[] | null;
  incident_meta: Record<string, unknown> | null;
  created_at: string;
}

export interface DailySecurityReportSummary {
  id: string;
  submitted_by: string;
  submitter_name: string;
  report_date: string;
  findings_summary: string;
  risk_notes: string | null;
  recommended_actions: string | null;
  delivered_to_guardian_at: string | null;
  created_at: string;
}

export interface SecBroadcastRequestSummary {
  id: string;
  requester_id: string;
  requester_name: string;
  type_: string;
  subject: string;
  content: string;
  target_scope: string;
  urgency: string | null;
  rationale: string | null;
  status: string;
  created_at: string;
}

export interface SecMessageSummary {
  id: string;
  from_id: string;
  from_name: string;
  subject: string;
  body: string;
  channel: string;
  recalled_at: string | null;
  created_at: string;
}

export interface SecurityPersonnelItem {
  id: string;
  full_name: string;
  role_name: string;
}

// ── Incident Reports (UC-SH-02, UC-SS-01) ────────────────────────────────────

export async function createIncidentReport(payload: {
  source: string;
  incident_type: string;
  location: string;
  occurred_at?: string;
  description: string;
  severity: string;
  recommended_action?: string;
  sector_or_base?: string;
  related_incident_ids?: string[];
  incident_meta?: Record<string, unknown>;
}): Promise<string> {
  return invoke<string>('sec_create_incident_report', { payload });
}

export async function getIncidentArchive(): Promise<IncidentReportSummary[]> {
  return invoke<IncidentReportSummary[]>('sec_get_incident_archive');
}

// ── Staff Assignment (UC-SH-03) ──────────────────────────────────────────────

export async function assignStaffToIncident(payload: {
  incident_id: string;
  user_id: string;
}): Promise<void> {
  await invoke('sec_assign_staff_to_incident', { payload });
}

// ── Daily Report (UC-SH-05) ──────────────────────────────────────────────────

export async function submitDailyReport(payload: {
  report_date: string;
  findings_summary: string;
  risk_notes?: string;
  recommended_actions?: string;
}): Promise<string> {
  return invoke<string>('sec_submit_daily_report', { payload });
}

export async function getMyDailyReports(): Promise<DailySecurityReportSummary[]> {
  return invoke<DailySecurityReportSummary[]>('sec_get_my_daily_reports');
}

// ── Broadcast Request (UC-SH-01) ─────────────────────────────────────────────

export async function secSubmitBroadcastRequest(payload: {
  subject: string;
  content: string;
  urgency?: string;
  rationale?: string;
}): Promise<string> {
  return invoke<string>('sec_submit_broadcast_request', { payload });
}

export async function getMyBroadcastRequests(): Promise<SecBroadcastRequestSummary[]> {
  return invoke<SecBroadcastRequestSummary[]>('sec_get_my_broadcast_requests');
}

// ── Security Messaging (UC-SH-04, UC-SS-02) ─────────────────────────────────

export async function secSendSecurityMessage(payload: {
  subject: string;
  body: string;
  recipients_to: string[];
  recipients_cc?: string[];
  recipients_bcc?: string[];
}): Promise<string> {
  return invoke<string>('sec_send_security_message', { payload });
}

export async function secGetSecurityMessages(): Promise<SecMessageSummary[]> {
  return invoke<SecMessageSummary[]>('sec_get_security_messages');
}

// ── Security Personnel ───────────────────────────────────────────────────────

export async function getSecurityPersonnel(): Promise<SecurityPersonnelItem[]> {
  return invoke<SecurityPersonnelItem[]>('sec_get_security_personnel');
}
