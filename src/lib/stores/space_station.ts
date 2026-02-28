// stores/space_station.ts — Frontend API for Space Station Settlers subsystem
// Source of truth: 08_SPACE_STATION_SETTLERS.md
// All business logic lives in Rust — this file only mirrors the Tauri command interface.

import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────────────────────

export interface StationSummary {
  id: string;
  name: string;
  sector: string;
  status: string;
  nearest_galactic_security_team_id: string | null;
  created_at: string;
}

export interface InventoryItem {
  id: string;
  station_id: string;
  item_name: string;
  category: string | null;
  quantity: number;
  unit: string | null;
  updated_at: string;
}

export interface ArchiveEntry {
  id: string;
  station_id: string;
  logged_by: string;
  logger_name: string;
  finding_type: string | null;
  archive_data: Record<string, unknown>;
  created_at: string;
}

export interface SupplyRequestSummary {
  id: string;
  station_id: string;
  submitted_by: string;
  submitter_name: string;
  items: Record<string, unknown>[];
  status: string;
  vote_session_id: string | null;
  created_at: string;
}

export interface StationMapSummary {
  id: string;
  station_id: string;
  image_storage_path: string;
  image_width: number | null;
  image_height: number | null;
  is_published: boolean;
  published_at: string | null;
  created_at: string;
}

export interface MapAnnotation {
  id: string;
  map_id: string;
  label: string;
  description: string | null;
  x_position: number;
  y_position: number;
}

export interface PublishedMapData {
  signed_url: string;
  image_width: number | null;
  image_height: number | null;
  annotations: MapAnnotation[];
  station_name: string;
}

export interface PersonnelLogEntry {
  id: string;
  station_id: string;
  person_name: string;
  role: string | null;
  arrived_at: string;
  departed_at: string | null;
  logged_by: string;
  created_at: string;
}

// ── UC-SSS-01: Report Finding to Galactic Security ───────────────────────────

export async function sstReportFinding(payload: {
  station_id: string;
  incident_type: string;
  location: string;
  occurred_at?: string;
  description: string;
  severity: string;
  recommended_action?: string;
  sector_or_base?: string;
  incident_meta?: Record<string, unknown>;
}): Promise<string> {
  return invoke<string>('sst_report_finding_to_security', { payload });
}

// ── UC-SSS-02: Add Finding to Private Station Archive ────────────────────────

export async function sstAddToArchive(payload: {
  station_id: string;
  finding_type: string;
  archive_data: Record<string, unknown>;
}): Promise<string> {
  return invoke<string>('sst_add_to_archive', { payload });
}

// ── UC-SSS-08: View Private Station Archive ──────────────────────────────────

export async function sstGetArchive(
  stationId: string,
  search?: string,
): Promise<ArchiveEntry[]> {
  return invoke<ArchiveEntry[]>('sst_get_archive', {
    stationId,
    search: search ?? null,
  });
}

// ── UC-SSS-03: Manage Station Inventory ──────────────────────────────────────

export async function sstManageInventory(payload: {
  station_id: string;
  item_name: string;
  category?: string;
  quantity_change: number;
  unit?: string;
}): Promise<void> {
  await invoke('sst_manage_inventory', { payload });
}

export async function sstGetInventory(
  stationId: string,
): Promise<InventoryItem[]> {
  return invoke<InventoryItem[]>('sst_get_inventory', { stationId });
}

// ── UC-SSS-04: Submit Supply Request ─────────────────────────────────────────

export async function sstSubmitSupplyRequest(payload: {
  station_id: string;
  items: Record<string, unknown>[];
}): Promise<string> {
  return invoke<string>('sst_submit_supply_request', { payload });
}

export async function sstGetSupplyRequests(
  stationId: string,
): Promise<SupplyRequestSummary[]> {
  return invoke<SupplyRequestSummary[]>('sst_get_supply_requests', { stationId });
}

// ── UC-SSS-05: Upload and Annotate Station Map ──────────────────────────────

export async function sstUploadMap(
  stationId: string,
  fileBytes: number[],
  filename: string,
  contentType: string,
  imageWidth?: number,
  imageHeight?: number,
): Promise<string> {
  return invoke<string>('sst_upload_map', {
    stationId,
    fileBytes,
    filename,
    contentType,
    imageWidth: imageWidth ?? null,
    imageHeight: imageHeight ?? null,
  });
}

export async function sstGetMaps(
  stationId: string,
): Promise<StationMapSummary[]> {
  return invoke<StationMapSummary[]>('sst_get_maps', { stationId });
}

export async function sstPublishMap(mapId: string): Promise<void> {
  await invoke('sst_publish_map', { mapId });
}

export async function sstAddAnnotation(payload: {
  map_id: string;
  label: string;
  description?: string;
  x_position: number;
  y_position: number;
}): Promise<string> {
  return invoke<string>('sst_add_annotation', { payload });
}

export async function sstDeleteAnnotation(annotationId: string): Promise<void> {
  await invoke('sst_delete_annotation', { annotationId });
}

export async function sstGetAnnotations(
  mapId: string,
): Promise<MapAnnotation[]> {
  return invoke<MapAnnotation[]>('sst_get_annotations', { mapId });
}

// ── UC-SSV-01: View Published Station Map (No auth — Visitor) ────────────────

export async function sstGetPublishedMap(
  stationId: string,
): Promise<PublishedMapData> {
  return invoke<PublishedMapData>('sst_get_published_map', { stationId });
}

// ── UC-SSS-06: Log Personnel On Board ────────────────────────────────────────

export async function sstLogArrival(payload: {
  station_id: string;
  person_name: string;
  role?: string;
  arrived_at: string;
}): Promise<string> {
  return invoke<string>('sst_log_arrival', { payload });
}

export async function sstLogDeparture(
  logId: string,
  departedAt: string,
): Promise<void> {
  await invoke('sst_log_departure', { logId, departedAt });
}

export async function sstGetPersonnel(
  stationId: string,
  showAll?: boolean,
): Promise<PersonnelLogEntry[]> {
  return invoke<PersonnelLogEntry[]>('sst_get_personnel', {
    stationId,
    showAll: showAll ?? null,
  });
}

// ── UC-SSS-07: Propose Station Abandonment ───────────────────────────────────

export async function sstProposeAbandonment(payload: {
  station_id: string;
  finding_reference_id?: string;
  reason: string;
}): Promise<string> {
  return invoke<string>('sst_propose_abandonment', { payload });
}

// ── Station List ─────────────────────────────────────────────────────────────

export async function sstGetStations(): Promise<StationSummary[]> {
  return invoke<StationSummary[]>('sst_get_stations');
}

export async function sstGetPublicStations(): Promise<StationSummary[]> {
  return invoke<StationSummary[]>('sst_get_public_stations');
}
