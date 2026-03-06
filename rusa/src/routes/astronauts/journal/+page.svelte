<!-- /astronauts/journal — UC-AS-05: Personal Mission Journal -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getPersonalJournal, type JournalSummary, type JournalEntry } from '$lib/stores/astronauts';

  let journal: JournalSummary | null = $state(null);
  let loading = $state(true);
  let error = $state('');
  let sortKey: 'date' | 'type' = $state('date');
  let sortAsc = $state(false);

  onMount(async () => {
    try {
      journal = await getPersonalJournal();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  let sortedMissions = $derived.by(() => {
    if (!journal) return [];
    const arr = [...journal.missions];
    arr.sort((a, b) => {
      if (sortKey === 'date') {
        const diff = new Date(a.completed_at).getTime() - new Date(b.completed_at).getTime();
        return sortAsc ? diff : -diff;
      }
      const diff = a.mission_type.localeCompare(b.mission_type);
      return sortAsc ? diff : -diff;
    });
    return arr;
  });

  function toggleSort(key: 'date' | 'type') {
    if (sortKey === key) { sortAsc = !sortAsc; }
    else { sortKey = key; sortAsc = true; }
  }

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleDateString('en-GB', { day: '2-digit', month: 'short', year: 'numeric' });
  }

  function dangerClass(level: string | null): string {
    switch (level) {
      case 'critical': return 'danger-critical';
      case 'high': return 'danger-high';
      case 'medium': return 'danger-medium';
      default: return 'danger-low';
    }
  }
</script>

<h1 class="title">Mission Journal</h1>
<p class="subtitle">Your completed missions and lifetime counters.</p>

{#if loading}
  <p class="loading">Loading journal…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if journal}
  <!-- Counter cards -->
  <div class="counter-row">
    <div class="counter-card interstellar">
      <span class="counter-value">{journal.interstellar_count}</span>
      <span class="counter-label">Interstellar</span>
    </div>
    <div class="counter-card terrain">
      <span class="counter-value">{journal.terrain_count}</span>
      <span class="counter-label">Terrain</span>
    </div>
    <div class="counter-card total">
      <span class="counter-value">{journal.interstellar_count + journal.terrain_count}</span>
      <span class="counter-label">Total Missions</span>
    </div>
  </div>

  <!-- Mission list -->
  <div class="card">
    <div class="sort-controls">
      <span class="sort-label">Sort by:</span>
      <button class:sort-active={sortKey === 'date'} onclick={() => toggleSort('date')}>
        Date {sortKey === 'date' ? (sortAsc ? '↑' : '↓') : ''}
      </button>
      <button class:sort-active={sortKey === 'type'} onclick={() => toggleSort('type')}>
        Type {sortKey === 'type' ? (sortAsc ? '↑' : '↓') : ''}
      </button>
    </div>

    {#if sortedMissions.length === 0}
      <p class="empty">No completed missions yet. Keep exploring!</p>
    {:else}
      <div class="mission-list">
        {#each sortedMissions as m}
          <div class="journal-item">
            <div class="journal-main">
              <span class="journal-title">{m.title}</span>
              <span class="journal-location">📍 {m.location}</span>
            </div>
            <div class="journal-meta">
              <span class="badge type-badge">{m.mission_type}</span>
              <span class="badge {dangerClass(m.danger_level)}">{m.danger_level ?? 'n/a'}</span>
              <span class="journal-date">{formatDate(m.completed_at)}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .loading { color:#94A3B8; }
  .error { color:#EF4444; }
  .empty { color:#475569;font-size:0.85rem; }

  /* Counters */
  .counter-row { display:flex;gap:1rem;margin-bottom:1.25rem;flex-wrap:wrap; }
  .counter-card { flex:1;min-width:140px;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:10px;padding:1rem;text-align:center;display:flex;flex-direction:column;align-items:center;gap:0.35rem; }
  .counter-value { font-family:'Orbitron',sans-serif;font-size:2rem;font-weight:700; }
  .counter-label { font-size:0.75rem;text-transform:uppercase;letter-spacing:0.5px; }
  .interstellar .counter-value { color:#3ABEFF; }
  .interstellar .counter-label { color:#3ABEFF; }
  .interstellar { border-color:rgba(58,190,255,0.25); }
  .terrain .counter-value { color:#10B981; }
  .terrain .counter-label { color:#10B981; }
  .terrain { border-color:rgba(16,185,129,0.25); }
  .total .counter-value { color:#C084FC; }
  .total .counter-label { color:#C084FC; }
  .total { border-color:rgba(139,92,246,0.25); }

  /* Card */
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem; }

  /* Sort controls */
  .sort-controls { display:flex;gap:0.5rem;align-items:center;margin-bottom:0.75rem; }
  .sort-label { color:#94A3B8;font-size:0.75rem; }
  .sort-controls button { background:none;border:1px solid rgba(58,190,255,0.1);color:#94A3B8;padding:0.25rem 0.6rem;border-radius:4px;font-size:0.75rem;cursor:pointer; }
  .sort-controls button:hover { color:#E6EDF3;border-color:rgba(58,190,255,0.3); }
  .sort-controls .sort-active { color:#3ABEFF;border-color:#3ABEFF; }

  /* Journal list */
  .mission-list { display:flex;flex-direction:column;gap:0.2rem; }
  .journal-item { display:flex;justify-content:space-between;align-items:center;padding:0.5rem 0.4rem;border-bottom:1px solid rgba(255,255,255,0.03);flex-wrap:wrap;gap:0.4rem; }
  .journal-item:last-child { border-bottom:none; }
  .journal-main { display:flex;flex-direction:column;gap:0.15rem; }
  .journal-title { font-weight:500;font-size:0.85rem; }
  .journal-location { color:#94A3B8;font-size:0.75rem; }
  .journal-meta { display:flex;gap:0.4rem;align-items:center; }
  .journal-date { color:#94A3B8;font-size:0.7rem; }

  .badge { display:inline-block;padding:0.12rem 0.4rem;border-radius:4px;font-size:0.65rem;font-weight:600;text-transform:capitalize; }
  .type-badge { background:rgba(139,92,246,0.15);color:#C084FC; }
  .danger-low { background:rgba(16,185,129,0.15);color:#10B981; }
  .danger-medium { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .danger-high { background:rgba(249,115,22,0.15);color:#F97316; }
  .danger-critical { background:rgba(239,68,68,0.2);color:#EF4444; }
</style>
