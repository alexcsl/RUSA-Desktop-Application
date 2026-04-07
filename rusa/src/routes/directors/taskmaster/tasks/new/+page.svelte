<!--
  /directors/taskmaster/tasks/new — Assign Task (UC-TM-01)
  TheTaskmaster assigns tasks to personnel across divisions.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { assignTask, getPersonnelList, type PersonnelListItem } from '$lib/stores/directors';

  let personnel: PersonnelListItem[] = $state([]);
  let assigneeId = $state('');
  let title = $state('');
  let description = $state('');
  let dueDate = $state('');
  let priority = $state('medium');
  let loading = $state(false);
  let error = $state('');
  let success = $state('');

  onMount(async () => {
    try { personnel = await getPersonnelList(); } catch {}
  });

  async function handleSubmit() {
    error = ''; success = '';
    if (!assigneeId || !title.trim()) {
      error = 'Assignee and title are required.';
      return;
    }
    loading = true;
    try {
      await assignTask({
        assignee_id: assigneeId,
        title: title.trim(),
        description: description.trim() || undefined,
        due_date: dueDate || undefined,
        priority,
      });
      success = 'Task assigned successfully.';
      assigneeId = ''; title = ''; description = ''; dueDate = ''; priority = 'medium';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }
</script>

<h1 class="title">Assign Task</h1>
<p class="subtitle">Create and assign a task to a personnel member.</p>

<div class="form-card">
  <label class="field">
    <span class="label">Assignee *</span>
    <select class="input" bind:value={assigneeId}>
      <option value="">Select personnel…</option>
      {#each personnel as p}
        <option value={p.id}>{p.full_name} — {p.role_name}</option>
      {/each}
    </select>
  </label>
  <label class="field">
    <span class="label">Title *</span>
    <input class="input" type="text" bind:value={title} placeholder="Task title…" />
  </label>
  <label class="field">
    <span class="label">Description</span>
    <textarea class="textarea" bind:value={description} rows="4" placeholder="Task details…"></textarea>
  </label>
  <label class="field">
    <span class="label">Due Date</span>
    <input class="input" type="date" bind:value={dueDate} />
  </label>
  <label class="field">
    <span class="label">Priority</span>
    <select class="input" bind:value={priority}>
      <option value="low">Low</option>
      <option value="medium">Medium</option>
      <option value="high">High</option>
      <option value="critical">Critical</option>
    </select>
  </label>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}
  <button class="btn-primary" onclick={handleSubmit} disabled={loading}>
    {loading ? 'Assigning…' : 'Assign Task'}
  </button>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1.2rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.5rem;max-width:500px;display:flex;flex-direction:column;gap:0.75rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem 0.6rem;font-size:0.82rem;font-family:'Inter',sans-serif;width:100%;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;padding:0.55rem;cursor:pointer;font-size:0.85rem;font-weight:600;margin-top:0.25rem; }
  .btn-primary:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:default; }
  .error { color:#EF4444;font-size:0.75rem; }
  .success { color:#22C55E;font-size:0.75rem; }
</style>
