// stores/sanitary.ts — Frontend API for the Sanitary Department subsystem
// Source of truth: 10_SANITARY.md
// All business logic lives in Rust — this file only mirrors the Tauri command interface.

import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────────────────────

// -- Divisions --

export interface DivisionRow {
  id: string;
  name: string;
  quota: number;
}

export interface SetQuotaPayload {
  division_id: string;
  quota: number;
}

// -- Staff roster --

export interface StaffRosterEntry {
  user_id: string;
  full_name: string;
  role_name: string;
  division_id: string | null;
  division_name: string | null;
  quarter: string | null;
}

// -- Shifts --

export interface SanitaryShift {
  id: string;
  user_id: string;
  staff_name: string;
  shift_start: string;
  shift_end: string;
  allocated_by: string;
  quarter: string;
  created_at: string;
}

export interface AllocateShiftPayload {
  user_id: string;
  shift_start: string;
  shift_end: string;
  quarter: string;
}

export interface DeleteShiftPayload {
  shift_id: string;
}

// -- Tasks --

export interface SanitaryTask {
  id: string;
  assigned_by: string;
  assigner_name: string;
  task_type: string;
  task_name: string | null;
  urgency: string | null;
  instructions: string | null;
  location: string | null;
  due_date: string | null;
  status: string;
  source_report_id: string | null;
  created_at: string;
}

export interface AssignTaskPayload {
  task_type: string;
  task_name?: string;
  urgency?: string;
  instructions?: string;
  location?: string;
  due_date?: string;
  assigned_to: string[];
  source_report_id?: string;
}

export interface UpdateTaskStatusPayload {
  task_id: string;
  status: string;
}

// -- Inventory --

export interface SanitaryInventoryItem {
  id: string;
  item_name: string;
  category: string | null;
  quantity: number;
  unit: string | null;
  updated_at: string;
}

export interface AddInventoryPayload {
  item_name: string;
  category?: string;
  quantity: number;
  unit?: string;
}

export interface UpdateInventoryPayload {
  item_id: string;
  quantity: number;
}

export interface RemoveInventoryPayload {
  item_id: string;
}

export interface InventoryLogEntry {
  id: string;
  item_id: string;
  item_name: string;
  logged_by: string;
  logger_name: string;
  action: string;
  quantity: number;
  purchase_note: string | null;
  logged_at: string;
}

export interface LogInventoryPayload {
  item_id: string;
  action: string; // 'add' | 'remove'
  quantity: number;
  purchase_note?: string;
}

// -- Transfer requests --

export interface TransferRequestRow {
  id: string;
  requested_by: string;
  requester_name: string;
  from_division_name: string;
  to_division_name: string;
  reason: string | null;
  status: string;
  decided_by: string | null;
  decided_at: string | null;
  created_at: string;
}

export interface SubmitTransferPayload {
  to_division_id: string;
  reason?: string;
}

export interface ReviewTransferPayload {
  request_id: string;
  decision: string; // 'approved' | 'rejected'
}

// -- Recruit --

export interface AssignRecruitPayload {
  user_id: string;
  division_id: string;
  quarter: string;
}

// -- Inspection reports --

export interface InspectionReport {
  id: string;
  reported_by: string;
  reporter_name: string;
  report_date: string;
  location: string;
  area_or_machine: string;
  findings: string;
  severity: string;
  recommendations: string | null;
  created_at: string;
}

export interface SubmitInspectionPayload {
  report_date: string;
  location: string;
  area_or_machine: string;
  findings: string;
  severity: string;
  recommendations?: string;
}

// -- Disposal docs --

export interface DisposalDoc {
  id: string;
  waste_category: string;
  procedure: string;
  safety_requirements: string | null;
  compliance_notes: string | null;
  revision_history: Record<string, unknown>[];
  authored_by: string;
  author_name: string;
  created_at: string;
}

export interface CreateDisposalDocPayload {
  waste_category: string;
  procedure: string;
  safety_requirements?: string;
  compliance_notes?: string;
}

export interface UpdateDisposalDocPayload {
  doc_id: string;
  procedure?: string;
  safety_requirements?: string;
  compliance_notes?: string;
  change_summary: string;
}

// -- Wastewater docs --

export interface WastewaterDoc {
  id: string;
  treatment_type: string;
  steps: string[];
  safety_requirements: string | null;
  compliance_notes: string | null;
  revision_history: Record<string, unknown>[];
  authored_by: string;
  author_name: string;
  created_at: string;
}

export interface CreateWastewaterDocPayload {
  treatment_type: string;
  steps: string[];
  safety_requirements?: string;
  compliance_notes?: string;
}

export interface UpdateWastewaterDocPayload {
  doc_id: string;
  steps?: string[];
  safety_requirements?: string;
  compliance_notes?: string;
  change_summary: string;
}

// -- Budget --

export interface SubmitBudgetRequestPayload {
  line_items: Record<string, unknown>[];
  total_amount: number;
  justification?: string;
  invoice_bytes: number[];
  invoice_filename: string;
  invoice_content_type: string;
}

export interface BudgetRequestSummary {
  id: string;
  total_amount: number | null;
  justification: string | null;
  status: string;
  line_items: Record<string, unknown>[];
  created_at: string;
}

// -- Expenditure --

export interface SubmitExpenditurePayload {
  line_items: Record<string, unknown>[];
  total_amount: number;
}

export interface ExpenditureReportSummary {
  id: string;
  total_amount: number | null;
  line_items: Record<string, unknown>[];
  foul_play_flag: boolean;
  foul_play_note: string | null;
  created_at: string;
}

// ── Invoke Wrappers ───────────────────────────────────────────────────────────

// -- UC-HS-07: Divisions & Quotas --

export async function sanGetDivisions(): Promise<DivisionRow[]> {
  return invoke<DivisionRow[]>('san_get_divisions');
}

export async function sanSetDivisionQuota(payload: SetQuotaPayload): Promise<void> {
  return invoke<void>('san_set_division_quota', { payload });
}

// -- UC-HS-04: Staff Roster & Shifts --

export async function sanGetStaffRoster(): Promise<StaffRosterEntry[]> {
  return invoke<StaffRosterEntry[]>('san_get_staff_roster');
}

export async function sanGetAllShifts(): Promise<SanitaryShift[]> {
  return invoke<SanitaryShift[]>('san_get_all_shifts');
}

export async function sanAllocateShift(payload: AllocateShiftPayload): Promise<string> {
  return invoke<string>('san_allocate_shift', { payload });
}

export async function sanDeleteShift(payload: DeleteShiftPayload): Promise<void> {
  return invoke<void>('san_delete_shift', { payload });
}

// -- UC-HS-05: Assign Task --

export async function sanAssignTask(payload: AssignTaskPayload): Promise<string> {
  return invoke<string>('san_assign_task', { payload });
}

export async function sanGetAllTasks(): Promise<SanitaryTask[]> {
  return invoke<SanitaryTask[]>('san_get_all_tasks');
}

// -- UC-HS-06: Transfer Requests (Head) --

export async function sanGetTransferRequests(): Promise<TransferRequestRow[]> {
  return invoke<TransferRequestRow[]>('san_get_transfer_requests');
}

export async function sanReviewTransferRequest(payload: ReviewTransferPayload): Promise<void> {
  return invoke<void>('san_review_transfer_request', { payload });
}

// -- UC-HS-08: Assign Recruit --

export async function sanAssignRecruit(payload: AssignRecruitPayload): Promise<string> {
  return invoke<string>('san_assign_recruit', { payload });
}

// -- UC-HS-03: Inventory (Macro — HeadOfSanitary) --

export async function sanGetInventory(): Promise<SanitaryInventoryItem[]> {
  return invoke<SanitaryInventoryItem[]>('san_get_inventory');
}

export async function sanAddInventory(payload: AddInventoryPayload): Promise<string> {
  return invoke<string>('san_add_inventory', { payload });
}

export async function sanUpdateInventory(payload: UpdateInventoryPayload): Promise<void> {
  return invoke<void>('san_update_inventory', { payload });
}

export async function sanRemoveInventory(payload: RemoveInventoryPayload): Promise<void> {
  return invoke<void>('san_remove_inventory', { payload });
}

// -- UC-STAS-01: Tasks (All Staff) --

export async function sanGetMyTasks(): Promise<SanitaryTask[]> {
  return invoke<SanitaryTask[]>('san_get_my_tasks');
}

export async function sanUpdateTaskStatus(payload: UpdateTaskStatusPayload): Promise<void> {
  return invoke<void>('san_update_task_status', { payload });
}

// -- UC-STAS-02: Schedule --

export async function sanGetMySchedule(): Promise<SanitaryShift[]> {
  return invoke<SanitaryShift[]>('san_get_my_schedule');
}

// -- UC-STAS-03: Transfer Request (Staff) --

export async function sanSubmitTransferRequest(payload: SubmitTransferPayload): Promise<string> {
  return invoke<string>('san_submit_transfer_request', { payload });
}

export async function sanGetMyTransfers(): Promise<TransferRequestRow[]> {
  return invoke<TransferRequestRow[]>('san_get_my_transfers');
}

// -- UC-STAS-04: Inventory Log --

export async function sanLogInventoryAction(payload: LogInventoryPayload): Promise<string> {
  return invoke<string>('san_log_inventory_action', { payload });
}

export async function sanGetInventoryLogs(): Promise<InventoryLogEntry[]> {
  return invoke<InventoryLogEntry[]>('san_get_inventory_logs');
}

// -- UC-IC-01: Inspection Reports --

export async function sanSubmitInspectionReport(payload: SubmitInspectionPayload): Promise<string> {
  return invoke<string>('san_submit_inspection_report', { payload });
}

export async function sanGetInspectionReports(): Promise<InspectionReport[]> {
  return invoke<InspectionReport[]>('san_get_inspection_reports');
}

// -- UC-DC-01: Disposal Docs --

export async function sanCreateDisposalDoc(payload: CreateDisposalDocPayload): Promise<string> {
  return invoke<string>('san_create_disposal_doc', { payload });
}

export async function sanUpdateDisposalDoc(payload: UpdateDisposalDocPayload): Promise<void> {
  return invoke<void>('san_update_disposal_doc', { payload });
}

export async function sanGetDisposalDocs(filter?: string): Promise<DisposalDoc[]> {
  return invoke<DisposalDoc[]>('san_get_disposal_docs', { filter: filter ?? null });
}

// -- UC-WC-01: Wastewater Docs --

export async function sanCreateWastewaterDoc(payload: CreateWastewaterDocPayload): Promise<string> {
  return invoke<string>('san_create_wastewater_doc', { payload });
}

export async function sanUpdateWastewaterDoc(payload: UpdateWastewaterDocPayload): Promise<void> {
  return invoke<void>('san_update_wastewater_doc', { payload });
}

export async function sanGetWastewaterDocs(filter?: string): Promise<WastewaterDoc[]> {
  return invoke<WastewaterDoc[]>('san_get_wastewater_docs', { filter: filter ?? null });
}

// -- UC-HS-01: Budget Request --

export async function sanSubmitBudgetRequest(payload: SubmitBudgetRequestPayload): Promise<string> {
  return invoke<string>('san_submit_budget_request', { payload });
}

export async function sanGetBudgetRequests(): Promise<BudgetRequestSummary[]> {
  return invoke<BudgetRequestSummary[]>('san_get_budget_requests');
}

// -- UC-HS-02: Expenditure Report --

export async function sanSubmitExpenditureReport(payload: SubmitExpenditurePayload): Promise<string> {
  return invoke<string>('san_submit_expenditure_report', { payload });
}

export async function sanGetExpenditureReports(): Promise<ExpenditureReportSummary[]> {
  return invoke<ExpenditureReportSummary[]>('san_get_expenditure_reports');
}
