<!-- /directors/nomad/relocate — TheNomad Personnel Relocation + Task Assignment (UC-NOM-01/02/03) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { relocatePersonnel, assignTask, getSubordinateTasks, getPersonnelList, type PersonnelListItem, type TaskSummary } from '$lib/stores/directors';

  const DIRECTOR_AND_ADMIN_ROLES = [
    'GeneralDirector','TheDirector','TheAccountant','TheLibrarian',
    'TheNomad','TheArtificer','TheObserver','TheWanderer',
    'TheTaskmaster','TheGuardian','TheStatistician','TheCoordinator',
    'TheOverseer','TheAnchorman','Administrator',
  ];

  const NOMAD_TASK_ROLES = ['SettlerCommander', 'HeadOfSanitary'];

  let personnel: PersonnelListItem[] = $state([]);
  let relocatablePersonnel: PersonnelListItem[] = $state([]);
  let taskablePersonnel: PersonnelListItem[] = $state([]);
  let tasks: TaskSummary[] = $state([]);
  let activeTab: 'relocate' | 'tasks' = $state('relocate');

  // Relocation form
  let targetUserId = $state('');
  let origin = $state('');
  let destination = $state('');
  let relocationType: 'temporary' | 'permanent' = $state('temporary');
  let effectiveDate = $state('');
  let error = $state('');
  let success = $state('');

  // Task form
  let taskAssignee = $state('');
  let taskTitle = $state('');
  let taskDesc = $state('');
  let taskDueDate = $state('');
  let taskError = $state('');
  let taskSuccess = $state('');

  onMount(async () => {
    const [p, t] = await Promise.all([getPersonnelList(), getSubordinateTasks()]);
    personnel = p;
    tasks = t;
    relocatablePersonnel = p.filter((x) => !DIRECTOR_AND_ADMIN_ROLES.includes(x.role_name));
    taskablePersonnel = p.filter((x) => NOMAD_TASK_ROLES.includes(x.role_name));
    effectiveDate = new Date().toISOString().split('T')[0];
  });

  async function handleSubmit() {
    error = ''; success = '';
    if (!targetUserId || !origin.trim() || !destination.trim() || !effectiveDate) {
      error = 'All fields are required.'; return;
    }
    try {
      const id = await relocatePersonnel({
        target_user_id: targetUserId, origin_location: origin,
        destination, relocation_type: relocationType, effective_date: effectiveDate,
      });
      const name = relocatablePersonnel.find(p => p.id === targetUserId)?.full_name ?? 'User';
      success = `${name} relocation recorded (ID: ${id.slice(0,8)}…)`;
      targetUserId = ''; origin = ''; destination = '';
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }

  async function handleTaskAssign() {
    taskError = ''; taskSuccess = '';
    if (!taskAssignee || !taskTitle.trim()) { taskError = 'Assignee and title required.'; return; }
    try {
      const id = await assignTask({
        assigned_to: taskAssignee, title: taskTitle,
        description: taskDesc || undefined, task_type: 'directive',
        due_date: taskDueDate || undefined,
      });
      taskSuccess = `Task assigned (${id.slice(0, 8)}…)`;
      taskTitle = ''; taskDesc = ''; taskAssignee = '';
      tasks = await getSubordinateTasks();
    } catch (e: unknown) { taskError = e instanceof Error ? e.message : String(e); }
  }
</script>

<h1 class="title">Nomad — Personnel & Tasks</h1>
<p class="subtitle">Relocate personnel and assign directives to Settler Commanders and Sanitary Heads.</p>

<div class="tabs">
  <button class="tab" class:active={activeTab === 'relocate'} onclick={() => (activeTab = 'relocate')}>Relocations</button>
  <button class="tab" class:active={activeTab === 'tasks'} onclick={() => (activeTab = 'tasks')}>Task Assignment</button>
</div>

{#if activeTab === 'relocate'}
<div class="form-card">
  <h2>Relocate Personnel</h2>
  <label class="field">
    <span class="label">Personnel</span>
    <select class="input" bind:value={targetUserId}>
      <option value="">— Select —</option>
      {#each relocatablePersonnel as p}
        <option value={p.id}>{p.full_name} ({p.role_name})</option>
      {/each}
    </select>
  </label>

  <label class="field">
    <span class="label">Origin Location</span>
    <input type="text" class="input" bind:value={origin} placeholder="Base Alpha, Lab 3…" />
  </label>

  <label class="field">
    <span class="label">Destination</span>
    <input type="text" class="input" bind:value={destination} placeholder="Base Beta, Sector 7…" />
  </label>

  <label class="field">
    <span class="label">Type</span>
    <select class="input" bind:value={relocationType}>
      <option value="temporary">Temporary</option>
      <option value="permanent">Permanent</option>
    </select>
  </label>

  <label class="field">
    <span class="label">Effective Date</span>
    <input type="date" class="input" bind:value={effectiveDate} />
  </label>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}

  <button class="btn-primary" onclick={handleSubmit}>Submit Relocation</button>
</div>
{:else}
<div class="layout">
  <div class="form-card">
    <h2>Assign Task</h2>
    <label class="field"><span class="label">Assignee</span>
      <select class="input" bind:value={taskAssignee}>
        <option value="">— Select —</option>
        {#each taskablePersonnel as p}<option value={p.id}>{p.full_name} ({p.role_name})</option>{/each}
      </select>
    </label>
    <label class="field"><span class="label">Title</span>
      <input type="text" class="input" bind:value={taskTitle} placeholder="Task title…" />
    </label>
    <label class="field"><span class="label">Description</span>
      <textarea class="textarea" bind:value={taskDesc} rows="3" placeholder="Directive details…"></textarea>
    </label>
    <label class="field"><span class="label">Due Date</span>
      <input type="date" class="input" bind:value={taskDueDate} />
    </label>
    {#if taskError}<p class="error">{taskError}</p>{/if}
    {#if taskSuccess}<p class="success">{taskSuccess}</p>{/if}
    <button class="btn-primary" onclick={handleTaskAssign}>Assign Task</button>
  </div>

  <div class="tasks-panel">
    <h2>Assigned Tasks</h2>
    {#each tasks as t}
      <div class="card">
        <div class="card-header">
          <span class="card-title">{t.title}</span>
          <span class="badge">{t.status}</span>
        </div>
        <span class="meta">{t.assignee_name}{t.due_date ? ` — Due: ${t.due_date}` : ''}</span>
      </div>
    {:else}
      <p class="empty">No tasks assigned.</p>
    {/each}
  </div>
</div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .tabs { display:flex;gap:0.5rem;margin-bottom:1rem; }
  .tab { background:transparent;border:1px solid rgba(58,190,255,0.15);color:#94A3B8;padding:0.4rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .tab.active { background:rgba(58,190,255,0.1);border-color:#3ABEFF;color:#3ABEFF; }
  .layout { display:flex;gap:1.5rem;flex-wrap:wrap; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.5rem;max-width:560px;display:flex;flex-direction:column;gap:0.75rem; }
  .form-card h2,.tasks-panel h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0 0 0.5rem; }
  .tasks-panel { flex:1;min-width:280px; }
  .field { display:flex;flex-direction:column;gap:0.25rem; }
  .label { font-size:0.75rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.5rem;font-size:0.85rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  select.input { cursor:pointer; }
  .btn-primary { padding:0.55rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.85rem;align-self:flex-start; }
  .btn-primary:hover { box-shadow:0 0 12px rgba(58,190,255,0.6); }
  .card { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.6rem;margin-bottom:0.35rem; }
  .card-header { display:flex;justify-content:space-between;align-items:center; }
  .card-title { font-size:0.8rem;font-weight:500; }
  .badge { font-size:0.65rem;padding:0.1rem 0.4rem;border-radius:4px;background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .meta { font-size:0.7rem;color:#94A3B8; }
  .empty { color:#475569;font-size:0.8rem;font-style:italic; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
</style>
