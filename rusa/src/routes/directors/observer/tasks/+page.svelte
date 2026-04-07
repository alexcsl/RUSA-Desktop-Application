<!--
  /directors/observer/tasks — Subordinate Task Overview (UC-OBS-02)
  TheObserver views all tasks assigned to scientists and data analysts.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getSubordinateTasks, type TaskSummary } from '$lib/stores/directors';

  let tasks: TaskSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => { await loadTasks(); });

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

  function formatDate(d: string | null) { return d ? new Date(d).toLocaleDateString() : '—'; }
  function statusColor(s: string) {
    const map: Record<string, string> = { open: '#3ABEFF', in_progress: '#8B5CF6', completed: '#22C55E' };
    return map[s] ?? '#94A3B8';
  }
</script>

<h1 class="title">Subordinate Tasks</h1>
<p class="subtitle">All tasks assigned within your division.</p>

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
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .task-list { display:flex;flex-direction:column;gap:0.5rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.9rem;display:flex;flex-direction:column;gap:0.3rem; }
  .card-header { display:flex;justify-content:space-between;align-items:center; }
  .card-title { font-size:0.88rem;font-weight:600;color:#E6EDF3; }
  .status-dot { font-size:0.72rem;font-weight:600;text-transform:capitalize; }
  .meta { font-size:0.72rem;color:#64748B;margin:0; }
  .desc { font-size:0.8rem;color:#94A3B8;margin:0;white-space:pre-wrap; }
  .error { color:#EF4444;font-size:0.75rem; }
</style>
