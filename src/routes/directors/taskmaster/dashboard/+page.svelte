<!-- /directors/taskmaster/dashboard — TheTaskmaster Unified Progress (UC-DIR-15) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getSubordinateTasks, approveClosureRequest, reviewHelpRequest,
    type TaskSummary,
  } from '$lib/stores/directors';

  let tasks: TaskSummary[] = $state([]);
  let selected: TaskSummary | null = $state(null);
  let filter: 'all' | 'pending' | 'in_progress' | 'completed' | 'closure_requested' = $state('all');

  let closureDecision: 'approved' | 'denied' = $state('approved');
  let closureReason = $state('');
  let helpDecision: 'forward' | 'reject' | 'convert' = $state('forward');
  let helpReason = $state('');
  let error = $state('');
  let success = $state('');

  onMount(async () => { tasks = await getSubordinateTasks(); });

  $effect(() => { selected = null; });

  function filteredTasks() {
    if (filter === 'all') return tasks;
    return tasks.filter(t => t.status === filter);
  }

  async function handleClosure() {
    if (!selected) return;
    error = ''; success = '';
    try {
      await approveClosureRequest(selected.id, closureDecision, closureReason || undefined);
      success = `Closure ${closureDecision}.`;
      tasks = await getSubordinateTasks();
      selected = null;
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }

  async function handleHelp() {
    if (!selected) return;
    error = ''; success = '';
    try {
      await reviewHelpRequest(selected.id, helpDecision, helpReason || undefined);
      success = `Help request ${helpDecision}ed.`;
      tasks = await getSubordinateTasks();
      selected = null;
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }

  function statusColor(s: string) {
    const m: Record<string,string> = { completed:'st-done', in_progress:'st-prog', pending:'st-new', closure_requested:'st-closure', help_requested:'st-help' };
    return m[s] ?? 'st-new';
  }
</script>

<h1 class="title">Taskmaster — Unified Dashboard</h1>
<p class="subtitle">Monitor all subordinate tasks. Approve closures and review help requests. Bypass: can approve closures without vote.</p>

<div class="toolbar">
  <select class="filter" bind:value={filter}>
    <option value="all">All ({tasks.length})</option>
    <option value="pending">Pending</option>
    <option value="in_progress">In Progress</option>
    <option value="completed">Completed</option>
    <option value="closure_requested">Closure Requested</option>
  </select>
</div>

<div class="grid">
  <div class="list-panel">
    {#each filteredTasks() as task}
      <button class="card" class:selected={selected?.id === task.id} onclick={() => { selected = task; error = ''; success = ''; }}>
        <div class="card-title">{task.title}</div>
        <div class="card-meta">
          <span class="badge {statusColor(task.status)}">{task.status}</span>
          <span>{task.assignee_name}</span>
        </div>
      </button>
    {:else}
      <p class="empty">No tasks match filter.</p>
    {/each}
  </div>

  <div class="detail-panel">
    {#if selected}
      <h2>{selected.title}</h2>
      <div class="kv"><span class="k">Status:</span><span class="badge {statusColor(selected.status)}">{selected.status}</span></div>
      <div class="kv"><span class="k">Assigned to:</span><span>{selected.assignee_name}</span></div>
      <div class="kv"><span class="k">Assigned by:</span><span>{selected.assigner_name}</span></div>
      {#if selected.due_date}<div class="kv"><span class="k">Due:</span><span>{selected.due_date}</span></div>{/if}
      {#if selected.description}<p class="desc">{selected.description}</p>{/if}

      {#if selected.status === 'closure_requested'}
        <div class="action-form">
          <h3>Closure Decision</h3>
          <select class="input" bind:value={closureDecision}>
            <option value="approved">Approve</option>
            <option value="denied">Deny</option>
          </select>
          <textarea class="textarea" placeholder="Reason…" bind:value={closureReason} rows="2"></textarea>
          <button class="btn-primary" onclick={handleClosure}>Submit</button>
        </div>
      {/if}

      {#if selected.status === 'help_requested'}
        <div class="action-form">
          <h3>Help Request Decision</h3>
          <select class="input" bind:value={helpDecision}>
            <option value="forward">Forward</option>
            <option value="reject">Reject</option>
            <option value="convert">Convert to Task</option>
          </select>
          <textarea class="textarea" placeholder="Reason…" bind:value={helpReason} rows="2"></textarea>
          <button class="btn-primary" onclick={handleHelp}>Submit</button>
        </div>
      {/if}

      {#if error}<p class="error">{error}</p>{/if}
      {#if success}<p class="success">{success}</p>{/if}
    {:else}
      <div class="empty-state"><p>Select a task to view details.</p></div>
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .toolbar { margin-bottom:0.75rem; }
  .filter { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem;font-size:0.8rem;cursor:pointer; }
  .grid { display:flex;gap:1rem;flex:1;overflow:hidden; }
  .list-panel { width:320px;overflow-y:auto; }
  .detail-panel { flex:1;overflow-y:auto; }
  .card { width:100%;text-align:left;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.65rem;margin-bottom:0.35rem;cursor:pointer;color:#E6EDF3; }
  .card:hover { border-color:rgba(58,190,255,0.3); }
  .card.selected { border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .card-title { font-size:0.8rem;font-weight:500;margin-bottom:0.2rem; }
  .card-meta { display:flex;justify-content:space-between;font-size:0.7rem;color:#94A3B8; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1rem;color:#3ABEFF;margin:0 0 0.5rem; }
  h3 { font-size:0.85rem;color:#E6EDF3;margin:0 0 0.4rem; }
  .kv { display:flex;gap:0.5rem;font-size:0.8rem;margin-bottom:0.3rem; }
  .k { color:#94A3B8;min-width:90px; }
  .desc { color:#94A3B8;font-size:0.8rem;margin:0.5rem 0; }
  .action-form { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-top:0.75rem;display:flex;flex-direction:column;gap:0.5rem; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .textarea { resize:vertical; }
  .badge { padding:0.1rem 0.35rem;border-radius:4px;font-size:0.65rem; }
  .st-new { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .st-prog { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .st-done { background:rgba(16,185,129,0.15);color:#10B981; }
  .st-closure { background:rgba(139,92,246,0.15);color:#C084FC; }
  .st-help { background:rgba(239,68,68,0.15);color:#EF4444; }
  .btn-primary { padding:0.45rem 1rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
  .empty-state { display:flex;justify-content:center;align-items:center;height:100%;color:#475569; }
</style>
