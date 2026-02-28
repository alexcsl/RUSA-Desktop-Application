<!-- /data/request/[id]/status — Requester status tracker -->
<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { getDataRequestDetail, getDataResponse, type DataRequest, type DataResponse } from '$lib/stores/data_analysts';

  let requestId = $derived($page.params.id ?? '');
  let request: DataRequest | null = $state(null);
  let response: DataResponse | null = $state(null);
  let loading = $state(true);
  let error = $state('');

  const STATUS_FLOW = [
    { key: 'pending_statistician', label: 'Pending Review' },
    { key: 'approved', label: 'Approved' },
    { key: 'processing', label: 'Analyst Processing' },
    { key: 'pending_outbound_review', label: 'Outbound Review' },
    { key: 'delivered', label: 'Delivered' },
  ];

  function statusIndex(status: string) {
    if (status === 'rejected' || status === 'withheld') return -1;
    return STATUS_FLOW.findIndex(s => s.key === status);
  }

  async function load() {
    loading = true; error = '';
    try {
      request = await getDataRequestDetail(requestId);
      // Only try to load response if request progressed past approval
      if (request && ['processing', 'pending_outbound_review', 'delivered'].includes(request.status)) {
        try { response = await getDataResponse(requestId); } catch { response = null; }
      }
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  onMount(load);
</script>

<div class="page">
  <h1 class="title">Data Request Status</h1>

  {#if loading}
    <p class="muted">Loading…</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if request}
    <!-- Status pipeline -->
    {#if request.status === 'rejected'}
      <div class="rejected-badge">Rejected by The Statistician</div>
      {#if request.statistician_decision_reason}
        <p class="reason">{request.statistician_decision_reason}</p>
      {/if}
    {:else if request.status === 'withheld'}
      <div class="rejected-badge">Response Withheld</div>
    {:else}
      <div class="pipeline">
        {#each STATUS_FLOW as step, i}
          {@const current = statusIndex(request.status)}
          <div class="step" class:done={i <= current} class:active={i === current}>
            <div class="dot"></div>
            <span class="step-label">{step.label}</span>
          </div>
          {#if i < STATUS_FLOW.length - 1}
            <div class="connector" class:done={i < current}></div>
          {/if}
        {/each}
      </div>
    {/if}

    <!-- Request details -->
    <div class="card">
      <h2 class="section-title">Request Details</h2>
      <div class="detail"><span class="label">Dataset</span><span class="value">{request.dataset_description}</span></div>
      <div class="detail"><span class="label">Scope</span><span class="value">{request.scope}</span></div>
      <div class="detail"><span class="label">Purpose</span><span class="value">{request.purpose}</span></div>
      <div class="detail"><span class="label">Urgency</span><span class="badge u-{request.urgency}">{request.urgency}</span></div>
      {#if request.sensitivity_note}
        <div class="detail"><span class="label">Sensitivity</span><span class="value sens">{request.sensitivity_note}</span></div>
      {/if}
      <div class="detail"><span class="label">Submitted</span><span class="value">{new Date(request.created_at).toLocaleString()}</span></div>
    </div>

    <!-- Response data (if delivered) -->
    {#if response && request.status === 'delivered'}
      <div class="card">
        <h2 class="section-title">Response Data</h2>
        {#if response.result_payload}
          <pre class="payload">{JSON.stringify(response.result_payload, null, 2)}</pre>
        {/if}
        {#if response.spreadsheet_storage_path}
          <p class="attachment">📎 Spreadsheet attached</p>
        {/if}
        <p class="muted" style="margin-top:0.5rem">Delivered {new Date(response.delivered_at ?? response.submitted_at).toLocaleString()}</p>
      </div>
    {/if}

    <button class="btn-secondary" onclick={load}>Refresh</button>
  {/if}
</div>

<style>
  .page { max-width:640px; }
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 1rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .error { color:#EF4444;font-size:0.8rem; }

  /* Pipeline */
  .pipeline { display:flex;align-items:center;gap:0;margin:0 0 1.2rem;flex-wrap:wrap; }
  .step { display:flex;flex-direction:column;align-items:center;gap:0.25rem; }
  .dot { width:14px;height:14px;border-radius:50%;background:#1E293B;border:2px solid #334155;transition:.2s; }
  .step.done .dot { background:#3ABEFF;border-color:#3ABEFF; }
  .step.active .dot { background:#3ABEFF;border-color:#3ABEFF;box-shadow:0 0 8px #3ABEFF80; }
  .step-label { font-size:0.65rem;color:#94A3B8;white-space:nowrap; }
  .step.done .step-label,.step.active .step-label { color:#E6EDF3; }
  .connector { flex:1;height:2px;min-width:20px;background:#334155;margin:0 0.15rem;align-self:flex-start;margin-top:7px; }
  .connector.done { background:#3ABEFF; }

  .rejected-badge { display:inline-block;background:#EF4444;color:white;padding:0.3rem 0.8rem;border-radius:6px;font-size:0.8rem;font-weight:600;margin-bottom:0.75rem; }
  .reason { color:#FDE68A;font-size:0.8rem;border-left:3px solid #F59E0B;padding-left:0.5rem;margin:0 0 1rem; }

  .card { background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.8rem;margin-bottom:0.75rem; }
  .section-title { font-family:'Orbitron',sans-serif;font-size:0.8rem;color:#E6EDF3;margin:0 0 0.5rem; }
  .detail { display:flex;gap:0.5rem;margin-bottom:0.35rem;font-size:0.8rem; }
  .label { color:#94A3B8;min-width:80px;flex-shrink:0; }
  .value { color:#E6EDF3; }
  .sens { color:#FDE68A; }
  .badge { padding:0.1rem 0.5rem;border-radius:4px;font-size:0.7rem;font-weight:600;text-transform:uppercase; }
  .u-low { background:#064E3B;color:#6EE7B7; }
  .u-medium { background:#1E3A5F;color:#93C5FD; }
  .u-high { background:#78350F;color:#FDE68A; }
  .u-critical { background:#7F1D1D;color:#FCA5A5; }

  .payload { background:#0B0F1A;border:1px solid #1E293B;border-radius:4px;padding:0.5rem;font-size:0.7rem;color:#93C5FD;overflow-x:auto;max-height:200px; }
  .attachment { color:#3ABEFF;font-size:0.8rem;margin:0.3rem 0 0; }
  .btn-secondary { padding:0.4rem 0.8rem;background:#1E293B;border:1px solid rgba(58,190,255,0.2);border-radius:6px;color:#E6EDF3;font-size:0.8rem;cursor:pointer; }
  .btn-secondary:hover { border-color:#3ABEFF; }
</style>
