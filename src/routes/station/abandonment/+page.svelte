<!-- /station/abandonment — UC-SSS-07: Propose Station Abandonment -->
<script lang="ts">
  import { getContext } from 'svelte';
  import type { Writable } from 'svelte/store';
  import { sstProposeAbandonment, sstGetArchive, type ArchiveEntry } from '$lib/stores/space_station';
  import { onMount } from 'svelte';

  const stationId = getContext<Writable<string>>('selectedStationId');

  let reason = $state('');
  let findingRefId = $state('');
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  // Archive entries for reference picker
  let archiveEntries: ArchiveEntry[] = $state([]);
  let loadingArchive = $state(true);

  let currentStationId = $state('');
  const unsub = stationId.subscribe((v) => {
    currentStationId = v;
    if (v) loadArchive();
  });

  async function loadArchive() {
    if (!currentStationId) return;
    loadingArchive = true;
    try {
      archiveEntries = await sstGetArchive(currentStationId);
    } catch {}
    loadingArchive = false;
  }

  onMount(() => { if (currentStationId) loadArchive(); });

  async function handleSubmit() {
    error = ''; success = '';
    if (!currentStationId) { error = 'No station selected.'; return; }
    if (!reason.trim()) { error = 'Reason is required.'; return; }

    submitting = true;
    try {
      const voteId = await sstProposeAbandonment({
        station_id: currentStationId,
        finding_reference_id: findingRefId || undefined,
        reason,
      });
      success = `Abandonment proposal submitted. Vote session created (${voteId.slice(0, 8)}…). Directors will review.`;
      reason = ''; findingRefId = '';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<h1 class="title">Propose Station Abandonment</h1>
<p class="subtitle">
  Extraordinary circumstances? Submit an abandonment proposal to Directors for voting.
  Linking a finding/archive report is recommended.
</p>

<div class="form-card">
  <h2>⚠ Abandonment Request</h2>

  <label class="field"><span class="label">Linked Finding (Optional)</span>
    {#if loadingArchive}
      <select class="input" disabled><option>Loading archive…</option></select>
    {:else if archiveEntries.length === 0}
      <select class="input" disabled><option>No archive entries found</option></select>
    {:else}
      <select class="input" bind:value={findingRefId}>
        <option value="">— None —</option>
        {#each archiveEntries as entry}
          <option value={entry.id}>
            {entry.finding_type ?? 'Unknown'} — {String((entry.archive_data as Record<string, unknown>).description ?? '').slice(0, 60)}…
          </option>
        {/each}
      </select>
    {/if}
  </label>

  <label class="field"><span class="label">Reason *</span>
    <textarea class="textarea" bind:value={reason} rows="6" placeholder="Detailed explanation of why abandonment is necessary…"></textarea>
  </label>

  <div class="warning-box">
    <span class="warning-icon">⚠</span>
    <p>This action will create a vote session visible to all Directors. Station status will change to <strong>abandoned</strong> only upon approval.</p>
  </div>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}

  <button class="btn-danger-primary" onclick={handleSubmit} disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Abandonment Proposal'}
  </button>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem;max-width:650px; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(239,68,68,0.15);border-radius:8px;padding:1.25rem;max-width:650px;display:flex;flex-direction:column;gap:0.6rem; }
  .form-card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#EF4444;margin:0; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .warning-box { display:flex;gap:0.5rem;align-items:flex-start;background:rgba(239,68,68,0.08);border:1px solid rgba(239,68,68,0.2);border-radius:6px;padding:0.6rem; }
  .warning-icon { font-size:1.1rem; }
  .warning-box p { font-size:0.75rem;color:#E6EDF3;margin:0;line-height:1.4; }
  .warning-box strong { color:#EF4444; }
  .btn-danger-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#EF4444 0%,#B91C1C 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-danger-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
</style>
