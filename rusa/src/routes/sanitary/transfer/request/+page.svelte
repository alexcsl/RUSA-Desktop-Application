<!-- /sanitary/transfer/request — UC-STAS-03: Request Division Transfer -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    sanSubmitTransferRequest, sanGetMyTransfers, sanGetDivisions,
    type TransferRequestRow, type DivisionRow,
  } from '$lib/stores/sanitary';

  let transfers: TransferRequestRow[] = $state([]);
  let divisions: DivisionRow[] = $state([]);
  let error = $state('');
  let success = $state('');

  let fToDivision = $state('');
  let fReason = $state('');

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = '';
    try {
      [transfers, divisions] = await Promise.all([sanGetMyTransfers(), sanGetDivisions()]);
    } catch (e: unknown) { error = String(e); }
  }

  async function handleSubmit() {
    if (!fToDivision) { error = 'Select the target division.'; return; }
    error = ''; success = '';
    try {
      await sanSubmitTransferRequest({ to_division_id: fToDivision, reason: fReason.trim() || undefined });
      success = 'Transfer request submitted. Head of Sanitary will review.';
      fToDivision = ''; fReason = '';
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }

  function statusBadge(s: string): string {
    const m: Record<string, string> = { pending: 'badge-warn', approved: 'badge-ok', rejected: 'badge-err' };
    return m[s] ?? 'badge-default';
  }
</script>

<h1 class="title">Division Transfer Request</h1>
<p class="subtitle">UC-STAS-03 — Request a transfer to another sanitary division. Must be reviewed by Head.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="form-card">
  <div class="row">
    <div class="form-group">
      <label for="div">Target Division</label>
      <select id="div" class="input" bind:value={fToDivision}>
        <option value="">— Select —</option>
        {#each divisions as d}
          <option value={d.id}>{d.name} (quota: {d.quota})</option>
        {/each}
      </select>
    </div>
  </div>
  <div class="form-group" style="margin-top:0.4rem;">
    <label for="reason">Reason</label>
    <textarea id="reason" class="input" rows="3" bind:value={fReason} placeholder="Optional explanation…"></textarea>
  </div>
  <button class="btn-primary" style="margin-top:0.5rem;" onclick={handleSubmit}>Submit Request</button>
</div>

<h2 class="section-title">My Transfer History</h2>
{#each transfers as t}
  <div class="card">
    <div class="card-top">
      <span class="dir">
        {t.from_division_name ?? '—'} <span class="arrow">→</span> {t.to_division_name ?? '—'}
      </span>
      <span class="badge {statusBadge(t.status)}">{t.status}</span>
    </div>
    {#if t.reason}<p class="reason">{t.reason}</p>{/if}
    <p class="meta">{new Date(t.created_at).toLocaleDateString()}</p>
  </div>
{:else}
  <p class="empty">No transfer requests yet.</p>
{/each}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .section-title { font-family:'Orbitron',sans-serif;font-size:0.95rem;color:#3ABEFF;margin:1.25rem 0 0.5rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-bottom:1rem; }
  .row { display:flex;gap:0.6rem;align-items:flex-end;flex-wrap:wrap; }
  .form-group { display:flex;flex-direction:column;gap:0.15rem;flex:1;min-width:160px; }
  .form-group label { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  select.input { cursor:pointer; }
  textarea.input { resize:vertical; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.7rem;margin-bottom:0.4rem; }
  .card-top { display:flex;justify-content:space-between;align-items:center; }
  .dir { font-size:0.82rem;color:#CBD5E1; }
  .arrow { color:#F59E0B;font-weight:700; }
  .reason { font-size:0.78rem;color:#94A3B8;margin:0.2rem 0 0;font-style:italic; }
  .meta { font-size:0.7rem;color:#64748B;margin:0.1rem 0 0; }
  .badge { padding:0.12rem 0.35rem;border-radius:4px;font-size:0.65rem;text-transform:capitalize; }
  .badge-warn { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .badge-ok { background:rgba(16,185,129,0.15);color:#10B981; }
  .badge-err { background:rgba(239,68,68,0.15);color:#EF4444; }
  .badge-default { background:rgba(148,163,184,0.15);color:#94A3B8; }
  .btn-primary { padding:0.45rem 0.75rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
</style>
