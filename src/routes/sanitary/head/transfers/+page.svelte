<!-- /sanitary/head/transfers — UC-HS-06: Review Division Transfer Requests -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    sanGetTransferRequests, sanReviewTransferRequest,
    type TransferRequestRow,
  } from '$lib/stores/sanitary';

  let requests: TransferRequestRow[] = $state([]);
  let error = $state('');
  let success = $state('');

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = ''; success = '';
    try { requests = await sanGetTransferRequests(); } catch (e: unknown) { error = String(e); }
  }

  async function handleReview(id: string, decision: 'approved' | 'rejected') {
    error = ''; success = '';
    try {
      await sanReviewTransferRequest({ request_id: id, decision });
      success = `Request ${decision}.`;
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }

  function statusBadge(s: string): string {
    const m: Record<string, string> = { pending: 'badge-warn', approved: 'badge-ok', rejected: 'badge-err' };
    return m[s] ?? 'badge-default';
  }
</script>

<h1 class="title">Transfer Requests</h1>
<p class="subtitle">UC-HS-06 — Review and approve or reject staff division transfer requests.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

{#each requests as r}
  <div class="card">
    <div class="card-top">
      <span class="card-name">{r.requester_name ?? r.requested_by.slice(0, 8)}</span>
      <span class="badge {statusBadge(r.status)}">{r.status}</span>
    </div>
    <div class="card-body">
      <span class="dir">From: <strong>{r.from_division_name ?? '—'}</strong></span>
      <span class="arrow">→</span>
      <span class="dir">To: <strong>{r.to_division_name ?? '—'}</strong></span>
    </div>
    {#if r.reason}<p class="reason">{r.reason}</p>{/if}
    <p class="meta">Requested: {new Date(r.created_at).toLocaleDateString()}</p>
    {#if r.status === 'pending'}
      <div class="actions">
        <button class="btn-approve" onclick={() => handleReview(r.id, 'approved')}>Approve</button>
        <button class="btn-reject" onclick={() => handleReview(r.id, 'rejected')}>Reject</button>
      </div>
    {/if}
  </div>
{:else}
  <p class="empty">No transfer requests.</p>
{/each}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.75rem;margin-bottom:0.5rem; }
  .card-top { display:flex;justify-content:space-between;align-items:center;margin-bottom:0.3rem; }
  .card-name { font-size:0.88rem;font-weight:500;color:#E6EDF3; }
  .card-body { display:flex;gap:0.4rem;align-items:center;font-size:0.8rem;color:#CBD5E1; }
  .arrow { color:#F59E0B;font-weight:700; }
  .dir strong { color:#E6EDF3; }
  .reason { font-size:0.78rem;color:#94A3B8;margin:0.25rem 0 0;font-style:italic; }
  .meta { font-size:0.7rem;color:#64748B;margin:0.15rem 0 0; }
  .actions { display:flex;gap:0.4rem;margin-top:0.5rem; }
  .btn-approve { padding:0.35rem 0.6rem;background:rgba(16,185,129,0.15);border:1px solid #10B981;color:#10B981;border-radius:6px;cursor:pointer;font-size:0.75rem; }
  .btn-approve:hover { background:rgba(16,185,129,0.25); }
  .btn-reject { padding:0.35rem 0.6rem;background:rgba(239,68,68,0.15);border:1px solid #EF4444;color:#EF4444;border-radius:6px;cursor:pointer;font-size:0.75rem; }
  .btn-reject:hover { background:rgba(239,68,68,0.25); }
  .badge { padding:0.12rem 0.35rem;border-radius:4px;font-size:0.65rem;text-transform:capitalize; }
  .badge-warn { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .badge-ok { background:rgba(16,185,129,0.15);color:#10B981; }
  .badge-err { background:rgba(239,68,68,0.15);color:#EF4444; }
  .badge-default { background:rgba(148,163,184,0.15);color:#94A3B8; }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1.5rem; }
</style>
