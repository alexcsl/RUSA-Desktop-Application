<!--
  UC-SC-01: Assign Task to Settler (Commander only)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { stlAssignTask, stlGetDashboard, type DashboardSettler } from '$lib/stores/settlers';

  let settlers: DashboardSettler[] = $state([]);
  let assignedTo = $state('');
  let title = $state('');
  let description = $state('');
  let scope = $state('maintenance');
  let urgency = $state('medium');
  let deadline = $state('');
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  onMount(async () => {
    try {
      const dash = await stlGetDashboard();
      settlers = dash.settlers;
    } catch (e: any) { error = e?.message ?? String(e); }
  });

  async function handleSubmit() {
    error = ''; success = '';
    if (!assignedTo) { error = 'Select a settler.'; return; }
    if (!title.trim()) { error = 'Title is required.'; return; }
    if (!deadline) { error = 'Deadline is required.'; return; }
    submitting = true;
    try {
      const id = await stlAssignTask({
        assigned_to: assignedTo,
        title,
        description,
        scope,
        urgency,
        deadline,
      });
      success = `Task assigned (ID: ${id.slice(0,8)}…).`;
      title = ''; description = ''; assignedTo = ''; deadline = '';
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }
</script>

<h2>Assign Task to Settler</h2>

<form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  <label>
    Settler *
    <select bind:value={assignedTo}>
      <option value="">— Select settler —</option>
      {#each settlers as s}
        <option value={s.user_id}>{s.full_name} ({s.role_name}){s.house_arrest ? ' 🔒' : ''}</option>
      {/each}
    </select>
  </label>

  <label>
    Title *
    <input type="text" bind:value={title} placeholder="Task title" />
  </label>

  <label>
    Description
    <textarea bind:value={description} rows="4" placeholder="Detailed instructions…"></textarea>
  </label>

  <div class="row">
    <label>
      Scope
      <select bind:value={scope}>
        <option value="maintenance">Maintenance</option>
        <option value="construction">Construction</option>
        <option value="farming">Farming</option>
        <option value="exploration">Exploration</option>
        <option value="logistics">Logistics</option>
        <option value="medical">Medical</option>
        <option value="other">Other</option>
      </select>
    </label>

    <label>
      Urgency
      <select bind:value={urgency}>
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
        <option value="critical">Critical</option>
      </select>
    </label>

    <label>
      Deadline *
      <input type="date" bind:value={deadline} />
    </label>
  </div>

  {#if error}<p class="err">{error}</p>{/if}
  {#if success}<p class="ok">{success}</p>{/if}

  <button type="submit" class="btn-primary" disabled={submitting}>
    {submitting ? 'Assigning…' : 'Assign Task'}
  </button>
</form>

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
  .form { display:flex;flex-direction:column;gap:0.75rem;max-width:560px; }
  .form label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8; }
  .form input, .form select, .form textarea { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem; }
  .form textarea { resize:vertical; }
  .row { display:flex;gap:1rem; }
  .row label { flex:1; }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem; }
  .btn-primary { background:#3ABEFF;color:#0B0F1A;border:none;border-radius:4px;padding:0.5rem 1.2rem;cursor:pointer;font-weight:600;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:#60CFFF; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
