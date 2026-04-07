<!--
  Artificer: Experiment Daily Logs view.
  Access: TheArtificer
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { dirGetExperimentLogs, type DirExperimentLogRow } from '$lib/stores/directors';

  let logs: DirExperimentLogRow[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    loading = true; error = '';
    try { logs = await dirGetExperimentLogs(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  });

  function ragColor(s: string | null): string {
    const m: Record<string, string> = { red: '#EF4444', amber: '#F59E0B', green: '#22C55E' };
    return s ? (m[s] ?? '#94A3B8') : '#94A3B8';
  }
</script>

<h2 class="title">Experiment Daily Logs</h2>
<p class="subtitle">Daily session logs submitted by your domain's scientists and engineers.</p>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if logs.length === 0}
  <p class="muted">No experiment logs found.</p>
{:else}
  <div class="log-list">
    {#each logs as log}
      <div class="log-card">
        <div class="log-header">
          <span class="exp-title">{log.experiment_title}</span>
          <span class="log-date">{log.log_date}</span>
          {#if log.rag_status}
            <span class="rag-dot" style="background:{ragColor(log.rag_status)}" title={log.rag_status}></span>
          {/if}
        </div>
        {#if log.logger_name}
          <p class="logger">Logged by: <strong>{log.logger_name}</strong></p>
        {/if}
        {#if log.completed_actions}
          <p class="actions-done">Completed: {log.completed_actions}</p>
        {/if}
        {#if log.pending_actions}
          <p class="actions-pending">Pending: {log.pending_actions}</p>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.25rem; }
  .subtitle { color:#64748B;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .error { color:#EF4444;font-size:0.82rem; }
  .log-list { display:flex;flex-direction:column;gap:0.75rem; }
  .log-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.9rem; }
  .log-header { display:flex;align-items:center;gap:0.6rem;margin-bottom:0.4rem; }
  .exp-title { font-size:0.88rem;font-weight:600;color:#E6EDF3;flex:1; }
  .log-date { font-size:0.72rem;color:#64748B;font-family:monospace; }
  .rag-dot { width:10px;height:10px;border-radius:50%;flex-shrink:0; }
  .logger { font-size:0.75rem;color:#94A3B8;margin:0 0 0.3rem; }
  .actions-done { font-size:0.78rem;color:#22C55E;margin:0 0 0.2rem; }
  .actions-pending { font-size:0.78rem;color:#F59E0B;margin:0; }
</style>
