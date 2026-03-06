<!-- /data/analyst/inbox — UC-DA-01: Data Analyst Inbox -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { getAnalystInbox, type DataRequestBrief } from '$lib/stores/data_analysts';

  let requests: DataRequestBrief[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  async function load() {
    loading = true; error = '';
    try {
      requests = await getAnalystInbox();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function urgencyOrder(u: string) {
    return { critical: 0, high: 1, medium: 2, low: 3 }[u] ?? 4;
  }

  onMount(load);
</script>

<div class="page">
  <h1 class="title">Data Analyst Inbox</h1>
  <p class="subtitle">Approved data requests awaiting processing. Sorted by urgency.</p>

  {#if loading}
    <p class="muted">Loading…</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if requests.length === 0}
    <div class="empty">No pending requests in queue.</div>
  {:else}
    <div class="list">
      {#each requests as req}
        <button class="card" onclick={() => goto(`/data/analyst/request/${req.id}`)}>
          <div class="card-header">
            <span class="badge u-{req.urgency}">{req.urgency}</span>
            <span class="status-chip" class:processing={req.status === 'processing'}>{req.status === 'processing' ? 'In Progress' : 'Approved'}</span>
            <span class="date">{new Date(req.created_at).toLocaleDateString()}</span>
          </div>
          <p class="desc">{req.dataset_description}</p>
          <div class="meta">
            <span>Scope: {req.scope}</span>
            <span>Requester: {req.requester_name}</span>
          </div>
        </button>
      {/each}
    </div>
  {/if}

  <button class="btn-secondary" onclick={load} style="margin-top:0.75rem">Refresh</button>
</div>

<style>
  .page { max-width:700px; }
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.2rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .error { color:#EF4444;font-size:0.8rem; }
  .empty { color:#94A3B8;font-size:0.85rem;background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:1.2rem;text-align:center; }
  .list { display:flex;flex-direction:column;gap:0.5rem; }
  .card { display:block;width:100%;text-align:left;background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.65rem 0.8rem;cursor:pointer;transition:.15s;font-family:inherit; }
  .card:hover { border-color:#3ABEFF;background:#0E1428; }
  .card-header { display:flex;align-items:center;gap:0.5rem;margin-bottom:0.3rem; }
  .badge { padding:0.1rem 0.45rem;border-radius:4px;font-size:0.65rem;font-weight:600;text-transform:uppercase; }
  .u-low { background:#064E3B;color:#6EE7B7; }
  .u-medium { background:#1E3A5F;color:#93C5FD; }
  .u-high { background:#78350F;color:#FDE68A; }
  .u-critical { background:#7F1D1D;color:#FCA5A5; }
  .status-chip { font-size:0.65rem;padding:0.1rem 0.4rem;border-radius:4px;background:#1E3A5F;color:#93C5FD; }
  .status-chip.processing { background:#064E3B;color:#6EE7B7; }
  .date { margin-left:auto;font-size:0.7rem;color:#94A3B8; }
  .desc { font-size:0.8rem;color:#E6EDF3;margin:0 0 0.3rem;line-height:1.35; }
  .meta { display:flex;gap:0.8rem;font-size:0.7rem;color:#94A3B8; }
  .btn-secondary { padding:0.4rem 0.8rem;background:#1E293B;border:1px solid rgba(58,190,255,0.2);border-radius:6px;color:#E6EDF3;font-size:0.8rem;cursor:pointer; }
  .btn-secondary:hover { border-color:#3ABEFF; }
</style>
