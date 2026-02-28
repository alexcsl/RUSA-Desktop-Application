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

  async function load() {
    loading = true; error = '';
    try { items = await stlGetIncomingQueue(); }
    catch (e: any) { error = e?.message ?? String(e); }
    finally { loading = false; }
  }
  onMount(load);

  async function reject(item: IncomingQueueItem) {
    const reason = prompt('Rejection reason:');
    if (!reason) return;
    actionMsg = '';
    try {
      await stlRejectIncoming({ item_id: item.id, item_type: item.item_type, reason });
      actionMsg = `Rejected ${item.item_type} ${item.id.slice(0,8)}…`;
      await load();
    } catch (e: any) { actionMsg = `Error: ${e?.message ?? e}`; }
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
</style>
