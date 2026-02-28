<!--
  /engineers/tasks — UC-GE-01: Engineer Task Inbox
  Displays tasks assigned by The Observer to the current engineer.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getMyTasks, type EngineerTask } from '$lib/stores/engineers';

  let tasks: EngineerTask[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try {
      tasks = await getMyTasks();
    } catch (e: any) {
      error = e?.toString() ?? 'Failed to load tasks.';
    } finally {
      loading = false;
    }
  });

  function statusColor(s: string): string {
    switch (s) {
      case 'completed': return '#22C55E';
      case 'in_progress': return '#F59E0B';
      case 'rejected': case 'cancelled': return '#EF4444';
      default: return '#94A3B8';
    }
  }
</script>

<h2>Task Inbox</h2>

{#if loading}
  <p class="muted">Loading tasks…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if tasks.length === 0}
  <p class="muted">No tasks assigned to you.</p>
{:else}
  <table class="tbl">
    <thead>
      <tr>
        <th>Title</th>
        <th>Type</th>
        <th>Status</th>
        <th>Assigned By</th>
        <th>Due</th>
        <th>Created</th>
        <th>Actions</th>
      </tr>
    </thead>
    <tbody>
      {#each tasks as t}
        <tr>
          <td>{t.title}</td>
          <td class="tag">{t.task_type}</td>
          <td><span class="badge" style="color:{statusColor(t.status)}">{t.status}</span></td>
          <td>{t.assigner_name}</td>
          <td>{t.due_date ?? '—'}</td>
          <td>{new Date(t.created_at).toLocaleDateString()}</td>
          <td>
            <a href="/engineers/tasks/{t.id}/progress-report" class="link-sm">Progress Report</a>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:1rem; }
  .muted { color:#64748B;font-size:0.85rem; }
  .error { color:#EF4444;font-size:0.85rem; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.5rem;color:#94A3B8;border-bottom:1px solid rgba(58,190,255,0.15);font-size:0.7rem;text-transform:uppercase; }
  .tbl td { padding:0.5rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .tag { background:rgba(139,92,246,0.15);color:#C084FC;padding:0.1rem 0.4rem;border-radius:4px;font-size:0.7rem; }
  .badge { font-weight:600;font-size:0.75rem; }
  .link-sm { font-size:0.75rem;color:#A5B4FC;text-decoration:none; }
  .link-sm:hover { text-decoration:underline; }
</style>
