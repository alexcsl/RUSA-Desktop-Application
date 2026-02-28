// stores/engineers.ts — Frontend API for Engineers subsystem
// Source of truth: 01_ENGINEERS.md
// All business logic lives in Rust — this file only mirrors the Tauri command interface.

import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────────────────────

export interface EngineerTask {
  id: string;
  title: string;
  description: string | null;
  task_type: string;
  status: string;
  assigned_by: string;
  assigner_name: string;
  due_date: string | null;
  payload: Record<string, unknown>;
  created_at: string;
}

export interface SubmitProgressReportPayload {
  task_id: string;
  current_status: string;
  work_completed: string;
  problems_encountered?: string;
  plans_next?: string;
}

export interface ProgressReport {
  id: string;
  task_id: string;
  submitted_by: string;
  submitter_name: string;
  report_date: string;
  current_status: string;
  work_completed: string;
  problems_encountered: string | null;
  plans_next: string | null;
  created_at: string;
}

export interface EngHelpRequestPayload {
  title: string;
  description: string;
  target_department?: string;
  urgency?: string;
}

export interface HelpRequestItem {
  id: string;
  title: string;
  description: string | null;
  status: string;
  payload: Record<string, unknown>;
  created_at: string;
}

export interface ApprovedTest {
  id: string;
  name: string;
  procedure: string;
  category: string;
  applicable_scope: string;
  accepted_at: string | null;
  created_at: string;
}

export interface ProposeTestPayload {
  name: string;
  goal: string;
  procedure?: string;
  species_scope?: string;
  category: string[];
  apparatuses?: string;
  required_data?: string;
  justification?: string;
}

export interface TestProposal {
  id: string;
  proposal_data: Record<string, unknown>;
  status: string;
  reviewer_note: string | null;
  created_at: string;
}

export interface LogDailyPayload {
  experiment_id: string;
  log_date: string;
  rag_status?: string;
  completed_actions?: string;
  pending_actions?: string;
  collected_data?: Record<string, unknown>;
  attendee_ids?: string[];
  test_ids?: string[];
  species_ids?: string[];
}

export interface ConclusionPayload {
  experiment_id: string;
  summary: string;
  final_outcomes?: string;
}

export interface SpeciesArchiveItem {
  id: string;
  archive_type: string;
  name: string;
  classification: string | null;
  detail: Record<string, unknown>;
  experiment_id: string | null;
  created_at: string;
}

export interface ExperimentSummary {
  id: string;
  title: string;
  experiment_type: string;
  status: string;
  proposed_by: string;
  proposer_name: string;
  metadata: Record<string, unknown>;
  created_at: string;
  approved_at: string | null;
  closed_at: string | null;
}

export interface ExperimentLogSummary {
  id: string;
  experiment_id: string;
  log_date: string;
  rag_status: string | null;
  completed_actions: string | null;
  pending_actions: string | null;
  collected_data: Record<string, unknown> | null;
  created_at: string;
}

// ── UC-GE-01: Task Inbox ─────────────────────────────────────────────────────

/** Get tasks assigned to the current engineer. */
export async function getMyTasks(): Promise<EngineerTask[]> {
  return invoke<EngineerTask[]>('eng_get_my_tasks');
}

// ── UC-GE-02: Progress Report ────────────────────────────────────────────────

/** Submit a progress report linked to a task. */
export async function submitProgressReport(
  payload: SubmitProgressReportPayload,
): Promise<string> {
  return invoke<string>('eng_submit_progress_report', { payload });
}

/** Get progress reports for a task. */
export async function getProgressReports(taskId: string): Promise<ProgressReport[]> {
  return invoke<ProgressReport[]>('eng_get_progress_reports', { taskId });
}

// ── UC-GE-03: Cross-Department Help Request ──────────────────────────────────

/** Submit a cross-department help request (routes to The Observer). */
export async function submitHelpRequest(
  payload: EngHelpRequestPayload,
): Promise<string> {
  return invoke<string>('eng_submit_help_request', { payload });
}

/** Get help requests submitted by the current engineer. */
export async function getMyHelpRequests(): Promise<HelpRequestItem[]> {
  return invoke<HelpRequestItem[]>('eng_get_my_help_requests');
}

// ── UC-AGE-01 / UC-BE-01: Approved Tests ─────────────────────────────────────

/** Get approved tests for the engineer's scope. */
export async function getApprovedTests(scope?: string): Promise<ApprovedTest[]> {
  return invoke<ApprovedTest[]>('eng_get_approved_tests', { scope });
}

// ── UC-AGE-02 / UC-BE-03: Propose New Test ───────────────────────────────────

/** Propose a new test (routes to The Observer for approval). */
export async function proposeNewTest(
  payload: ProposeTestPayload,
): Promise<string> {
  return invoke<string>('eng_propose_new_test', { payload });
}

/** Get test proposals submitted by the current engineer. */
export async function getMyTestProposals(): Promise<TestProposal[]> {
  return invoke<TestProposal[]>('eng_get_my_test_proposals');
}

// ── UC-AGE-03 / UC-BE-04: Log Daily Experiment ──────────────────────────────

/** Log a daily experiment session. */
export async function logDailyExperiment(
  payload: LogDailyPayload,
): Promise<string> {
  return invoke<string>('eng_log_daily_experiment', { payload });
}

// ── UC-AGE-04 / UC-BE-05: Submit Experiment Conclusion ──────────────────────

/** Submit an experiment conclusion request (routes to The Taskmaster). */
export async function submitExperimentConclusion(
  payload: ConclusionPayload,
): Promise<string> {
  return invoke<string>('eng_submit_experiment_conclusion', { payload });
}

// ── UC-AGE-05 / UC-BE-01: Species Archive ───────────────────────────────────

/** Get species archive entries (filtered by role scope). */
export async function getSpeciesArchive(
  search?: string,
): Promise<SpeciesArchiveItem[]> {
  return invoke<SpeciesArchiveItem[]>('eng_get_species_archive', { search });
}

// ── UC-AGE-06 / UC-BE-02: Experiment Archive ────────────────────────────────

/** Get experiment archive filtered by engineer discipline. */
export async function getExperimentArchive(
  statusFilter?: string,
): Promise<ExperimentSummary[]> {
  return invoke<ExperimentSummary[]>('eng_get_experiment_archive', { statusFilter });
}

/** Get experiment detail including daily logs. */
export async function getExperimentDetail(
  experimentId: string,
): Promise<[ExperimentSummary, ExperimentLogSummary[]]> {
  return invoke<[ExperimentSummary, ExperimentLogSummary[]]>(
    'eng_get_experiment_detail',
    { experimentId },
  );
}
