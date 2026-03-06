// stores/psychiatry.ts — Frontend API for Psychiatry Division subsystem (Sub-07)
// Source of truth: 07_PSYCHIATRY.md
// All business logic lives in Rust — this file only mirrors the Tauri command interface.

import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────────────────────

export interface PatientSummary {
  id: string;
  user_id: string;
  patient_name: string;
  care_status: string;
  initial_notes: string | null;
  created_at: string | null;
}

export interface PatientDetail {
  id: string;
  user_id: string;
  psychiatrist_id: string;
  patient_profile: Record<string, unknown>;
  initial_notes: string | null;
  created_at: string | null;
  updated_at: string | null;
}

export interface AppointmentRecord {
  id: string;
  patient_id: string;
  psychiatrist_id: string;
  scheduled_at: string;
  status: string;
  booked_by: string | null;
  appointment_log: Record<string, unknown> | null;
  created_at: string | null;
}

export interface AppointmentSummary {
  id: string;
  patient_id: string;
  scheduled_at: string;
  status: string;
  booked_by: string | null;
}

export interface RecoveryLogEntry {
  id: string;
  patient_id: string;
  milestone: string;
  status: string;
  details: string | null;
  logged_at: string | null;
  logged_by: string | null;
}

export interface ScheduleSlot {
  id: string;
  psychiatrist_id: string;
  slot_start: string;
  slot_end: string;
  is_available: boolean;
  blocked_reason: string | null;
}

export interface PatientIndexEntry {
  id: string;
  user_id: string;
  patient_name: string;
  department: string;
  care_status: string;
  last_updated: string | null;
}

export interface PatientListEntry {
  id: string;
  user_id: string;
  patient_name: string;
  care_status: string;
}

export interface AccessSetting {
  assistant_id: string;
  assistant_name: string;
  granted: boolean;
  updated_at: string | null;
}

export interface UserPickerEntry {
  id: string;
  full_name: string;
  username: string;
  role_name: string;
}

export interface CreatedId {
  id: string;
}

// ── Payloads ──────────────────────────────────────────────────────────────────

export interface CreatePatientPayload {
  user_id: string;
  patient_profile: Record<string, unknown>;
  initial_notes?: string;
}

export interface LogAppointmentPayload {
  patient_id: string;
  scheduled_at: string;
  appointment_log: Record<string, unknown>;
}

export interface UpdateRecoveryLogPayload {
  patient_id: string;
  milestone: string;
  status: string;
  details?: string;
}

export interface ScheduleSlotInput {
  slot_start: string;
  slot_end: string;
  is_available: boolean;
  blocked_reason?: string;
}

export interface ManageSchedulePayload {
  slots: ScheduleSlotInput[];
}

export interface ScheduleAppointmentPayload {
  patient_id: string;
  psychiatrist_id: string;
  scheduled_at: string;
}

export interface GrantAccessPayload {
  assistant_id: string;
  grant: boolean;
}

// ══════════════════════════════════════════════════════════════════════════════
// PSYCHIATRIST commands
// ══════════════════════════════════════════════════════════════════════════════

/** UC-PSY-01: Create a new patient record */
export async function psyCreatePatientRecord(payload: CreatePatientPayload): Promise<CreatedId> {
  return invoke('psy_create_patient_record', { payload });
}

/** Get eligible user list for patient picker (non-Director, non-Admin) */
export async function psyGetUserDirectory(): Promise<UserPickerEntry[]> {
  return invoke('psy_get_user_directory');
}

/** Get list of patients assigned to current psychiatrist */
export async function psyGetMyPatients(): Promise<PatientSummary[]> {
  return invoke('psy_get_my_patients');
}

/** Get full patient detail (own patients only) */
export async function psyGetPatientDetail(patientId: string): Promise<PatientDetail> {
  return invoke('psy_get_patient_detail', { patientId });
}

/** UC-PSY-02: Log appointment findings */
export async function psyLogAppointment(payload: LogAppointmentPayload): Promise<CreatedId> {
  return invoke('psy_log_appointment', { payload });
}

/** Get appointments for a patient (full clinical data) */
export async function psyGetAppointments(patientId: string): Promise<AppointmentRecord[]> {
  return invoke('psy_get_appointments', { patientId });
}

/** UC-PSY-03: Add recovery log entry */
export async function psyUpdateRecoveryLog(payload: UpdateRecoveryLogPayload): Promise<CreatedId> {
  return invoke('psy_update_recovery_log', { payload });
}

/** Get recovery logs for a patient (full details) */
export async function psyGetRecoveryLogs(patientId: string): Promise<RecoveryLogEntry[]> {
  return invoke('psy_get_recovery_logs', { patientId });
}

/** UC-PSY-04: Manage schedule (add slots) */
export async function psyManageSchedule(payload: ManageSchedulePayload): Promise<void> {
  return invoke('psy_manage_schedule', { payload });
}

/** Get own schedule */
export async function psyGetSchedule(): Promise<ScheduleSlot[]> {
  return invoke('psy_get_schedule');
}

/** Delete a schedule slot */
export async function psyDeleteScheduleSlot(slotId: string): Promise<void> {
  return invoke('psy_delete_schedule_slot', { slotId });
}

/** UC-PSY-05: View shared patient index */
export async function psyGetPatientIndex(): Promise<PatientIndexEntry[]> {
  return invoke('psy_get_patient_index');
}

// ══════════════════════════════════════════════════════════════════════════════
// PSYCHIATRIST ASSISTANT commands
// ══════════════════════════════════════════════════════════════════════════════

/** UC-PA-01: Schedule a patient appointment */
export async function psyScheduleAppointment(payload: ScheduleAppointmentPayload): Promise<CreatedId> {
  return invoke('psy_schedule_appointment', { payload });
}

/** UC-PA-02: Request access to a patient's schedule */
export async function psyRequestScheduleAccess(patientUserId: string): Promise<void> {
  return invoke('psy_request_schedule_access', { patientUserId });
}

/** UC-PA-03: Get patient list (restricted — names only) */
export async function psyAssistantGetPatients(): Promise<PatientListEntry[]> {
  return invoke('psy_assistant_get_patients');
}

/** UC-PA-04: Get patient recovery log (GDPR-checked) */
export async function psyAssistantGetRecoveryLog(patientId: string): Promise<RecoveryLogEntry[]> {
  return invoke('psy_assistant_get_recovery_log', { patientId });
}

/** Get psychiatrist schedule (assistant read-only) */
export async function psyAssistantGetSchedule(psychiatristId: string): Promise<ScheduleSlot[]> {
  return invoke('psy_assistant_get_schedule', { psychiatristId });
}

/** Get appointments for a patient (assistant — no clinical data) */
export async function psyAssistantGetAppointments(patientId: string): Promise<AppointmentSummary[]> {
  return invoke('psy_assistant_get_appointments', { patientId });
}

// ══════════════════════════════════════════════════════════════════════════════
// PATIENT commands (any authenticated RUSA user under care)
// ══════════════════════════════════════════════════════════════════════════════

/** UC-PAT-01: Grant or deny assistant schedule access */
export async function psyGrantScheduleAccess(payload: GrantAccessPayload): Promise<void> {
  return invoke('psy_grant_schedule_access', { payload });
}

/** Get current access settings (pending requests + grants) */
export async function psyGetAccessSettings(): Promise<AccessSetting[]> {
  return invoke('psy_get_access_settings');
}
