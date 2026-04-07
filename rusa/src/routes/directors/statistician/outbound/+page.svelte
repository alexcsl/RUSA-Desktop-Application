<!--
  /directors/statistician/outbound — Outbound Data Response Review (UC-STAT-03)
  TheStatistician reviews outbound data responses before they are released.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getOutboundReviewQueue,
    reviewOutboundDataResponse,
    type OutboundResponseSummary,
  } from '$lib/stores/directors';

  let queue: OutboundResponseSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let reviewing: string | null = $state(null);
  let reviewNotes = $state('');
  let actionMsg = $state('');

  onMount(async () => { await loadQueue(); });

  async function loadQueue() {
    loading = true; error = '';
    try { queue = await getOutboundReviewQueue(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  }

  async function decide(id: string, decision: 'approved' | 'rejected') {
    actionMsg = '';
    try {
      await reviewOutboundDataResponse({ response_id: id, decision, notes: reviewNotes.trim() || undefined });
      actionMsg = `Response ${decision}.`;
      reviewing = null; reviewNotes = '';
      await loadQueue();
    } catch (e: unknown) { actionMsg = e instanceof Error ? e.message : String(e); }
  }

  function formatDate(d: string) { return new Date(d).toLocaleString(); }
</script>

<h1 class="title">Outbound Response Queue</h1>
<p class="subtitle">Review data analyst responses before they are released externally.</p>

{#if actionMsg}<p class="action-msg">{actionMsg}</p>{/if}

{#if loading}<p class="muted">Loading…</p>
{:else if error}<p class="error">{error}</p>
{:else if queue.length === 0}<p class="muted">Queue is empty.</p>
{:else}
  {#each queue as item}
    <div class="card">
      <div class="card-header">
        <span class="card-title">{item.subject ?? 'Data Response'}</span>
        <span class="date-label">{formatDate(item.created_at)}</span>
      </div>
      <p class="meta">Analyst: <strong>{item.analyst_name}</strong></p>
      {#if item.summary}<p class="body">{item.summary}</p>{/if}

      {#if reviewing === item.id}
        <label class="field">
          <span class="label">Review Notes (optional)</span>
          <textarea class="textarea" bind:value={reviewNotes} rows="2" placeholder="Optional notes…"></textarea>
        </label>
        <div class="action-row">
          <button class="btn-cancel-sm" onclick={() => { reviewing = null; reviewNotes = ''; }}>Cancel</button>
          <button class="btn-reject" onclick={() => decide(item.id, 'rejected')}>Reject</button>
          <button class="btn-approve" onclick={() => decide(item.id, 'approved')}>Approve</button>
        </div>
      {:else}
        <button class="btn-review" onclick={() => { reviewing = item.id; reviewNotes = ''; }}>Review</button>
      {/if}
    </div>
  {/each}
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .action-msg { font-size:0.78rem;color:#22C55E;margin-bottom:0.5rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.9rem;margin-bottom:0.6rem;display:flex;flex-direction:column;gap:0.35rem; }
  .card-header { display:flex;justify-content:space-between;align-items:center; }
  .card-title { font-size:0.88rem;font-weight:600;color:#E6EDF3; }
  .date-label { font-size:0.68rem;color:#475569; }
  .meta { font-size:0.72rem;color:#64748B;margin:0; }
  .body { font-size:0.8rem;color:#94A3B8;margin:0;white-space:pre-wrap; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.68rem;color:#64748B; }
  .textarea { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem;font-size:0.8rem;font-family:'Inter',sans-serif;width:100%;box-sizing:border-box;resize:vertical; }
  .textarea:focus { outline:none;border-color:#3ABEFF; }
  .action-row { display:flex;gap:0.4rem;justify-content:flex-end;margin-top:0.2rem; }
  .btn-review { align-self:flex-start;background:rgba(58,190,255,0.1);border:1px solid rgba(58,190,255,0.3);color:#3ABEFF;border-radius:5px;padding:0.28rem 0.65rem;cursor:pointer;font-size:0.75rem; }
  .btn-approve { background:rgba(34,197,94,0.12);border:1px solid rgba(34,197,94,0.35);color:#22C55E;border-radius:5px;padding:0.3rem 0.7rem;cursor:pointer;font-size:0.75rem; }
  .btn-reject { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);color:#EF4444;border-radius:5px;padding:0.3rem 0.7rem;cursor:pointer;font-size:0.75rem; }
  .btn-cancel-sm { background:transparent;border:1px solid rgba(148,163,184,0.2);color:#94A3B8;border-radius:5px;padding:0.3rem 0.6rem;cursor:pointer;font-size:0.75rem; }
  .error { color:#EF4444;font-size:0.75rem; }
</style>
