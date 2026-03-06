<!-- /data/request/mine — List of current user's data requests with status -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { getMyDataRequests, type DataRequestBrief } from '$lib/stores/data_analysts';

  let requests: DataRequestBrief[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  async function load() {
    loading = true; error = '';
    try {
      requests = await getMyDataRequests();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function statusLabel(s: string): string {
    const map: Record<string, string> = {
      pending_statistician: 'Pending Review',
      approved: 'Approved',
      rejected: 'Rejected',
      processing: 'Analyst Processing',
      pending_outbound_review: 'Outbound Review',
      delivered: 'Delivered',
      withheld: 'Withheld',
    };
    return map[s] ?? s;
  }

  function statusClass(s: string): string {
    if (s === 'delivered') return 'st-delivered';
    if (s === 'rejected' || s === 'withheld') return 'st-rejected';
    if (s === 'processing' || s === 'approved') return 'st-active';
    return 'st-pending';
  }

  onMount(load);
</script>

<div class="page">
  <div class="header-row">
    <h1 class="title">My Data Requests</h1>
    <button class="btn-primary" onclick={() => goto('/data/request/new')}>+ New Request</button>
  </div>

  {#if loading}
    <p class="muted">Loading…</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if requests.length === 0}
    <div class="empty">
      <p>You haven't submitted any data requests yet.</p>
      <button class="btn-secondary" onclick={() => goto('/data/request/new')}>Submit Your First Request</button>
    </div>
  {:else}
    <div class="list">
      {#each requests as req}
        <button class="card" onclick={() => goto(`/data/request/${req.id}/status`)}>
          <div class="card-top">
            <span class="badge u-{req.urgency}">{req.urgency}</span>
            <span class="status-chip {statusClass(req.status)}">{statusLabel(req.status)}</span>
            <span class="date">{new Date(req.created_at).toLocaleDateString()}</span>
          </div>
          <p class="desc">{req.dataset_description}</p>
          <p class="scope">Scope: {req.scope}</p>
        </button>
      {/each}
    </div>
  {/if}

  <button class="btn-secondary" onclick={load} style="margin-top:0.75rem">Refresh</button>
</div>

<style>
  .page { max-width:700px; }
  .header-row { display:flex;align-items:center;justify-content:space-between;margin-bottom:0.75rem; }
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .error { color:#EF4444;font-size:0.8rem; }
  .empty { color:#94A3B8;font-size:0.85rem;background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:1.2rem;text-align:center; }
  .list { display:flex;flex-direction:column;gap:0.5rem; }
  .card { display:block;width:100%;text-align:left;background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.65rem 0.8rem;cursor:pointer;transition:.15s;font-family:inherit; }
  .card:hover { border-color:#3ABEFF;background:#0E1428; }
  .card-top { display:flex;align-items:center;gap:0.5rem;margin-bottom:0.3rem; }
  .badge { padding:0.1rem 0.45rem;border-radius:4px;font-size:0.65rem;font-weight:600;text-transform:uppercase; }
  .u-low { background:#064E3B;color:#6EE7B7; }
  .u-medium { background:#1E3A5F;color:#93C5FD; }
  .u-high { background:#78350F;color:#FDE68A; }
  .u-critical { background:#7F1D1D;color:#FCA5A5; }
  .status-chip { font-size:0.65rem;padding:0.1rem 0.4rem;border-radius:4px; }
  .st-pending { background:#1E293B;color:#94A3B8; }
  .st-active { background:#1E3A5F;color:#93C5FD; }
  .st-delivered { background:#064E3B;color:#6EE7B7; }
  .st-rejected { background:#7F1D1D;color:#FCA5A5; }
  .date { margin-left:auto;font-size:0.7rem;color:#94A3B8; }
  .desc { font-size:0.8rem;color:#E6EDF3;margin:0 0 0.15rem;line-height:1.35; }
  .scope { font-size:0.7rem;color:#94A3B8;margin:0; }
  .btn-primary { padding:0.4rem 0.9rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem; }
  .btn-secondary { padding:0.4rem 0.8rem;background:#1E293B;border:1px solid rgba(58,190,255,0.2);border-radius:6px;color:#E6EDF3;font-size:0.8rem;cursor:pointer; }
  .btn-secondary:hover { border-color:#3ABEFF; }
</style>
