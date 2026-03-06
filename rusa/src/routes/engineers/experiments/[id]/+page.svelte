<!--
  /engineers/experiments/[id] — Experiment Detail + Daily Log List
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { getExperimentDetail, type ExperimentSummary, type ExperimentLogSummary } from '$lib/stores/engineers';

  let experimentId = $state('');
  const unsubPage = page.subscribe((p) => (experimentId = p.params.id ?? ''));
  onDestroy(unsubPage);

  let experiment: ExperimentSummary | null = $state(null);
  let logs: ExperimentLogSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try {
      const [exp, logList] = await getExperimentDetail(experimentId);
      experiment = exp;
      logs = logList;
    } catch (e: any) {
      error = e?.toString() ?? 'Failed to load experiment.';
    } finally {
      loading = false;
    }
  });

  function ragColor(r: string | null): string {
    if (r === 'red') return '#EF4444';
    if (r === 'amber') return '#F59E0B';
    if (r === 'green') return '#22C55E';
    return '#94A3B8';
  }
</script>

<h2>Experiment Detail</h2>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if !experiment}
  <p class="muted">Experiment not found.</p>
{:else}
  <div class="card">
    <h3>{experiment.title}</h3>
    <div class="meta">
      <span>Type: <strong>{experiment.experiment_type}</strong></span>
      <span>Status: <strong>{experiment.status}</strong></span>
      <span>Proposed by: <strong>{experiment.proposer_name}</strong></span>
      {#if experiment.approved_at}<span>Approved: {new Date(experiment.approved_at).toLocaleDateString()}</span>{/if}
      {#if experiment.closed_at}<span>Closed: {new Date(experiment.closed_at).toLocaleDateString()}</span>{/if}
    </div>
    {#if experiment.metadata}
      <div class="metadata-block">
        {#if experiment.metadata.introduction}<p><strong>Introduction:</strong> {experiment.metadata.introduction}</p>{/if}
        {#if experiment.metadata.problem_statement}<p><strong>Problem:</strong> {experiment.metadata.problem_statement}</p>{/if}
        {#if experiment.metadata.methodology}<p><strong>Methodology:</strong> {experiment.metadata.methodology}</p>{/if}
      </div>
    {/if}
  </div>

  <!-- Action links -->
  <div class="action-bar">
    {#if experiment.status === 'approved' || experiment.status === 'active'}
      <a href="/engineers/experiments/{experimentId}/log" class="btn-link">+ Log Session</a>
    {/if}
    {#if experiment.status === 'active'}
      <a href="/engineers/experiments/{experimentId}/conclusion" class="btn-link">Request Conclusion</a>
    {/if}
  </div>

  <!-- Daily logs -->
  <h3 class="section-title">Daily Logs ({logs.length})</h3>
  {#if logs.length === 0}
    <p class="muted">No logs recorded yet.</p>
  {:else}
    <table class="tbl">
      <thead><tr><th>Date</th><th>RAG</th><th>Completed</th><th>Pending</th></tr></thead>
      <tbody>
        {#each logs as l}
          <tr>
            <td>{l.log_date}</td>
            <td><span style="color:{ragColor(l.rag_status)};font-weight:700">{l.rag_status ?? '—'}</span></td>
            <td class="trunc">{l.completed_actions ?? '—'}</td>
            <td class="trunc">{l.pending_actions ?? '—'}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:1rem; }
  h3 { font-size:0.95rem;color:#E6EDF3;margin:0 0 0.5rem; }
  .section-title { margin-top:1.5rem;font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#3ABEFF; }
  .muted { color:#64748B;font-size:0.8rem; }
  .error { color:#EF4444;font-size:0.85rem; }
  .card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem; }
  .meta { display:flex;gap:1rem;flex-wrap:wrap;font-size:0.8rem;color:#94A3B8;margin-bottom:0.5rem; }
  .metadata-block p { font-size:0.8rem;color:#CBD5E1;margin:0.25rem 0; }
  .action-bar { display:flex;gap:0.75rem;margin:1rem 0; }
  .btn-link { color:#3ABEFF;font-size:0.8rem;text-decoration:none;padding:0.3rem 0.6rem;border:1px solid rgba(58,190,255,0.3);border-radius:4px; }
  .btn-link:hover { background:rgba(58,190,255,0.1); }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.5rem;color:#94A3B8;border-bottom:1px solid rgba(58,190,255,0.15);font-size:0.7rem;text-transform:uppercase; }
  .tbl td { padding:0.5rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .trunc { max-width:200px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
</style>
