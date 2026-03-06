<!-- /station/map — UC-SSV-01: View Interactive Station Map (Visitor, no auth) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { sstGetPublishedMap, sstGetPublicStations, type PublishedMapData, type StationSummary } from '$lib/stores/space_station';

  let stations: StationSummary[] = $state([]);
  let selectedStationId = $state('');
  let mapData: PublishedMapData | null = $state(null);
  let loading = $state(true);
  let loadingMap = $state(false);
  let error = $state('');
  let expandedAnnotation: string | null = $state(null);

  onMount(async () => {
    try {
      stations = await sstGetPublicStations();
      if (stations.length > 0) {
        selectedStationId = stations[0].id;
        await loadMap();
      }
    } catch {}
    loading = false;
  });

  async function loadMap() {
    if (!selectedStationId) return;
    loadingMap = true;
    error = '';
    mapData = null;
    try {
      mapData = await sstGetPublishedMap(selectedStationId);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
    loadingMap = false;
  }

  async function handleStationChange() {
    await loadMap();
  }

  function toggleAnnotation(id: string) {
    expandedAnnotation = expandedAnnotation === id ? null : id;
  }
</script>

<div class="visitor-shell">
  <header class="visitor-header">
    <div class="brand">
      <img src="/Logo.png" alt="RUSA" class="logo" />
      <h1>Station Map — Visitor View</h1>
    </div>
    {#if stations.length > 1}
      <select class="station-select" bind:value={selectedStationId} onchange={handleStationChange}>
        {#each stations as st}
          <option value={st.id}>{st.name} — {st.sector}</option>
        {/each}
      </select>
    {:else if stations.length === 1}
      <span class="station-name">{stations[0].name}</span>
    {/if}
  </header>

  <main class="visitor-main">
    {#if loading}
      <p class="muted">Loading stations…</p>
    {:else if loadingMap}
      <p class="muted">Loading map…</p>
    {:else if error}
      <div class="no-map">
        <span class="no-map-icon">🗺️</span>
        <p>{error}</p>
      </div>
    {:else if mapData}
      <h2 class="map-title">{mapData.station_name}</h2>

      <div class="map-container">
        <div class="map-image-wrapper">
          <img
            src={mapData.signed_url}
            alt="Station Map"
            class="map-image"
            style={mapData.image_width && mapData.image_height ? `aspect-ratio:${mapData.image_width}/${mapData.image_height}` : ''}
          />
          <!-- Annotation markers overlaid on the map image -->
          {#each mapData.annotations as a}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <button
              class="anno-marker"
              class:active={expandedAnnotation === a.id}
              style="left:{a.x_position}%;top:{a.y_position}%"
              onclick={() => toggleAnnotation(a.id)}
              title={a.label}
            >
              📌
            </button>
            {#if expandedAnnotation === a.id}
              <div class="anno-popup" style="left:{Math.min(a.x_position + 2, 70)}%;top:{a.y_position + 3}%">
                <strong>{a.label}</strong>
                {#if a.description}<p>{a.description}</p>{/if}
              </div>
            {/if}
          {/each}
        </div>
      </div>

      <!-- Annotation list below the map -->
      {#if mapData.annotations.length > 0}
        <div class="anno-legend">
          <h3>Annotations</h3>
          <div class="anno-grid">
            {#each mapData.annotations as a}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <div class="anno-card" class:highlighted={expandedAnnotation === a.id} role="button" tabindex="0" onclick={() => toggleAnnotation(a.id)}>
                <span class="anno-label">📌 {a.label}</span>
                {#if a.description}<p class="anno-desc">{a.description}</p>{/if}
              </div>
            {/each}
          </div>
        </div>
      {/if}
    {:else}
      <div class="no-map">
        <span class="no-map-icon">🗺️</span>
        <p>No published map available for this station.</p>
      </div>
    {/if}
  </main>
</div>

<style>
  .visitor-shell { display:flex;flex-direction:column;height:100vh;background:#0B0F1A;color:#E6EDF3;font-family:'Inter',sans-serif; }
  .visitor-header { display:flex;justify-content:space-between;align-items:center;padding:0.75rem 1.25rem;background:#111827;border-bottom:1px solid rgba(58,190,255,0.15); }
  .brand { display:flex;align-items:center;gap:0.6rem; }
  .brand h1 { font-family:'Orbitron',sans-serif;font-size:1rem;color:#3ABEFF;margin:0; }
  .logo { width:28px;height:28px; }
  .station-select { background:#0E1428;border:1px solid rgba(58,190,255,0.2);color:#E6EDF3;border-radius:4px;padding:0.3rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .station-name { font-size:0.85rem;color:#3ABEFF;font-weight:500; }

  .visitor-main { flex:1;overflow-y:auto;padding:1.25rem;display:flex;flex-direction:column;align-items:center; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .map-title { font-family:'Orbitron',sans-serif;font-size:1rem;color:#8B5CF6;margin:0 0 1rem;text-align:center; }

  .map-container { position:relative;max-width:900px;width:100%;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;overflow:hidden; }
  .map-image-wrapper { position:relative; }
  .map-image { width:100%;display:block;border-radius:8px; }

  .anno-marker { position:absolute;transform:translate(-50%,-50%);background:rgba(58,190,255,0.2);border:1px solid #3ABEFF;border-radius:50%;width:28px;height:28px;display:flex;align-items:center;justify-content:center;cursor:pointer;font-size:0.8rem;transition:all 0.15s;z-index:10; }
  .anno-marker:hover,.anno-marker.active { background:rgba(58,190,255,0.4);transform:translate(-50%,-50%) scale(1.15); }
  .anno-popup { position:absolute;z-index:20;background:#1F2937;border:1px solid rgba(58,190,255,0.3);border-radius:8px;padding:0.6rem;max-width:220px;box-shadow:0 4px 16px rgba(0,0,0,0.4);animation:fadeIn 0.15s ease; }
  .anno-popup strong { color:#3ABEFF;font-size:0.85rem; }
  .anno-popup p { color:#94A3B8;font-size:0.75rem;margin:0.2rem 0 0; }

  .no-map { display:flex;flex-direction:column;align-items:center;gap:0.5rem;margin-top:4rem; }
  .no-map-icon { font-size:3rem;opacity:0.4; }
  .no-map p { color:#94A3B8;font-size:0.9rem; }

  .anno-legend { margin-top:1rem;max-width:900px;width:100%; }
  .anno-legend h3 { font-family:'Orbitron',sans-serif;font-size:0.85rem;color:#8B5CF6;margin:0 0 0.5rem; }
  .anno-grid { display:grid;grid-template-columns:repeat(auto-fill, minmax(200px, 1fr));gap:0.5rem; }
  .anno-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.6rem;cursor:pointer;transition:border-color 0.15s; }
  .anno-card:hover,.anno-card.highlighted { border-color:#3ABEFF; }
  .anno-label { font-size:0.8rem;color:#E6EDF3; }
  .anno-desc { font-size:0.75rem;color:#94A3B8;margin:0.2rem 0 0; }

  @keyframes fadeIn { from{opacity:0}to{opacity:1} }
</style>
