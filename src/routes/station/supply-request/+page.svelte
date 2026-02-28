<!-- /station/supply-request — UC-SSS-04: Submit Supply Request -->
<script lang="ts">
  import { getContext, onMount } from 'svelte';
  import type { Writable } from 'svelte/store';
  import { sstSubmitSupplyRequest, sstGetSupplyRequests, type SupplyRequestSummary } from '$lib/stores/space_station';

  const stationId = getContext<Writable<string>>('selectedStationId');

  // Form state
  let formItems: { item: string; quantity: number; justification: string }[] = $state([
    { item: '', quantity: 1, justification: '' },
  ]);
  let formError = $state('');
  let formSuccess = $state('');
  let submitting = $state(false);

  // List state
  let requests: SupplyRequestSummary[] = $state([]);
  let loading = $state(true);
  let activeTab: 'form' | 'history' = $state('form');

  let currentStationId = $state('');
  const unsub = stationId.subscribe((v) => {
    currentStationId = v;
    if (v) loadRequests();
  });

  async function loadRequests() {
    if (!currentStationId) return;
    loading = true;
    try {
      requests = await sstGetSupplyRequests(currentStationId);
    } catch {}
    loading = false;
  }

  onMount(() => { if (currentStationId) loadRequests(); });

  function addItem() {
    formItems = [...formItems, { item: '', quantity: 1, justification: '' }];
  }

  function removeItem(idx: number) {
    formItems = formItems.filter((_, i) => i !== idx);
  }

  async function handleSubmit() {
    formError = ''; formSuccess = '';
    if (!currentStationId) { formError = 'No station selected.'; return; }
    const valid = formItems.filter((it) => it.item.trim());
    if (valid.length === 0) { formError = 'At least one item is required.'; return; }

    submitting = true;
    try {
      await sstSubmitSupplyRequest({
        station_id: currentStationId,
        items: valid,
      });
      formSuccess = 'Supply request submitted. Directors will vote on approval.';
      formItems = [{ item: '', quantity: 1, justification: '' }];
      await loadRequests();
    } catch (e: unknown) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  function statusColor(s: string): string {
    switch (s) {
      case 'approved': return '#10B981';
      case 'rejected': return '#EF4444';
      default: return '#F59E0B';
    }
  }
</script>

<h1 class="title">Supply Requests</h1>
<p class="subtitle">Request supplies for the station. Approval is routed to Directors for voting.</p>

<div class="tabs">
  <button class="tab" class:active={activeTab === 'form'} onclick={() => (activeTab = 'form')}>New Request</button>
  <button class="tab" class:active={activeTab === 'history'} onclick={() => (activeTab = 'history')}>History ({requests.length})</button>
</div>

{#if activeTab === 'form'}
  <div class="form-card">
    <h2>Request Items</h2>

    {#each formItems as fi, idx}
      <div class="item-row">
        <label class="field flex2"><span class="label">Item *</span>
          <input type="text" class="input" bind:value={fi.item} placeholder="e.g. Oxygen Canister…" />
        </label>
        <label class="field flex05"><span class="label">Qty</span>
          <input type="number" class="input" bind:value={fi.quantity} min="1" />
        </label>
        <label class="field flex2"><span class="label">Justification</span>
          <input type="text" class="input" bind:value={fi.justification} placeholder="Reason…" />
        </label>
        {#if formItems.length > 1}
          <button class="btn-remove" onclick={() => removeItem(idx)}>×</button>
        {/if}
      </div>
    {/each}

    <button class="btn-add-item" onclick={addItem}>+ Add Another Item</button>

    {#if formError}<p class="error">{formError}</p>{/if}
    {#if formSuccess}<p class="success">{formSuccess}</p>{/if}

    <button class="btn-primary" onclick={handleSubmit} disabled={submitting}>
      {submitting ? 'Submitting…' : 'Submit Supply Request'}
    </button>
  </div>
{:else}
  {#if loading}
    <p class="muted">Loading requests…</p>
  {:else if requests.length === 0}
    <p class="muted">No supply requests found.</p>
  {:else}
    <div class="request-list">
      {#each requests as req}
        <div class="request-card">
          <div class="req-header">
            <span class="req-by">Submitted by: {req.submitter_name}</span>
            <span class="status-badge" style="color:{statusColor(req.status)};background:rgba({statusColor(req.status) === '#10B981' ? '16,185,129' : statusColor(req.status) === '#EF4444' ? '239,68,68' : '245,158,11'},0.15)">{req.status.replace(/_/g, ' ')}</span>
          </div>
          <div class="req-items">
            {#each (req.items as unknown as { item: string; quantity: number; justification: string }[]) as it}
              <div class="req-item">
                <span class="item-name">{it.item}</span>
                <span class="item-qty">×{it.quantity}</span>
                {#if it.justification}
                  <span class="item-just">— {it.justification}</span>
                {/if}
              </div>
            {/each}
          </div>
          <span class="req-date">{new Date(req.created_at).toLocaleString()}</span>
        </div>
      {/each}
    </div>
  {/if}
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .tabs { display:flex;gap:0.25rem;margin-bottom:1rem; }
  .tab { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);color:#94A3B8;padding:0.4rem 1rem;border-radius:6px 6px 0 0;cursor:pointer;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .tab.active { color:#3ABEFF;border-bottom-color:transparent;background:rgba(58,190,255,0.08); }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:0 8px 8px 8px;padding:1.25rem;max-width:750px;display:flex;flex-direction:column;gap:0.6rem; }
  .form-card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0; }
  .item-row { display:flex;gap:0.5rem;align-items:flex-end; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .flex2 { flex:2; }
  .flex05 { flex:0.5; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .btn-remove { background:none;border:none;color:#EF4444;font-size:1.1rem;cursor:pointer;padding:0.3rem; }
  .btn-add-item { background:none;border:1px dashed rgba(58,190,255,0.2);color:#3ABEFF;padding:0.3rem 0.75rem;border-radius:6px;font-size:0.75rem;cursor:pointer;align-self:flex-start; }
  .btn-add-item:hover { border-color:#3ABEFF; }
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .request-list { display:flex;flex-direction:column;gap:0.5rem;max-width:750px; }
  .request-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.85rem; }
  .req-header { display:flex;justify-content:space-between;align-items:center;margin-bottom:0.4rem; }
  .req-by { font-size:0.8rem;color:#E6EDF3; }
  .status-badge { padding:0.1rem 0.4rem;border-radius:4px;font-size:0.65rem;font-weight:600;text-transform:uppercase; }
  .req-items { display:flex;flex-direction:column;gap:0.2rem;margin-bottom:0.3rem; }
  .req-item { display:flex;gap:0.4rem;font-size:0.8rem;color:#C9D1D9; }
  .item-name { font-weight:500; }
  .item-qty { color:#3ABEFF; }
  .item-just { color:#94A3B8;font-style:italic; }
  .req-date { font-size:0.7rem;color:#94A3B8; }
</style>
