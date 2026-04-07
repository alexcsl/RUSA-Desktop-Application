<!--
  Observer: Experiments view — all experiments from Biologist / Chemist / AgEng / BioEng.
  Access: TheObserver
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { dirGetExperiments, type DirExperimentRow } from '$lib/stores/directors';

  let experiments: DirExperimentRow[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    loading = true; error = '';
    try { experiments = await dirGetExperiments(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  });

  function statusColor(s: string): string {
    const m: Record<string, string> = { pending: '#F59E0B', approved: '#22C55E', active: '#3ABEFF', closed: '#94A3B8', rejected: '#EF4444' };
    return m[s] ?? '#94A3B8';
  }
</script>

<h2 class="title">Experiments</h2>
<p class="subtitle">All experiments from biological, chemical, and engineering personnel.</p>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if experiments.length === 0}
  <p class="muted">No experiments found.</p>
{:else}
  <div class="table-wrap">
    <table class="table">
      <thead>
        <tr>
          <th>Title</th>
          <th>Type</th>
          <th>Status</th>
          <th>Proposed By</th>
          <th>Date</th>
        </tr>
      </thead>
      <tbody>
        {#each experiments as exp}
          <tr>
            <td class="title-cell">{exp.title}</td>
            <td class="type-cell">{exp.experiment_type.replace(/_/g, ' ')}</td>
            <td>
              <span class="status-badge" style="color:{statusColor(exp.status)}">{exp.status}</span>
            </td>
            <td>{exp.proposer_name}</td>
            <td class="mono">{new Date(exp.created_at).toLocaleDateString()}</td>
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
  .title-cell { font-weight:600; }
  .type-cell { text-transform:capitalize;color:#94A3B8; }
  .status-badge { font-weight:700;text-transform:capitalize; }
  .mono { font-size:0.72rem;color:#64748B; }
</style>
