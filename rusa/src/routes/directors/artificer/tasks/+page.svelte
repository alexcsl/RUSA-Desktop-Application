<!--
  /directors/artificer/tasks — Subordinate Task Overview (UC-ART-02)
  TheArtificer views all tasks assigned to their subordinates.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getSubordinateTasks, approveClosureRequest, type TaskSummary } from '$lib/stores/directors';

  let tasks: TaskSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let actionMsg = $state('');

  onMount(async () => {
    await loadTasks();
  });

  async function loadTasks() {
    loading = true; error = '';
    try {
      tasks = await getSubordinateTasks();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function handleApprove(taskId: string) {
    actionMsg = '';
    try {
      await approveClosureRequest(taskId, 'approved');
      actionMsg = 'Closure approved.';
      await loadTasks();
    } catch (e: unknown) {
      actionMsg = e instanceof Error ? e.message : String(e);
    }
  }

  function formatDate(d: string | null) { return d ? new Date(d).toLocaleDateString() : '—'; }
  function statusColor(s: string) {
    const map: Record<string, string> = { open: '#3ABEFF', in_progress: '#8B5CF6', completed: '#22C55E', closure_requested: '#F59E0B' };
    return map[s] ?? '#94A3B8';
  }
</script>

<h1 class="title">Subordinate Tasks</h1>
<p class="subtitle">All tasks assigned within your division. Approve closure requests here.</p>

{#if actionMsg}<p class="action-msg">{actionMsg}</p>{/if}

{#if loading}<p class="muted">Loading…</p>
{:else if error}<p class="error">{error}</p>
{:else if tasks.length === 0}<p class="muted">No tasks found.</p>
{:else}
  <div class="task-list">
    {#each tasks as task}
      <div class="card">
        <div class="card-header">
          <span class="card-title">{task.title}</span>
          <span class="status-dot" style="color:{statusColor(task.status)}">{task.status.replace('_', ' ')}</span>
        </div>
        <p class="meta">
          Assigned to: <strong>{task.assignee_name ?? '—'}</strong>
          · Due: {formatDate(task.due_date ?? null)}
        </p>
        {#if task.description}<p class="desc">{task.description}</p>{/if}
        {#if task.status === 'closure_requested'}
          <button class="btn-approve" onclick={() => handleApprove(task.id)}>Approve Closure</button>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .action-msg { font-size:0.78rem;color:#22C55E;margin-bottom:0.75rem; }
  .task-list { display:flex;flex-direction:column;gap:0.5rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.9rem;display:flex;flex-direction:column;gap:0.3rem; }
  .card-header { display:flex;justify-content:space-between;align-items:center; }
  .card-title { font-size:0.88rem;font-weight:600;color:#E6EDF3; }
  .status-dot { font-size:0.72rem;font-weight:600;text-transform:capitalize; }
  .meta { font-size:0.72rem;color:#64748B;margin:0; }
  .desc { font-size:0.8rem;color:#94A3B8;margin:0;white-space:pre-wrap; }
  .btn-approve { align-self:flex-start;background:rgba(34,197,94,0.12);border:1px solid rgba(34,197,94,0.35);color:#22C55E;border-radius:5px;padding:0.3rem 0.7rem;cursor:pointer;font-size:0.75rem;margin-top:0.2rem; }
  .btn-approve:hover { background:rgba(34,197,94,0.22); }
  .error { color:#EF4444;font-size:0.75rem; }
</style>
