<!--
  /directors/artificer/help-requests — Review Help Requests (UC-ART-03/04)
  TheArtificer reviews incoming help requests from subordinates and can respond.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { reviewHelpRequest, reviewHelpResponse } from '$lib/stores/directors';

  interface HelpRequest {
    id: string;
    requester_name: string;
    subject: string;
    body: string;
    status: string;
    created_at: string;
    response?: string;
  }

  let requests: HelpRequest[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let responding: string | null = $state(null);
  let responseText = $state('');
  let actionError = $state('');
  let actionSuccess = $state('');

  onMount(async () => {
    await loadRequests();
  });

  async function loadRequests() {
    loading = true; error = '';
    try {
      requests = await invoke<HelpRequest[]>('dir_get_help_requests');
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function submitResponse(id: string) {
    actionError = ''; actionSuccess = '';
    if (!responseText.trim()) { actionError = 'Response cannot be empty.'; return; }
    try {
      await reviewHelpResponse({ request_id: id, response: responseText.trim() });
      actionSuccess = 'Response submitted.';
      responding = null; responseText = '';
      await loadRequests();
    } catch (e: unknown) {
      actionError = e instanceof Error ? e.message : String(e);
    }
  }

  function formatDate(d: string) { return new Date(d).toLocaleString(); }
  function statusColor(s: string) {
    return s === 'open' ? '#3ABEFF' : s === 'resolved' ? '#22C55E' : '#94A3B8';
  }
</script>

<h1 class="title">Help Requests</h1>
<p class="subtitle">Incoming help requests from your subordinates requiring review.</p>

{#if loading}<p class="muted">Loading…</p>
{:else if error}<p class="error">{error}</p>
{:else if requests.length === 0}<p class="muted">No help requests.</p>
{:else}
  {#each requests as req}
    <div class="card">
      <div class="card-header">
        <span class="card-title">{req.subject}</span>
        <span class="status-badge" style="color:{statusColor(req.status)}">{req.status}</span>
      </div>
      <p class="meta">From: <strong>{req.requester_name}</strong> · {formatDate(req.created_at)}</p>
      <p class="body-text">{req.body}</p>
      {#if req.response}
        <p class="response-label">Response:</p>
        <p class="response-text">{req.response}</p>
      {/if}
      {#if req.status === 'open'}
        {#if responding === req.id}
          <textarea class="textarea" bind:value={responseText} rows="3" placeholder="Your response…"></textarea>
          {#if actionError}<p class="error">{actionError}</p>{/if}
          {#if actionSuccess}<p class="success">{actionSuccess}</p>{/if}
          <div class="row-actions">
            <button class="btn-sm-cancel" onclick={() => { responding = null; responseText = ''; }}>Cancel</button>
            <button class="btn-sm" onclick={() => submitResponse(req.id)}>Submit Response</button>
          </div>
        {:else}
          <button class="btn-sm" onclick={() => { responding = req.id; responseText = ''; actionError = ''; actionSuccess = ''; }}>Respond</button>
        {/if}
      {/if}
    </div>
  {/each}
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.9rem;margin-bottom:0.7rem;display:flex;flex-direction:column;gap:0.35rem; }
  .card-header { display:flex;justify-content:space-between;align-items:center; }
  .card-title { font-size:0.9rem;font-weight:600;color:#E6EDF3; }
  .status-badge { font-size:0.72rem;font-weight:600; }
  .meta { font-size:0.72rem;color:#64748B;margin:0; }
  .body-text { font-size:0.82rem;color:#CBD5E1;margin:0;white-space:pre-wrap; }
  .response-label { font-size:0.7rem;color:#8B5CF6;margin:0; }
  .response-text { font-size:0.8rem;color:#C4B5FD;margin:0;white-space:pre-wrap; }
  .textarea { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif;width:100%;box-sizing:border-box;resize:vertical; }
  .textarea:focus { outline:none;border-color:#3ABEFF; }
  .row-actions { display:flex;gap:0.4rem;justify-content:flex-end; }
  .btn-sm { background:rgba(58,190,255,0.12);border:1px solid rgba(58,190,255,0.35);color:#3ABEFF;border-radius:5px;padding:0.3rem 0.7rem;cursor:pointer;font-size:0.75rem; }
  .btn-sm:hover { background:rgba(58,190,255,0.22); }
  .btn-sm-cancel { background:transparent;border:1px solid rgba(148,163,184,0.25);color:#94A3B8;border-radius:5px;padding:0.3rem 0.7rem;cursor:pointer;font-size:0.75rem; }
  .error { color:#EF4444;font-size:0.73rem; }
  .success { color:#22C55E;font-size:0.73rem; }
</style>
