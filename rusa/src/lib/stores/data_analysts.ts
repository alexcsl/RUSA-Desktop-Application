// stores/data_analysts.ts — Frontend API for Data Analysts + Data Request subsystem
// Source of truth: 02_DATA_ANALYSTS.md
// All business logic lives in Rust — this file only mirrors the Tauri command interface.

import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────────────────────

export interface DataRequest {
  id: string;
  requested_by: string;
  requester_name: string;
  dataset_description: string;
  scope: string;
  purpose: string;
  urgency: 'low' | 'medium' | 'high' | 'critical';
  sensitivity_note: string | null;
  status: string;
  statistician_decision_reason: string | null;
  assigned_analyst_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface DataRequestBrief {
  id: string;
  requester_name: string;
  dataset_description: string;
  scope: string;
  urgency: string;
  status: string;
  created_at: string;
}

export interface DataResponse {
  id: string;
  request_id: string;
  prepared_by: string;
  analyst_name: string;
  result_payload: Record<string, unknown> | null;
  spreadsheet_storage_path: string | null;
  status: string;
  statistician_review_note: string | null;
  submitted_at: string;
  cleared_at: string | null;
  delivered_at: string | null;
}

export interface SubmitDataRequestPayload {
  dataset_description: string;
  scope: string;
  purpose: string;
  urgency: 'low' | 'medium' | 'high' | 'critical';
  sensitivity_note?: string;
}

// ── Cross-Role: Any Authenticated User ────────────────────────────────────────

/** UC-DRQ-01: Submit a cross-department data request. */
export async function submitDataRequest(
  payload: SubmitDataRequestPayload,
): Promise<string> {
  return invoke<string>('submit_data_request', { payload });
}

/** Get all data requests submitted by the current user (status tracker). */
export async function getMyDataRequests(): Promise<DataRequestBrief[]> {
  return invoke<DataRequestBrief[]>('get_my_data_requests');
}

/** Get full detail for a single data request. */
export async function getDataRequestDetail(
  requestId: string,
): Promise<DataRequest> {
  return invoke<DataRequest>('get_data_request_detail', {
    requestId,
  });
}

/** Get the data response for a given request (if exists). */
export async function getDataResponse(
  requestId: string,
): Promise<DataResponse | null> {
  return invoke<DataResponse | null>('get_data_response', {
    requestId,
  });
}

// ── Data Analyst Only ─────────────────────────────────────────────────────────

/** UC-DA-01: Get approved data requests for the analyst inbox. */
export async function getAnalystInbox(): Promise<DataRequestBrief[]> {
  return invoke<DataRequestBrief[]>('get_analyst_inbox');
}

/** UC-DA-02: Start processing a data request (status → 'processing'). */
export async function processDataRequest(
  requestId: string,
): Promise<void> {
  await invoke('process_data_request', {
    payload: { request_id: requestId, action: 'start_processing' },
  });
}

/** UC-DA-03: Submit a data response (with optional spreadsheet file). */
export async function submitDataResponse(
  requestId: string,
  resultPayload?: Record<string, unknown>,
  fileBytes?: number[],
  filename?: string,
): Promise<string> {
  return invoke<string>('submit_data_response', {
    payload: {
      request_id: requestId,
      result_payload: resultPayload ?? null,
      analyst_notes: null,
    },
    fileBytes: fileBytes ?? null,
    filename: filename ?? null,
  });
}
