<!--
  /directors/wanderer/territories — Territory Renaming (UC-WAN-04)
  TheWanderer views and renames territories, split by type (Sector / Planet).
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getTerritories, renameTerritory, type TerritorySummary } from '$lib/stores/directors';

  type TabType = 'sector' | 'planet' | 'all';
  let activeTab = $state<TabType>('sector');

  let territories: TerritorySummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let renaming: string | null = $state(null);
  let newName = $state('');
  let actionMsg = $state('');

  onMount(async () => { await loadTerritories(); });

  async function loadTerritories() {
    loading = true; error = '';
    try { territories = await getTerritories(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  }

  let filtered = $derived(
    activeTab === 'all'
      ? territories
      : territories.filter((t) => t.territory_type.toLowerCase() === activeTab)
  );

  let sectors = $derived(territories.filter((t) => t.territory_type.toLowerCase() === 'sector'));
  let planets = $derived(territories.filter((t) => t.territory_type.toLowerCase() === 'planet'));

  async function handleRename(id: string) {
    actionMsg = '';
    if (!newName.trim()) { actionMsg = 'New name cannot be empty.'; return; }
    try {
      await renameTerritory({ territory_id: id, new_name: newName.trim() });
      actionMsg = 'Territory renamed.';
      renaming = null; newName = '';
      await loadTerritories();
    } catch (e: unknown) {
      actionMsg = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<h1 class="title">Territories</h1>
<p class="subtitle">View and rename sectors and planets under Wanderer oversight.</p>

{#if loading}<p class="muted">Loading…</p>
{:else if error}<p class="error">{error}</p>
{:else}
  {#if actionMsg}<p class="action-msg">{actionMsg}</p>{/if}

  <div class="tabs">
    <button class="tab" class:active={activeTab === 'sector'} onclick={() => { activeTab = 'sector'; renaming = null; }}>
      Sectors <span class="count">{sectors.length}</span>
    </button>
    <button class="tab" class:active={activeTab === 'planet'} onclick={() => { activeTab = 'planet'; renaming = null; }}>
      Planets <span class="count">{planets.length}</span>
    </button>
    <button class="tab" class:active={activeTab === 'all'} onclick={() => { activeTab = 'all'; renaming = null; }}>
      All <span class="count">{territories.length}</span>
    </button>
  </div>

  {#if filtered.length === 0}
    <p class="muted">No {activeTab === 'all' ? 'territories' : activeTab + 's'} found.</p>
  {:else}
    <div class="list">
      {#each filtered as t}
        <div class="card">
          <div class="card-header">
            <div class="card-info">
              <span class="territory-name">{t.name}</span>
              <span class="type-tag type-{t.territory_type.toLowerCase()}">{t.territory_type}</span>
              {#if t.previous_name}
                <span class="prev-name">formerly: {t.previous_name}</span>
              {/if}
            </div>
            {#if renaming === t.id}
              <div class="rename-row">
                <input class="input-sm" type="text" bind:value={newName} placeholder="New name…" />
                <button class="btn-sm" onclick={() => handleRename(t.id)}>Save</button>
                <button class="btn-cancel" onclick={() => { renaming = null; newName = ''; }}>✕</button>
              </div>
            {:else}
              <button class="btn-rename" onclick={() => { renaming = t.id; newName = t.name; }}>Rename</button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .action-msg { font-size:0.78rem;color:#22C55E;margin-bottom:0.5rem; }
  .tabs { display:flex;gap:0.4rem;margin-bottom:1rem; }
  .tab { padding:0.3rem 0.65rem;border:1px solid #374151;background:#1F2937;color:#94A3B8;border-radius:4px;cursor:pointer;font-size:0.76rem;display:flex;align-items:center;gap:0.35rem; }
  .tab.active { border-color:#3ABEFF;color:#3ABEFF;background:rgba(58,190,255,0.1); }
  .count { background:rgba(255,255,255,0.08);border-radius:3px;padding:0 0.3rem;font-size:0.65rem; }
  .list { display:flex;flex-direction:column;gap:0.5rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.9rem; }
  .card-header { display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:0.5rem; }
  .card-info { display:flex;align-items:center;gap:0.5rem;flex-wrap:wrap; }
  .territory-name { font-size:0.9rem;font-weight:600;color:#E6EDF3; }
  .type-tag { font-size:0.65rem;border-radius:3px;padding:0.1rem 0.35rem;font-weight:600; }
  .type-sector { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .type-planet { background:rgba(139,92,246,0.15);color:#C084FC; }
  .prev-name { font-size:0.7rem;color:#475569;font-style:italic; }
  .rename-row { display:flex;align-items:center;gap:0.35rem; }
  .input-sm { background:#0E1428;border:1px solid rgba(58,190,255,0.2);color:#E6EDF3;border-radius:5px;padding:0.3rem 0.45rem;font-size:0.78rem;font-family:'Inter',sans-serif;width:150px; }
  .input-sm:focus { outline:none;border-color:#3ABEFF; }
  .btn-sm { background:rgba(58,190,255,0.12);border:1px solid rgba(58,190,255,0.35);color:#3ABEFF;border-radius:5px;padding:0.28rem 0.6rem;cursor:pointer;font-size:0.75rem; }
  .btn-rename { background:transparent;border:1px solid rgba(58,190,255,0.2);color:#3ABEFF;border-radius:5px;padding:0.25rem 0.55rem;cursor:pointer;font-size:0.73rem; }
  .btn-rename:hover { background:rgba(58,190,255,0.08); }
  .btn-cancel { background:transparent;border:none;color:#94A3B8;cursor:pointer;font-size:0.8rem;padding:0.2rem 0.3rem; }
  .error { color:#EF4444;font-size:0.75rem; }
</style>
