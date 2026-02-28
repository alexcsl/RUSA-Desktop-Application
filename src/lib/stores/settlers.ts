// stores/settlers.ts — Exoplanet Settlers subsystem store
// Source of truth: 06_EXOPLANET_SETTLERS.md
//
// 23 Tauri invoke wrappers + types for all settler use cases.

import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────────────────────

export interface SettlerTaskSummary {
  id: string;
  title: string;
  scope: string | null;
  urgency: string | null;
  deadline: string | null;
  status: string;
  assigned_by_name: string;
  created_at: string;
}

export interface SettlerTaskDetail {
  id: string;
  settlement_id: string;
  assigned_by: string;
  assigned_by_name: string;
  assigned_to: string;
  assigned_to_name: string;
  title: string;
  description: string | null;
  scope: string | null;
  urgency: string | null;
  deadline: string | null;
  status: string;
  created_at: string;
}

export interface InventoryItem {
  id: string;
  item_name: string;
  category: string | null;
  quantity: number;
  unit: string | null;
  min_threshold: number;
  updated_at: string;
}

export interface IncomingQueueItem {
  id: string;
  item_type: string;
  submitted_by: string;
  submitted_by_name: string;
  summary: string;
  status: string;
  created_at: string;
}

export interface DashboardSettler {
  user_id: string;
  full_name: string;
  role_name: string;
  assignment_type: string;
  residence_unit: string | null;
  house_arrest: boolean;
  assigned_at: string;
}

export interface DashboardData {
  settlement_name: string;
  planet: string;
  status: string;
  settlers: DashboardSettler[];
  open_anomalies: number;
  pending_requests: number;
  house_arrests: number;
}

export interface ResidenceInfo {
  settlement_name: string;
  planet: string;
  residence_unit: string | null;
  arrived_at: string | null;
}

// ── Supply request item shape ─────────────────────────────────────────────────

export interface SupplyItem {
  item: string;
  specification?: string;
  spec?: string;
  quantity: number;
  unit?: string;
  reason?: string;
}

// ── Shared settler commands ───────────────────────────────────────────────────

/** UC-PS-01: Get tasks assigned to the current settler */
export async function stlGetMyTasks(): Promise<SettlerTaskSummary[]> {
  return invoke('stl_get_my_tasks');
}

/** UC-PS-01 detail: Get task detail by ID */
export async function stlGetTaskDetail(taskId: string): Promise<SettlerTaskDetail> {
  return invoke('stl_get_task_detail', { taskId });
}

/** UC-PS-02: Submit progress report linked to task */
export async function stlSubmitProgressReport(payload: {
  task_id: string;
  week?: string;
  rag_status?: string;
  progress_made: string;
  materials_equipment?: string;
}): Promise<string> {
  return invoke('stl_submit_progress_report', { payload });
}

/** UC-PS-03 / UC-TS-01: Submit anomaly report */
export async function stlSubmitAnomalyReport(payload: {
  description: string;
  location_in_settlement?: string;
  danger_level?: string;
  severity: string;
  category?: string[];
}): Promise<string> {
  return invoke('stl_submit_anomaly_report', { payload });
}

/** UC-PS-04 / UC-TS-02: Submit complaint about another settler */
export async function stlSubmitComplaint(payload: {
  subject_user_id: string;
  incident_description: string;
}): Promise<string> {
  return invoke('stl_submit_complaint', { payload });
}

/** UC-PS-05 / UC-TS-03: Submit supply request (routed to Commander) */
export async function stlSubmitSupplyRequest(payload: {
  items: SupplyItem[];
  justification?: string;
}): Promise<string> {
  return invoke('stl_submit_supply_request', { payload });
}

/** UC-PS-06: Get settlement inventory (read-only) */
export async function stlGetSettlementInventory(): Promise<InventoryItem[]> {
  return invoke('stl_get_settlement_inventory');
}

// ── Commander commands ────────────────────────────────────────────────────────

/** UC-SC-01: Assign task to a settler */
export async function stlAssignTask(payload: {
  assigned_to: string;
  title: string;
  description?: string;
  scope?: string;
  urgency?: string;
  deadline?: string;
}): Promise<string> {
  return invoke('stl_assign_task', { payload });
}

/** UC-SC-02: Get commander's incoming review queue */
export async function stlGetIncomingQueue(): Promise<IncomingQueueItem[]> {
  return invoke('stl_get_incoming_queue');
}

/** UC-SC-02 (action): Reject an incoming item */
export async function stlRejectIncoming(payload: {
  item_id: string;
  item_type: string;
  reason: string;
}): Promise<void> {
  return invoke('stl_reject_incoming', { payload });
}

/** UC-SC-02 (action): Forward an incoming item to Directors for voting */
export async function stlForwardToDirectors(payload: {
  item_id: string;
  item_type: string;
  topic?: string;
  context?: string;
}): Promise<string> {
  return invoke('stl_forward_to_directors', { payload });
}

/** UC-SC-02A: Submit commander anomaly report (direct to Directors) */
export async function stlSubmitCommanderAnomaly(payload: {
  description: string;
  location_in_settlement?: string;
  danger_level?: string;
  severity: string;
  category?: string[];
}): Promise<string> {
  return invoke('stl_submit_commander_anomaly', { payload });
}

/** UC-SC-03: Request settlement abandonment (requires anomaly report) */
export async function stlRequestAbandonment(payload: {
  anomaly_report_id: string;
  reason: string;
}): Promise<string> {
  return invoke('stl_request_abandonment', { payload });
}

/** UC-SC-04: Request settler repatriation (requires complaint) */
export async function stlRequestRepatriation(payload: {
  settler_id: string;
  complaint_id: string;
  reason: string;
}): Promise<string> {
  return invoke('stl_request_repatriation', { payload });
}

/** UC-SC-05: Toggle house arrest flag on a settler */
export async function stlSetHouseArrest(payload: {
  settler_id: string;
  arrest: boolean;
}): Promise<void> {
  return invoke('stl_set_house_arrest', { payload });
}

/** UC-SC-06: Get settlement dashboard data */
export async function stlGetDashboard(): Promise<DashboardData> {
  return invoke('stl_get_dashboard');
}

/** UC-SC-07: Submit commander supply request (direct to Directors) */
export async function stlSubmitCommanderSupply(payload: {
  items: SupplyItem[];
  justification?: string;
}): Promise<string> {
  return invoke('stl_submit_commander_supply', { payload });
}

/** UC-SC-08: Manage settlement inventory (add/update/delete) */
export async function stlManageInventory(payload: {
  action: 'add' | 'update' | 'delete';
  id?: string;
  item_name?: string;
  category?: string;
  quantity?: number;
  unit?: string;
  min_threshold?: number;
}): Promise<void> {
  return invoke('stl_manage_inventory', { payload });
}

// ── Civil Engineer commands ───────────────────────────────────────────────────

/** UC-CE-01: Submit construction progress report */
export async function stlSubmitConstructionReport(payload: {
  task_id: string;
  week?: string;
  rag_status?: string;
  materials_used?: { material: string; quantity: number; unit: string }[];
  construction_progress: string;
  issues?: string;
}): Promise<string> {
  return invoke('stl_submit_construction_report', { payload });
}

/** UC-CE-02: Request building materials */
export async function stlRequestMaterials(payload: {
  items: SupplyItem[];
  justification?: string;
}): Promise<string> {
  return invoke('stl_request_materials', { payload });
}

/** UC-CE-03: Log building health check */
export async function stlLogBuildingHealth(payload: {
  building_name: string;
  check_date: string;
  findings?: string;
  status: 'pass' | 'fail' | 'needs_repair';
}): Promise<string> {
  return invoke('stl_log_building_health', { payload });
}

/** UC-CE-04: View assigned residence */
export async function stlGetResidence(): Promise<ResidenceInfo> {
  return invoke('stl_get_residence');
}

// ── Farmer commands ───────────────────────────────────────────────────────────

/** UC-FA-01: Request farming supplies */
export async function stlRequestFarmingSupplies(payload: {
  items: SupplyItem[];
  justification?: string;
}): Promise<string> {
  return invoke('stl_request_farming_supplies', { payload });
}

/** UC-FA-02: Log farm health check (plant or livestock) */
export async function stlLogFarmHealth(payload: {
  log_date: string;
  subject_type: 'plant' | 'livestock';
  subject_name: string;
  condition: string;
  treatment?: string;
  notes?: string;
}): Promise<string> {
  return invoke('stl_log_farm_health', { payload });
}
