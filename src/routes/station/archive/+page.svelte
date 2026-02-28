<!-- /station/archive — UC-SSS-08: View Private Station Archive -->
<script lang="ts">
  import { getContext, onMount } from 'svelte';
  import type { Writable } from 'svelte/store';
  import { sstGetArchive, type ArchiveEntry } from '$lib/stores/space_station';

  const stationId = getContext<Writable<string>>('selectedStationId');

  let entries: ArchiveEntry[] = $state([]);
  let loading = $state(true);
  let searchTerm = $state('');
  let filterType = $state('all');
  let expandedId: string | null = $state(null);

  let currentStationId = $state('');
  const unsub = stationId.subscribe((v) => {
    currentStationId = v;
    if (v) loadArchive();
  });

  async function loadArchive() {
    if (!currentStationId) return;
    loading = true;
    try {
      entries = await sstGetArchive(currentStationId, searchTerm.trim() || undefined);
    } catch {}
    loading = false;
  }

  onMount(() => { if (currentStationId) loadArchive(); });

  function handleSearch() { loadArchive(); }

  let findingTypes = $derived([...new Set(entries.map((e) => e.finding_type).filter(Boolean))]);

  let filtered = $derived(
    entries.filter((e) => {
      if (filterType !== 'all' && e.finding_type !== filterType) return false;
      return true;
    })
  );

  function severityColor(sev: string): string {
    switch (sev) {
      case 'critical': return '#EF4444';
      case 'high': return '#F59E0B';
      case 'medium': return '#3ABEFF';
      case 'low': return '#10B981';
      default: return '#94A3B8';
    }
  }

  function toggleExpand(id: string) {
    expandedId = expandedId === id ? null : id;
  }
</script>

<h1 class="title">Private Station Archive</h1>
<p class="subtitle">Historical internal findings. Search by date, type, or keyword.</p>

<div class="controls">
  <input type="text" class="search-input" bind:value={searchTerm} placeholder="Search archive…"
    onkeydown={(e) => { if (e.key === 'Enter') handleSearch(); }} />
  <button class="btn-search" onclick={handleSearch}>Search</button>
  <select class="filter-select" bind:value={filterType}>
    <option value="all">All Types</option>
    {#each findingTypes as ft}
      <option value={ft}>{ft}</option>
    {/each}
  </select>
</div>

{#if loading}
  <p class="muted">Loading archive…</p>
{:else if filtered.length === 0}
  <p class="muted">No archive entries found.</p>
{:else}
  <div class="archive-list">
    {#each filtered as entry}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div class="archive-card" role="button" tabindex="0" onclick={() => toggleExpand(entry.id)}>
        <div class="arc-header">
          <div class="arc-title-row">
            <span class="arc-type">{entry.finding_type ?? 'Unknown'}</span>
            {#if (entry.archive_data as Record<string, unknown>).severity}
              {@const sev = String((entry.archive_data as Record<string, unknown>).severity)}
              <span class="badge" style="background:rgba({severityColor(sev) === '#EF4444' ? '239,68,68' : severityColor(sev) === '#F59E0B' ? '245,158,11' : severityColor(sev) === '#3ABEFF' ? '58,190,255' : '16,185,129'},0.15);color:{severityColor(sev)}">{sev}</span>
            {/if}
          </div>
          <span class="arc-date">{new Date(entry.created_at).toLocaleString()}</span>
        </div>
        <div class="arc-meta">
          <span>👤 {entry.logger_name}</span>
          {#if (entry.archive_data as Record<string, unknown>).location_on_station}
            <span>📍 {(entry.archive_data as Record<string, unknown>).location_on_station}</span>
          {/if}
          {#if (entry.archive_data as Record<string, unknown>).is_reported_to_security}
            <span class="reported-tag">Reported to Security</span>
          {/if}
        </div>

        {#if expandedId === entry.id}
          <div class="arc-detail">
            {#if (entry.archive_data as Record<string, unknown>).description}
              <p class="arc-desc">{(entry.archive_data as Record<string, unknown>).description}</p>
            {/if}
            {#if (entry.archive_data as Record<string, unknown>).response_procedure}
              <p class="arc-action"><strong>Response:</strong> {(entry.archive_data as Record<string, unknown>).response_procedure}</p>
            {/if}
            {#if Array.isArray((entry.archive_data as Record<string, unknown>).category)}
              <div class="arc-cats">
                {#each ((entry.archive_data as Record<string, unknown>).category as string[]) as cat}
                  <span class="cat-tag">{cat}</span>
                {/each}
              </div>
            {/if}
          </div>
        {:else}
          <p class="arc-preview">
            {String((entry.archive_data as Record<string, unknown>).description ?? '').slice(0, 120)}{String((entry.archive_data as Record<string, unknown>).description ?? '').length > 120 ? '…' : ''}
          </p>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .controls { display:flex;gap:0.5rem;margin-bottom:1rem;max-width:650px; }
  .search-input { flex:1;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .search-input:focus { outline:none;border-color:#3ABEFF; }
  .btn-search { background:rgba(58,190,255,0.15);border:1px solid rgba(58,190,255,0.25);color:#3ABEFF;border-radius:6px;padding:0.45rem 0.75rem;font-size:0.8rem;cursor:pointer; }
  .btn-search:hover { background:rgba(58,190,255,0.25); }
  .filter-select { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .archive-list { display:flex;flex-direction:column;gap:0.5rem;max-width:750px; }
  .archive-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.85rem;cursor:pointer;transition:border-color 0.15s; }
  .archive-card:hover { border-color:rgba(58,190,255,0.2); }
  .arc-header { display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:0.4rem; }
  .arc-title-row { display:flex;align-items:center;gap:0.5rem; }
  .arc-type { font-size:0.9rem;font-weight:600;color:#E6EDF3; }
  .badge { padding:0.1rem 0.4rem;border-radius:4px;font-size:0.65rem;font-weight:600;text-transform:uppercase; }
  .arc-date { font-size:0.7rem;color:#94A3B8; }
  .arc-meta { display:flex;flex-wrap:wrap;gap:0.5rem;font-size:0.75rem;color:#94A3B8;margin-bottom:0.4rem; }
  .reported-tag { background:rgba(16,185,129,0.15);color:#10B981;padding:0.05rem 0.3rem;border-radius:3px; }
  .arc-preview { font-size:0.8rem;color:#94A3B8;margin:0;line-height:1.4; }
  .arc-detail { margin-top:0.3rem; }
  .arc-desc { font-size:0.8rem;color:#C9D1D9;margin:0.3rem 0;line-height:1.4; }
  .arc-action { font-size:0.75rem;color:#94A3B8;margin:0.2rem 0; }
  .arc-cats { display:flex;flex-wrap:wrap;gap:0.3rem;margin-top:0.3rem; }
  .cat-tag { background:rgba(139,92,246,0.15);color:#C084FC;padding:0.05rem 0.3rem;border-radius:3px;font-size:0.7rem;text-transform:capitalize; }
</style>
