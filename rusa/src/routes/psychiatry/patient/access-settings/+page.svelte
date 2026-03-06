<!--
  /psychiatry/patient/access-settings — Patient Access Settings (UC-PAT-01)
  Any RUSA user under care can grant/deny assistant schedule access.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    psyGetAccessSettings,
    psyGrantScheduleAccess,
    type AccessSetting,
  } from '$lib/stores/psychiatry';

  let settings: AccessSetting[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let success = $state('');
  let toggling = $state(false);

  onMount(async () => { await loadSettings(); });

  async function loadSettings() {
    loading = true;
    try { settings = await psyGetAccessSettings(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    loading = false;
  }

  async function toggleAccess(s: AccessSetting) {
    toggling = true;
    error = '';
    success = '';
    try {
      await psyGrantScheduleAccess({ assistant_id: s.assistant_id, grant: !s.granted });
      success = `Access ${!s.granted ? 'granted to' : 'revoked from'} ${s.assistant_name}.`;
      await loadSettings();
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    toggling = false;
  }
</script>

<div class="page">
  <h2>Schedule Access Settings</h2>
  <p class="muted sub">Control which assistants can view and book from your schedule.</p>

  {#if success}<div class="toast-success">{success}</div>{/if}
  {#if error}<div class="toast-error">{error}</div>{/if}

  {#if loading}
    <p class="muted">Loading…</p>
  {:else if settings.length === 0}
    <div class="empty-state">
      <p class="muted">No access requests yet.</p>
      <p class="muted">When a psychiatrist assistant requests access to your schedule, it will appear here.</p>
    </div>
  {:else}
    <div class="list">
      {#each settings as s}
        <div class="row">
          <div class="info">
            <span class="aname">{s.assistant_name}</span>
            {#if s.granted}
              <span class="badge badge-granted">Granted</span>
            {:else}
              <span class="badge badge-pending">Pending</span>
            {/if}
            <span class="muted ts">{s.updated_at ? new Date(s.updated_at).toLocaleDateString() : ''}</span>
          </div>
          <button
            class={s.granted ? 'btn-danger-sm' : 'btn-grant-sm'}
            onclick={() => toggleAccess(s)}
            disabled={toggling}
          >
            {s.granted ? 'Revoke' : 'Grant Access'}
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page { max-width:700px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#3ABEFF;margin-bottom:0.2rem; }
  .sub { margin-bottom:1rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .empty-state { text-align:center;padding:2rem;background:#111827;border-radius:8px;border:1px solid rgba(58,190,255,0.08); }

  .list { display:flex;flex-direction:column;gap:0.3rem; }
  .row { display:flex;justify-content:space-between;align-items:center;padding:0.7rem 0.9rem;background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:6px; }
  .info { display:flex;align-items:center;gap:0.5rem; }
  .aname { font-size:0.85rem;color:#E6EDF3; }
  .ts { font-size:0.7rem; }
  .badge { padding:0.15rem 0.5rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .badge-granted { background:rgba(16,185,129,0.15);color:#34D399; }
  .badge-pending { background:rgba(245,158,11,0.15);color:#FBBF24; }

  .btn-grant-sm { background:rgba(16,185,129,0.1);color:#34D399;border:1px solid rgba(16,185,129,0.3);border-radius:4px;padding:0.3rem 0.7rem;cursor:pointer;font-size:0.75rem; }
  .btn-grant-sm:hover:not(:disabled) { background:rgba(16,185,129,0.2); }
  .btn-danger-sm { background:rgba(239,68,68,0.1);color:#EF4444;border:1px solid rgba(239,68,68,0.3);border-radius:4px;padding:0.3rem 0.7rem;cursor:pointer;font-size:0.75rem; }
  .btn-danger-sm:hover:not(:disabled) { background:rgba(239,68,68,0.2); }
  .btn-grant-sm:disabled, .btn-danger-sm:disabled { opacity:0.5;cursor:not-allowed; }

  .toast-success { background:rgba(16,185,129,0.1);border:1px solid rgba(16,185,129,0.3);color:#34D399;padding:0.5rem 0.8rem;border-radius:6px;font-size:0.8rem;margin-bottom:0.8rem; }
  .toast-error { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);color:#EF4444;padding:0.5rem 0.8rem;border-radius:6px;font-size:0.8rem;margin-bottom:0.8rem; }
</style>
