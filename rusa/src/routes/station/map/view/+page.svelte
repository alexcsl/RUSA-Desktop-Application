<!-- /station/map/view — View all uploaded station maps (published + draft) -->
<script lang="ts">
  import { getContext, onMount } from 'svelte';
  import type { Writable } from 'svelte/store';
  import {
    sstGetMaps, sstGetMapWithUrl,
    type StationMapSummary, type PublishedMapData,
  } from '$lib/stores/space_station';

  const stationId = getContext<Writable<string>>('selectedStationId');

  let maps: StationMapSummary[] = $state([]);
  let selectedMapId: string | null = $state(null);
  let mapData: PublishedMapData | null = $state(null);
  let loading = $state(true);
  let loadingImage = $state(false);
  let error = $state('');
  let expandedAnnotation: string | null = $state(null);

  let currentStationId = $state('');
  stationId.subscribe((v) => {
    currentStationId = v;
    if (v) loadMaps();
  });

  async function loadMaps() {
    if (!currentStationId) return;
    loading = true;
    error = '';
    try {
      maps = await sstGetMaps(currentStationId);
      if (maps.length > 0) {
        await selectMap(maps[0].id);
      }
    } catch (e: any) {
      error = e?.message ?? String(e);
    } finally {
      loading = false;
    }
  }

  async function selectMap(id: string) {
    selectedMapId = id;
    mapData = null;
    expandedAnnotation = null;
    loadingImage = true;
    try {
      mapData = await sstGetMapWithUrl(id);
    } catch (e: any) {
      error = `Could not load map image: ${e?.message ?? String(e)}`;
    } finally {
      loadingImage = false;
    }
  }

  function toggleAnnotation(id: string) {
    expandedAnnotation = expandedAnnotation === id ? null : id;
  }

  function mapFileName(path: string): string {
    return path.split('/').pop() ?? path;
  }

  onMount(() => { if (currentStationId) loadMaps(); });
</script>

<h1 class="title">Station Maps</h1>
<p class="subtitle">All uploaded maps for this station — select one to view.</p>

{#if loading}
  <p class="muted">Loading maps…</p>
{:else if error && maps.length === 0}
  <p class="err">{error}</p>
{:else if maps.length === 0}
  <div class="empty">
    <span class="empty-icon">🗺️</span>
    <p>No maps have been uploaded yet. Use the Map Editor to upload one.</p>
  </div>
{:else}
  <div class="viewer-layout">
    <!-- Sidebar: map list -->
    <div class="map-list">
      {#each maps as m}
        <button
          class="map-item"
          class:selected={selectedMapId === m.id}
          onclick={() => selectMap(m.id)}
        >
          <span class="map-name">{mapFileName(m.image_storage_path)}</span>
          <div class="map-meta">
            {#if m.is_published}
              <span class="tag tag-pub">Published</span>
            {:else}
              <span class="tag tag-draft">Draft</span>
            {/if}
            <span class="map-date">{new Date(m.created_at).toLocaleDateString()}</span>
          </div>
        </button>
      {/each}
    </div>

    <!-- Main: image + annotations -->
    <div class="map-view">
      {#if loadingImage}
        <p class="muted">Loading image…</p>
      {:else if error}
        <p class="err">{error}</p>
      {:else if mapData}
        <div class="map-header">
          <h2 class="map-title">{mapData.station_name}</h2>
          {#if mapData.image_width && mapData.image_height}
            <span class="dim-badge">{mapData.image_width} × {mapData.image_height} px</span>
          {/if}
        </div>

        <div class="map-container">
          <div class="map-image-wrapper">
            <img
              src={mapData.signed_url}
              alt="Station Map"
              class="map-image"
              style={mapData.image_width && mapData.image_height
                ? `aspect-ratio:${mapData.image_width}/${mapData.image_height}`
                : ''}
            />
            {#each mapData.annotations as a}
              <button
                class="anno-marker"
                class:active={expandedAnnotation === a.id}
                style="left:{a.x_position}%;top:{a.y_position}%"
                onclick={() => toggleAnnotation(a.id)}
                title={a.label}
              >📌</button>
              {#if expandedAnnotation === a.id}
                <div class="anno-popup" style="left:{Math.min(a.x_position + 2, 70)}%;top:{a.y_position + 3}%">
                  <strong>{a.label}</strong>
                  {#if a.description}<p>{a.description}</p>{/if}
                </div>
              {/if}
            {/each}
          </div>
        </div>

        {#if mapData.annotations.length > 0}
          <div class="anno-legend">
            <h3>Annotations ({mapData.annotations.length})</h3>
            <div class="anno-grid">
              {#each mapData.annotations as a}
                <button
                  class="anno-card"
                  class:highlighted={expandedAnnotation === a.id}
                  onclick={() => toggleAnnotation(a.id)}
                >
                  <span class="anno-label">📌 {a.label}</span>
                  <span class="anno-coords">({a.x_position.toFixed(1)}, {a.y_position.toFixed(1)})</span>
                  {#if a.description}<p class="anno-desc">{a.description}</p>{/if}
                </button>
              {/each}
            </div>
          </div>
        {:else}
          <p class="muted no-anno">No annotations on this map.</p>
        {/if}
      {:else}
        <p class="muted">Select a map from the list.</p>
      {/if}
    </div>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#64748B;font-size:0.75rem;margin:0 0 1rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .err { color:#EF4444;font-size:0.8rem; }
  .empty { display:flex;flex-direction:column;align-items:center;gap:0.5rem;margin-top:4rem; }
  .empty-icon { font-size:3rem;opacity:0.35; }
  .empty p { color:#94A3B8;font-size:0.9rem; }

  .viewer-layout { display:flex;gap:1rem;height:calc(100vh - 120px);overflow:hidden; }

  /* Map list sidebar */
  .map-list { width:240px;min-width:180px;display:flex;flex-direction:column;gap:0.3rem;overflow-y:auto; }
  .map-item { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.55rem 0.65rem;cursor:pointer;text-align:left;display:flex;flex-direction:column;gap:0.25rem;transition:border-color 0.15s; }
  .map-item:hover { border-color:rgba(58,190,255,0.2); }
  .map-item.selected { border-color:#3ABEFF;background:rgba(58,190,255,0.07); }
  .map-name { font-size:0.78rem;color:#E6EDF3;word-break:break-all;line-height:1.3; }
  .map-meta { display:flex;justify-content:space-between;align-items:center;gap:0.3rem; }
  .tag { font-size:0.63rem;padding:0.05rem 0.3rem;border-radius:3px; }
  .tag-pub { background:rgba(16,185,129,0.15);color:#10B981; }
  .tag-draft { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .map-date { font-size:0.63rem;color:#64748B; }

  /* Map view area */
  .map-view { flex:1;overflow-y:auto;display:flex;flex-direction:column;gap:0.75rem; }
  .map-header { display:flex;align-items:baseline;gap:0.75rem; }
  .map-title { font-family:'Orbitron',sans-serif;font-size:0.95rem;color:#8B5CF6;margin:0; }
  .dim-badge { font-size:0.7rem;color:#64748B;font-family:monospace; }

  .map-container { position:relative;max-width:900px;width:100%;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;overflow:hidden; }
  .map-image-wrapper { position:relative; }
  .map-image { width:100%;display:block;border-radius:8px; }

  .anno-marker { position:absolute;transform:translate(-50%,-50%);background:rgba(58,190,255,0.2);border:1px solid #3ABEFF;border-radius:50%;width:28px;height:28px;display:flex;align-items:center;justify-content:center;cursor:pointer;font-size:0.8rem;transition:all 0.15s;z-index:10; }
  .anno-marker:hover,.anno-marker.active { background:rgba(58,190,255,0.4);transform:translate(-50%,-50%) scale(1.15); }
  .anno-popup { position:absolute;z-index:20;background:#1F2937;border:1px solid rgba(58,190,255,0.3);border-radius:8px;padding:0.6rem;max-width:220px;box-shadow:0 4px 16px rgba(0,0,0,0.4);animation:fadeIn 0.15s ease; }
  .anno-popup strong { color:#3ABEFF;font-size:0.85rem; }
  .anno-popup p { color:#94A3B8;font-size:0.75rem;margin:0.2rem 0 0; }

  .anno-legend { max-width:900px; }
  .anno-legend h3 { font-family:'Orbitron',sans-serif;font-size:0.8rem;color:#8B5CF6;margin:0 0 0.5rem; }
  .anno-grid { display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:0.4rem; }
  .anno-card { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.55rem;cursor:pointer;text-align:left;display:flex;flex-direction:column;gap:0.15rem;transition:border-color 0.15s; }
  .anno-card:hover,.anno-card.highlighted { border-color:#3ABEFF; }
  .anno-label { font-size:0.8rem;color:#E6EDF3; }
  .anno-coords { font-size:0.65rem;color:#64748B;font-family:monospace; }
  .anno-desc { font-size:0.75rem;color:#94A3B8;margin:0; }
  .no-anno { margin-top:0.5rem; }

  @keyframes fadeIn { from{opacity:0}to{opacity:1} }
</style>
