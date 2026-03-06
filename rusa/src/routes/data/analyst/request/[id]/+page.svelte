<!-- /data/analyst/request/[id] — UC-DA-02/03: Analyst Workbench + Submit Response -->
<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import {
    getDataRequestDetail,
    processDataRequest,
    submitDataResponse,
    getDataResponse,
    type DataRequest,
    type DataResponse,
  } from '$lib/stores/data_analysts';

  let requestId = $derived($page.params.id ?? '');
  let request: DataRequest | null = $state(null);
  let existingResponse: DataResponse | null = $state(null);
  let loading = $state(true);
  let error = $state('');
  let actionError = $state('');
  let actionSuccess = $state('');
  let starting = $state(false);
  let submitting = $state(false);

  // Response form fields
  let resultJson = $state('{}');
  let jsonError = $state('');
  let selectedFile: File | null = $state(null);

  async function load() {
    loading = true; error = '';
    try {
      request = await getDataRequestDetail(requestId);
      try { existingResponse = await getDataResponse(requestId); } catch { existingResponse = null; }
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function handleStartProcessing() {
    starting = true; actionError = ''; actionSuccess = '';
    try {
      await processDataRequest(requestId);
      actionSuccess = 'Now processing this request.';
      await load();
    } catch (e: unknown) {
      actionError = e instanceof Error ? e.message : String(e);
    } finally {
      starting = false;
    }
  }

  function validateJson() {
    jsonError = '';
    try {
      JSON.parse(resultJson);
    } catch {
      jsonError = 'Invalid JSON. Please fix before submitting.';
    }
  }

  async function handleSubmitResponse() {
    actionError = ''; actionSuccess = '';
    // Validate JSON
    let payload: Record<string, unknown> | undefined;
    try {
      const parsed = JSON.parse(resultJson);
      if (typeof parsed === 'object' && parsed !== null && Object.keys(parsed).length > 0) {
        payload = parsed;
      }
    } catch {
      actionError = 'Invalid JSON in result payload.';
      return;
    }

    submitting = true;
    try {
      // Read file bytes if a file is selected
      let fileBytes: number[] | undefined;
      let filename: string | undefined;
      if (selectedFile) {
        const buf = await selectedFile.arrayBuffer();
        fileBytes = Array.from(new Uint8Array(buf));
        filename = selectedFile.name;
      }

      await submitDataResponse(requestId, payload, fileBytes, filename);
      actionSuccess = 'Response submitted! Routing to The Statistician for outbound review.';
      await load();
    } catch (e: unknown) {
      actionError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  function handleFileInput(e: Event) {
    const input = e.target as HTMLInputElement;
    selectedFile = input.files?.[0] ?? null;
  }

  onMount(load);
</script>

<div class="page">
  <button class="back-link" onclick={() => goto('/data/analyst/inbox')}>&larr; Back to Inbox</button>

  {#if loading}
    <p class="muted">Loading…</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if request}
    <h1 class="title">Request Workbench</h1>

    <!-- Request details card -->
    <div class="card">
      <h2 class="section-title">Request Details</h2>
      <div class="detail"><span class="label">Dataset</span><span class="value">{request.dataset_description}</span></div>
      <div class="detail"><span class="label">Scope</span><span class="value">{request.scope}</span></div>
      <div class="detail"><span class="label">Purpose</span><span class="value">{request.purpose}</span></div>
      <div class="detail"><span class="label">Urgency</span><span class="badge u-{request.urgency}">{request.urgency}</span></div>
      {#if request.sensitivity_note}
        <div class="detail"><span class="label">Sensitivity</span><span class="value sens">{request.sensitivity_note}</span></div>
      {/if}
      <div class="detail"><span class="label">Status</span><span class="value status-{request.status}">{request.status.replace(/_/g, ' ')}</span></div>
      <div class="detail"><span class="label">Submitted</span><span class="value">{new Date(request.created_at).toLocaleString()}</span></div>
    </div>

    <!-- Existing response info -->
    {#if existingResponse}
      <div class="card">
        <h2 class="section-title">Submitted Response</h2>
        <div class="detail"><span class="label">Status</span><span class="value">{existingResponse.status}</span></div>
        {#if existingResponse.statistician_review_note}
          <div class="detail"><span class="label">Review Note</span><span class="value sens">{existingResponse.statistician_review_note}</span></div>
        {/if}
        <div class="detail"><span class="label">Submitted</span><span class="value">{new Date(existingResponse.submitted_at).toLocaleString()}</span></div>
        {#if existingResponse.result_payload}
          <pre class="payload">{JSON.stringify(existingResponse.result_payload, null, 2)}</pre>
        {/if}
        {#if existingResponse.spreadsheet_storage_path}
          <p class="attachment">📎 Spreadsheet attached</p>
        {/if}
      </div>
    {/if}

    {#if actionError}<p class="error">{actionError}</p>{/if}
    {#if actionSuccess}<p class="success">{actionSuccess}</p>{/if}

    <!-- Action: Start Processing (if status is 'approved') -->
    {#if request.status === 'approved'}
      <div class="action-card">
        <p class="action-text">This request is approved and awaiting an analyst. Claim it to begin working.</p>
        <button class="btn-primary" onclick={handleStartProcessing} disabled={starting}>
          {starting ? 'Starting…' : 'Start Processing'}
        </button>
      </div>
    {/if}

    <!-- Action: Submit Response (if status is 'processing') -->
    {#if request.status === 'processing'}
      <div class="action-card">
        <h2 class="section-title">Submit Response</h2>
        <p class="action-text">Enter the result data and optionally attach a spreadsheet.</p>

        <div class="field">
          <label class="field-label" for="result-json">Result Payload (JSON)</label>
          <textarea id="result-json" class="textarea" rows="8" placeholder={'{"key": "value"}'}
            bind:value={resultJson} oninput={validateJson}></textarea>
          {#if jsonError}<p class="json-error">{jsonError}</p>{/if}
        </div>

        <div class="field">
          <label class="field-label" for="spreadsheet">Spreadsheet Attachment (optional)</label>
          <input id="spreadsheet" type="file" class="file-input" accept=".xlsx,.xls,.csv,.ods"
            onchange={handleFileInput} />
          {#if selectedFile}
            <p class="file-name">{selectedFile.name} ({(selectedFile.size / 1024).toFixed(1)} KB)</p>
          {/if}
        </div>

        <button class="btn-primary" onclick={handleSubmitResponse} disabled={submitting || !!jsonError}>
          {submitting ? 'Submitting…' : 'Submit Response'}
        </button>
      </div>
    {/if}

    <!-- If withheld, analyst can resubmit -->
    {#if request.status === 'processing' && existingResponse?.status === 'withheld'}
      <div class="withheld-notice">
        The Statistician withheld your previous response. Please revise and resubmit above.
      </div>
    {/if}
  {/if}
</div>

<style>
  .page { max-width:640px; }
  .back-link { background:none;border:none;color:#3ABEFF;font-size:0.8rem;cursor:pointer;padding:0;margin-bottom:0.75rem;display:inline-block; }
  .back-link:hover { text-decoration:underline; }
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.75rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .error { color:#EF4444;font-size:0.8rem;margin:0.25rem 0; }
  .success { color:#10B981;font-size:0.8rem;margin:0.25rem 0; }

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

  .status-approved { color:#6EE7B7; }
  .status-processing { color:#93C5FD; }
  .status-pending_outbound_review { color:#FDE68A; }
  .status-delivered { color:#3ABEFF; }

  .action-card { background:#0E1428;border:1px solid rgba(58,190,255,0.15);border-radius:8px;padding:0.8rem;margin-bottom:0.75rem; }
  .action-text { color:#94A3B8;font-size:0.8rem;margin:0 0 0.5rem; }

  .field { display:flex;flex-direction:column;gap:0.25rem;margin-bottom:0.6rem; }
  .field-label { font-size:0.8rem;color:#E6EDF3;font-weight:500; }
  .textarea { width:100%;background:#0B0F1A;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.5rem;font-size:0.75rem;font-family:'Fira Code','Consolas',monospace;resize:vertical; }
  .textarea:focus { outline:none;border-color:#3ABEFF; }
  .json-error { color:#EF4444;font-size:0.7rem;margin:0; }
  .file-input { font-size:0.8rem;color:#E6EDF3; }
  .file-name { font-size:0.75rem;color:#93C5FD;margin:0; }

  .payload { background:#0B0F1A;border:1px solid #1E293B;border-radius:4px;padding:0.5rem;font-size:0.7rem;color:#93C5FD;overflow-x:auto;max-height:200px; }
  .attachment { color:#3ABEFF;font-size:0.8rem;margin:0.3rem 0 0; }

  .btn-primary { padding:0.5rem 1.2rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.85rem; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }

  .withheld-notice { background:#7F1D1D20;border:1px solid #EF444440;border-radius:6px;padding:0.6rem;color:#FCA5A5;font-size:0.8rem;margin-bottom:0.75rem; }
</style>
