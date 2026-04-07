<!--
  Observer: Task Conclusions view — completed tasks with conclusion notes from subordinates.
  Access: TheObserver
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { dirGetClosureRequestsForDirector, type DirClosureRequestRow } from '$lib/stores/directors';

  let conclusions: DirClosureRequestRow[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    loading = true; error = '';
    try { conclusions = await dirGetClosureRequestsForDirector(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  });

  function formatDate(d: string | null): string {
    return d ? new Date(d).toLocaleDateString() : '—';
  }
</script>

<h2 class="title">Task Conclusions</h2>
<p class="subtitle">Completed tasks with conclusion notes from your domain's personnel.</p>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if conclusions.length === 0}
  <p class="muted">No task conclusions found.</p>
{:else}
  <div class="table-wrap">
    <table class="table">
      <thead>
        <tr>
          <th>Task</th>
          <th>Completed By</th>
          <th>Conclusion</th>
          <th>Completed</th>
        </tr>
      </thead>
      <tbody>
        {#each conclusions as c}
          <tr>
            <td class="task-title">{c.title}</td>
            <td>{c.submitter_name}</td>
            <td class="summary-cell">{c.conclusion_summary ?? '—'}</td>
            <td class="mono">{formatDate(c.completed_at)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.25rem; }
  .subtitle { color:#64748B;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .error { color:#EF4444;font-size:0.82rem; }
  .table-wrap { overflow-x:auto; }
  .table { width:100%;border-collapse:collapse;font-size:0.78rem; }
  .table th { text-align:left;color:#64748B;font-weight:600;padding:0.5rem 0.6rem;border-bottom:1px solid rgba(58,190,255,0.1); }
  .table td { padding:0.45rem 0.6rem;color:#E6EDF3;border-bottom:1px solid rgba(255,255,255,0.04); }
  .table tr:hover td { background:rgba(58,190,255,0.03); }
  .task-title { font-weight:600; }
  .summary-cell { max-width:300px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:#94A3B8; }
  .mono { font-size:0.72rem;color:#64748B; }
</style>
