<!--
  Guardian / Overseer: Lost and Found Logs view
  Access: TheGuardian, TheOverseer
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { dirGuardianGetLostFoundLogs, type DirLostFoundRow } from '$lib/stores/directors';

  let logs: DirLostFoundRow[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    loading = true; error = '';
    try { logs = await dirGuardianGetLostFoundLogs(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  });

  function statusColor(s: string): string {
    const m: Record<string, string> = { found: '#3ABEFF', claimed: '#22C55E', unclaimed: '#F59E0B' };
    return m[s] ?? '#94A3B8';
  }

  function formatDate(d: string | null): string {
    return d ? new Date(d).toLocaleString() : '—';
  }
</script>

<h2 class="title">Lost and Found Logs</h2>
<p class="subtitle">All reported lost-and-found items across the station.</p>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if logs.length === 0}
  <p class="muted">No lost-and-found entries recorded.</p>
{:else}
  <div class="table-wrap">
    <table class="table">
      <thead>
        <tr>
          <th>Item Description</th>
          <th>Location Found</th>
          <th>Status</th>
          <th>Reporter</th>
          <th>Found At</th>
          <th>Logged</th>
        </tr>
      </thead>
      <tbody>
        {#each logs as log}
          <tr>
            <td>{log.item_description}</td>
            <td>{log.location_found}</td>
            <td>
              <span class="status-badge" style="color:{statusColor(log.status)}">
                {log.status}
              </span>
            </td>
            <td>{log.reporter_name}</td>
            <td class="mono">{formatDate(log.found_at)}</td>
            <td class="mono">{formatDate(log.created_at)}</td>
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
  .table td { padding:0.45rem 0.6rem;color:#E6EDF3;border-bottom:1px solid rgba(255,255,255,0.04);vertical-align:top; }
  .table tr:hover td { background:rgba(58,190,255,0.03); }
  .status-badge { font-weight:700;text-transform:capitalize; }
  .mono { font-size:0.72rem;color:#64748B; }
</style>
