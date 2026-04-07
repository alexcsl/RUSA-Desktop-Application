<!--
  /settlers/commander/farm-logs — Commander views farm health logs
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { stlGetFarmHealthLogs, type FarmHealthRow } from '$lib/stores/settlers';

  let logs: FarmHealthRow[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try { logs = await stlGetFarmHealthLogs(); }
    catch (e: any) { error = e?.message ?? String(e); }
    finally { loading = false; }
  });
</script>

<h2>Farm Health Logs</h2>
<p class="hint">All farm health check logs submitted by farmers.</p>

{#if loading}
  <p class="dim">Loading…</p>
{:else if error}
  <p class="err">{error}</p>
{:else if logs.length === 0}
  <p class="dim">No farm health logs submitted yet.</p>
{:else}
  <div class="table-wrap">
    <table>
      <thead>
        <tr><th>Type</th><th>Subject</th><th>Farmer</th><th>Date</th><th>Condition</th><th>Treatment</th><th>Notes</th></tr>
      </thead>
      <tbody>
        {#each logs as l}
          <tr>
            <td><span class="badge {l.subject_type === 'plant' ? 'plant' : 'livestock'}">{l.subject_type}</span></td>
            <td class="cell-title">{l.subject_name}</td>
            <td>{l.submitted_by_name}</td>
            <td class="dim">{new Date(l.log_date).toLocaleDateString()}</td>
            <td>{l.condition}</td>
            <td class="dim">{l.treatment ?? '—'}</td>
            <td class="dim cell-notes">{l.notes ?? '—'}</td>
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
  .cell-notes { max-width:200px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .badge { display:inline-block;padding:0.15rem 0.45rem;border-radius:4px;font-size:0.7rem;font-weight:600;text-transform:capitalize; }
  .plant { background:rgba(74,222,128,0.15);color:#4ADE80; }
  .livestock { background:rgba(245,158,11,0.15);color:#F59E0B; }
</style>
