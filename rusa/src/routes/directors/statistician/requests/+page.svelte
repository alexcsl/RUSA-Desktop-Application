<!-- /directors/statistician/requests — TheStatistician Data Request Pipeline (UC-STAT-01/02/03) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getDataRequestQueue,
    decideDataRequest,
    getOutboundReviewQueue,
    reviewOutboundDataResponse,
    type DataRequestSummary,
    type OutboundResponseSummary,
  } from '$lib/stores/directors';

  // ── Tab state ──────────────────────────────────────────────────────────────
  let activeTab: 'incoming' | 'outbound' = $state('incoming');

  // ── Incoming queue state ───────────────────────────────────────────────────
  let queue: DataRequestSummary[] = $state([]);
  let selected: DataRequestSummary | null = $state(null);
  let decision: 'approved' | 'rejected' = $state('approved');
  let reason = $state('');
  let error = $state('');
  let success = $state('');

  // ── Outbound review state ─────────────────────────────────────────────────
  let outboundQueue: OutboundResponseSummary[] = $state([]);
  let selectedOutbound: OutboundResponseSummary | null = $state(null);
  let outboundDecision: 'forward' | 'withhold' = $state('forward');
  let outboundReason = $state('');
  let outboundError = $state('');
  let outboundSuccess = $state('');

  onMount(async () => {
    queue = await getDataRequestQueue();
    outboundQueue = await getOutboundReviewQueue();
  });

  async function handleDecide() {
    if (!selected) return;
    error = ''; success = '';
    try {
      await decideDataRequest(selected.id, decision, reason || undefined);
      success = `Request ${decision}.`;
      selected = null; reason = '';
      queue = await getDataRequestQueue();
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }

  async function handleOutbound() {
    if (!selectedOutbound) return;
    outboundError = ''; outboundSuccess = '';
    try {
      await reviewOutboundDataResponse(selectedOutbound.id, outboundDecision, outboundReason || undefined);
      outboundSuccess = `Response ${outboundDecision === 'forward' ? 'forwarded to requester' : 'withheld — returned to analyst'}.`;
      selectedOutbound = null; outboundReason = '';
      outboundQueue = await getOutboundReviewQueue();
    } catch (e: unknown) { outboundError = e instanceof Error ? e.message : String(e); }
  }

  function switchTab(tab: 'incoming' | 'outbound') {
    activeTab = tab;
    error = ''; success = ''; outboundError = ''; outboundSuccess = '';
    selected = null; selectedOutbound = null;
  }

  function urgencyColor(u: string): string {
    switch (u) {
      case 'critical': return '#EF4444';
      case 'high': return '#F59E0B';
      case 'medium': return '#3ABEFF';
      default: return '#94A3B8';
    }
  }
</script>

<h1 class="title">Statistician — Data Request Pipeline</h1>
<p class="subtitle">Bypass authority: approve/reject requests without Directors vote. Gate outbound responses.</p>

<!-- Tabs -->
<div class="view-tabs">
  <button class="tab-btn" class:active={activeTab === 'incoming'} onclick={() => switchTab('incoming')}>
    Incoming Queue ({queue.length})
  </button>
  <button class="tab-btn" class:active={activeTab === 'outbound'} onclick={() => switchTab('outbound')}>
    Outbound Review ({outboundQueue.length})
  </button>
</div>

<!-- ── Incoming Queue Tab ──────────────────────────────────────────────────── -->
{#if activeTab === 'incoming'}
<div class="grid">
  <div class="list-panel">
    {#each queue as req}
      <button class="card" class:selected={selected?.id === req.id} onclick={() => { selected = req; error = ''; success = ''; }}>
        <div class="card-title">{req.dataset_description}</div>
        <div class="card-meta">
          <span class="badge" style="color:{urgencyColor(req.urgency)}">{req.urgency}</span>
          <span>{req.requester_name}</span>
        </div>
      </button>
    {:else}
      <p class="empty">No pending data requests.</p>
    {/each}
  </div>

  <div class="detail-panel">
    {#if selected}
      <h2>{selected.dataset_description}</h2>
      <div class="kv"><span class="k">Requester:</span><span>{selected.requester_name}</span></div>
      <div class="kv"><span class="k">Scope:</span><span>{selected.scope}</span></div>
      <div class="kv"><span class="k">Purpose:</span><span>{selected.purpose}</span></div>
      <div class="kv"><span class="k">Urgency:</span><span style="color:{urgencyColor(selected.urgency)}">{selected.urgency}</span></div>
      <div class="kv"><span class="k">Date:</span><span>{new Date(selected.created_at).toLocaleDateString()}</span></div>
      {#if selected.sensitivity_note}
        <div class="kv"><span class="k">Sensitivity:</span><span class="sensitivity">{selected.sensitivity_note}</span></div>
      {/if}

      <div class="action-form">
        <h3>Decision</h3>
        <select class="input" bind:value={decision}>
          <option value="approved">Approve → Forward to Analysts</option>
          <option value="rejected">Reject → Return to Requester</option>
        </select>
        <textarea class="textarea" placeholder="Reason (required for rejection)…" bind:value={reason} rows="2"></textarea>
        {#if error}<p class="error">{error}</p>{/if}
        {#if success}<p class="success">{success}</p>{/if}
        <button class="btn-primary" onclick={handleDecide}>Submit Decision</button>
      </div>
    {:else}
      <div class="empty-state"><p>Select a request to review.</p></div>
    {/if}
  </div>
</div>
{/if}

<!-- ── Outbound Review Tab ─────────────────────────────────────────────────── -->
{#if activeTab === 'outbound'}
<div class="grid">
  <div class="list-panel">
    {#each outboundQueue as resp}
      <button class="card" class:selected={selectedOutbound?.id === resp.id} onclick={() => { selectedOutbound = resp; outboundError = ''; outboundSuccess = ''; }}>
        <div class="card-title">{resp.dataset_description}</div>
        <div class="card-meta">
          <span>By {resp.analyst_name}</span>
          <span>→ {resp.requester_name}</span>
        </div>
      </button>
    {:else}
      <p class="empty">No responses awaiting outbound review.</p>
    {/each}
  </div>

  <div class="detail-panel">
    {#if selectedOutbound}
      <h2>Outbound Response Review</h2>
      <div class="kv"><span class="k">Dataset:</span><span>{selectedOutbound.dataset_description}</span></div>
      <div class="kv"><span class="k">Analyst:</span><span>{selectedOutbound.analyst_name}</span></div>
      <div class="kv"><span class="k">Requester:</span><span>{selectedOutbound.requester_name}</span></div>
      <div class="kv"><span class="k">Submitted:</span><span>{new Date(selectedOutbound.submitted_at).toLocaleDateString()}</span></div>
      {#if selectedOutbound.spreadsheet_storage_path}
        <div class="kv"><span class="k">Spreadsheet:</span><span class="file-badge">📎 Attached</span></div>
      {/if}

      {#if selectedOutbound.result_payload}
        <h3>Response Payload</h3>
        <pre class="json">{JSON.stringify(selectedOutbound.result_payload, null, 2)}</pre>
      {/if}

      <div class="action-form">
        <h3>Outbound Decision</h3>
        <select class="input" bind:value={outboundDecision}>
          <option value="forward">Clear & Deliver to Requester</option>
          <option value="withhold">Withhold — Return to Analyst</option>
        </select>
        <textarea class="textarea" placeholder="Review notes…" bind:value={outboundReason} rows="2"></textarea>
        {#if outboundError}<p class="error">{outboundError}</p>{/if}
        {#if outboundSuccess}<p class="success">{outboundSuccess}</p>{/if}
        <button class="btn-primary" onclick={handleOutbound}>Confirm</button>
      </div>
    {:else}
      <div class="empty-state"><p>Select a response to review.</p></div>
    {/if}
  </div>
</div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }

  .view-tabs { display:flex;gap:0.25rem;margin-bottom:1rem; }
  .tab-btn { padding:0.4rem 1rem;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px 6px 0 0;color:#94A3B8;cursor:pointer;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .tab-btn.active { background:rgba(58,190,255,0.08);border-color:#3ABEFF;color:#3ABEFF;font-weight:600; }

  .grid { display:flex;gap:1rem;flex:1;overflow:hidden; }
  .list-panel { width:300px;overflow-y:auto; }
  .detail-panel { flex:1;overflow-y:auto; }
  .card { width:100%;text-align:left;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.65rem;margin-bottom:0.35rem;cursor:pointer;color:#E6EDF3; }
  .card:hover { border-color:rgba(58,190,255,0.3); }
  .card.selected { border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .card-title { font-size:0.8rem;font-weight:500;margin-bottom:0.2rem;white-space:nowrap;overflow:hidden;text-overflow:ellipsis; }
  .card-meta { display:flex;justify-content:space-between;font-size:0.7rem;color:#94A3B8; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1rem;color:#3ABEFF;margin:0 0 0.5rem; }
  h3 { font-size:0.85rem;color:#E6EDF3;margin:0.75rem 0 0.4rem; }
  .kv { display:flex;gap:0.5rem;font-size:0.8rem;margin-bottom:0.3rem; }
  .k { color:#94A3B8;min-width:80px; }
  .sensitivity { color:#F59E0B;font-style:italic; }
  .file-badge { color:#10B981; }
  .json { background:#0E1428;border:1px solid rgba(58,190,255,0.1);border-radius:6px;padding:0.6rem;font-size:0.7rem;color:#94A3B8;overflow-x:auto;white-space:pre-wrap;max-height:200px;overflow-y:auto; }
  .action-form { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-top:0.75rem;display:flex;flex-direction:column;gap:0.5rem; }
  .action-form h3 { margin:0; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .btn-primary { padding:0.45rem 1rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .badge { padding:0.1rem 0.35rem;border-radius:4px;font-size:0.65rem;text-transform:uppercase;font-weight:600; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
  .empty-state { display:flex;justify-content:center;align-items:center;height:100%;color:#475569; }
</style>
