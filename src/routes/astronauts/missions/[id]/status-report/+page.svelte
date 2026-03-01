<!-- /astronauts/missions/[id]/status-report — UC-AS-03: Submit Status Report -->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { submitStatusReport } from '$lib/stores/astronauts';

  let missionId = $state('');
  const unsub = page.subscribe((p) => (missionId = p.params.id ?? ''));
  onDestroy(unsub);

  // Form state
  let reportDate = $state(new Date().toISOString().slice(0, 10));
  let monthTracker = $state('');
  let ragStatus = $state<'red' | 'amber' | 'green' | ''>('');
  let currentStatus = $state('');
  let issuesBlockers = $state('');
  let collectedSamples = $state('');
  let progressLastMonth = $state('');
  let plansNextMonth = $state('');

  let submitting = $state(false);
  let error = $state('');
  let success = $state('');

  async function handleSubmit() {
    error = '';
    success = '';
    if (!currentStatus.trim()) {
      error = 'Current status is required.';
      return;
    }
    submitting = true;
    try {
      await submitStatusReport({
        mission_id: missionId,
        report_date: reportDate,
        month_tracker: monthTracker || undefined,
        rag_status: ragStatus || undefined,
        current_status: currentStatus,
        issues_blockers: issuesBlockers || undefined,
        collected_samples_last_month: collectedSamples || undefined,
        progress_last_month: progressLastMonth || undefined,
        plans_next_month: plansNextMonth || undefined,
      });
      success = 'Status report submitted successfully.';
      setTimeout(() => goto(`/astronauts/missions/${missionId}`), 1500);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<a class="btn-back" href="/astronauts/missions/{missionId}">← Back to Mission</a>

<h1 class="title">Submit Status Report</h1>
<p class="subtitle">Mission status report for mission {missionId.slice(0, 8)}…</p>

<div class="form-card">
  <label class="field">
    <span class="label">Report Date</span>
    <input type="date" class="input" bind:value={reportDate} />
  </label>

  <label class="field">
    <span class="label">Month Tracker</span>
    <input type="text" class="input" bind:value={monthTracker} placeholder="e.g. Month 3 of 12" />
  </label>

  <fieldset class="rag-fieldset">
    <legend class="label">RAG Status</legend>
    <div class="rag-group">
      <label class="rag-option">
        <input type="radio" bind:group={ragStatus} value="green" />
        <span class="rag-chip rag-green">Green</span>
      </label>
      <label class="rag-option">
        <input type="radio" bind:group={ragStatus} value="amber" />
        <span class="rag-chip rag-amber">Amber</span>
      </label>
      <label class="rag-option">
        <input type="radio" bind:group={ragStatus} value="red" />
        <span class="rag-chip rag-red">Red</span>
      </label>
    </div>
  </fieldset>

  <label class="field">
    <span class="label">Current Status <span class="required">*</span></span>
    <textarea class="textarea" bind:value={currentStatus} rows="3" placeholder="Describe current mission status…"></textarea>
  </label>

  <label class="field">
    <span class="label">Issues / Blockers</span>
    <textarea class="textarea" bind:value={issuesBlockers} rows="2" placeholder="Any issues or blockers?"></textarea>
  </label>

  <label class="field">
    <span class="label">Collected Samples (last month)</span>
    <textarea class="textarea" bind:value={collectedSamples} rows="2" placeholder="Specimens, data points…"></textarea>
  </label>

  <label class="field">
    <span class="label">Progress (last month)</span>
    <textarea class="textarea" bind:value={progressLastMonth} rows="2" placeholder="What was accomplished…"></textarea>
  </label>

  <label class="field">
    <span class="label">Plans (next month)</span>
    <textarea class="textarea" bind:value={plansNextMonth} rows="2" placeholder="Upcoming objectives…"></textarea>
  </label>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}

  <button class="btn-primary" onclick={handleSubmit} disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Report'}
  </button>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0.5rem 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .btn-back { color:#94A3B8;font-size:0.75rem;text-decoration:none; }
  .btn-back:hover { color:#E6EDF3; }

  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:640px;display:flex;flex-direction:column;gap:0.6rem; }

  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .required { color:#EF4444; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }

  .rag-fieldset { border:none;padding:0;margin:0; }
  .rag-group { display:flex;gap:0.5rem;margin-top:0.2rem; }
  .rag-option { cursor:pointer;display:flex;align-items:center;gap:0.25rem; }
  .rag-option input { display:none; }
  .rag-chip { padding:0.3rem 0.75rem;border-radius:20px;font-size:0.75rem;font-weight:600;border:2px solid transparent;transition:border-color 0.15s; }
  .rag-green { background:rgba(16,185,129,0.15);color:#10B981; }
  .rag-amber { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .rag-red { background:rgba(239,68,68,0.15);color:#EF4444; }
  .rag-option input:checked + .rag-green { border-color:#10B981; }
  .rag-option input:checked + .rag-amber { border-color:#F59E0B; }
  .rag-option input:checked + .rag-red { border-color:#EF4444; }

  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
</style>
