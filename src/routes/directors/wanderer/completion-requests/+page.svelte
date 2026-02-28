<!-- /directors/wanderer/completion-requests — UC-WAN-02: Review Completion Requests -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getCompletionRequestsWanderer, processCompletionRequest, getEvidenceUrls,
    type CompletionRequestSummary, type EvidenceFileUrl,
  } from '$lib/stores/astronauts';

  let requests: CompletionRequestSummary[] = $state([]);
  let selected: CompletionRequestSummary | null = $state(null);
  let evidenceUrls: EvidenceFileUrl[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let success = $state('');

  let decision: 'forward' | 'reject' = $state('forward');
  let note = $state('');
  let processing = $state(false);

  onMount(async () => {
    try {
      requests = await getCompletionRequestsWanderer();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  async function selectRequest(req: CompletionRequestSummary) {
    selected = req;
    evidenceUrls = [];
    error = '';
    success = '';
    note = '';
    decision = 'forward';
    if (req.evidence_storage_paths?.length) {
      try {
        evidenceUrls = await getEvidenceUrls(req.id);
      } catch {}
    }
  }

  async function handleDecision() {
    if (!selected) return;
    error = '';
    success = '';
    processing = true;
    try {
      await processCompletionRequest({
        request_id: selected.id,
        decision,
        note: note || undefined,
      });
      success = decision === 'forward'
        ? 'Forwarded to Taskmaster for final approval.'
        : 'Request rejected. Mission returns to active.';
      requests = await getCompletionRequestsWanderer();
      selected = null;
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      processing = false;
    }
  }

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleDateString('en-GB', { day: '2-digit', month: 'short', year: 'numeric' });
  }

  function statusClass(s: string): string {
    switch (s) {
      case 'pending_wanderer': return 'st-pending';
      case 'pending_taskmaster': return 'st-forwarded';
      case 'approved': return 'st-approved';
      case 'rejected': return 'st-rejected';
      default: return '';
    }
  }
</script>

<h1 class="title">Wanderer — Completion Requests</h1>
<p class="subtitle">Review mission completion evidence from astronauts. Forward to Taskmaster or reject.</p>

{#if loading}
  <p class="loading">Loading…</p>
{:else}
  <div class="grid">
    <div class="list-panel">
      {#each requests as req}
        <button class="card" class:selected={selected?.id === req.id} onclick={() => selectRequest(req)}>
          <div class="card-title">{req.mission_title}</div>
          <div class="card-meta">
            <span>{req.submitter_name}</span>
            <span class="badge {statusClass(req.status)}">{req.status.replace(/_/g, ' ')}</span>
          </div>
          <div class="card-date">{formatDate(req.created_at)}</div>
        </button>
      {:else}
        <p class="empty">No completion requests.</p>
      {/each}
    </div>

    <div class="detail-panel">
      {#if selected}
        <h2>{selected.mission_title}</h2>
        <div class="kv"><span class="k">Submitter:</span><span>{selected.submitter_name}</span></div>
        <div class="kv"><span class="k">Status:</span><span class="badge {statusClass(selected.status)}">{selected.status.replace(/_/g, ' ')}</span></div>
        <div class="kv"><span class="k">Submitted:</span><span>{formatDate(selected.created_at)}</span></div>

        <div class="section">
          <h3>Findings Summary</h3>
          <p class="body-text">{selected.findings_summary}</p>
        </div>

        {#if evidenceUrls.length > 0}
          <div class="section">
            <h3>Evidence Files ({evidenceUrls.length})</h3>
            {#each evidenceUrls as ev}
              <a class="evidence-link" href={ev.signed_url} target="_blank" rel="noopener">📎 {ev.filename}</a>
            {/each}
          </div>
        {/if}

        {#if selected.wanderer_note}
          <div class="note-box"><strong>Your Note:</strong> {selected.wanderer_note}</div>
        {/if}

        {#if selected.status === 'pending_wanderer'}
          <div class="action-form">
            <h3>Decision</h3>
            <select class="input" bind:value={decision}>
              <option value="forward">Forward to Taskmaster</option>
              <option value="reject">Reject (Return to Active)</option>
            </select>
            <textarea class="textarea" bind:value={note} placeholder="Optional note…" rows="2"></textarea>
            {#if error}<p class="error">{error}</p>{/if}
            {#if success}<p class="success">{success}</p>{/if}
            <button class="btn-primary" onclick={handleDecision} disabled={processing}>
              {processing ? 'Processing…' : decision === 'forward' ? 'Forward to Taskmaster' : 'Reject'}
            </button>
          </div>
        {/if}
      {:else}
        <div class="empty-state"><p>Select a request to review.</p></div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .loading { color:#94A3B8; }
  .grid { display:flex;gap:1rem;flex:1;overflow:hidden; }
  .list-panel { width:320px;overflow-y:auto; }
  .detail-panel { flex:1;overflow-y:auto; }
  .card { width:100%;text-align:left;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.65rem;margin-bottom:0.35rem;cursor:pointer;color:#E6EDF3; }
  .card:hover { border-color:rgba(58,190,255,0.3); }
  .card.selected { border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .card-title { font-size:0.8rem;font-weight:500;margin-bottom:0.2rem; }
  .card-meta { display:flex;justify-content:space-between;font-size:0.7rem;color:#94A3B8; }
  .card-date { font-size:0.65rem;color:#475569;margin-top:0.15rem; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1rem;color:#3ABEFF;margin:0 0 0.5rem; }
  h3 { font-size:0.85rem;color:#E6EDF3;margin:0 0 0.4rem; }
  .kv { display:flex;gap:0.5rem;font-size:0.8rem;margin-bottom:0.3rem; }
  .k { color:#94A3B8;min-width:90px; }
  .section { margin-top:0.75rem; }
  .body-text { color:#CBD5E1;font-size:0.85rem;line-height:1.5;margin:0;white-space:pre-wrap; }
  .evidence-link { display:block;color:#3ABEFF;font-size:0.8rem;text-decoration:none;padding:0.15rem 0; }
  .evidence-link:hover { text-decoration:underline; }
  .note-box { padding:0.5rem;border-radius:6px;font-size:0.8rem;background:rgba(139,92,246,0.1);border-left:3px solid #8B5CF6;color:#CBD5E1;margin-top:0.75rem; }
  .action-form { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-top:0.75rem;display:flex;flex-direction:column;gap:0.5rem; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .textarea { resize:vertical; }
  .badge { padding:0.1rem 0.35rem;border-radius:4px;font-size:0.65rem;font-weight:600; }
  .st-pending { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .st-forwarded { background:rgba(139,92,246,0.15);color:#C084FC; }
  .st-approved { background:rgba(16,185,129,0.15);color:#10B981; }
  .st-rejected { background:rgba(239,68,68,0.15);color:#EF4444; }
  .btn-primary { padding:0.45rem 1rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
  .empty-state { display:flex;justify-content:center;align-items:center;height:100%;color:#475569; }
</style>
