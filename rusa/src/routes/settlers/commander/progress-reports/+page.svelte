<!--
  /settlers/commander/progress-reports — Commander views all settler progress reports
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { stlGetProgressReports, type ProgressReportRow } from '$lib/stores/settlers';

  let reports: ProgressReportRow[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try { reports = await stlGetProgressReports(); }
    catch (e: any) { error = e?.message ?? String(e); }
    finally { loading = false; }
  });

  function ragClass(r: string | null): string {
    if (!r) return '';
    return r === 'red' ? 'rag-red' : r === 'amber' ? 'rag-amber' : 'rag-green';
  }
</script>

<h2>Progress Reports</h2>
<p class="hint">All progress reports submitted by settlers in your settlement.</p>

{#if loading}
  <p class="dim">Loading…</p>
{:else if error}
  <p class="err">{error}</p>
{:else if reports.length === 0}
  <p class="dim">No progress reports submitted yet.</p>
{:else}
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Task</th><th>Submitted By</th><th>Week</th><th>RAG</th><th>Progress</th><th>Materials / Equipment</th><th>Date</th></tr>
      </thead>
      <tbody>
        {#each reports as r}
          <tr>
            <td class="cell-title">{r.task_title}</td>
            <td>{r.submitted_by_name}</td>
            <td class="dim">{r.week ?? '—'}</td>
            <td>{#if r.rag_status}<span class="rag {ragClass(r.rag_status)}">{r.rag_status}</span>{:else}—{/if}</td>
            <td class="cell-prog">{r.progress_made}</td>
            <td class="dim">{r.materials_equipment ?? '—'}</td>
            <td class="dim">{new Date(r.created_at).toLocaleDateString()}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 0.25rem; }
  .hint { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .dim { color:#64748B;font-size:0.8rem; }
  .err { color:#EF4444;font-size:0.85rem; }
  .table-wrap { overflow-x:auto; }
  table { width:100%;border-collapse:collapse;font-size:0.8rem; }
  th { text-align:left;color:#94A3B8;font-size:0.7rem;text-transform:uppercase;padding:0.4rem 0.6rem;border-bottom:1px solid #1E293B; }
  td { padding:0.4rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.03); }
  .cell-title { font-weight:600; }
  .cell-prog { max-width:240px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .rag { display:inline-block;padding:0.1rem 0.45rem;border-radius:4px;font-size:0.7rem;font-weight:700;text-transform:uppercase; }
  .rag-green { background:rgba(74,222,128,0.15);color:#4ADE80; }
  .rag-amber { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .rag-red { background:rgba(239,68,68,0.15);color:#EF4444; }
</style>
