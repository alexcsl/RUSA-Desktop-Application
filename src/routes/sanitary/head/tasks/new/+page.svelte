<!-- /sanitary/head/tasks/new — UC-HS-05: Assign Task to Crew -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    sanAssignTask, sanGetAllTasks, sanGetStaffRoster,
    type SanitaryTask, type StaffRosterEntry,
  } from '$lib/stores/sanitary';

  let tasks: SanitaryTask[] = $state([]);
  let roster: StaffRosterEntry[] = $state([]);
  let error = $state('');
  let success = $state('');

  // Form
  let fTaskName = $state('');
  let fInstructions = $state('');
  let fTaskType: string = $state('cleaning');
  let fUrgency: string = $state('low');
  let fLocation = $state('');
  let fDueDate = $state('');
  let fAssignees: string[] = $state([]);

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = '';
    try {
      [tasks, roster] = await Promise.all([
        sanGetAllTasks(),
        sanGetStaffRoster(),
      ]);
    } catch (e: unknown) { error = String(e); }
  }

  function toggleAssignee(uid: string) {
    if (fAssignees.includes(uid)) {
      fAssignees = fAssignees.filter((a) => a !== uid);
    } else {
      fAssignees = [...fAssignees, uid];
    }
  }

  async function handleSubmit() {
    if (fAssignees.length === 0) { error = 'At least one assignee is required.'; return; }
    error = ''; success = '';
    try {
      await sanAssignTask({
        task_name: fTaskName.trim() || undefined,
        instructions: fInstructions.trim() || undefined,
        task_type: fTaskType,
        urgency: fUrgency || undefined,
        location: fLocation.trim() || undefined,
        due_date: fDueDate ? new Date(fDueDate + 'T23:59:59Z').toISOString() : undefined,
        assigned_to: fAssignees,
      });
      success = 'Task assigned and crew notified.';
      fTaskName = ''; fInstructions = ''; fTaskType = 'cleaning'; fUrgency = 'low'; fLocation = ''; fDueDate = ''; fAssignees = [];
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }

  function urgencyBadge(u: string): string {
    const m: Record<string, string> = { low: 'badge-info', medium: 'badge-warn', high: 'badge-err', critical: 'badge-err' };
    return m[u] ?? 'badge-info';
  }

  function statusBadge(s: string): string {
    const m: Record<string, string> = { pending: 'badge-default', in_progress: 'badge-warn', completed: 'badge-ok' };
    return m[s] ?? 'badge-default';
  }
</script>

<h1 class="title">Assign Task to Crew</h1>
<p class="subtitle">UC-HS-05 — Create tasks and assign to one or more staff members.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="form-card">
  <div class="row">
    <div class="form-group" style="flex:2;">
      <label for="taskname">Task Name</label>
      <input id="taskname" class="input" bind:value={fTaskName} placeholder="Task name (optional)" />
    </div>
    <div class="form-group">
      <label for="type">Type</label>
      <select id="type" class="input" bind:value={fTaskType}>
        <option value="cleaning">Cleaning</option>
        <option value="inspection">Inspection</option>
        <option value="disposal">Disposal</option>
        <option value="transport">Transport</option>
        <option value="other">Other</option>
      </select>
    </div>
    <div class="form-group">
      <label for="urgency">Urgency</label>
      <select id="urgency" class="input" bind:value={fUrgency}>
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
        <option value="critical">Critical</option>
      </select>
    </div>
  </div>
  <div class="form-group" style="margin-top:0.4rem;">
    <label for="instr">Instructions</label>
    <textarea id="instr" class="input" rows="3" bind:value={fInstructions} placeholder="Detailed instructions…"></textarea>
  </div>
  <div class="row" style="margin-top:0.4rem;">
    <div class="form-group">
      <label for="loc">Location</label>
      <input id="loc" class="input" bind:value={fLocation} placeholder="e.g. Sector B" />
    </div>
    <div class="form-group">
      <label for="due">Due Date</label>
      <input id="due" class="input" type="date" bind:value={fDueDate} />
    </div>
  </div>

  <div class="assignee-section">
    <span class="label">Assignees</span>
    <div class="assignee-chips">
      {#each roster as s}
        <button class="chip" class:chip-selected={fAssignees.includes(s.user_id)} onclick={() => toggleAssignee(s.user_id)}>
          {s.full_name} <span class="chip-role">({s.role_name})</span>
        </button>
      {/each}
    </div>
  </div>

  <button class="btn-primary" style="margin-top:0.6rem;" onclick={handleSubmit}>Assign Task</button>
</div>

<h2 class="section-title">Existing Tasks ({tasks.length})</h2>
{#each tasks as t}
  <div class="card">
    <div class="card-top">
      <span class="card-title">{t.task_name ?? t.task_type}</span>
      <div class="badge-row">
        {#if t.urgency}<span class="badge {urgencyBadge(t.urgency)}">{t.urgency}</span>{/if}
        <span class="badge {statusBadge(t.status)}">{t.status.replace(/_/g,' ')}</span>
      </div>
    </div>
    {#if t.instructions}<p class="card-desc">{t.instructions}</p>{/if}
    <div class="card-meta">
      Type: {t.task_type}{#if t.due_date} · Due: {t.due_date}{/if}
    </div>
  </div>
{:else}
  <p class="empty">No tasks yet.</p>
{/each}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .section-title { font-family:'Orbitron',sans-serif;font-size:0.95rem;color:#3ABEFF;margin:1.25rem 0 0.5rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-bottom:1rem; }
  .row { display:flex;gap:0.6rem;align-items:flex-end;flex-wrap:wrap; }
  .form-group { display:flex;flex-direction:column;gap:0.15rem;flex:1;min-width:130px; }
  .form-group label { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  select.input { cursor:pointer; }
  textarea.input { resize:vertical; }
  .assignee-section { margin-top:0.6rem; }
  .assignee-section .label { font-size:0.75rem;color:#94A3B8;display:block;margin-bottom:0.3rem; }
  .assignee-chips { display:flex;flex-wrap:wrap;gap:0.3rem; }
  .chip { padding:0.3rem 0.5rem;border-radius:6px;border:1px solid #475569;background:transparent;color:#94A3B8;cursor:pointer;font-size:0.75rem;transition:all .15s; }
  .chip:hover { border-color:#3ABEFF;color:#E6EDF3; }
  .chip-selected { background:rgba(58,190,255,0.15);border-color:#3ABEFF;color:#3ABEFF; }
  .chip-role { font-size:0.65rem;opacity:0.7; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.7rem;margin-bottom:0.4rem; }
  .card-top { display:flex;justify-content:space-between;align-items:center; }
  .card-title { font-size:0.85rem;font-weight:500; }
  .badge-row { display:flex;gap:0.3rem; }
  .badge { padding:0.12rem 0.35rem;border-radius:4px;font-size:0.6rem;text-transform:capitalize; }
  .badge-info { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .badge-warn { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .badge-err { background:rgba(239,68,68,0.15);color:#EF4444; }
  .badge-ok { background:rgba(16,185,129,0.15);color:#10B981; }
  .badge-default { background:rgba(148,163,184,0.15);color:#94A3B8; }
  .card-desc { font-size:0.78rem;color:#CBD5E1;margin:0.25rem 0 0; }
  .card-meta { font-size:0.7rem;color:#94A3B8;margin-top:0.2rem; }
  .btn-primary { padding:0.45rem 0.75rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
</style>
