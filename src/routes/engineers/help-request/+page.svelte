<!--
  /engineers/help-request — UC-GE-03: Cross-Department Help Request
  Available to both Agricultural and Biological Engineers.
  Routes to The Observer (proxy Director) for triage.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { submitHelpRequest, getMyHelpRequests, type HelpRequestItem } from '$lib/stores/engineers';

  let title = $state('');
  let description = $state('');
  let urgency = $state('medium');
  let targetDepartment = $state('');
  let submitting = $state(false);
  let message = $state('');
  let isError = $state(false);

  let requests: HelpRequestItem[] = $state([]);
  let loadingRequests = $state(true);
  let tab: 'form' | 'history' = $state('form');

  onMount(async () => {
    try { requests = await getMyHelpRequests(); } catch {}
    loadingRequests = false;
  });

  async function handleSubmit() {
    if (!title.trim() || !description.trim()) return;
    submitting = true;
    message = '';
    try {
      await submitHelpRequest({
        title,
        description,
        urgency,
        target_department: targetDepartment || undefined,
      });
      message = 'Help request submitted. The Observer has been notified.';
      isError = false;
      title = ''; description = ''; targetDepartment = '';
      try { requests = await getMyHelpRequests(); } catch {}
    } catch (e: any) {
      message = e?.toString() ?? 'Failed to submit.';
      isError = true;
    } finally {
      submitting = false;
    }
  }

  function statusColor(s: string): string {
    switch (s) {
      case 'resolved': return '#22C55E';
      case 'in_progress': return '#F59E0B';
      case 'rejected': return '#EF4444';
      default: return '#94A3B8';
    }
  }
</script>

<h2>Cross-Department Help Request</h2>

<div class="tabs">
  <button class:active={tab === 'form'} onclick={() => (tab = 'form')}>New Request</button>
  <button class:active={tab === 'history'} onclick={() => (tab = 'history')}>My Requests ({requests.length})</button>
</div>

{#if tab === 'form'}
  <p class="hint">Submit a help request to The Observer. They will route it to the appropriate department.</p>
  <div class="form-card">
    <div class="form-group">
      <label for="title">Title *</label>
      <input id="title" type="text" bind:value={title} placeholder="Brief request title" />
    </div>
    <div class="form-group">
      <label for="desc">Description *</label>
      <textarea id="desc" bind:value={description} rows="4" placeholder="Describe the help needed…"></textarea>
    </div>
    <div class="row">
      <div class="form-group" style="flex:1">
        <label for="urgency">Urgency</label>
        <select id="urgency" bind:value={urgency}>
          <option value="low">Low</option>
          <option value="medium">Medium</option>
          <option value="high">High</option>
          <option value="critical">Critical</option>
        </select>
      </div>
      <div class="form-group" style="flex:1">
        <label for="dept">Target Department</label>
        <input id="dept" bind:value={targetDepartment} placeholder="e.g. Medical, Security" />
      </div>
    </div>
    <button class="btn-primary" onclick={handleSubmit} disabled={submitting || !title.trim() || !description.trim()}>
      {submitting ? 'Submitting…' : 'Submit Help Request'}
    </button>
    {#if message}<p class:msg-ok={!isError} class:msg-err={isError}>{message}</p>{/if}
  </div>
{:else}
  {#if loadingRequests}
    <p class="muted">Loading…</p>
  {:else if requests.length === 0}
    <p class="muted">You have not submitted any help requests.</p>
  {:else}
    <table class="tbl">
      <thead><tr><th>Title</th><th>Status</th><th>Submitted</th></tr></thead>
      <tbody>
        {#each requests as r}
          <tr>
            <td>{r.title}</td>
            <td><span class="badge" style="color:{statusColor(r.status)}">{r.status}</span></td>
            <td>{new Date(r.created_at).toLocaleDateString()}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.75rem; }
  .tabs { display:flex;gap:0.5rem;margin-bottom:1rem; }
  .tabs button { background:transparent;border:1px solid #334155;color:#94A3B8;padding:0.35rem 0.75rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .tabs button.active { border-color:#3ABEFF;color:#3ABEFF;background:rgba(58,190,255,0.1); }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:1rem; }
  .form-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:600px; }
  .row { display:flex;gap:1rem; }
  .form-group { margin-bottom:0.75rem; }
  .form-group label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.25rem; }
  .form-group input, .form-group textarea, .form-group select { width:100%;background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem;font-size:0.8rem; }
  .form-group textarea { resize:vertical;font-family:'Fira Code',monospace; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .msg-ok { color:#22C55E;font-size:0.8rem;margin-top:0.5rem; }
  .msg-err { color:#EF4444;font-size:0.8rem;margin-top:0.5rem; }
  .muted { color:#64748B;font-size:0.85rem; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.5rem;color:#94A3B8;border-bottom:1px solid rgba(58,190,255,0.15);font-size:0.7rem;text-transform:uppercase; }
  .tbl td { padding:0.5rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .badge { font-weight:600;font-size:0.75rem; }
</style>
