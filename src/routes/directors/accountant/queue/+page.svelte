<!-- /directors/accountant/queue — TheAccountant Financial Monitoring (UC-DIR-11) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getFinancialQueue, flagBudgetReport, type FinancialDocument } from '$lib/stores/directors';

  let queue: FinancialDocument[] = $state([]);
  let selected: FinancialDocument | null = $state(null);
  let flagReason = $state('');
  let error = $state('');
  let success = $state('');

  onMount(async () => { queue = await getFinancialQueue(); });

  async function handleFlag() {
    if (!selected || !flagReason.trim()) { error = 'Reason required.'; return; }
    error = ''; success = '';
    try {
      await flagBudgetReport(selected.id, flagReason);
      success = `Flagged "${selected.title}" — vote session created.`;
      flagReason = '';
      selected = null;
      queue = await getFinancialQueue();
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }
</script>

<h1 class="title">Financial Monitoring Queue</h1>
<p class="subtitle">Review budget/financial requests. Flag discrepancies to trigger a Director vote.</p>

<div class="grid">
  <div class="list-panel">
    {#each queue as doc}
      <button class="card" class:selected={selected?.id === doc.id} onclick={() => { selected = doc; error = ''; success = ''; }}>
        <div class="card-title">{doc.title}</div>
        <div class="card-meta">
          <span class="badge badge-{doc.status}">{doc.status}</span>
          <span>{doc.requester_name}</span>
        </div>
      </button>
    {:else}
      <p class="empty">No financial documents in queue.</p>
    {/each}
  </div>

  <div class="detail-panel">
    {#if selected}
      <h2>{selected.title}</h2>
      <div class="kv"><span class="k">Type:</span><span>{selected.type_}</span></div>
      <div class="kv"><span class="k">Requester:</span><span>{selected.requester_name}</span></div>
      <div class="kv"><span class="k">Status:</span><span class="badge badge-{selected.status}">{selected.status}</span></div>
      <div class="kv"><span class="k">Date:</span><span>{new Date(selected.created_at).toLocaleDateString()}</span></div>

      <h3>Payload</h3>
      <pre class="json">{JSON.stringify(selected.payload, null, 2)}</pre>

      {#if selected.status === 'pending'}
        <div class="flag-form">
          <h3>Flag Discrepancy</h3>
          <textarea class="textarea" placeholder="Describe the issue..." bind:value={flagReason} rows="3"></textarea>
          {#if error}<p class="error">{error}</p>{/if}
          {#if success}<p class="success">{success}</p>{/if}
          <button class="btn-warn" onclick={handleFlag}>Flag & Create Vote</button>
        </div>
      {/if}
    {:else}
      <div class="empty-state"><p>Select a document to review.</p></div>
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .grid { display:flex;gap:1rem;flex:1;overflow:hidden; }
  .list-panel { width:320px;overflow-y:auto; }
  .detail-panel { flex:1;overflow-y:auto; }
  .card { width:100%;text-align:left;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.7rem;margin-bottom:0.4rem;cursor:pointer;color:#E6EDF3; }
  .card:hover { border-color:rgba(58,190,255,0.3); }
  .card.selected { border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .card-title { font-size:0.85rem;font-weight:500;margin-bottom:0.2rem; }
  .card-meta { display:flex;justify-content:space-between;font-size:0.7rem;color:#94A3B8; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1rem;color:#3ABEFF;margin:0 0 0.75rem; }
  h3 { font-size:0.85rem;color:#E6EDF3;margin:0.75rem 0 0.4rem; }
  .kv { display:flex;gap:0.5rem;font-size:0.8rem;margin-bottom:0.3rem; }
  .k { color:#94A3B8;min-width:80px; }
  .json { background:#0E1428;border:1px solid rgba(58,190,255,0.1);border-radius:6px;padding:0.75rem;font-size:0.75rem;color:#94A3B8;overflow-x:auto;white-space:pre-wrap; }
  .flag-form { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-top:1rem;display:flex;flex-direction:column;gap:0.5rem; }
  .textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif;resize:vertical; }
  .textarea:focus { outline:none;border-color:#3ABEFF; }
  .btn-warn { padding:0.5rem 1rem;background:rgba(245,158,11,0.15);border:1px solid #F59E0B;color:#F59E0B;border-radius:6px;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .badge { padding:0.12rem 0.4rem;border-radius:4px;font-size:0.65rem; }
  .badge-pending { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .badge-approved { background:rgba(16,185,129,0.15);color:#10B981; }
  .badge-flagged { background:rgba(239,68,68,0.15);color:#EF4444; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1.5rem; }
  .empty-state { display:flex;justify-content:center;align-items:center;height:100%;color:#475569; }
</style>
