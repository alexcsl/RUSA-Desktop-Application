// stores/auth.ts — Central auth store for the Svelte frontend
// Source of truth: AUTH_GUIDE.md §6.1
//
// Session data is NEVER the source of truth — Rust AppState is.
// This store mirrors what Rust exposes via Tauri commands for UI purposes only.

import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface SessionUser {
  id: string;
  username: string;
  full_name: string;
  role: string;
  base_location_id: string | null;
  base_has_data_regulation: boolean;
}

export const currentUser = writable<SessionUser | null>(null);

// Derived booleans for common role checks — used in route guards
export const isAdmin = derived(currentUser, (u) => u?.role === 'Administrator');
export const isDirector = derived(currentUser, (u) =>
  DIRECTOR_ROLES.includes(u?.role ?? '')
);
export const isAuthenticated = derived(currentUser, (u) => u !== null);

const DIRECTOR_ROLES = [
  'GeneralDirector',
  'TheDirector',
  'TheAccountant',
  'TheLibrarian',
  'TheNomad',
  'TheArtificer',
  'TheObserver',
  'TheWanderer',
  'TheTaskmaster',
  'TheGuardian',
  'TheStatistician',
  'TheCoordinator',
  'TheOverseer',
  'TheAnchorman',
  'Administrator',
];

/** Call once on app boot to restore session state. */
export async function initSession(): Promise<void> {
  try {
    const session = await invoke<SessionUser | null>('get_current_session');
    currentUser.set(session);
  } catch {
    currentUser.set(null);
  }
}

export async function login(
  username: string,
  password: string
): Promise<void> {
  const user = await invoke<SessionUser>('login', {
    payload: { username, password },
  });
  currentUser.set(user);
}

export async function logout(): Promise<void> {
  await invoke('logout');
  currentUser.set(null);
}

// ── Account Management (TheDirector / Administrator only) ─────────────────────

export interface CreateAccountPayload {
  full_name: string;
  username: string;
  email: string | null;
  initial_password: string;
  role: string;
  base_location_id: string | null;
}

export interface CreatedUser {
  id: string;
  username: string;
  full_name: string;
  role: string;
}

/** Create a new personnel account. Only callable by TheDirector or Administrator. */
export async function createAccount(
  payload: CreateAccountPayload
): Promise<CreatedUser> {
  return await invoke<CreatedUser>('create_personnel_account', { payload });
}

/** Terminate (soft-delete) a personnel account. Only callable by TheAnchorman or Administrator. */
export async function terminateAccount(
  targetUserId: string
): Promise<void> {
  await invoke('terminate_personnel_account', { targetUserId });
}
