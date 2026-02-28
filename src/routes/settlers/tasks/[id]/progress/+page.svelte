<!--
  UC-PS-02: Submit Progress Report (linked to task)
-->
<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { stlGetTaskDetail, stlSubmitProgressReport, type SettlerTaskDetail } from '$lib/stores/settlers';

  let taskId = $state('');
  const unsubPage = page.subscribe((p) => { taskId = p.params.id ?? ''; });
  import { onDestroy } from 'svelte';
  onDestroy(() => unsubPage());

  let task: SettlerTaskDetail | null = $state(null);
  let loading = $state(true);
  let error = $state('');
  let success = $state('');

  let week = $state('');
  let ragStatus = $state('green');
  let progressMade = $state('');
  let materialsEquipment = $state('');
  let submitting = $state(false);

  onMount(async () => {
    try { task = await stlGetTaskDetail(taskId); }
    catch (e: any) { error = e?.message ?? String(e); }
    finally { loading = false; }
  });

  async function handleSubmit() {
    error = ''; success = '';
    if (!progressMade.trim()) { error = 'Progress description is required.'; return; }
    submitting = true;
    try {
      await stlSubmitProgressReport({
        task_id: taskId,
        week: week || undefined,
        rag_status: ragStatus || undefined,
        progress_made: progressMade,
        materials_equipment: materialsEquipment || undefined,
      });
      success = 'Progress report submitted successfully.';
      progressMade = ''; materialsEquipment = ''; week = '';
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }
</script>

<h2>Submit Progress Report</h2>

{#if loading}
  <p class="dim">Loading task…</p>
{:else if task}
  <div class="task-info">
    <strong>{task.title}</strong>
    <span class="meta">Scope: {task.scope ?? '—'} | Status: {task.status} | Deadline: {task.deadline ?? '—'}</span>
  </div>

  <form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    <label>
      Week
      <input type="text" bind:value={week} placeholder="e.g. Week 3" />
    </label>

    <label>
      RAG Status
      <select bind:value={ragStatus}>
        <option value="green">Green</option>
        <option value="amber">Amber</option>
        <option value="red">Red</option>
      </select>
    </label>

    <label>
      Progress Made *
      <textarea bind:value={progressMade} rows="4" required></textarea>
    </label>

    <label>
      Materials / Equipment Used
      <textarea bind:value={materialsEquipment} rows="2"></textarea>
    </label>

    {#if error}<p class="err">{error}</p>{/if}
    {#if success}<p class="ok">{success}</p>{/if}

    <button type="submit" class="btn-primary" disabled={submitting}>
      {submitting ? 'Submitting…' : 'Submit Report'}
    </button>
  </form>
{:else}
  <p class="err">Task not found.</p>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
  .dim { color:#64748B;font-size:0.85rem; }
  .task-info { background:#1F2937;padding:0.75rem;border-radius:6px;margin-bottom:1rem; }
  .task-info strong { display:block;color:#E6EDF3;font-size:0.9rem; }
  .meta { font-size:0.75rem;color:#94A3B8; }
  .form { display:flex;flex-direction:column;gap:0.75rem;max-width:560px; }
  .form label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8; }
  .form input, .form select, .form textarea { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem; }
  .form textarea { resize:vertical; }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem; }
  .btn-primary { background:#3ABEFF;color:#0B0F1A;border:none;border-radius:4px;padding:0.5rem 1.2rem;cursor:pointer;font-weight:600;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:#60CFFF; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
