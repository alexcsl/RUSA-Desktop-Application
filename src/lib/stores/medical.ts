// stores/medical.ts — Frontend API for the Medical Department subsystem
// Source of truth: 09_MEDICAL.md
// All business logic lives in Rust — this file only mirrors the Tauri command interface.

import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────────────────────

// -- Patients --

export interface PatientSummary {
  id: string;
  user_id: string;
  full_name: string;
  created_at: string;
}

export interface TreatmentLog {
  id: string;
  patient_id: string;
  treated_by: string;
  treater_name: string;
  treatment_date: string;
  diagnosis: string;
  treatment_provided: string;
  medications: string | null;
  follow_up_notes: string | null;
  created_at: string;
}

export interface LogTreatmentPayload {
  patient_id: string;
  treatment_date: string;
  diagnosis: string;
  treatment_provided: string;
  medications?: string;
  follow_up_notes?: string;
}

export interface RegisterPatientPayload {
  user_id: string;
}

// -- Inventory --

export interface InventoryItem {
  id: string;
  base_id: string;
  item_name: string;
  item_type: string; // 'medicine' | 'equipment' | 'supply'
  quantity: number;
  unit: string | null;
  updated_at: string;
}

export interface AddInventoryPayload {
  item_name: string;
  item_type: string;
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

// -- Shifts --

export interface ShiftEntry {
  id: string;
  user_id: string;
  staff_name: string;
  specialty: string | null;
  base_id: string;
  shift_start: string;
  shift_end: string;
  allocated_by: string;
  created_at: string;
}

export interface AllocateShiftPayload {
  user_id: string;
  shift_start: string;
  shift_end: string;
}

export interface DeleteShiftPayload {
  shift_id: string;
}

// -- Staff --

export interface MedicalStaffMember {
  id: string;
  full_name: string;
  specialty: string | null;
  base_id: string | null;
  base_name: string | null;
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

// -- User lookup --

export interface UserLookupItem {
  id: string;
  full_name: string;
  role_name: string;
}

// ── Invoke Wrappers ───────────────────────────────────────────────────────────

// UC-MED-01: Log Patient Treatment

export async function medRegisterPatient(payload: RegisterPatientPayload): Promise<string> {
  return invoke<string>('med_register_patient', { payload });
}

export async function medLogTreatment(payload: LogTreatmentPayload): Promise<string> {
  return invoke<string>('med_log_treatment', { payload });
}

// UC-MED-02: View Patient Record

export async function medGetPatients(): Promise<PatientSummary[]> {
  return invoke<PatientSummary[]>('med_get_patients');
}

export async function medGetPatientRecord(patientId: string): Promise<TreatmentLog[]> {
  return invoke<TreatmentLog[]>('med_get_patient_record', { patientId });
}

// UC-MED-03: Manage Medical Inventory

export async function medGetInventory(): Promise<InventoryItem[]> {
  return invoke<InventoryItem[]>('med_get_inventory');
}

export async function medAddInventory(payload: AddInventoryPayload): Promise<string> {
  return invoke<string>('med_add_inventory', { payload });
}

export async function medUpdateInventory(payload: UpdateInventoryPayload): Promise<void> {
  return invoke<void>('med_update_inventory', { payload });
}

export async function medRemoveInventory(payload: RemoveInventoryPayload): Promise<void> {
  return invoke<void>('med_remove_inventory', { payload });
}

// UC-MED-04: View Work Schedule

export async function medGetMyShifts(): Promise<ShiftEntry[]> {
  return invoke<ShiftEntry[]>('med_get_my_shifts');
}

// UC-HOM-01: Allocate Staff Shifts

export async function medGetStaffList(): Promise<MedicalStaffMember[]> {
  return invoke<MedicalStaffMember[]>('med_get_staff_list');
}

export async function medGetAllShifts(): Promise<ShiftEntry[]> {
  return invoke<ShiftEntry[]>('med_get_all_shifts');
}

export async function medAllocateShift(payload: AllocateShiftPayload): Promise<string> {
  return invoke<string>('med_allocate_shift', { payload });
}

export async function medDeleteShift(payload: DeleteShiftPayload): Promise<void> {
  return invoke<void>('med_delete_shift', { payload });
}

// UC-HOM-02: Submit Budget Funding Request

export async function medSubmitBudgetRequest(payload: SubmitBudgetRequestPayload): Promise<string> {
  return invoke<string>('med_submit_budget_request', { payload });
}

export async function medGetBudgetRequests(): Promise<BudgetRequestSummary[]> {
  return invoke<BudgetRequestSummary[]>('med_get_budget_requests');
}

// UC-HOM-03: Submit Budget Expenditure Report

export async function medSubmitExpenditureReport(payload: SubmitExpenditurePayload): Promise<string> {
  return invoke<string>('med_submit_expenditure_report', { payload });
}

export async function medGetExpenditureReports(): Promise<ExpenditureReportSummary[]> {
  return invoke<ExpenditureReportSummary[]>('med_get_expenditure_reports');
}

// Helpers

export async function medGetUserLookup(): Promise<UserLookupItem[]> {
  return invoke<UserLookupItem[]>('med_get_user_lookup');
}
