<!-- /scientists/tasks/conclude — Task Conclusion (all scientist roles) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getMyTasks, concludeTask, type ScientistTask } from '$lib/stores/scientists';

  let tasks: ScientistTask[] = $state([]);
  let selectedTaskId = $state('');
  let conclusionSummary = $state('');
  let submitting = $state(false);
  let error = $state('');
  let success = $state('');

  let activeTasks = $derived(tasks.filter((t) => t.status !== 'completed' && t.status !== 'cancelled'));

  onMount(async () => {
    try { tasks = await getMyTasks(); } catch {}
  });

  async function handleSubmit() {
    error = ''; success = '';
    if (!selectedTaskId) { error = 'Please select a task.'; return; }
    if (!conclusionSummary.trim()) { error = 'Conclusion summary is required.'; return; }

    submitting = true;
    try {
      await concludeTask({ task_id: selectedTaskId, conclusion_summary: conclusionSummary });
      success = 'Task concluded successfully.';
      conclusionSummary = ''; selectedTaskId = '';
      tasks = await getMyTasks();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<h1 class="title">Task Conclusion</h1>
<p class="subtitle">Mark an assigned task as completed and submit your conclusion summary.</p>

{#if activeTasks.length === 0 && tasks.length > 0}
  <div class="empty">All your tasks are already completed.</div>
{:else}
  <div class="form-card">
    <label class="field">
      <span class="label">Task *</span>
      <select class="input" bind:value={selectedTaskId}>
        <option value="">— Select a task to conclude —</option>
        {#each activeTasks as t}
          <option value={t.id}>[{t.status}] {t.title}</option>
        {/each}
      </select>
    </label>

    <label class="field">
      <span class="label">Conclusion Summary *</span>
      <textarea class="textarea" bind:value={conclusionSummary} rows="5"
        placeholder="Summarise what was accomplished, findings, and the outcome of this task…"></textarea>
    </label>

    {#if error}<p class="error">{error}</p>{/if}
    {#if success}<p class="success">{success}</p>{/if}

    <button class="btn-primary" onclick={handleSubmit} disabled={submitting}>
      {submitting ? 'Submitting…' : 'Conclude Task'}
    </button>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.78rem;margin:0 0 0.75rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:600px;display:flex;flex-direction:column;gap:0.6rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .empty { background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:1.25rem;text-align:center;color:#94A3B8;font-size:0.85rem; }
</style>
