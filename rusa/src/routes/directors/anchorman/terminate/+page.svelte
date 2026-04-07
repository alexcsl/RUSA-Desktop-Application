<!--
  /directors/anchorman/terminate — Terminate Personnel (UC-ANC-03)
  TheAnchorman terminates a personnel account. Includes history view via activity log.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    terminatePersonnel, getPersonnelList, getPersonnelActivityLog,
    type PersonnelListItem, type PersonnelActivityEntry,
  } from '$lib/stores/directors';

  type Tab = 'terminate' | 'history';
  let activeTab = $state<Tab>('terminate');

  // Terminate form
  let personnel: PersonnelListItem[] = $state([]);
  let targetId = $state('');
  let reason = $state('');
  let effectiveDate = $state('');
  let confirming = $state(false);
  let loading = $state(false);
  let error = $state('');
  let success = $state('');

  // History
  let activityLog: PersonnelActivityEntry[] = $state([]);
  let historyLoading = $state(false);
  let historyError = $state('');

  onMount(async () => {
    try {
      personnel = await getPersonnelList();
    } catch (e) {
      error = String(e);
    }
  });

  async function loadHistory() {
    if (activityLog.length > 0) return;
    historyLoading = true;
    historyError = '';
    try {
      const all = await getPersonnelActivityLog();
      activityLog = all.filter(e => e.entry_type === 'termination');
    } catch (e) {
      historyError = String(e);
    } finally {
      historyLoading = false;
    }
  }

  function switchTab(tab: Tab) {
    activeTab = tab;
    if (tab === 'history') loadHistory();
  }

  function requestConfirm() {
    error = '';
    if (!targetId) { error = 'Select a personnel member.'; return; }
    if (!reason.trim()) { error = 'Reason is required.'; return; }
    if (!effectiveDate) { error = 'Effective date is required.'; return; }
    confirming = true;
  }

  async function handleTerminate() {
    loading = true; error = '';
    try {
      await terminatePersonnel({
        target_user_id: targetId,
        reason: reason.trim(),
        effective_date: effectiveDate,
      });
      const name = personnel.find(p => p.id === targetId)?.full_name ?? targetId.slice(0, 8) + '…';
      success = `${name} has been terminated.`;
      targetId = ''; reason = ''; effectiveDate = ''; confirming = false;
      // Reset history cache so it reloads next time
      activityLog = [];
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
      confirming = false;
    } finally {
      loading = false;
    }
  }

  function fmtDate(d: string | null) {
    if (!d) return '—';
    return new Date(d).toLocaleDateString();
  }
</script>

<div class="page">
  <h2>Staff Termination</h2>

  <div class="tabs">
    <button class="tab" class:active={activeTab === 'terminate'} onclick={() => switchTab('terminate')}>Terminate</button>
    <button class="tab" class:active={activeTab === 'history'} onclick={() => switchTab('history')}>Termination History</button>
  </div>

  {#if activeTab === 'terminate'}
    <p class="subtitle">Permanently deactivate a personnel account. This action is irreversible.</p>

    <div class="form-card">
      <label class="field">
        <span class="label">Personnel *</span>
        <select class="input" bind:value={targetId} disabled={confirming}>
          <option value="">— Select personnel —</option>
          {#each personnel as p}
            <option value={p.id}>{p.full_name} ({p.role_name})</option>
          {/each}
        </select>
      </label>

      <label class="field">
        <span class="label">Effective Date *</span>
        <input class="input" type="date" bind:value={effectiveDate} disabled={confirming} />
      </label>

      <label class="field">
        <span class="label">Reason *</span>
        <textarea class="textarea" bind:value={reason} rows="4" placeholder="Grounds for termination…" disabled={confirming}></textarea>
      </label>

      {#if error}<p class="msg error">{error}</p>{/if}
      {#if success}<p class="msg success">{success}</p>{/if}

      {#if !confirming}
        <button class="btn-danger" onclick={requestConfirm}>Terminate Personnel</button>
      {:else}
        <div class="confirm-box">
          <p class="confirm-text">⚠ Confirm termination of <strong>{personnel.find(p => p.id === targetId)?.full_name ?? '…'}</strong> effective {effectiveDate}?</p>
          <div class="confirm-actions">
            <button class="btn-cancel" onclick={() => (confirming = false)} disabled={loading}>Cancel</button>
            <button class="btn-danger" onclick={handleTerminate} disabled={loading}>
              {loading ? 'Terminating…' : 'Confirm Terminate'}
            </button>
          </div>
        </div>
      {/if}
    </div>

  {:else}
    {#if historyLoading}
      <p class="loading">Loading termination history…</p>
    {:else if historyError}
      <div class="banner error">{historyError}</div>
    {:else}
      <table class="tbl">
        <thead><tr><th>Personnel</th><th>Reason</th><th>Effective Date</th><th>Terminated By</th><th>Date</th></tr></thead>
        <tbody>
          {#each activityLog as e}
            <tr>
              <td>{e.target_name}</td>
              <td class="reason">{e.description}</td>
              <td>{fmtDate(e.effective_date)}</td>
              <td>{e.actor_name}</td>
              <td>{fmtDate(e.created_at)}</td>
            </tr>
          {/each}
          {#if activityLog.length === 0}
            <tr><td colspan="5" class="empty">No termination records found.</td></tr>
          {/if}
        </tbody>
      </table>
    {/if}
  {/if}
</div>

<style>
  .page { max-width:780px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#EF4444;margin:0 0 0.75rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1.2rem; }
  .tabs { display:flex;gap:0.4rem;margin-bottom:1rem; }
  .tab { padding:0.35rem 0.75rem;border:1px solid #374151;background:#1F2937;color:#94A3B8;border-radius:4px;cursor:pointer;font-size:0.78rem; }
  .tab.active { border-color:#EF4444;color:#EF4444;background:rgba(239,68,68,0.08); }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(239,68,68,0.15);border-radius:8px;padding:1.5rem;max-width:480px;display:flex;flex-direction:column;gap:0.75rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem 0.6rem;font-size:0.82rem;font-family:'Inter',sans-serif;width:100%;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .input:disabled,.textarea:disabled { opacity:0.5; }
  .textarea { resize:vertical; }
  .btn-danger { background:rgba(239,68,68,0.15);border:1px solid #EF4444;color:#EF4444;border-radius:6px;padding:0.55rem;cursor:pointer;font-size:0.85rem;font-weight:600; }
  .btn-danger:hover:not(:disabled) { background:rgba(239,68,68,0.25); }
  .btn-danger:disabled { opacity:0.5;cursor:default; }
  .btn-cancel { background:rgba(148,163,184,0.1);border:1px solid rgba(148,163,184,0.3);color:#94A3B8;border-radius:6px;padding:0.55rem 1rem;cursor:pointer;font-size:0.85rem; }
  .confirm-box { background:rgba(239,68,68,0.05);border:1px solid rgba(239,68,68,0.2);border-radius:6px;padding:0.75rem; }
  .confirm-text { color:#FCA5A5;font-size:0.82rem;margin:0 0 0.6rem; }
  .confirm-actions { display:flex;gap:0.5rem;justify-content:flex-end; }
  .msg { font-size:0.75rem;margin:0; }
  .msg.error { color:#EF4444; }
  .msg.success { color:#22C55E; }
  .banner { padding:0.6rem 1rem;border-radius:6px;font-size:0.8rem;margin-bottom:1rem; }
  .banner.error { background:rgba(239,68,68,0.15);color:#EF4444;border:1px solid rgba(239,68,68,0.3); }
  .loading { color:#94A3B8;font-size:0.85rem; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { color:#64748B;font-weight:500;text-align:left;padding:0.4rem 0.6rem;border-bottom:1px solid #1F2937; }
  .tbl td { color:#CBD5E1;padding:0.45rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .reason { max-width:240px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis; }
  .empty { color:#4B5563;font-style:italic;text-align:center; }
</style>
