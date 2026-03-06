// stores/scientists.ts — Frontend API for Scientists subsystem
// Source of truth: 04_SCIENTISTS.md
// All business logic lives in Rust — this file only mirrors the Tauri command interface.

import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────────────────────

export interface ScientistTask {
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

export interface SubmitHelpRequestPayload {
  title: string;
  description: string;
  urgency?: string;
  calculations_area?: string;
}

export interface SubmitMathResultsPayload {
  task_id: string;
  content_latex: string;
  workings?: string;
  calculations_area?: string;
}

export interface ProposeExperimentPayload {
  title: string;
  experiment_type: string;
  introduction?: string;
  problem_statement?: string;
  research_questions?: string;
  hypotheses?: string;
  methodology?: string;
  expected_outcomes?: string;
  location?: string;
  related_objects?: string[];
}

export interface LogExperimentSessionPayload {
  experiment_id: string;
  log_date: string;
  rag_status?: string;
  completed_actions?: string;
  pending_actions?: string;
  collected_data?: Record<string, unknown>;
  attendee_ids?: string[];
  test_ids?: string[];
}

export interface SubmitExperimentConclusionPayload {
  experiment_id: string;
  reason: string;
}

export interface SubmitFinalDocumentPayload {
  experiment_id: string;
  doc_type: string;
  document_data: Record<string, unknown>;
}

export interface ProposeNewTestPayload {
  name: string;
  goal: string;
  procedure?: string;
  species_scope?: string;
  category: string | string[];
  apparatuses?: string | string[];
  required_data?: string;
  justification?: string;
}

export interface ProposeNewSpeciesPayload {
  title: string;
  introduction?: string;
  problem_statement?: string;
  research_questions?: string;
  hypotheses?: string;
  location?: string;
  common_name?: string;
  scientific_name?: string;
  classification?: string;
  habitat?: string;
  diet?: string;
  description?: string;
  estimated_population?: number;
  threat_level?: string;
}

export interface ArchiveItem {
  id: string;
  archive_type: string;
  name: string;
  classification: string | null;
  detail: Record<string, unknown>;
  storage_path: string | null;
  experiment_id: string | null;
  created_at: string;
}

export interface ApprovedTest {
  id: string;
  name: string;
  procedure: string;
  category: string;
  applicable_scope: string;
  species_scope?: string;
  accepted_at: string | null;
  approved_at?: string | null;
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

export interface MathResult {
  id: string;
  task_id: string;
  submitted_by: string;
  content: Record<string, unknown>;
  pdf_storage_path: string | null;
  created_at: string;
}

export interface FinalDocSummary {
  id: string;
  experiment_id: string;
  doc_type: string;
  document_data: Record<string, unknown>;
  status: string;
  submitted_by: string;
  created_at: string;
}

export interface TestProposal {
  id: string;
  name: string;
  proposal_data: Record<string, unknown>;
  status: string;
  reviewer_note: string | null;
  created_at: string;
}

// ── UC-GS-01: Task Inbox ─────────────────────────────────────────────────────

/** Get tasks assigned to the current scientist. */
export async function getMyTasks(): Promise<ScientistTask[]> {
  return invoke<ScientistTask[]>('get_my_tasks');
}

// ── UC-GS-02: Cross-Department Help Request ──────────────────────────────────

/** Submit a cross-department help request (routes to proxy Director). */
export async function submitHelpRequest(
  payload: SubmitHelpRequestPayload,
): Promise<string> {
  return invoke<string>('submit_help_request', { payload });
}

// ── UC-MA-01: Submit Math Results ────────────────────────────────────────────

/** Submit calculation results with LaTeX content for an assigned task. */
export async function submitMathResults(
  payload: SubmitMathResultsPayload,
): Promise<string> {
  return invoke<string>('submit_math_results', { payload });
}

// ── UC-PH-01 / UC-CH-04 / UC-BIO-02: Propose Experiment ─────────────────────

/** Propose a new experiment. Enters approval workflow. */
export async function proposeExperiment(
  payload: ProposeExperimentPayload,
): Promise<string> {
  return invoke<string>('propose_experiment', { payload });
}

// ── UC-PH-02 / UC-CH-05 / UC-BIO-03: Log Experiment Session ─────────────────

/** Log a daily experiment / observation session. */
export async function logExperimentSession(
  payload: LogExperimentSessionPayload,
): Promise<string> {
  return invoke<string>('log_experiment_session', { payload });
}

// ── UC-PH-07 / UC-CH-06 / UC-BIO-07: Submit Experiment Conclusion ───────────

/** Submit an experiment conclusion request (routes to The Taskmaster). */
export async function submitExperimentConclusion(
  payload: SubmitExperimentConclusionPayload,
): Promise<string> {
  return invoke<string>('submit_experiment_conclusion', { payload });
}

// ── UC-PH-08 / UC-CH-07 / UC-BIO-04: Submit Final Document ──────────────────

/** Submit a final object/matter/species document post-experiment. */
export async function submitFinalDocument(
  payload: SubmitFinalDocumentPayload,
): Promise<string> {
  return invoke<string>('submit_final_document', { payload });
}

// ── Archives ─────────────────────────────────────────────────────────────────

/** Get science archive entries filtered by type (matter, physical_object, species). */
export async function getArchive(archiveType: string): Promise<ArchiveItem[]> {
  return invoke<ArchiveItem[]>('get_archive', { archiveType });
}

/** Get experiment archive, optionally filtered by experiment type. */
export async function getExperimentArchive(
  experimentType?: string,
): Promise<ExperimentSummary[]> {
  return invoke<ExperimentSummary[]>('get_experiment_archive', { experimentType });
}

/** Get experiment detail including daily logs. */
export async function getExperimentDetail(
  experimentId: string,
): Promise<[ExperimentSummary, ExperimentLogSummary[]]> {
  return invoke<[ExperimentSummary, ExperimentLogSummary[]]>('get_experiment_detail', {
    experimentId,
  });
}

/** Get math results for a task. */
export async function getMathResults(taskId: string): Promise<MathResult[]> {
  return invoke<MathResult[]>('get_math_results', { taskId });
}

/** Get final documents for an experiment. */
export async function getFinalDocuments(
  experimentId: string,
): Promise<FinalDocSummary[]> {
  return invoke<FinalDocSummary[]>('get_final_documents', { experimentId });
}

// ── Approved Tests ───────────────────────────────────────────────────────────

/** Get approved tests optionally filtered by scope. */
export async function getApprovedTests(
  scope?: string,
): Promise<ApprovedTest[]> {
  return invoke<ApprovedTest[]>('get_approved_tests', { scope });
}

// ── Test Proposals ───────────────────────────────────────────────────────────

/** Propose a new test method (routes to supervisor Director). */
export async function proposeNewTest(
  payload: ProposeNewTestPayload,
): Promise<string> {
  return invoke<string>('propose_new_test', { payload });
}

/** Get test proposals submitted by the current user. */
export async function getMyTestProposals(): Promise<TestProposal[]> {
  return invoke<TestProposal[]>('get_my_test_proposals');
}

// ── Species Proposal ─────────────────────────────────────────────────────────

/** Propose a new species for documentation (Biologist only). */
export async function proposeNewSpecies(
  payload: ProposeNewSpeciesPayload,
): Promise<string> {
  return invoke<string>('propose_new_species', { payload });
}
