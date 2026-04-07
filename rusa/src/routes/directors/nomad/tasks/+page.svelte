<!--
  /directors/nomad/tasks — Tasks for Sanitary/Settlers (UC-NOM-02/03)
  TheNomad views tasks assigned to sanitary workers and settlers.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getSubordinateTasks, type TaskSummary } from '$lib/stores/directors';

  let tasks: TaskSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let filterStatus = $state('all');

  onMount(async () => { await loadTasks(); });

  async function loadTasks() {
    loading = true; error = '';
    try { tasks = await getSubordinateTasks(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  }

  function filtered() {
    return filterStatus === 'all' ? tasks : tasks.filter(t => t.status === filterStatus);
  }
  function formatDate(d: string | null) { return d ? new Date(d).toLocaleDateString() : '—'; }
  function statusColor(s: string) {
    const m: Record<string, string> = { open:'#3ABEFF', in_progress:'#8B5CF6', completed:'#22C55E' };
    return m[s] ?? '#94A3B8';
  }
</script>

<h1 class="title">Division Tasks</h1>
<p class="subtitle">Tasks for sanitary workers and settlers under your oversight.</p>

<div class="toolbar">
  {#each ['all','open','in_progress','completed'] as s}
    <button class="filter-btn" class:active={filterStatus === s} onclick={() => (filterStatus = s)}>
      {s.replace('_',' ')}
    </button>
  {/each}
</div>

{#if loading}<p class="muted">Loading…</p>
{:else if error}<p class="error">{error}</p>
{:else}
  {@const list = filtered()}
  {#if list.length === 0}<p class="muted">No tasks match this filter.</p>
  {:else}
    <div class="task-list">
      {#each list as task}
        <div class="card">
          <div class="card-header">
            <span class="card-title">{task.title}</span>
            <span class="status-dot" style="color:{statusColor(task.status)}">{task.status.replace('_',' ')}</span>
          </div>
          <p class="meta">Assignee: <strong>{task.assignee_name ?? '—'}</strong> · Due: {formatDate(task.due_date ?? null)}</p>
          {#if task.description}<p class="desc">{task.description}</p>{/if}
        </div>
      {/each}
    </div>
  {/if}
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .toolbar { display:flex;gap:0.3rem;flex-wrap:wrap;margin-bottom:0.8rem; }
  .filter-btn { background:transparent;border:1px solid rgba(58,190,255,0.15);color:#94A3B8;border-radius:4px;padding:0.22rem 0.55rem;cursor:pointer;font-size:0.72rem;font-family:'Inter',sans-serif;text-transform:capitalize; }
  .filter-btn.active { border-color:#3ABEFF;color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .task-list { display:flex;flex-direction:column;gap:0.5rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.9rem;display:flex;flex-direction:column;gap:0.3rem; }
  .card-header { display:flex;justify-content:space-between;align-items:center; }
  .card-title { font-size:0.88rem;font-weight:600;color:#E6EDF3; }
  .status-dot { font-size:0.72rem;font-weight:600;text-transform:capitalize; }
  .meta { font-size:0.72rem;color:#64748B;margin:0; }
  .desc { font-size:0.8rem;color:#94A3B8;margin:0; }
  .error { color:#EF4444;font-size:0.75rem; }
</style>
