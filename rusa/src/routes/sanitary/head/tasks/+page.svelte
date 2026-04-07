<!-- /sanitary/head/tasks — UC-HS-05: View All Assigned Tasks (HeadOfSanitary) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { sanGetAllTasks, sanUpdateTaskStatus, type SanitaryTask } from '$lib/stores/sanitary';

  let tasks: SanitaryTask[] = $state([]);
  let selected: SanitaryTask | null = $state(null);
  let filterStatus = $state('all');
  let loading = $state(false);
  let error = $state('');
  let success = $state('');

  const STATUS_OPTIONS = ['all', 'pending', 'in_progress', 'completed'];

  onMount(async () => { await refresh(); });

  async function refresh() {
    loading = true;
    error = ''; success = '';
    try {
      tasks = await sanGetAllTasks();
    } catch (e: unknown) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  const filteredTasks = $derived(
    filterStatus === 'all' ? tasks : tasks.filter((t) => t.status === filterStatus)
  );

  async function updateStatus(taskId: string, status: string) {
    error = ''; success = '';
    try {
      await sanUpdateTaskStatus({ task_id: taskId, status });
      success = `Task updated to ${status.replace(/_/g, ' ')}.`;
      if (selected?.id === taskId) selected = { ...selected, status };
      await refresh();
    } catch (e: unknown) {
      error = String(e);
    }
  }

  function urgencyBadge(u: string): string {
    const m: Record<string, string> = { low: 'badge-info', medium: 'badge-warn', high: 'badge-err', critical: 'badge-crit' };
    return m[u] ?? 'badge-info';
  }
  function statusBadge(s: string): string {
    const m: Record<string, string> = { pending: 'badge-default', in_progress: 'badge-warn', completed: 'badge-ok' };
    return m[s] ?? 'badge-default';
  }
</script>

<h1 class="title">All Assigned Tasks</h1>
<p class="subtitle">UC-HS-05 — Overview of all tasks assigned to sanitary staff.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="filter-row">
  <span class="filter-label">Filter:</span>
  {#each STATUS_OPTIONS as s}
    <button
      class="filter-btn"
      class:filter-active={filterStatus === s}
      onclick={() => { filterStatus = s; }}
    >
      {s === 'all' ? 'All' : s.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())}
    </button>
  {/each}
</div>

{#if loading}<p class="loading">Loading tasks…</p>{/if}

<div class="grid">
  <div class="list-panel">
    {#each filteredTasks as t}
      <button
        class="card"
        class:card-selected={selected?.id === t.id}
        onclick={() => { selected = t; }}
      >
        <div class="card-row">
          <span class="card-name">{t.task_name ?? t.task_type}</span>
          <span class="badge {urgencyBadge(t.urgency ?? 'low')}">{t.urgency ?? '—'}</span>
        </div>
        <div class="card-meta">
          <span class="badge {statusBadge(t.status)}">{t.status.replace(/_/g, ' ')}</span>
          {#if t.due_date}<span class="due">Due: {t.due_date}</span>{/if}
        </div>
      </button>
    {:else}
      {#if !loading}<p class="empty">No tasks found.</p>{/if}
    {/each}
  </div>

  <div class="detail-panel">
    {#if selected}
      <h2 class="detail-title">{selected.task_name ?? selected.task_type}</h2>
      <div class="detail-row"><span class="label">Type</span><span>{selected.task_type}</span></div>
      <div class="detail-row"><span class="label">Urgency</span>
        <span class="badge {urgencyBadge(selected.urgency ?? 'low')}">{selected.urgency ?? '—'}</span>
      </div>
      <div class="detail-row"><span class="label">Status</span>
        <span class="badge {statusBadge(selected.status)}">{selected.status.replace(/_/g, ' ')}</span>
      </div>
      {#if selected.location}
        <div class="detail-row"><span class="label">Location</span><span>{selected.location}</span></div>
      {/if}
      {#if selected.due_date}
        <div class="detail-row"><span class="label">Due Date</span><span>{selected.due_date}</span></div>
      {/if}
      {#if selected.instructions}
        <div class="instructions">
          <span class="label">Instructions</span>
          <p>{selected.instructions}</p>
        </div>
      {/if}
      <div class="detail-row"><span class="label">Assigned By</span><span>{selected.assigner_name}</span></div>
      <div class="detail-row"><span class="label">Created</span>
        <span>{new Date(selected.created_at).toLocaleDateString()}</span>
      </div>

      {#if selected.status !== 'completed'}
        <div class="detail-actions">
          {#if selected.status === 'pending'}
            <button class="btn-action" onclick={() => updateStatus(selected!.id, 'in_progress')}>Mark In Progress</button>
          {/if}
          <button class="btn-ok" onclick={() => updateStatus(selected!.id, 'completed')}>Mark Completed</button>
        </div>
      {/if}
    {:else}
      <p class="placeholder-text">Select a task to see details.</p>
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#F59E0B;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .filter-row { display:flex;gap:0.35rem;align-items:center;margin-bottom:0.75rem;flex-wrap:wrap; }
  .filter-label { font-size:0.72rem;color:#94A3B8;text-transform:uppercase; }
  .filter-btn { padding:0.25rem 0.55rem;border:1px solid rgba(245,158,11,0.2);background:transparent;color:#94A3B8;border-radius:4px;cursor:pointer;font-size:0.72rem; }
  .filter-btn:hover { color:#E6EDF3;border-color:rgba(245,158,11,0.4); }
  .filter-active { border-color:#F59E0B;color:#F59E0B;background:rgba(245,158,11,0.1); }
  .grid { display:grid;grid-template-columns:1fr 1.3fr;gap:0.75rem; }
  .list-panel { display:flex;flex-direction:column;gap:0.35rem;max-height:65vh;overflow-y:auto; }
  .detail-panel { background:rgba(14,20,40,0.6);border:1px solid rgba(245,158,11,0.1);border-radius:8px;padding:1rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(245,158,11,0.08);border-radius:6px;padding:0.6rem;cursor:pointer;text-align:left;width:100%;color:#E6EDF3;transition:border-color .15s; }
  .card:hover { border-color:rgba(245,158,11,0.3); }
  .card-selected { border-color:#F59E0B;background:rgba(245,158,11,0.06); }
  .card-row { display:flex;justify-content:space-between;align-items:center; }
  .card-name { font-size:0.82rem;font-weight:500; }
  .card-meta { display:flex;gap:0.4rem;align-items:center;margin-top:0.2rem; }
  .due { font-size:0.7rem;color:#94A3B8; }
  .detail-title { font-size:0.95rem;font-family:'Orbitron',sans-serif;color:#F59E0B;margin:0 0 0.7rem; }
  .detail-row { display:flex;justify-content:space-between;padding:0.3rem 0;border-bottom:1px solid rgba(245,158,11,0.06);font-size:0.8rem; }
  .label { color:#94A3B8; }
  .instructions { margin-top:0.5rem;font-size:0.8rem; }
  .instructions p { color:#CBD5E1;margin:0.2rem 0 0;white-space:pre-wrap; }
  .detail-actions { display:flex;gap:0.5rem;margin-top:0.8rem; }
  .badge { padding:0.12rem 0.35rem;border-radius:4px;font-size:0.6rem;text-transform:capitalize;display:inline-block; }
  .badge-info { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .badge-warn { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .badge-err { background:rgba(239,68,68,0.15);color:#EF4444; }
  .badge-crit { background:rgba(239,68,68,0.3);color:#FCA5A5;font-weight:700; }
  .badge-ok { background:rgba(16,185,129,0.15);color:#10B981; }
  .badge-default { background:rgba(148,163,184,0.15);color:#94A3B8; }
  .btn-action { padding:0.4rem 0.65rem;background:rgba(245,158,11,0.15);border:1px solid #F59E0B;color:#F59E0B;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-action:hover { background:rgba(245,158,11,0.25); }
  .btn-ok { padding:0.4rem 0.65rem;background:rgba(16,185,129,0.15);border:1px solid #10B981;color:#10B981;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-ok:hover { background:rgba(16,185,129,0.25); }
  .placeholder-text { color:#475569;font-size:0.85rem;text-align:center;padding:2rem 0; }
  .loading { color:#94A3B8;font-size:0.8rem; }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1.5rem; }
</style>
