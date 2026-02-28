// stores/directors.ts — Frontend API for Directors + Voting subsystem
// All business logic lives in Rust — this file only mirrors the Tauri command interface.

import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────────────────────

export interface VoteSessionSummary {
  id: string;
  topic: string;
  context: string | null;
  status: string;
  opens_at: string;
  closes_at: string | null;
  result: string | null;
  total_yay: number;
  total_nay: number;
  total_abstain: number;
  admin_overridden: boolean;
  created_by: string;
  created_at: string;
}

export interface VoteRecord {
  id: string;
  session_id: string;
  director_id: string;
  vote: string;
  reason: string;
  changed_at: string | null;
  created_at: string;
}

export interface VoteRecordWithName extends VoteRecord {
  director_name: string;
}

export interface VoteSessionWithRecords {
  session: VoteSessionSummary & {
    admin_override_decision: string | null;
    admin_override_reason: string | null;
    admin_terminated: boolean;
    request_id: string | null;
  };
  records: VoteRecordWithName[];
  my_vote: VoteRecord | null;
}

export interface MeetingSummary {
  id: string;
  title: string;
  agenda: string | null;
  scheduled_at: string;
  created_by: string;
  created_at: string;
}

export interface TaskSummary {
  id: string;
  title: string;
  description: string | null;
  status: string;
  assigned_to: string;
  assignee_name: string;
  assigned_by: string;
  assigner_name: string;
  due_date: string | null;
  created_at: string;
}

export interface FinancialDocument {
  id: string;
  type_: string;
  title: string;
  status: string;
  requester_id: string;
  requester_name: string;
  payload: Record<string, unknown>;
  invoice_storage_path: string | null;
  created_at: string;
}

export interface BroadcastRequestSummary {
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

export interface DataRequestSummary {
  id: string;
  dataset_description: string;
  scope: string;
  purpose: string;
  urgency: string;
  sensitivity_note: string | null;
  status: string;
  requester_id: string;
  requester_name: string;
  created_at: string;
}

export interface OutboundResponseSummary {
  id: string;
  request_id: string;
  analyst_name: string;
  requester_name: string;
  dataset_description: string;
  result_payload: Record<string, unknown> | null;
  spreadsheet_storage_path: string | null;
  status: string;
  submitted_at: string;
}

export interface EventSummary {
  id: string;
  title: string;
  event_date: string | null;
  location: string | null;
  agenda: string | null;
  created_by: string;
  created_at: string;
}

export interface EventDocSummary {
  id: string;
  event_id: string;
  document_type: string | null;
  original_filename: string;
  content_type: string;
  storage_path: string;
  uploaded_at: string;
}

export interface TerritorySummary {
  id: string;
  territory_type: string;
  name: string;
  previous_name: string | null;
  renamed_at: string;
}

export interface NotificationItem {
  id: string;
  type_: string;
  payload: Record<string, unknown>;
  read_at: string | null;
  created_at: string;
}

export interface PersonnelListItem {
  id: string;
  full_name: string;
  username: string;
  role_name: string;
}

// ── Voting API ────────────────────────────────────────────────────────────────

export async function castVote(
  sessionId: string,
  vote: 'yay' | 'nay' | 'abstain',
  reason: string,
): Promise<void> {
  await invoke('cast_vote', {
    payload: { session_id: sessionId, vote, reason },
  });
}

export async function changeVote(
  sessionId: string,
  vote: 'yay' | 'nay' | 'abstain',
  reason: string,
): Promise<void> {
  await invoke('change_vote', {
    payload: { session_id: sessionId, vote, reason },
  });
}

export async function getPendingVotes(): Promise<VoteSessionSummary[]> {
  return invoke<VoteSessionSummary[]>('get_pending_votes');
}

export async function getVoteSessionDetail(
  sessionId: string,
): Promise<VoteSessionWithRecords> {
  return invoke<VoteSessionWithRecords>('get_vote_session_detail', {
    sessionId,
  });
}

export async function getVoteSessions(): Promise<VoteSessionSummary[]> {
  return invoke<VoteSessionSummary[]>('get_vote_sessions');
}

export async function initiateAdHocVote(
  topic: string,
  context?: string,
): Promise<string> {
  return invoke<string>('initiate_ad_hoc_vote', {
    payload: { topic, context },
  });
}

export async function adminOverrideVote(
  sessionId: string,
  decision: 'approved' | 'denied',
  reason: string,
): Promise<void> {
  await invoke('admin_override_vote', {
    payload: { session_id: sessionId, decision, reason },
  });
}

export async function adminTerminateVote(
  sessionId: string,
  reason: string,
): Promise<void> {
  await invoke('admin_terminate_vote', { sessionId, reason });
}

// ── Meeting API ───────────────────────────────────────────────────────────────

export async function createMeeting(payload: {
  title: string;
  agenda?: string;
  scheduled_at: string;
  invitee_ids: string[];
}): Promise<{ id: string }> {
  return invoke<{ id: string }>('create_meeting', { payload });
}

export async function getMeetings(): Promise<MeetingSummary[]> {
  return invoke<MeetingSummary[]>('get_meetings');
}

// ── Account API (update) ──────────────────────────────────────────────────────

export async function updatePersonnelAccount(
  targetUserId: string,
  payload: {
    full_name?: string;
    email?: string;
    role?: string;
    base_location_id?: string;
  },
): Promise<void> {
  await invoke('update_personnel_account', { targetUserId, payload });
}

// ── Financial API ─────────────────────────────────────────────────────────────

export async function getFinancialQueue(): Promise<FinancialDocument[]> {
  return invoke<FinancialDocument[]>('get_financial_queue');
}

export async function flagBudgetReport(
  requestId: string,
  reason: string,
): Promise<void> {
  await invoke('flag_budget_report', {
    payload: { request_id: requestId, reason },
  });
}

// ── Relocation API ────────────────────────────────────────────────────────────

export async function relocatePersonnel(payload: {
  target_user_id: string;
  origin_location: string;
  destination: string;
  relocation_type: 'temporary' | 'permanent';
  effective_date: string;
}): Promise<string> {
  return invoke<string>('relocate_personnel', { payload });
}

// ── Task API ──────────────────────────────────────────────────────────────────

export async function assignTask(payload: {
  assigned_to: string;
  title: string;
  description?: string;
  task_type?: string;
  due_date?: string;
  payload?: Record<string, unknown>;
}): Promise<string> {
  return invoke<string>('assign_task', { payload });
}

export async function getSubordinateTasks(): Promise<TaskSummary[]> {
  return invoke<TaskSummary[]>('get_subordinate_tasks');
}

export async function approveClosureRequest(
  requestId: string,
  decision: 'approved' | 'denied',
  reason?: string,
): Promise<void> {
  await invoke('approve_closure_request', {
    payload: { request_id: requestId, decision, reason },
  });
}

export async function reviewHelpRequest(
  taskId: string,
  decision: 'forward' | 'reject' | 'convert',
  reason?: string,
): Promise<void> {
  await invoke('review_help_request', {
    payload: { task_id: taskId, decision, reason },
  });
}

export async function reviewHelpResponse(
  taskId: string,
  decision: 'forward' | 'withhold',
  reason?: string,
): Promise<void> {
  await invoke('review_help_response', {
    payload: { task_id: taskId, decision, reason },
  });
}

// ── Territory API ─────────────────────────────────────────────────────────────

export async function renameTerritory(
  territoryId: string,
  newName: string,
): Promise<void> {
  await invoke('rename_territory', {
    payload: { territory_id: territoryId, new_name: newName },
  });
}

export async function getTerritories(): Promise<TerritorySummary[]> {
  return invoke<TerritorySummary[]>('get_territories');
}

// ── Broadcast API ─────────────────────────────────────────────────────────────

export async function submitBroadcastRequest(payload: {
  type_: string;
  subject: string;
  content: string;
  target_scope?: string;
  target_ids?: string[];
  urgency?: string;
  rationale?: string;
}): Promise<string> {
  return invoke<string>('submit_broadcast_request', { payload });
}

export async function getBroadcastRequestQueue(): Promise<BroadcastRequestSummary[]> {
  return invoke<BroadcastRequestSummary[]>('get_broadcast_request_queue');
}

export async function decideBroadcastRequest(
  requestId: string,
  decision: 'approved' | 'rejected',
  reason?: string,
): Promise<void> {
  await invoke('decide_broadcast_request', {
    payload: { request_id: requestId, decision, reason },
  });
}

// ── Experiment Proposal Queue (Artificer / Observer) ─────────────────────────

export interface ExperimentProposalSummary {
  request_id: string;
  experiment_id: string;
  experiment_type: string;
  title: string;
  description: string | null;
  requester_id: string;
  requester_name: string;
  status: string;
  created_at: string;
}

export async function getExperimentProposalQueue(): Promise<ExperimentProposalSummary[]> {
  return invoke<ExperimentProposalSummary[]>('get_experiment_proposal_queue');
}

export async function decideExperimentProposal(
  requestId: string,
  decision: 'approved' | 'denied',
  reason?: string,
): Promise<void> {
  await invoke('decide_experiment_proposal', {
    payload: { request_id: requestId, decision, reason },
  });
}

// ── Math Results for Director (Artificer) ─────────────────────────────────────

export interface MathResultItem {
  id: string;
  task_id: string;
  submitted_by: string;
  submitted_by_name: string;
  content: {
    content_latex?: string;
    workings?: string;
    calculations_area?: string;
  };
  pdf_storage_path: string | null;
  created_at: string;
}

export async function getMathResultsForDirector(): Promise<MathResultItem[]> {
  return invoke<MathResultItem[]>('get_math_results_for_director');
}

// ── Test Proposal Queue (Artificer / Observer) ────────────────────────────────

export interface TestProposalQueueItem {
  id: string;
  proposed_by: string;
  proposer_name: string;
  proposal_data: {
    name?: string;
    goal?: string;
    procedure?: string;
    species_scope?: string;
    category?: string;
    apparatuses?: string;
    required_data?: string;
    justification?: string;
  };
  status: string;
  reviewer_note: string | null;
  created_at: string;
}

export async function getTestProposalQueue(): Promise<TestProposalQueueItem[]> {
  return invoke<TestProposalQueueItem[]>('get_test_proposal_queue');
}

export async function decideTestProposal(
  proposalId: string,
  decision: 'approved' | 'rejected',
  reason?: string,
): Promise<void> {
  await invoke('decide_test_proposal', {
    payload: { proposal_id: proposalId, decision, reason },
  });
}

// ── Final Document Queue (Artificer / Observer) ───────────────────────────────

export interface FinalDocQueueItem {
  id: string;
  experiment_id: string;
  experiment_title: string;
  doc_type: string;
  document_data: Record<string, unknown>;
  status: string;
  submitted_by: string;
  submitter_name: string;
  reviewer_note: string | null;
  created_at: string;
}

export async function getFinalDocumentQueue(): Promise<FinalDocQueueItem[]> {
  return invoke<FinalDocQueueItem[]>('get_final_document_queue');
}

export async function decideFinalDocument(
  documentId: string,
  decision: 'approved' | 'rejected',
  reason?: string,
): Promise<void> {
  await invoke('decide_final_document', {
    payload: { document_id: documentId, decision, reason },
  });
}

export async function sendEmergencyBroadcast(payload: {
  subject: string;
  content: string;
  target_scope?: string;
  target_ids?: string[];
}): Promise<string> {
  return invoke<string>('send_emergency_broadcast', {
    payload: { ...payload, type_: 'security' },
  });
}

export async function sendInformationalBroadcast(payload: {
  subject: string;
  content: string;
  target_scope?: string;
  target_ids?: string[];
  urgency?: string;
}): Promise<string> {
  return invoke<string>('send_informational_broadcast', {
    payload: { ...payload, type_: 'informational' },
  });
}

// ── Data Request API (Statistician) ───────────────────────────────────────────

export async function getDataRequestQueue(): Promise<DataRequestSummary[]> {
  return invoke<DataRequestSummary[]>('get_data_request_queue');
}

export async function decideDataRequest(
  requestId: string,
  decision: 'approved' | 'rejected',
  reason?: string,
): Promise<void> {
  await invoke('decide_data_request', {
    payload: { request_id: requestId, decision, reason },
  });
}

export async function getOutboundReviewQueue(): Promise<OutboundResponseSummary[]> {
  return invoke<OutboundResponseSummary[]>('get_outbound_review_queue');
}

export async function reviewOutboundDataResponse(
  responseId: string,
  decision: 'forward' | 'withhold',
  reason?: string,
): Promise<void> {
  await invoke('review_outbound_data_response', {
    payload: { response_id: responseId, decision, reason },
  });
}

// ── Event API (Coordinator) ───────────────────────────────────────────────────

export interface EventAttendeeInput {
  name: string;
  is_external: boolean;
  user_id?: string;
}

export async function createEvent(payload: {
  title: string;
  event_date?: string;
  location?: string;
  agenda?: string;
  attendees: EventAttendeeInput[];
}): Promise<string> {
  return invoke<string>('create_event', { payload });
}

export async function getEvents(): Promise<EventSummary[]> {
  return invoke<EventSummary[]>('get_events');
}

export async function uploadEventDocument(
  eventId: string,
  fileBytes: number[],
  filename: string,
  contentType: string,
  documentType: string,
): Promise<string> {
  return invoke<string>('upload_event_document', {
    eventId,
    fileBytes,
    filename,
    contentType,
    documentType,
  });
}

export async function getEventDocuments(
  eventId: string,
): Promise<EventDocSummary[]> {
  return invoke<EventDocSummary[]>('get_event_documents', { eventId });
}

// ── Termination API ───────────────────────────────────────────────────────────

export async function terminatePersonnel(payload: {
  target_user_id: string;
  reason: string;
  effective_date: string;
}): Promise<void> {
  await invoke('terminate_personnel', { payload });
}

// ── Notifications API ─────────────────────────────────────────────────────────

export async function getNotifications(): Promise<NotificationItem[]> {
  return invoke<NotificationItem[]>('get_notifications');
}

export async function markNotificationRead(
  notificationId: string,
): Promise<void> {
  await invoke('mark_notification_read', { notificationId });
}

// ── Personnel API ─────────────────────────────────────────────────────────────

export async function getPersonnelList(): Promise<PersonnelListItem[]> {
  return invoke<PersonnelListItem[]>('get_personnel_list');
}

// ── Secure Messaging API (UC-GUA-04, UC-OVR-02) ──────────────────────────────

export interface MessageSummary {
  id: string;
  from_id: string;
  from_name: string;
  subject: string;
  body: string;
  channel: string;
  recalled_at: string | null;
  created_at: string;
}

export interface MessageRecipient {
  user_id: string;
  full_name: string;
  type_: string;
  read_at: string | null;
}

export async function sendSecurityMessage(payload: {
  subject: string;
  body: string;
  channel: string;
  recipients_to: string[];
  recipients_cc?: string[];
  recipients_bcc?: string[];
}): Promise<string> {
  return invoke<string>('send_security_message', { payload });
}

export async function getSecurityMessages(): Promise<MessageSummary[]> {
  return invoke<MessageSummary[]>('get_security_messages');
}

export async function getMessageRecipients(
  messageId: string,
): Promise<MessageRecipient[]> {
  return invoke<MessageRecipient[]>('get_message_recipients', { messageId });
}

export async function recallMessage(messageId: string): Promise<void> {
  await invoke('recall_message', { messageId });
}

export async function markMessageRead(messageId: string): Promise<void> {
  await invoke('mark_message_read', { messageId });
}

// ── Daily Security Reports (Guardian view) ────────────────────────────────────

export interface DirDailySecurityReport {
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

export async function getDailySecurityReports(): Promise<
  DirDailySecurityReport[]
> {
  return invoke<DirDailySecurityReport[]>('dir_get_daily_security_reports');
}
