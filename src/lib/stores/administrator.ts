// stores/administrator.ts — Administrator-exclusive invoke wrappers
// Source of truth: 12_ADMINISTRATOR.md
//
// These functions call admin-only Tauri commands.  Other admin actions
// (account creation, termination, voting override) live in auth.ts and directors.ts
// since those commands are shared with Director-level roles.

import { invoke } from '@tauri-apps/api/core';

// ── Types ─────────────────────────────────────────────────────────────────────

export interface SystemStats {
  total_users: number;
  active_users: number;
  deactivated_users: number;
  terminated_users: number;
  total_roles: number;
  open_vote_sessions: number;
  decided_vote_sessions: number;
  overridden_vote_sessions: number;
  terminated_vote_sessions: number;
  total_audit_entries: number;
  recent_audit_entries_24h: number;
  pending_requests: number;
  active_meetings: number;
  total_notifications_unread: number;
  total_base_locations: number;
}

export interface AuditLogEntry {
  id: string;
  table_name: string;
  record_id: string;
  operation: string;
  performed_by: string | null;
  performer_name: string | null;
  performed_at: string;
  before_data: Record<string, unknown> | null;
  after_data: Record<string, unknown> | null;
}

export interface AuditLogPage {
  entries: AuditLogEntry[];
  total_count: number;
  page: number;
  page_size: number;
}

export interface AuditLogFilter {
  page?: number;
  page_size?: number;
  table_name?: string | null;
  operation?: string | null;
  performer_id?: string | null;
  date_from?: string | null;
  date_to?: string | null;
}

export interface AdminUserEntry {
  id: string;
  username: string;
  full_name: string;
  email: string | null;
  role_name: string;
  base_location_name: string | null;
  is_active: boolean;
  created_at: string;
  updated_at: string | null;
  deleted_at: string | null;
}

export interface RecentActivity {
  id: string;
  table_name: string;
  operation: string;
  performer_name: string | null;
  performed_at: string;
  summary: string | null;
}

export interface RoleEntry {
  id: string;
  name: string;
  description: string | null;
}

export interface BaseLocationEntry {
  id: string;
  name: string;
  has_data_regulation: boolean;
}

// ── Dashboard ─────────────────────────────────────────────────────────────────

/** Fetch aggregate system statistics for the admin dashboard. */
export async function getSystemStats(): Promise<SystemStats> {
  return await invoke<SystemStats>('admin_get_system_stats');
}

/** Fetch recent activity feed for the dashboard. */
export async function getRecentActivity(limit?: number): Promise<RecentActivity[]> {
  return await invoke<RecentActivity[]>('admin_get_recent_activity', { limit: limit ?? null });
}

// ── Audit Log ─────────────────────────────────────────────────────────────────

/** Fetch paginated, filterable audit log. */
export async function getAuditLog(filter: AuditLogFilter): Promise<AuditLogPage> {
  return await invoke<AuditLogPage>('admin_get_audit_log', { filter });
}

// ── User Directory ────────────────────────────────────────────────────────────

/** Fetch all users (optionally including soft-deleted/terminated). */
export async function getAllUsers(includeTerminated?: boolean): Promise<AdminUserEntry[]> {
  return await invoke<AdminUserEntry[]>('admin_get_all_users', {
    includeTerminated: includeTerminated ?? false,
  });
}

/** Toggle a user's is_active status. */
export async function toggleUserStatus(targetUserId: string, isActive: boolean): Promise<void> {
  await invoke('admin_toggle_user_status', { targetUserId, isActive });
}

/** Reset any user's password. */
export async function resetUserPassword(targetUserId: string, newPassword: string): Promise<void> {
  await invoke('admin_reset_password', {
    payload: { target_user_id: targetUserId, new_password: newPassword },
  });
}

// ── Lookups ───────────────────────────────────────────────────────────────────

/** Fetch all roles from the database. */
export async function getRoles(): Promise<RoleEntry[]> {
  return await invoke<RoleEntry[]>('admin_get_roles');
}

/** Fetch all base locations. */
export async function getBaseLocations(): Promise<BaseLocationEntry[]> {
  return await invoke<BaseLocationEntry[]>('admin_get_base_locations');
}
