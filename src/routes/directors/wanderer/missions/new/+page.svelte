<!-- /directors/wanderer/missions/new — TheWanderer Territory & Missions (UC-DIR-14) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    renameTerritory, getTerritories, assignTask, getSubordinateTasks, getPersonnelList,
    type TerritorySummary, type TaskSummary, type PersonnelListItem,
  } from '$lib/stores/directors';

  let territories: TerritorySummary[] = $state([]);
  let tasks: TaskSummary[] = $state([]);
  let personnel: PersonnelListItem[] = $state([]);

  // Rename form
  let selectedTerritory = $state('');
  let newName = $state('');
  let renameError = $state('');
  let renameSuccess = $state('');

  // Task form
  let assignedTo = $state('');
  let taskTitle = $state('');
  let taskDesc = $state('');
  let dueDate = $state('');
  let taskError = $state('');
  let taskSuccess = $state('');

  const validRoles = ['Astronaut'];

  onMount(async () => {
    const [t, ts, p] = await Promise.all([getTerritories(), getSubordinateTasks(), getPersonnelList()]);
    territories = t;
    tasks = ts;
    personnel = p.filter(x => validRoles.includes(x.role_name));
  });

  async function handleRename() {
    renameError = ''; renameSuccess = '';
    if (!selectedTerritory || !newName.trim()) { renameError = 'Select territory and enter name.'; return; }
    try {
      await renameTerritory(selectedTerritory, newName);
      renameSuccess = 'Territory renamed.';
      newName = '';
      territories = await getTerritories();
    } catch (e: unknown) { renameError = e instanceof Error ? e.message : String(e); }
  }

  async function handleAssign() {
    taskError = ''; taskSuccess = '';
    if (!assignedTo || !taskTitle.trim()) { taskError = 'Assignee and title required.'; return; }
    try {
      await assignTask({
        assigned_to: assignedTo, title: taskTitle,
        description: taskDesc || undefined, task_type: 'mission',
        due_date: dueDate || undefined,
      });
      taskSuccess = 'Mission assigned.';
      taskTitle = ''; taskDesc = ''; assignedTo = '';
      tasks = await getSubordinateTasks();
    } catch (e: unknown) { taskError = e instanceof Error ? e.message : String(e); }
  }
</script>

<h1 class="title">Wanderer — Territory & Missions</h1>
<p class="subtitle">Rename territories and assign exploration missions to Astronauts.</p>

<div class="layout">
  <!-- Territory naming -->
  <div class="form-card">
    <h2>Territories</h2>
    {#each territories as t}
      <div class="terr-item">
        <span class="terr-name">{t.name}</span>
        <span class="terr-type">{t.territory_type}</span>
        {#if t.previous_name}<span class="terr-prev">was: {t.previous_name}</span>{/if}
      </div>
    {:else}
      <p class="empty">No territories recorded.</p>
    {/each}

    <h3>Rename Territory</h3>
    <select class="input" bind:value={selectedTerritory}>
      <option value="">— Select —</option>
      {#each territories as t}<option value={t.id}>{t.name}</option>{/each}
    </select>
    <input type="text" class="input" bind:value={newName} placeholder="New name…" />
    {#if renameError}<p class="error">{renameError}</p>{/if}
    {#if renameSuccess}<p class="success">{renameSuccess}</p>{/if}
    <button class="btn-primary" onclick={handleRename}>Rename</button>
  </div>

  <!-- Mission assignment -->
  <div class="form-card">
    <h2>Assign Mission</h2>
    <label class="field"><span class="label">Astronaut</span>
      <select class="input" bind:value={assignedTo}>
        <option value="">— Select —</option>
        {#each personnel as p}<option value={p.id}>{p.full_name}</option>{/each}
      </select>
    </label>
    <label class="field"><span class="label">Mission Title</span>
      <input type="text" class="input" bind:value={taskTitle} placeholder="e.g. Survey Sector 9" />
    </label>
    <label class="field"><span class="label">Description</span>
      <textarea class="textarea" bind:value={taskDesc} rows="3"></textarea>
    </label>
    <label class="field"><span class="label">Due Date</span>
      <input type="date" class="input" bind:value={dueDate} />
    </label>
    {#if taskError}<p class="error">{taskError}</p>{/if}
    {#if taskSuccess}<p class="success">{taskSuccess}</p>{/if}
    <button class="btn-primary" onclick={handleAssign}>Assign Mission</button>

    <h3>Active Missions</h3>
    {#each tasks as task}
      <div class="task-item">
        <span class="task-name">{task.title}</span>
        <span class="badge">{task.status}</span>
      </div>
    {:else}
      <p class="empty">No missions.</p>
    {/each}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .layout { display:flex;gap:1.5rem;flex-wrap:wrap; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;flex:1;min-width:320px;display:flex;flex-direction:column;gap:0.5rem; }
  .form-card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0; }
  h3 { font-size:0.8rem;color:#E6EDF3;margin:0.75rem 0 0.3rem; }
  .terr-item { display:flex;gap:0.75rem;align-items:center;font-size:0.8rem;padding:0.3rem 0;border-bottom:1px solid rgba(255,255,255,0.03); }
  .terr-name { font-weight:500; }
  .terr-type { color:#8B5CF6;font-size:0.7rem; }
  .terr-prev { color:#94A3B8;font-size:0.7rem;font-style:italic; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .task-item { display:flex;justify-content:space-between;align-items:center;font-size:0.8rem;padding:0.25rem 0;border-bottom:1px solid rgba(255,255,255,0.03); }
  .task-name { font-weight:500; }
  .badge { padding:0.1rem 0.35rem;border-radius:4px;font-size:0.65rem;background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .btn-primary { padding:0.45rem 1rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .empty { color:#475569;font-size:0.8rem; }
</style>
