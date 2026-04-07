<!-- /scientists/tasks/progress — Task Progress Report (UC-GS-01: progress update) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getMyTasks, submitTaskProgress, type ScientistTask } from '$lib/stores/scientists';

  let tasks: ScientistTask[] = $state([]);
  let selectedTaskId = $state('');
  let progressSummary = $state('');
  let ragStatus = $state('');
  let submitting = $state(false);
  let error = $state('');
  let success = $state('');

  let pendingTasks = $derived(tasks.filter((t) => t.status !== 'completed' && t.status !== 'cancelled'));

  onMount(async () => {
    try { tasks = await getMyTasks(); } catch {}
  });

  async function handleSubmit() {
    error = ''; success = '';
    if (!selectedTaskId) { error = 'Please select a task.'; return; }
    if (!progressSummary.trim()) { error = 'Progress summary is required.'; return; }

    submitting = true;
    try {
      await submitTaskProgress({
        task_id: selectedTaskId,
        progress_summary: progressSummary,
        rag_status: ragStatus || undefined,
      });
      success = 'Progress report submitted.';
      progressSummary = ''; ragStatus = ''; selectedTaskId = '';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<h1 class="title">Task Progress Report</h1>
<p class="subtitle">Submit a progress update for an active assigned task.</p>

<div class="form-card">
  <label class="field">
    <span class="label">Task *</span>
    <select class="input" bind:value={selectedTaskId}>
      <option value="">— Select a task —</option>
      {#each pendingTasks as t}
        <option value={t.id}>{t.title} <span>({t.task_type})</span></option>
      {/each}
    </select>
  </label>

  <div class="form-row">
    <label class="field">
      <span class="label">RAG Status</span>
      <select class="input" bind:value={ragStatus}>
        <option value="">Not specified</option>
        <option value="green">Green — On track</option>
        <option value="amber">Amber — At risk</option>
        <option value="red">Red — Blocked</option>
      </select>
    </label>
  </div>

  <label class="field">
    <span class="label">Progress Summary *</span>
    <textarea class="textarea" bind:value={progressSummary} rows="5"
      placeholder="Describe the current progress, actions completed, and any blockers…"></textarea>
  </label>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}

  <button class="btn-primary" onclick={handleSubmit} disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Progress Report'}
  </button>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.78rem;margin:0 0 0.75rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:600px;display:flex;flex-direction:column;gap:0.6rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .form-row { display:grid;grid-template-columns:1fr;gap:0.6rem; }
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
</style>
