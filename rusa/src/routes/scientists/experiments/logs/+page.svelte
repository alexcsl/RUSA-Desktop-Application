<!-- /scientists/experiments/logs — Experiment Logs list (Physicist/Chemist/Biologist) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getMyExperimentLogs, type ExperimentLogWithExp } from '$lib/stores/scientists';

  let logs: ExperimentLogWithExp[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let search = $state('');

  let filtered = $derived(
    logs.filter((l) => {
      const q = search.toLowerCase();
      return (
        l.experiment_title.toLowerCase().includes(q) ||
        (l.completed_actions ?? '').toLowerCase().includes(q) ||
        (l.rag_status ?? '').toLowerCase().includes(q)
      );
    })
  );

  onMount(async () => {
    try { logs = await getMyExperimentLogs(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  });

  function ragColor(rag: string | null): string {
    switch (rag) {
      case 'green': return '#10B981';
      case 'amber': return '#F59E0B';
      case 'red': return '#EF4444';
      default: return '#94A3B8';
    }
  }
</script>

<h1 class="title">Experiment Logs</h1>
<div class="toolbar">
  <input class="search" type="text" placeholder="Search logs…" bind:value={search} />
  <span class="count">{filtered.length} log{filtered.length !== 1 ? 's' : ''}</span>
</div>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="err">{error}</p>
{:else if filtered.length === 0}
  <div class="empty">No experiment logs found.</div>
{:else}
  <div class="log-list">
    {#each filtered as log}
      <div class="log-card">
        <div class="card-header">
          <span class="exp-title">{log.experiment_title}</span>
          <div class="card-meta">
            {#if log.rag_status}
              <span class="rag-badge" style="color:{ragColor(log.rag_status)};border-color:{ragColor(log.rag_status)}40">
                {log.rag_status.toUpperCase()}
              </span>
            {/if}
            <span class="date">{log.log_date}</span>
          </div>
        </div>
        {#if log.completed_actions}
          <div class="section">
            <span class="section-label">Completed Actions</span>
            <p class="section-body">{log.completed_actions}</p>
          </div>
        {/if}
        {#if log.pending_actions}
          <div class="section">
            <span class="section-label">Pending Actions</span>
            <p class="section-body">{log.pending_actions}</p>
          </div>
        {/if}
        <div class="card-footer">
          <a href="/scientists/experiments/{log.experiment_id}" class="exp-link">View Experiment →</a>
          <span class="id-tag">{new Date(log.created_at).toLocaleString()}</span>
        </div>
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.6rem; }
  .toolbar { display:flex;align-items:center;gap:0.75rem;margin-bottom:0.75rem; }
  .search { flex:1;max-width:380px;background:#0E1428;border:1px solid rgba(58,190,255,0.15);border-radius:6px;color:#E6EDF3;padding:0.4rem 0.6rem;font-size:0.8rem; }
  .search:focus { outline:none;border-color:#3ABEFF; }
  .count { font-size:0.75rem;color:#6B7280; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .err { color:#EF4444;font-size:0.8rem; }
  .empty { background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:1.25rem;text-align:center;color:#94A3B8;font-size:0.85rem; }
  .log-list { display:flex;flex-direction:column;gap:0.5rem;max-width:700px; }
  .log-card { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:0.75rem;display:flex;flex-direction:column;gap:0.4rem; }
  .card-header { display:flex;justify-content:space-between;align-items:center; }
  .exp-title { font-size:0.88rem;font-weight:600;color:#E6EDF3; }
  .card-meta { display:flex;align-items:center;gap:0.5rem; }
  .rag-badge { font-size:0.65rem;font-weight:700;padding:0.1rem 0.35rem;border-radius:4px;border:1px solid; }
  .date { font-size:0.7rem;color:#6B7280; }
  .section { display:flex;flex-direction:column;gap:0.1rem; }
  .section-label { font-size:0.65rem;color:#94A3B8;text-transform:uppercase;letter-spacing:0.05em; }
  .section-body { font-size:0.78rem;color:#CBD5E1;margin:0;line-height:1.4; }
  .card-footer { display:flex;justify-content:space-between;align-items:center; }
  .exp-link { font-size:0.72rem;color:#3ABEFF;text-decoration:none; }
  .exp-link:hover { text-decoration:underline; }
  .id-tag { font-size:0.65rem;color:#374151; }
</style>
