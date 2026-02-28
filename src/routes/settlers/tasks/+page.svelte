<!--
  UC-PS-01: Settler Task Inbox
  Shows tasks assigned by the Commander to the current settler.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { stlGetMyTasks, type SettlerTaskSummary } from '$lib/stores/settlers';

  let tasks: SettlerTaskSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try { tasks = await stlGetMyTasks(); }
    catch (e: any) { error = e?.message ?? String(e); }
    finally { loading = false; }
  });

  function urgencyClass(u: string | null): string {
    if (!u) return '';
    return u === 'critical' ? 'urg-critical' : u === 'high' ? 'urg-high' : u === 'medium' ? 'urg-medium' : 'urg-low';
  }
</script>

<h2>Task Inbox</h2>

{#if loading}
  <p class="dim">Loading tasks…</p>
{:else if error}
  <p class="err">{error}</p>
{:else if tasks.length === 0}
  <p class="dim">No tasks assigned.</p>
{:else}
  <table class="tbl">
    <thead>
      <tr><th>Title</th><th>Scope</th><th>Urgency</th><th>Deadline</th><th>Status</th><th>Assigned By</th><th>Actions</th></tr>
    </thead>
    <tbody>
      {#each tasks as t}
        <tr>
          <td>{t.title}</td>
          <td>{t.scope ?? '—'}</td>
          <td><span class="badge {urgencyClass(t.urgency)}">{t.urgency ?? '—'}</span></td>
          <td>{t.deadline ?? '—'}</td>
          <td><span class="badge st-{t.status}">{t.status.replace(/_/g,' ')}</span></td>
          <td>{t.assigned_by_name}</td>
          <td><a href="/settlers/tasks/{t.id}/progress" class="link">Progress Report</a></td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
  .dim { color:#64748B;font-size:0.85rem; }
  .err { color:#EF4444;font-size:0.85rem; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;color:#94A3B8;border-bottom:1px solid rgba(58,190,255,0.15);padding:0.5rem 0.6rem;font-weight:600; }
  .tbl td { padding:0.5rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .badge { padding:0.15rem 0.4rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .urg-critical { background:rgba(239,68,68,0.2);color:#F87171; }
  .urg-high { background:rgba(245,158,11,0.2);color:#F59E0B; }
  .urg-medium { background:rgba(59,130,246,0.2);color:#60A5FA; }
  .urg-low { background:rgba(34,197,94,0.2);color:#4ADE80; }
  .st-assigned { background:rgba(59,130,246,0.15);color:#60A5FA; }
  .st-in_progress { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .st-completed { background:rgba(34,197,94,0.15);color:#4ADE80; }
  .link { color:#3ABEFF;text-decoration:none;font-size:0.75rem; }
  .link:hover { text-decoration:underline; }
</style>
