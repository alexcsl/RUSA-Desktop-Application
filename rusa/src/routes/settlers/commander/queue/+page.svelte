<!--
  UC-SC-02: Incoming Queue — Review settler requests, anomalies, complaints
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    stlGetIncomingQueue,
    stlRejectIncoming,
    stlForwardToDirectors,
    type IncomingQueueItem,
  } from '$lib/stores/settlers';

  let items: IncomingQueueItem[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let actionMsg = $state('');
  let rejectingItem: IncomingQueueItem | null = $state(null);
  let rejectReason = $state('');

  async function load() {
    loading = true; error = '';
    try { items = await stlGetIncomingQueue(); }
    catch (e: any) { error = e?.message ?? String(e); }
    finally { loading = false; }
  }
  onMount(load);

  async function reject(item: IncomingQueueItem) {
    rejectingItem = item;
    rejectReason = '';
  }

  async function confirmReject() {
    if (!rejectingItem || !rejectReason.trim()) return;
    actionMsg = '';
    try {
      await stlRejectIncoming({ item_id: rejectingItem.id, item_type: rejectingItem.item_type, reason: rejectReason });
      actionMsg = `Rejected ${rejectingItem.item_type} ${rejectingItem.id.slice(0,8)}…`;
      rejectingItem = null;
      rejectReason = '';
      await load();
    } catch (e: any) { actionMsg = `Error: ${e?.message ?? e}`; }
  }

  function cancelReject() {
    rejectingItem = null;
    rejectReason = '';
  }

  async function forward(item: IncomingQueueItem) {
    actionMsg = '';
    try {
      await stlForwardToDirectors({ item_id: item.id, item_type: item.item_type });
      actionMsg = `Forwarded ${item.item_type} ${item.id.slice(0,8)}… to Directors.`;
      await load();
    } catch (e: any) { actionMsg = `Error: ${e?.message ?? e}`; }
  }
</script>

<h2>Incoming Queue</h2>

{#if actionMsg}<p class="ok">{actionMsg}</p>{/if}

{#if loading}
  <p class="dim">Loading…</p>
{:else if error}
  <p class="err">{error}</p>
{:else if items.length === 0}
  <p class="dim">No pending items in the queue.</p>
{:else}
  <table class="tbl">
    <thead>
      <tr>
        <th>Type</th><th>Submitted By</th><th>Summary</th><th>Date</th><th>Actions</th>
      </tr>
    </thead>
    <tbody>
      {#each items as it}
        <tr>
          <td><span class="badge type">{it.item_type}</span></td>
          <td>{it.submitted_by_name}</td>
          <td class="summary">{it.summary}</td>
          <td class="dim">{new Date(it.created_at).toLocaleDateString()}</td>
          <td class="actions">
            <button class="btn-sm fwd" onclick={() => forward(it)}>Forward ▶</button>
            <button class="btn-sm rej" onclick={() => reject(it)}>Reject ✕</button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

<!-- Reject confirmation dialog -->
{#if rejectingItem}
  <div class="modal-overlay" onclick={cancelReject} role="presentation">
    <div class="modal" onclick={(e) => e.stopPropagation()} role="dialog">
      <h3>Reject {rejectingItem.item_type}</h3>
      <p class="modal-sub">From: {rejectingItem.submitted_by_name}</p>
      <label class="modal-label">
        Reason *
        <textarea bind:value={rejectReason} rows="3" placeholder="Why are you rejecting this item?"></textarea>
      </label>
      <div class="modal-actions">
        <button class="btn-danger" disabled={!rejectReason.trim()} onclick={confirmReject}>Reject</button>
        <button class="btn-ghost" onclick={cancelReject}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
  .dim { color:#64748B;font-size:0.8rem; }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem;margin-bottom:0.5rem; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.4rem 0.6rem;border-bottom:1px solid #374151;color:#94A3B8;font-weight:500; }
  .tbl td { padding:0.4rem 0.6rem;border-bottom:1px solid rgba(55,65,81,0.4);vertical-align:middle; }
  .summary { max-width:280px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .badge { display:inline-block;padding:0.15rem 0.55rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .type { background:rgba(139,92,246,0.15);color:#A78BFA;text-transform:capitalize; }
  .actions { display:flex;gap:0.4rem; }
  .btn-sm { padding:0.25rem 0.6rem;border-radius:4px;font-size:0.7rem;font-weight:600;border:none;cursor:pointer; }
  .fwd { background:rgba(58,190,255,0.12);color:#3ABEFF; }
  .fwd:hover { background:rgba(58,190,255,0.25); }
  .rej { background:rgba(239,68,68,0.12);color:#EF4444; }
  .rej:hover { background:rgba(239,68,68,0.25); }
  .modal-overlay { position:fixed;inset:0;background:rgba(0,0,0,0.55);display:flex;align-items:center;justify-content:center;z-index:50; }
  .modal { background:#1F2937;border:1px solid #374151;border-radius:8px;padding:1.5rem;max-width:400px;width:90%; }
  .modal h3 { font-family:'Orbitron',sans-serif;color:#EF4444;font-size:0.9rem;margin:0 0 0.3rem;text-transform:capitalize; }
  .modal-sub { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .modal-label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8;margin-bottom:0.75rem; }
  .modal-label textarea { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem;resize:vertical; }
  .modal-actions { display:flex;gap:0.75rem; }
  .btn-danger { background:#EF4444;color:#fff;border:none;border-radius:4px;padding:0.45rem 1rem;cursor:pointer;font-weight:600;font-size:0.8rem; }
  .btn-danger:hover { background:#DC2626; }
  .btn-danger:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-ghost { background:none;border:1px solid #374151;color:#94A3B8;padding:0.45rem 0.9rem;border-radius:4px;cursor:pointer;font-size:0.8rem; }
</style>
