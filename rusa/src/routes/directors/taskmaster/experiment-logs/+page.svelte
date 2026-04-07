<!--
  /directors/taskmaster/experiment-logs — All experiment daily logs (read-only)
  TheTaskmaster views all experiment logs across the organization.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { dirGetExperimentLogs, type DirExperimentLogRow } from '$lib/stores/directors';

  let logs: DirExperimentLogRow[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try { logs = await dirGetExperimentLogs(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  });

  function formatDate(d: string) { return new Date(d).toLocaleDateString(); }

  function ragColor(rag: string | null): string {
    if (!rag) return '#94A3B8';
    const m: Record<string, string> = { red: '#EF4444', amber: '#F59E0B', green: '#22C55E' };
    return m[rag.toLowerCase()] ?? '#94A3B8';
  }
</script>

<h1 class="title">Experiment Logs</h1>
<p class="subtitle">Daily logs for all experiments across the organization.</p>

{#if loading}<p class="muted">Loading…</p>
{:else if error}<p class="error">{error}</p>
{:else if logs.length === 0}<p class="muted">No experiment logs found.</p>
{:else}
  <div class="list">
    {#each logs as log}
      <div class="card">
        <div class="card-header">
          <span class="card-title">{log.experiment_title}</span>
          {#if log.rag_status}
            <span class="rag-badge" style="color:{ragColor(log.rag_status)};border-color:{ragColor(log.rag_status)}">
              {log.rag_status.toUpperCase()}
            </span>
          {/if}
        </div>
        <p class="meta">
          Logged by: <strong>{log.logger_name ?? '—'}</strong> · {formatDate(log.log_date)}
        </p>
        {#if log.completed_actions}
          <div class="section-block">
            <span class="section-label">Completed:</span>
            <span class="section-text">{log.completed_actions}</span>
          </div>
        {/if}
        {#if log.pending_actions}
          <div class="section-block">
            <span class="section-label">Pending:</span>
            <span class="section-text">{log.pending_actions}</span>
          </div>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .error { color:#EF4444;font-size:0.75rem; }
  .list { display:flex;flex-direction:column;gap:0.5rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.9rem;display:flex;flex-direction:column;gap:0.35rem; }
  .card-header { display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:0.4rem; }
  .card-title { font-size:0.88rem;font-weight:600;color:#E6EDF3; }
  .rag-badge { font-size:0.65rem;font-weight:700;border:1px solid;border-radius:4px;padding:0.1rem 0.4rem; }
  .meta { font-size:0.72rem;color:#64748B;margin:0; }
  .section-block { display:flex;gap:0.4rem;font-size:0.77rem; }
  .section-label { color:#8B5CF6;font-weight:600;white-space:nowrap; }
  .section-text { color:#94A3B8; }
</style>
