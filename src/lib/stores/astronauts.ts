// stores/astronauts.ts — Frontend API for Astronauts subsystem
// All business logic lives in Rust — this file only mirrors Tauri command interfaces.
// Source of truth: 05_ASTRONAUTS.md

import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────────────────────

export interface MissionSummary {
  id: string;
  title: string;
  mission_type: string; // 'interstellar' | 'terrain'
  danger_level: string | null;
  location: string;
  status: string; // 'active' | 'completion_requested' | 'completed' | 'rejected'
  assigned_by_name: string;
  created_at: string;
}

export interface MissionDocument {
  id: string;
  title: string;
  mission_type: string;
  danger_level: string | null;
  location: string;
  mission_objective: string | null;
  procedures: string | null;
  known_dangers: string | null;
  status: string;
  assigned_by: string;
  assigned_by_name: string;
  created_at: string;
}

export interface TeamMemberCounter {
  astronaut_id: string;
  full_name: string;
  interstellar_count: number;
  terrain_count: number;
}

export interface StatusReportItem {
  id: string;
  mission_id: string;
  submitted_by: string;
  submitter_name: string;
  report_date: string;
  month_tracker: string | null;
  rag_status: string | null; // 'red' | 'amber' | 'green'
  current_status: string;
  issues_blockers: string | null;
  collected_samples_last_month: string | null;
  progress_last_month: string | null;
  plans_next_month: string | null;
  created_at: string;
}

export interface CompletionRequestItem {
  id: string;
  mission_id: string;
  submitted_by: string;
  findings_summary: string;
  evidence_storage_paths: string[] | null;
  status: string; // 'pending_wanderer' | 'pending_taskmaster' | 'approved' | 'rejected'
  wanderer_note: string | null;
  taskmaster_note: string | null;
  created_at: string;
}

export interface JournalEntry {
  mission_id: string;
  title: string;
  mission_type: string;
  location: string;
  danger_level: string | null;
  completed_at: string;
}

export interface JournalSummary {
  interstellar_count: number;
  terrain_count: number;
  missions: JournalEntry[];
}

export interface MissionDetailBundle {
  mission: MissionDocument;
  team: TeamMemberCounter[];
  status_reports: StatusReportItem[];
  completion_request: CompletionRequestItem | null;
}

export interface CompletionRequestSummary {
  id: string;
  mission_id: string;
  mission_title: string;
  submitted_by: string;
  submitter_name: string;
  findings_summary: string;
  evidence_storage_paths: string[] | null;
  status: string;
  wanderer_note: string | null;
  taskmaster_note: string | null;
  created_at: string;
}

export interface EvidenceFileUrl {
  storage_path: string;
  signed_url: string;
  filename: string;
}

// ── Astronaut Commands ────────────────────────────────────────────────────────

/** UC-AS-01: Get the current astronaut's mission inbox. */
export async function getMyMissions(): Promise<MissionSummary[]> {
  return invoke<MissionSummary[]>('ast_get_my_missions');
}

/** UC-AS-02 + UC-AS-06: Get full mission detail including team counters, reports, and completion status. */
export async function getMissionDetail(missionId: string): Promise<MissionDetailBundle> {
  return invoke<MissionDetailBundle>('ast_get_mission_detail', { missionId });
}

/** UC-AS-03: Submit a mission status report. */
export async function submitStatusReport(payload: {
  mission_id: string;
  report_date: string;
  month_tracker?: string;
  rag_status?: string;
  current_status: string;
  issues_blockers?: string;
  collected_samples_last_month?: string;
  progress_last_month?: string;
  plans_next_month?: string;
}): Promise<string> {
  return invoke<string>('ast_submit_status_report', { payload });
}

/** UC-AS-04: Submit a mission completion request with evidence files. */
export async function submitCompletionRequest(
  payload: {
    mission_id: string;
    findings_summary: string;
    file_names: string[];
    content_types: string[];
  },
  fileBytesList: number[][],
): Promise<string> {
  return invoke<string>('ast_submit_completion_request', { payload, fileBytesList });
}

/** UC-AS-05: Get personal journal (completed missions + counters). */
export async function getPersonalJournal(): Promise<JournalSummary> {
  return invoke<JournalSummary>('ast_get_personal_journal');
}

/** UC-AS-06: Get colleague counters for a specific mission. */
export async function getColleagueCounters(missionId: string): Promise<TeamMemberCounter[]> {
  return invoke<TeamMemberCounter[]>('ast_get_colleague_counters', { missionId });
}

/** Get signed URLs for evidence files on a completion request. */
export async function getEvidenceUrls(requestId: string): Promise<EvidenceFileUrl[]> {
  return invoke<EvidenceFileUrl[]>('ast_get_evidence_urls', { requestId });
}

// ── Director-side Commands (Wanderer / Taskmaster) ────────────────────────────

/** UC-WAN-01: Assign a new mission (Wanderer/Taskmaster). */
export async function assignMission(payload: {
  title: string;
  type: string;
  danger_level?: string;
  location: string;
  mission_objective?: string;
  procedures?: string;
  known_dangers?: string;
  astronaut_ids: string[];
}): Promise<string> {
  return invoke<string>('ast_assign_mission', { payload });
}

/** Get all missions — Wanderer / Taskmaster overview. */
export async function getAllMissions(): Promise<MissionSummary[]> {
  return invoke<MissionSummary[]>('ast_get_all_missions');
}

/** UC-WAN-02: Process completion request (Wanderer). */
export async function processCompletionRequest(payload: {
  request_id: string;
  decision: 'forward' | 'reject';
  note?: string;
}): Promise<void> {
  await invoke('ast_process_completion_request', { payload });
}

/** UC-TM-03 variant: Taskmaster decides on forwarded completion request. */
export async function taskmasterDecideCompletion(payload: {
  request_id: string;
  decision: 'approved' | 'rejected';
  note?: string;
}): Promise<void> {
  await invoke('ast_taskmaster_decide_completion', { payload });
}

/** UC-WAN-03: Get all mission status reports (Wanderer feed). */
export async function getAllStatusReports(): Promise<StatusReportItem[]> {
  return invoke<StatusReportItem[]>('ast_get_all_status_reports');
}

/** UC-WAN-02: Get completion requests for Wanderer queue. */
export async function getCompletionRequestsWanderer(): Promise<CompletionRequestSummary[]> {
  return invoke<CompletionRequestSummary[]>('ast_get_completion_requests_wanderer');
}

/** Get completion requests for Taskmaster queue. */
export async function getCompletionRequestsTaskmaster(): Promise<CompletionRequestSummary[]> {
  return invoke<CompletionRequestSummary[]>('ast_get_completion_requests_taskmaster');
}
