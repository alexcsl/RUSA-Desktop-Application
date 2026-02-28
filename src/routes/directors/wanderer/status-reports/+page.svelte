<!-- /directors/wanderer/status-reports — UC-WAN-03: All Mission Status Reports -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getAllStatusReports, type StatusReportItem } from '$lib/stores/astronauts';

  let reports: StatusReportItem[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let filterRag: 'all' | 'red' | 'amber' | 'green' = $state('all');

  onMount(async () => {
    try {
      reports = await getAllStatusReports();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  let filtered = $derived.by(() => {
    if (filterRag === 'all') return reports;
    return reports.filter((r) => r.rag_status === filterRag);
  });

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleDateString('en-GB', { day: '2-digit', month: 'short', year: 'numeric' });
  }

  function ragColor(rag: string | null): string {
    switch (rag) {
      case 'red': return '#EF4444';
      case 'amber': return '#F59E0B';
      case 'green': return '#10B981';
      default: return '#94A3B8';
    }
  }

  function ragBgStyle(rag: string | null): string {
    switch (rag) {
      case 'red': return 'background:rgba(239,68,68,0.08);border-left:3px solid #EF4444;';
      case 'amber': return 'background:rgba(245,158,11,0.08);border-left:3px solid #F59E0B;';
      case 'green': return 'background:rgba(16,185,129,0.08);border-left:3px solid #10B981;';
      default: return 'border-left:3px solid #475569;';
    }
  }
</script>

<h1 class="title">Wanderer — Status Reports Feed</h1>
<p class="subtitle">All mission status reports submitted by astronauts.</p>

{#if loading}
  <p class="loading">Loading reports…</p>
{:else if error}
  <p class="error">{error}</p>
{:else}
  <div class="toolbar">
    <span class="filter-label">Filter:</span>
    <button class:active={filterRag === 'all'} onclick={() => (filterRag = 'all')}>All ({reports.length})</button>
    <button class:active={filterRag === 'red'} onclick={() => (filterRag = 'red')}>🔴 Red</button>
    <button class:active={filterRag === 'amber'} onclick={() => (filterRag = 'amber')}>🟡 Amber</button>
    <button class:active={filterRag === 'green'} onclick={() => (filterRag = 'green')}>🟢 Green</button>
  </div>

  {#if filtered.length === 0}
    <p class="empty">No reports match the filter.</p>
  {:else}
    <div class="report-list">
      {#each filtered as r}
        <div class="report-card" style={ragBgStyle(r.rag_status)}>
          <div class="report-header">
            <span class="rag-dot" style="background:{ragColor(r.rag_status)}"></span>
            <span class="report-date">{formatDate(r.report_date)}</span>
            <span class="report-author">{r.submitter_name}</span>
            {#if r.month_tracker}<span class="report-month">{r.month_tracker}</span>{/if}
          </div>
          <p class="report-body">{r.current_status}</p>
          {#if r.issues_blockers}
            <p class="report-issues"><strong>Issues:</strong> {r.issues_blockers}</p>
          {/if}
          {#if r.collected_samples_last_month}
            <p class="report-detail"><strong>Samples:</strong> {r.collected_samples_last_month}</p>
          {/if}
          {#if r.progress_last_month}
            <p class="report-detail"><strong>Progress:</strong> {r.progress_last_month}</p>
          {/if}
          {#if r.plans_next_month}
            <p class="report-detail"><strong>Plans:</strong> {r.plans_next_month}</p>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .loading { color:#94A3B8; }
  .error { color:#EF4444; }
  .empty { color:#475569;font-size:0.85rem; }

  .toolbar { display:flex;gap:0.4rem;align-items:center;margin-bottom:1rem; }
  .filter-label { color:#94A3B8;font-size:0.75rem; }
  .toolbar button { background:none;border:1px solid rgba(58,190,255,0.1);color:#94A3B8;padding:0.3rem 0.6rem;border-radius:4px;font-size:0.75rem;cursor:pointer; }
  .toolbar button:hover { color:#E6EDF3;border-color:rgba(58,190,255,0.3); }
  .toolbar button.active { color:#3ABEFF;border-color:#3ABEFF; }

  .report-list { display:flex;flex-direction:column;gap:0.5rem; }
  .report-card { padding:0.75rem;border-radius:6px;background:rgba(14,20,40,0.4); }
  .report-header { display:flex;gap:0.5rem;align-items:center;margin-bottom:0.3rem; }
  .rag-dot { width:10px;height:10px;border-radius:50%;flex-shrink:0; }
  .report-date { color:#E6EDF3;font-weight:600;font-size:0.8rem; }
  .report-author { color:#94A3B8;font-size:0.75rem; }
  .report-month { color:#8B5CF6;font-size:0.7rem; }
  .report-body { color:#CBD5E1;font-size:0.8rem;margin:0.15rem 0; }
  .report-issues { color:#F59E0B;font-size:0.78rem;margin:0.15rem 0; }
  .report-detail { color:#94A3B8;font-size:0.78rem;margin:0.1rem 0; }
</style>
