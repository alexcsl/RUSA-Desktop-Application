<!--
  /admin/personnel — Enhanced Personnel Management
  Full user directory with search, status toggling, password reset.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getAllUsers,
    toggleUserStatus,
    resetUserPassword,
    type AdminUserEntry,
  } from '$lib/stores/administrator';

  let users: AdminUserEntry[] = $state([]);
  let filtered: AdminUserEntry[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  /* Filters */
  let search = $state('');
  let roleFilter = $state('');
  let statusFilter = $state('all'); // all | active | deactivated | terminated
  let includeTerminated = $state(true);

  /* Modals */
  let resetTarget: AdminUserEntry | null = $state(null);
  let newPassword = $state('');
  let resetBusy = $state(false);
  let resetError = $state('');
  let resetDone = $state(false);

  let toggleBusy = $state<string | null>(null);

  onMount(loadUsers);

  async function loadUsers() {
    loading = true;
    error = '';
    try {
      users = await getAllUsers(includeTerminated);
      applyFilter();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function applyFilter() {
    let list = users;
    const q = search.toLowerCase();
    if (q) list = list.filter(u => u.full_name.toLowerCase().includes(q) || u.username.toLowerCase().includes(q) || u.role_name.toLowerCase().includes(q));
    if (roleFilter) list = list.filter(u => u.role_name === roleFilter);
    if (statusFilter === 'active') list = list.filter(u => u.is_active && !u.deleted_at);
    else if (statusFilter === 'deactivated') list = list.filter(u => !u.is_active && !u.deleted_at);
    else if (statusFilter === 'terminated') list = list.filter(u => u.deleted_at);
    filtered = list;
  }

  /* Reactive filter */
  $effect(() => { search; roleFilter; statusFilter; applyFilter(); });

  /* Unique roles for filter dropdown */
  function uniqueRoles(): string[] {
    const s = new Set(users.map(u => u.role_name));
    return [...s].sort();
  }

  /* Status badge */
  function badge(u: AdminUserEntry): { label:string;color:string } {
    if (u.deleted_at) return { label: 'Terminated', color: '#EF4444' };
    if (!u.is_active) return { label: 'Deactivated', color: '#F59E0B' };
    return { label: 'Active', color: '#10B981' };
  }

  /* Toggle activate/deactivate */
  async function handleToggle(u: AdminUserEntry) {
    if (u.deleted_at) return; // cannot toggle terminated
    toggleBusy = u.id;
    try {
      await toggleUserStatus(u.id, !u.is_active);
      await loadUsers();
    } catch (e: unknown) {
      alert(e instanceof Error ? e.message : String(e));
    } finally {
      toggleBusy = null;
    }
  }

  /* Reset password modal */
  function openReset(u: AdminUserEntry) {
    resetTarget = u;
    newPassword = '';
    resetError = '';
    resetDone = false;
  }

  async function submitReset() {
    if (!resetTarget) return;
    if (newPassword.length < 8) { resetError = 'Minimum 8 characters.'; return; }
    resetBusy = true;
    resetError = '';
    try {
      await resetUserPassword(resetTarget.id, newPassword);
      resetDone = true;
    } catch (e: unknown) {
      resetError = e instanceof Error ? e.message : String(e);
    } finally {
      resetBusy = false;
    }
  }

  function closeReset() { resetTarget = null; }
</script>

<div class="personnel-page">
  <div class="header-row">
    <h1 class="title">Personnel Management</h1>
    <span class="count">{filtered.length} / {users.length} users</span>
  </div>

  <!-- Filter Bar -->
  <div class="filter-bar">
    <input class="input grow" placeholder="Search name, username, or role…" bind:value={search} />
    <select class="input" bind:value={roleFilter}>
      <option value="">All Roles</option>
      {#each uniqueRoles() as r}
        <option value={r}>{r}</option>
      {/each}
    </select>
    <select class="input" bind:value={statusFilter}>
      <option value="all">All Statuses</option>
      <option value="active">Active</option>
      <option value="deactivated">Deactivated</option>
      <option value="terminated">Terminated</option>
    </select>
    <label class="check-label"><input type="checkbox" bind:checked={includeTerminated} onchange={loadUsers} /> Include terminated</label>
  </div>

  {#if loading}
    <p class="hint">Loading personnel…</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else}
    <table class="table">
      <thead>
        <tr>
          <th>Name</th><th>Username</th><th>Role</th><th>Location</th><th>Status</th><th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each filtered as u}
          {@const b = badge(u)}
          <tr class="row" class:deactivated={!u.is_active && !u.deleted_at} class:terminated={!!u.deleted_at}>
            <td class="cell name">{u.full_name}</td>
            <td class="cell user">{u.username}</td>
            <td class="cell role">{u.role_name}</td>
            <td class="cell loc">{u.base_location_name ?? '—'}</td>
            <td class="cell"><span class="badge" style="color:{b.color};border-color:{b.color}">{b.label}</span></td>
            <td class="cell actions">
              {#if !u.deleted_at}
                <button
                  class="act-btn"
                  class:deact={u.is_active}
                  class:act={!u.is_active}
                  disabled={toggleBusy === u.id}
                  onclick={() => handleToggle(u)}
                >
                  {u.is_active ? 'Deactivate' : 'Activate'}
                </button>
                <button class="act-btn reset" onclick={() => openReset(u)}>Reset PW</button>
              {:else}
                <span class="muted-txt">N/A</span>
              {/if}
            </td>
          </tr>
        {:else}
          <tr><td colspan="6" class="empty">No personnel match your filters.</td></tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<!-- Password Reset Modal -->
{#if resetTarget}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="overlay" onclick={closeReset}>
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <h2 class="modal-title">Reset Password</h2>
      <p class="modal-sub">For <strong>{resetTarget.full_name}</strong> ({resetTarget.username})</p>
      {#if resetDone}
        <p class="success">Password updated successfully. The user's sessions have been invalidated.</p>
        <button class="btn" onclick={closeReset}>Close</button>
      {:else}
        <label class="field-label">New Password
          <input class="input full" type="password" placeholder="Min 8 characters" bind:value={newPassword} />
        </label>
        {#if resetError}<p class="field-error">{resetError}</p>{/if}
        <div class="modal-actions">
          <button class="btn" disabled={resetBusy} onclick={submitReset}>{resetBusy ? 'Saving…' : 'Reset Password'}</button>
          <button class="btn btn-ghost" onclick={closeReset}>Cancel</button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .personnel-page { max-width:1200px; }
  .header-row { display:flex;align-items:baseline;gap:0.75rem;margin-bottom:0.75rem; }
  .title { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#EF4444;margin:0; }
  .count { color:#94A3B8;font-size:0.78rem; }

  .filter-bar { display:flex;flex-wrap:wrap;gap:0.5rem;margin-bottom:0.75rem;align-items:center; }
  .input { background:#0E1428;border:1px solid rgba(239,68,68,0.18);border-radius:6px;color:#E6EDF3;font-size:0.78rem;padding:0.4rem 0.6rem;outline:none; }
  .input:focus { border-color:#EF4444; }
  .input.full { width:100%; }
  .grow { flex:1;min-width:180px; }
  .check-label { color:#94A3B8;font-size:0.75rem;display:flex;align-items:center;gap:0.3rem;cursor:pointer; }

  .btn { background:rgba(239,68,68,0.15);color:#EF4444;border:1px solid rgba(239,68,68,0.25);border-radius:6px;padding:0.4rem 0.9rem;cursor:pointer;font-size:0.78rem;font-weight:600;transition:background 0.2s; }
  .btn:hover { background:rgba(239,68,68,0.25); }
  .btn-ghost { background:transparent;color:#94A3B8;border-color:rgba(148,163,184,0.2); }
  .btn-ghost:hover { background:rgba(148,163,184,0.08); }
  .btn:disabled { opacity:0.4;cursor:default; }

  .hint { color:#94A3B8;font-size:0.82rem; }
  .error { color:#EF4444;font-size:0.82rem; }

  .table { width:100%;border-collapse:collapse; }
  th { text-align:left;font-size:0.68rem;color:#475569;text-transform:uppercase;letter-spacing:0.03em;padding:0.5rem 0.4rem;border-bottom:1px solid rgba(255,255,255,0.06); }
  .row { transition:background 0.15s; }
  .row:hover { background:rgba(239,68,68,0.04); }
  .row.deactivated { opacity:0.65; }
  .row.terminated { opacity:0.4; }
  .cell { padding:0.4rem;font-size:0.78rem;border-bottom:1px solid rgba(255,255,255,0.03); }
  .name { color:#E6EDF3;font-weight:600; }
  .user { color:#94A3B8;font-family:monospace;font-size:0.72rem; }
  .role { color:#C084FC; }
  .loc { color:#94A3B8; }
  .badge { font-size:0.68rem;border:1px solid;border-radius:10px;padding:0.1rem 0.5rem;font-weight:600; }
  .actions { display:flex;gap:0.35rem; }
  .act-btn { font-size:0.7rem;padding:0.25rem 0.55rem;border-radius:5px;cursor:pointer;border:1px solid;font-weight:600;transition:background 0.2s; }
  .act-btn.deact { background:rgba(239,68,68,0.1);color:#EF4444;border-color:rgba(239,68,68,0.25); }
  .act-btn.deact:hover { background:rgba(239,68,68,0.2); }
  .act-btn.act { background:rgba(16,185,129,0.1);color:#10B981;border-color:rgba(16,185,129,0.25); }
  .act-btn.act:hover { background:rgba(16,185,129,0.2); }
  .act-btn.reset { background:rgba(139,92,246,0.1);color:#8B5CF6;border-color:rgba(139,92,246,0.25); }
  .act-btn.reset:hover { background:rgba(139,92,246,0.2); }
  .act-btn:disabled { opacity:0.4;cursor:default; }
  .muted-txt { color:#475569;font-size:0.72rem; }
  .empty { text-align:center;color:#475569;font-size:0.8rem;padding:2rem 0; }

  /* Modal overlay */
  .overlay { position:fixed;inset:0;background:rgba(0,0,0,0.65);display:flex;align-items:center;justify-content:center;z-index:200; }
  .modal { background:#111827;border:1px solid rgba(239,68,68,0.2);border-radius:10px;padding:1.5rem 1.8rem;width:380px;max-width:90vw; }
  .modal-title { font-family:'Orbitron',sans-serif;font-size:0.95rem;color:#EF4444;margin:0 0 0.3rem; }
  .modal-sub { font-size:0.78rem;color:#94A3B8;margin:0 0 0.75rem; }
  .field-label { display:flex;flex-direction:column;gap:0.25rem;font-size:0.75rem;color:#94A3B8;margin-bottom:0.5rem; }
  .field-error { color:#EF4444;font-size:0.75rem;margin:0 0 0.5rem; }
  .success { color:#10B981;font-size:0.8rem;margin-bottom:0.5rem; }
  .modal-actions { display:flex;gap:0.5rem; }
</style>
