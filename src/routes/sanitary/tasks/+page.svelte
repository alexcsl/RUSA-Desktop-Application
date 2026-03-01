<!-- /sanitary/tasks — UC-STAS-01: Receive & Execute Task -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    sanGetMyTasks, sanUpdateTaskStatus,
    type SanitaryTask,
  } from '$lib/stores/sanitary';

  let tasks: SanitaryTask[] = $state([]);
  let selected: SanitaryTask | null = $state(null);
  let error = $state('');
  let success = $state('');

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = ''; success = '';
    try { tasks = await sanGetMyTasks(); } catch (e: unknown) { error = String(e); }
  }

  async function updateStatus(taskId: string, status: string) {
    error = ''; success = '';
    try {
      await sanUpdateTaskStatus({ task_id: taskId, status });
      success = `Task marked as ${status.replace(/_/g,' ')}.`;
      if (selected?.id === taskId) selected = { ...selected, status };
      await refresh();
    } catch (e: unknown) { error = String(e); }
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

<h1 class="title">My Tasks</h1>
<p class="subtitle">UC-STAS-01 — View assigned tasks, start, and mark completed.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="grid">
  <div class="list-panel">
    {#each tasks as t}
      <button class="card" class:card-selected={selected?.id === t.id} onclick={() => { selected = t; }}>
        <div class="card-row">
          <span class="card-name">{t.task_name ?? t.task_type}</span>
          <span class="badge {urgencyBadge(t.urgency ?? 'low')}">{t.urgency ?? '—'}</span>
        </div>
        <div class="card-meta">
          <span class="badge {statusBadge(t.status)}">{t.status.replace(/_/g,' ')}</span>
          {#if t.due_date}<span class="due">Due: {t.due_date}</span>{/if}
        </div>
      </button>
    {:else}
      <p class="empty">No tasks assigned to you.</p>
    {/each}
  </div>

  <div class="detail-panel">
    {#if selected}
      <h2 class="detail-title">{selected.task_name ?? selected.task_type}</h2>
      <div class="detail-row"><span class="label">Type</span><span>{selected.task_type}</span></div>
      <div class="detail-row"><span class="label">Urgency</span><span class="badge {urgencyBadge(selected.urgency ?? 'low')}">{selected.urgency ?? '—'}</span></div>
      <div class="detail-row"><span class="label">Status</span><span class="badge {statusBadge(selected.status)}">{selected.status.replace(/_/g,' ')}</span></div>
      {#if selected.due_date}<div class="detail-row"><span class="label">Due Date</span><span>{selected.due_date}</span></div>{/if}
      {#if selected.instructions}
        <div class="instructions">
          <span class="label">Instructions</span>
          <p>{selected.instructions}</p>
        </div>
      {/if}
      <div class="detail-actions">
        {#if selected.status === 'pending'}
          <button class="btn-primary" onclick={() => updateStatus(selected!.id, 'in_progress')}>Start Task</button>
        {/if}
        {#if selected.status === 'in_progress'}
          <button class="btn-ok" onclick={() => updateStatus(selected!.id, 'completed')}>Mark Completed</button>
        {/if}
      </div>
    {:else}
      <p class="placeholder-text">Select a task to see details.</p>
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .grid { display:grid;grid-template-columns:1fr 1.3fr;gap:0.75rem; }
  .list-panel { display:flex;flex-direction:column;gap:0.35rem;max-height:70vh;overflow-y:auto; }
  .detail-panel { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.6rem;cursor:pointer;text-align:left;width:100%;color:#E6EDF3;transition:border-color .15s; }
  .card:hover { border-color:rgba(58,190,255,0.3); }
  .card-selected { border-color:#3ABEFF;background:rgba(58,190,255,0.06); }
  .card-row { display:flex;justify-content:space-between;align-items:center; }
  .card-name { font-size:0.82rem;font-weight:500; }
  .card-meta { display:flex;gap:0.4rem;align-items:center;margin-top:0.2rem; }
  .due { font-size:0.7rem;color:#94A3B8; }
  .detail-title { font-size:0.95rem;font-family:'Orbitron',sans-serif;color:#3ABEFF;margin:0 0 0.7rem; }
  .detail-row { display:flex;justify-content:space-between;padding:0.3rem 0;border-bottom:1px solid rgba(58,190,255,0.06);font-size:0.8rem; }
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
  .btn-primary { padding:0.4rem 0.65rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-ok { padding:0.4rem 0.65rem;background:rgba(16,185,129,0.15);border:1px solid #10B981;color:#10B981;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-ok:hover { background:rgba(16,185,129,0.25); }
  .placeholder-text { color:#475569;font-size:0.85rem;text-align:center;padding:2rem 0; }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1.5rem; }
</style>
