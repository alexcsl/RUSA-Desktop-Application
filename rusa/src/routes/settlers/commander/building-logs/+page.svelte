<!--
  /settlers/commander/building-logs — Commander views building health logs
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { stlGetBuildingLogs, type BuildingLogRow } from '$lib/stores/settlers';

  let logs: BuildingLogRow[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try { logs = await stlGetBuildingLogs(); }
    catch (e: any) { error = e?.message ?? String(e); }
    finally { loading = false; }
  });

  function statusClass(s: string): string {
    return s === 'pass' ? 'st-pass' : s === 'fail' ? 'st-fail' : 'st-repair';
  }
</script>

<h2>Building Health Logs</h2>
<p class="hint">All building health check logs submitted by civil engineers.</p>

{#if loading}
  <p class="dim">Loading…</p>
{:else if error}
  <p class="err">{error}</p>
{:else if logs.length === 0}
  <p class="dim">No building health logs submitted yet.</p>
{:else}
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Building</th><th>Engineer</th><th>Date</th><th>Status</th><th>Findings</th></tr>
      </thead>
      <tbody>
        {#each logs as l}
          <tr>
            <td class="cell-title">{l.building_name}</td>
            <td>{l.submitted_by_name}</td>
            <td class="dim">{new Date(l.check_date).toLocaleDateString()}</td>
            <td><span class="badge {statusClass(l.status)}">{l.status.replace(/_/g,' ')}</span></td>
            <td class="cell-find">{l.findings ?? '—'}</td>
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
  .cell-find { max-width:260px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .badge { display:inline-block;padding:0.15rem 0.45rem;border-radius:4px;font-size:0.7rem;font-weight:700;text-transform:capitalize; }
  .st-pass { background:rgba(74,222,128,0.15);color:#4ADE80; }
  .st-fail { background:rgba(239,68,68,0.15);color:#EF4444; }
  .st-repair { background:rgba(245,158,11,0.15);color:#F59E0B; }
</style>
