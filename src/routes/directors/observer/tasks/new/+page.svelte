<!-- /directors/observer/tasks/new — TheObserver (UC-OBS-01/02/03/04)
     Task assignment, tracking, help request proxy review, outbound response review.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    assignTask, getSubordinateTasks, getPersonnelList,
    reviewHelpRequest, reviewHelpResponse,
    type TaskSummary, type PersonnelListItem,
  } from '$lib/stores/directors';

  let tasks: TaskSummary[] = $state([]);
  let personnel: PersonnelListItem[] = $state([]);
  let activeTab: 'assign' | 'help_in' | 'help_out' = $state('assign');

  // Assignment form
  let assignedTo = $state('');
  let title = $state('');
  let description = $state('');
  let taskType = $state('research');
  let dueDate = $state('');
  let error = $state('');
  let success = $state('');

  // Help review
  let helpDecision: 'forward' | 'reject' | 'convert' = $state('forward');
  let helpReason = $state('');
  let outDecision: 'forward' | 'withhold' = $state('forward');
  let outReason = $state('');

  const validRoles = ['Biologist', 'Chemist', 'AgriculturalEngineer', 'BiologicalEngineer'];

  onMount(async () => {
    const [t, p] = await Promise.all([getSubordinateTasks(), getPersonnelList()]);
    tasks = t;
    personnel = p.filter(x => validRoles.includes(x.role_name));
  });

  async function handleAssign() {
    error = ''; success = '';
    if (!assignedTo || !title.trim()) { error = 'Assignee and title are required.'; return; }
    try {
      const id = await assignTask({
        assigned_to: assignedTo, title, description: description || undefined,
        task_type: taskType, due_date: dueDate || undefined,
      });
      success = `Task created (${id.slice(0,8)}…)`;
      title = ''; description = ''; assignedTo = '';
      tasks = await getSubordinateTasks();
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }

  async function handleHelpReview(taskId: string) {
    error = ''; success = '';
    try {
      await reviewHelpRequest(taskId, helpDecision, helpReason || undefined);
      success = `Help request ${helpDecision}ed.`;
      helpReason = '';
      tasks = await getSubordinateTasks();
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }

  async function handleOutboundReview(taskId: string) {
    error = ''; success = '';
    try {
      await reviewHelpResponse(taskId, outDecision, outReason || undefined);
      success = `Outbound response ${outDecision}ed.`;
      outReason = '';
      tasks = await getSubordinateTasks();
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }

  function helpRequests() { return tasks.filter(t => t.status === 'help_requested'); }
  function outboundResponses() { return tasks.filter(t => t.status === 'response_pending'); }

  function statusColor(s: string) {
    return s === 'completed' ? 'st-done' : s === 'in_progress' ? 'st-prog' : s === 'help_requested' ? 'st-help' : s === 'response_pending' ? 'st-out' : 'st-new';
  }
</script>

<h1 class="title">Observer — Task Management</h1>
<p class="subtitle">Assign tasks, review incoming help requests, and gate outbound responses for Bio, Chem, AgEng & BioEng.</p>

<div class="tabs">
  <button class:active={activeTab === 'assign'} onclick={() => (activeTab = 'assign')}>Assign & Track</button>
  <button class:active={activeTab === 'help_in'} onclick={() => (activeTab = 'help_in')}>Help Requests ({helpRequests().length})</button>
  <button class:active={activeTab === 'help_out'} onclick={() => (activeTab = 'help_out')}>Outbound Review ({outboundResponses().length})</button>
</div>

{#if activeTab === 'assign'}
<div class="layout">
  <div class="form-card">
    <h2>New Task</h2>
    <label class="field"><span class="label">Assignee</span>
      <select class="input" bind:value={assignedTo}>
        <option value="">— Select —</option>
        {#each personnel as p}<option value={p.id}>{p.full_name} ({p.role_name})</option>{/each}
      </select>
    </label>
    <label class="field"><span class="label">Title</span>
      <input type="text" class="input" bind:value={title} placeholder="Task title…" />
    </label>
    <label class="field"><span class="label">Description</span>
      <textarea class="textarea" bind:value={description} rows="3" placeholder="Details…"></textarea>
    </label>
    <label class="field"><span class="label">Type</span>
      <select class="input" bind:value={taskType}>
        <option value="research">Research</option>
        <option value="experiment">Experiment</option>
        <option value="analysis">Analysis</option>
        <option value="field_study">Field Study</option>
      </select>
    </label>
    <label class="field"><span class="label">Due Date</span>
      <input type="date" class="input" bind:value={dueDate} />
    </label>
    {#if error}<p class="error">{error}</p>{/if}
    {#if success}<p class="success">{success}</p>{/if}
    <button class="btn-primary" onclick={handleAssign}>Assign Task</button>
  </div>
  <div class="task-list">
    <h2>Subordinate Tasks</h2>
    {#each tasks as task}
      <div class="task-card">
        <div class="task-header">
          <span class="task-title">{task.title}</span>
          <span class="badge {statusColor(task.status)}">{task.status}</span>
        </div>
        <div class="task-meta">
          <span>Assigned to: {task.assignee_name}</span>
          {#if task.due_date}<span>Due: {task.due_date}</span>{/if}
        </div>
        {#if task.description}<p class="task-desc">{task.description}</p>{/if}
      </div>
    {:else}
      <p class="empty">No tasks assigned yet.</p>
    {/each}
  </div>
</div>

{:else if activeTab === 'help_in'}
<!-- UC-OBS-03: Incoming cross-department help request proxy review -->
<div class="review-section">
  <h2 class="section-title">Incoming Help Requests</h2>
  <p class="hint">Decide: Forward to target department, Reject with reason, or Convert to formal task.</p>
  {#each helpRequests() as req}
    <div class="review-card">
      <div class="review-header">
        <span class="review-title">{req.title}</span>
        <span class="badge st-help">{req.status}</span>
      </div>
      {#if req.description}<p class="review-body">{req.description}</p>{/if}
      <div class="review-meta">From: {req.assignee_name}{req.due_date ? ` — Due: ${req.due_date}` : ''}</div>
      <div class="review-actions">
        <select class="input input-sm" bind:value={helpDecision}>
          <option value="forward">Forward</option>
          <option value="reject">Reject</option>
          <option value="convert">Convert to Task</option>
        </select>
        <input type="text" class="input input-sm" bind:value={helpReason} placeholder="Reason (optional)…" />
        <button class="btn-action" onclick={() => handleHelpReview(req.id)}>Submit</button>
      </div>
    </div>
  {:else}
    <p class="empty">No incoming help requests.</p>
  {/each}
  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}
</div>

{:else}
<!-- UC-OBS-04: Outgoing help response proxy review -->
<div class="review-section">
  <h2 class="section-title">Outbound Response Review</h2>
  <p class="hint">Gate responses from subordinates going to other departments. Forward or withhold.</p>
  {#each outboundResponses() as resp}
    <div class="review-card">
      <div class="review-header">
        <span class="review-title">{resp.title}</span>
        <span class="badge st-out">{resp.status}</span>
      </div>
      {#if resp.description}<p class="review-body">{resp.description}</p>{/if}
      <div class="review-meta">From: {resp.assignee_name}</div>
      <div class="review-actions">
        <select class="input input-sm" bind:value={outDecision}>
          <option value="forward">Forward</option>
          <option value="withhold">Withhold</option>
        </select>
        <input type="text" class="input input-sm" bind:value={outReason} placeholder="Reason (optional)…" />
        <button class="btn-action" onclick={() => handleOutboundReview(resp.id)}>Submit</button>
      </div>
    </div>
  {:else}
    <p class="empty">No outbound responses pending review.</p>
  {/each}
  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}
</div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .tabs { display:flex;gap:0.25rem;margin-bottom:1rem; }
  .tabs button { padding:0.45rem 0.9rem;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);color:#94A3B8;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .tabs button.active { color:#3ABEFF;border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .layout { display:flex;gap:1.5rem;flex-wrap:wrap; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;width:400px;display:flex;flex-direction:column;gap:0.6rem; }
  .form-card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0; }
  .task-list { flex:1;min-width:320px; }
  .task-list h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0 0 0.75rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .input-sm { width:auto;flex:1; }
  .task-card { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.65rem;margin-bottom:0.4rem; }
  .task-header { display:flex;justify-content:space-between;align-items:center; }
  .task-title { font-size:0.85rem;font-weight:500; }
  .task-meta { font-size:0.7rem;color:#94A3B8;margin-top:0.2rem;display:flex;gap:1rem; }
  .task-desc { font-size:0.75rem;color:#94A3B8;margin:0.3rem 0 0; }
  .badge { padding:0.1rem 0.35rem;border-radius:4px;font-size:0.65rem; }
  .st-new { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .st-prog { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .st-done { background:rgba(16,185,129,0.15);color:#10B981; }
  .st-help { background:rgba(239,68,68,0.15);color:#EF4444; }
  .st-out { background:rgba(139,92,246,0.15);color:#C084FC; }
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }

  .review-section { max-width:700px; }
  .section-title { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0 0 0.3rem; }
  .hint { font-size:0.75rem;color:#94A3B8;margin:0 0 0.75rem; }
  .review-card { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.75rem;margin-bottom:0.5rem; }
  .review-header { display:flex;justify-content:space-between;align-items:center;margin-bottom:0.3rem; }
  .review-title { font-size:0.85rem;font-weight:500; }
  .review-body { font-size:0.8rem;color:#94A3B8;margin:0.2rem 0; }
  .review-meta { font-size:0.7rem;color:#475569;margin-bottom:0.4rem; }
  .review-actions { display:flex;gap:0.4rem;align-items:center; }
  .btn-action { padding:0.35rem 0.7rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:4px;cursor:pointer;font-size:0.75rem;white-space:nowrap; }
  .btn-action:hover { background:rgba(58,190,255,0.3); }

  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .empty { color:#475569;font-size:0.8rem; }
</style>
