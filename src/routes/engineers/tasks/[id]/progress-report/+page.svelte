<!--
  /engineers/tasks/[id]/progress-report — UC-GE-02: Submit Progress Report
  Also shows previous reports for this task.
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import {
    submitProgressReport, getProgressReports,
    type ProgressReport,
  } from '$lib/stores/engineers';

  let taskId = $state('');
  const unsubPage = page.subscribe((p) => (taskId = p.params.id ?? ''));
  onDestroy(unsubPage);

  let currentStatus = $state('in_progress');
  let workCompleted = $state('');
  let problemsEncountered = $state('');
  let plansNext = $state('');
  let submitting = $state(false);
  let message = $state('');
  let isError = $state(false);

  let reports: ProgressReport[] = $state([]);
  let loadingReports = $state(true);

  onMount(async () => {
    try {
      reports = await getProgressReports(taskId);
    } catch {}
    loadingReports = false;
  });

  async function handleSubmit() {
    if (!workCompleted.trim()) return;
    submitting = true;
    message = '';
    try {
      await submitProgressReport({
        task_id: taskId,
        current_status: currentStatus,
        work_completed: workCompleted,
        problems_encountered: problemsEncountered || undefined,
        plans_next: plansNext || undefined,
      });
      message = 'Progress report submitted. The Observer has been notified.';
      isError = false;
      workCompleted = ''; problemsEncountered = ''; plansNext = '';
      // Refresh list
      try { reports = await getProgressReports(taskId); } catch {}
    } catch (e: any) {
      message = e?.toString() ?? 'Failed to submit report.';
      isError = true;
    } finally {
      submitting = false;
    }
  }
</script>

<h2>Progress Report</h2>
<p class="hint">Task <code>{taskId.slice(0, 8)}…</code></p>

<div class="form-card">
  <div class="form-group">
    <label for="status">Current Status</label>
    <select id="status" bind:value={currentStatus}>
      <option value="in_progress">In Progress</option>
      <option value="completed">Completed</option>
      <option value="blocked">Blocked</option>
      <option value="on_hold">On Hold</option>
    </select>
  </div>
  <div class="form-group">
    <label for="work">Work Completed *</label>
    <textarea id="work" bind:value={workCompleted} rows="4" placeholder="Describe work accomplished since last report…"></textarea>
  </div>
  <div class="form-group">
    <label for="problems">Problems Encountered</label>
    <textarea id="problems" bind:value={problemsEncountered} rows="3" placeholder="Any blockers or issues…"></textarea>
  </div>
  <div class="form-group">
    <label for="plans">Plans Next</label>
    <textarea id="plans" bind:value={plansNext} rows="3" placeholder="Next steps planned…"></textarea>
  </div>
  <div class="btn-row">
    <button class="btn-primary" onclick={handleSubmit} disabled={submitting || !workCompleted.trim()}>
      {submitting ? 'Submitting…' : 'Submit Report'}
    </button>
    <button class="btn-secondary" onclick={() => goto('/engineers/tasks')}>Back to Tasks</button>
  </div>
  {#if message}<p class:msg-ok={!isError} class:msg-err={isError}>{message}</p>{/if}
</div>

<!-- Previous reports -->
<h3 class="section-title">Previous Reports ({reports.length})</h3>
{#if loadingReports}
  <p class="muted">Loading…</p>
{:else if reports.length === 0}
  <p class="muted">No previous reports for this task.</p>
{:else}
  <table class="tbl">
    <thead><tr><th>Date</th><th>Status</th><th>Work Completed</th><th>Problems</th></tr></thead>
    <tbody>
      {#each reports as r}
        <tr>
          <td>{r.report_date}</td>
          <td class="tag">{r.current_status}</td>
          <td class="trunc">{r.work_completed}</td>
          <td class="trunc">{r.problems_encountered ?? '—'}</td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.5rem; }
  h3 { font-size:0.95rem;color:#E6EDF3; }
  .section-title { margin-top:1.5rem;font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#3ABEFF; }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:1rem; }
  .hint code { color:#A5B4FC; }
  .form-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:650px; }
  .form-group { margin-bottom:0.75rem; }
  .form-group label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.25rem; }
  .form-group input, .form-group textarea, .form-group select { width:100%;background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem;font-size:0.8rem; }
  .form-group textarea { resize:vertical;font-family:'Fira Code',monospace; }
  .btn-row { display:flex;gap:0.75rem; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-secondary { background:transparent;border:1px solid #475569;color:#94A3B8;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-secondary:hover { color:#E6EDF3;border-color:#3ABEFF; }
  .msg-ok { color:#22C55E;font-size:0.8rem;margin-top:0.5rem; }
  .msg-err { color:#EF4444;font-size:0.8rem;margin-top:0.5rem; }
  .muted { color:#64748B;font-size:0.85rem; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.5rem;color:#94A3B8;border-bottom:1px solid rgba(58,190,255,0.15);font-size:0.7rem;text-transform:uppercase; }
  .tbl td { padding:0.5rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .tag { background:rgba(139,92,246,0.15);color:#C084FC;padding:0.1rem 0.4rem;border-radius:4px;font-size:0.7rem; }
  .trunc { max-width:200px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
</style>
