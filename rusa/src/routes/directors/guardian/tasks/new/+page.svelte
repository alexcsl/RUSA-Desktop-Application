<!--
  Guardian: Assign Security Task — assign a task to a security team member.
  Access: TheGuardian
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getPersonnelList, assignTask, type PersonnelListItem } from '$lib/stores/directors';
  import { goto } from '$app/navigation';

  const GUARDIAN_TARGET_ROLES = ['GalacticSecurityHead', 'GalacticSecurityStaff', 'PlanetarySecurityHead', 'SecuritySupervisor', 'SecurityOfficer'];

  let loading = $state(false);
  let loadingPeople = $state(true);
  let error = $state('');
  let success = $state('');

  let personnel: PersonnelListItem[] = $state([]);
  let targets = $derived(personnel.filter((p) => GUARDIAN_TARGET_ROLES.includes(p.role_name)));

  let assignedTo = $state('');
  let title = $state('');
  let description = $state('');
  let taskType = $state('general');
  let dueDate = $state('');

  onMount(async () => {
    try {
      personnel = await getPersonnelList();
    } catch (err) {
      error = String(err);
    } finally {
      loadingPeople = false;
    }
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!assignedTo || !title.trim()) {
      error = 'Please select an assignee and enter a task title.';
      return;
    }
    loading = true;
    error = '';
    success = '';
    try {
      await assignTask({
        assigned_to: assignedTo,
        title: title.trim(),
        description: description.trim() || undefined,
        task_type: taskType,
        due_date: dueDate || undefined,
      });
      success = 'Task assigned successfully.';
      assignedTo = '';
      title = '';
      description = '';
      taskType = 'general';
      dueDate = '';
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }
</script>

<div class="page">
  <h2>Assign Security Task</h2>
  <p class="subtitle">Assign a task to a security team member.</p>

  {#if success}
    <div class="banner success">
      {success}
      <button class="link" onclick={() => goto('/directors/guardian/tasks')}>View all tasks →</button>
    </div>
  {/if}
  {#if error}
    <div class="banner error">{error}</div>
  {/if}

  {#if loadingPeople}
    <div class="loading">Loading personnel…</div>
  {:else}
    <form onsubmit={handleSubmit} class="form">
      <div class="field">
        <label for="assignedTo">Assign To</label>
        <select id="assignedTo" bind:value={assignedTo} required>
          <option value="">— Select personnel —</option>
          {#each targets as p}
            <option value={p.id}>{p.full_name} ({p.role_name})</option>
          {/each}
        </select>
        {#if targets.length === 0}
          <span class="hint">No eligible security personnel found.</span>
        {/if}
      </div>

      <div class="field">
        <label for="title">Task Title</label>
        <input id="title" type="text" bind:value={title} placeholder="e.g. Complete sector patrol report" required />
      </div>

      <div class="row2">
        <div class="field">
          <label for="taskType">Task Type</label>
          <select id="taskType" bind:value={taskType}>
            <option value="general">General</option>
            <option value="inspection">Inspection</option>
            <option value="report">Report</option>
            <option value="maintenance">Maintenance</option>
            <option value="logistics">Logistics</option>
            <option value="relocation">Relocation</option>
          </select>
        </div>
        <div class="field">
          <label for="dueDate">Due Date (optional)</label>
          <input id="dueDate" type="date" bind:value={dueDate} />
        </div>
      </div>

      <div class="field">
        <label for="description">Description (optional)</label>
        <textarea id="description" bind:value={description} rows="4" placeholder="Additional details or instructions…"></textarea>
      </div>

      <div class="actions">
        <button type="submit" class="btn-primary" disabled={loading || targets.length === 0}>
          {loading ? 'Assigning…' : 'Assign Task'}
        </button>
        <button type="button" class="btn-secondary" onclick={() => goto('/directors/guardian/tasks')}>
          View Tasks
        </button>
      </div>
    </form>
  {/if}
</div>

<style>
  .page { max-width:600px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.25rem; }
  .subtitle { color:#64748B;font-size:0.8rem;margin:0 0 1.25rem; }
  .loading { color:#94A3B8;font-size:0.85rem; }
  .banner { padding:0.6rem 1rem;border-radius:6px;font-size:0.8rem;margin-bottom:1rem;display:flex;align-items:center;gap:0.75rem; }
  .banner.success { background:rgba(16,185,129,0.15);color:#10B981;border:1px solid rgba(16,185,129,0.3); }
  .banner.error { background:rgba(239,68,68,0.15);color:#EF4444;border:1px solid rgba(239,68,68,0.3); }
  .link { background:none;border:none;color:#3ABEFF;cursor:pointer;font-size:0.8rem;text-decoration:underline; }
  .form { display:flex;flex-direction:column;gap:0.9rem; }
  .field { display:flex;flex-direction:column;gap:0.3rem; }
  .row2 { display:grid;grid-template-columns:1fr 1fr;gap:0.9rem; }
  .hint { font-size:0.72rem;color:#F59E0B; }
  label { font-size:0.75rem;color:#94A3B8;font-weight:500; }
  input,select,textarea { background:#1F2937;border:1px solid #374151;color:#E6EDF3;border-radius:6px;padding:0.45rem 0.6rem;font-size:0.82rem;font-family:inherit; }
  input:focus,select:focus,textarea:focus { outline:none;border-color:#3ABEFF; }
  textarea { resize:vertical; }
  .actions { display:flex;gap:0.6rem;margin-top:0.25rem; }
  .btn-primary { padding:0.55rem 1.2rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.85rem;font-weight:600; }
  .btn-primary:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-secondary { padding:0.55rem 1.2rem;background:transparent;border:1px solid #374151;color:#94A3B8;border-radius:6px;cursor:pointer;font-size:0.85rem; }
  .btn-secondary:hover { color:#E6EDF3;border-color:#3ABEFF; }
</style>
