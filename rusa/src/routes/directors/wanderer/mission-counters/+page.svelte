<!--
  /directors/wanderer/mission-counters — Astronaut mission counters view
  TheWanderer views counters for all astronauts to prioritize mission assignments.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getAllAstronautCounters, type AstroCounterItem } from '$lib/stores/astronauts';

  let counters: AstroCounterItem[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try { counters = await getAllAstronautCounters(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  });

  let sorted = $derived(
    [...counters].sort(
      (a, b) => (b.interstellar_count + b.terrain_count) - (a.interstellar_count + a.terrain_count)
    )
  );
</script>

<h1 class="title">Mission Counters</h1>
<p class="subtitle">Astronaut mission counts for prioritizing assignments.</p>
<p class="note">Higher terrain count = prioritize for terrain missions; Higher galactic count = prioritize for interstellar missions.</p>

{#if loading}<p class="muted">Loading…</p>
{:else if error}<p class="error">{error}</p>
{:else if sorted.length === 0}<p class="muted">No astronaut counters found.</p>
{:else}
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>Astronaut</th>
          <th class="num">Interstellar</th>
          <th class="num">Terrain</th>
          <th class="num">Total</th>
        </tr>
      </thead>
      <tbody>
        {#each sorted as item}
          {@const total = item.interstellar_count + item.terrain_count}
          <tr>
            <td class="name">{item.full_name}</td>
            <td class="num">
              <span class="count blue">{item.interstellar_count}</span>
            </td>
            <td class="num">
              <span class="count purple">{item.terrain_count}</span>
            </td>
            <td class="num">
              <span class="count total">{total}</span>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.4rem; }
  .note { font-size:0.75rem;color:#64748B;font-style:italic;margin:0 0 1rem;padding:0.5rem;background:rgba(58,190,255,0.05);border-left:2px solid rgba(58,190,255,0.3);border-radius:0 4px 4px 0; }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .error { color:#EF4444;font-size:0.75rem; }
  .table-wrap { overflow-x:auto; }
  table { width:100%;border-collapse:collapse;font-size:0.82rem; }
  thead tr { border-bottom:1px solid rgba(58,190,255,0.15); }
  th { color:#64748B;font-weight:600;font-size:0.72rem;padding:0.5rem 0.75rem;text-align:left;text-transform:uppercase;letter-spacing:0.04em; }
  th.num { text-align:center; }
  td { padding:0.5rem 0.75rem;border-bottom:1px solid rgba(255,255,255,0.04);color:#E6EDF3; }
  td.num { text-align:center; }
  td.name { font-weight:500; }
  .count { display:inline-block;padding:0.15rem 0.55rem;border-radius:4px;font-weight:600;font-size:0.8rem; }
  .count.blue { color:#3ABEFF;background:rgba(58,190,255,0.1); }
  .count.purple { color:#8B5CF6;background:rgba(139,92,246,0.1); }
  .count.total { color:#E6EDF3;background:rgba(255,255,255,0.07); }
  tbody tr:hover { background:rgba(58,190,255,0.03); }
</style>
