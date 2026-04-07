<!-- /astronauts/colleagues — View all astronauts' mission counters -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getAllAstronautCounters, type AstroCounterItem } from '$lib/stores/astronauts';

  let counters: AstroCounterItem[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let search = $state('');

  onMount(async () => {
    try {
      counters = await getAllAstronautCounters();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  const filtered = $derived(
    search.trim()
      ? counters.filter((c) =>
          c.full_name.toLowerCase().includes(search.trim().toLowerCase())
        )
      : counters
  );
</script>

<h1 class="title">Colleagues' Missions</h1>
<p class="subtitle">Mission counters for all astronauts.</p>

<div class="search-row">
  <input
    class="search-input"
    type="text"
    placeholder="Filter by name…"
    bind:value={search}
  />
</div>

{#if loading}
  <p class="loading">Loading counters…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if filtered.length === 0}
  <div class="empty-state">
    <p>{search ? 'No astronauts match your search.' : 'No astronauts found.'}</p>
  </div>
{:else}
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>Name</th>
          <th>Interstellar</th>
          <th>Terrain</th>
          <th>Total</th>
        </tr>
      </thead>
      <tbody>
        {#each filtered as c}
          <tr>
            <td class="cell-name">{c.full_name}</td>
            <td><span class="count interstellar">{c.interstellar_count}</span></td>
            <td><span class="count terrain">{c.terrain_count}</span></td>
            <td><span class="count total">{c.interstellar_count + c.terrain_count}</span></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .loading { color:#94A3B8;font-size:0.85rem; }
  .error { color:#EF4444;font-size:0.85rem; }
  .empty-state { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:2rem;text-align:center;color:#475569;font-size:0.85rem; }

  .search-row { margin-bottom:1rem; }
  .search-input { background:#111827;border:1px solid rgba(58,190,255,0.2);border-radius:6px;color:#E6EDF3;font-size:0.8rem;padding:0.4rem 0.75rem;width:260px;outline:none; }
  .search-input:focus { border-color:#3ABEFF; }
  .search-input::placeholder { color:#475569; }

  .table-wrap { overflow-x:auto; }
  table { width:100%;border-collapse:collapse;font-size:0.8rem; }
  thead th { text-align:left;color:#94A3B8;font-weight:600;font-size:0.7rem;text-transform:uppercase;padding:0.5rem 0.75rem;border-bottom:1px solid rgba(58,190,255,0.1); }
  tbody tr { border-bottom:1px solid rgba(255,255,255,0.03); }
  tbody tr:hover { background:rgba(58,190,255,0.03); }
  td { padding:0.55rem 0.75rem;vertical-align:middle; }
  .cell-name { font-weight:500;color:#E6EDF3; }

  .count { display:inline-block;padding:0.15rem 0.6rem;border-radius:4px;font-size:0.75rem;font-weight:600;min-width:2rem;text-align:center; }
  .interstellar { color:#3ABEFF;background:rgba(58,190,255,0.12); }
  .terrain { color:#10B981;background:rgba(16,185,129,0.12); }
  .total { color:#C084FC;background:rgba(139,92,246,0.12); }
</style>
