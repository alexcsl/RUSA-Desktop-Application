<!-- /station/map/edit — UC-SSS-05: Upload and Annotate Station Map -->
<script lang="ts">
  import { getContext, onMount } from 'svelte';
  import type { Writable } from 'svelte/store';
  import {
    sstUploadMap, sstGetMaps, sstPublishMap,
    sstAddAnnotation, sstDeleteAnnotation, sstGetAnnotations,
    type StationMapSummary, type MapAnnotation,
  } from '$lib/stores/space_station';

  const stationId = getContext<Writable<string>>('selectedStationId');

  let maps: StationMapSummary[] = $state([]);
  let selectedMapId: string | null = $state(null);
  let annotations: MapAnnotation[] = $state([]);
  let loading = $state(true);

  // Upload form
  let fileInput: HTMLInputElement | null = $state(null);
  let uploadError = $state('');
  let uploadSuccess = $state('');
  let uploading = $state(false);

  // Annotation form
  let annoLabel = $state('');
  let annoDescription = $state('');
  let annoX = $state(0);
  let annoY = $state(0);
  let annoError = $state('');
  let addingAnno = $state(false);

  // Publishing
  let publishing = $state(false);

  let currentStationId = $state('');
  const unsub = stationId.subscribe((v) => {
    currentStationId = v;
    if (v) loadMaps();
  });

  async function loadMaps() {
    if (!currentStationId) return;
    loading = true;
    try {
      maps = await sstGetMaps(currentStationId);
      if (maps.length > 0 && !selectedMapId) {
        selectedMapId = maps[0].id;
        await loadAnnotations();
      }
    } catch {}
    loading = false;
  }

  async function loadAnnotations() {
    if (!selectedMapId) return;
    try {
      annotations = await sstGetAnnotations(selectedMapId);
    } catch {}
  }

  onMount(() => { if (currentStationId) loadMaps(); });

  async function handleSelectMap(mapId: string) {
    selectedMapId = mapId;
    await loadAnnotations();
  }

  async function handleUpload() {
    uploadError = ''; uploadSuccess = '';
    if (!currentStationId) { uploadError = 'No station selected.'; return; }
    if (!fileInput?.files?.length) { uploadError = 'Please select an image file.'; return; }

    const file = fileInput.files[0];
    if (!file.type.startsWith('image/')) { uploadError = 'Only image files are allowed.'; return; }

    uploading = true;
    try {
      const arrayBuf = await file.arrayBuffer();
      const bytes = Array.from(new Uint8Array(arrayBuf));

      // Try to get image dimensions
      const img = new Image();
      const dimPromise = new Promise<{ w: number; h: number }>((resolve) => {
        img.onload = () => resolve({ w: img.naturalWidth, h: img.naturalHeight });
        img.onerror = () => resolve({ w: 0, h: 0 });
        img.src = URL.createObjectURL(file);
      });
      const dims = await dimPromise;

      const mapId = await sstUploadMap(
        currentStationId,
        bytes,
        file.name,
        file.type,
        dims.w > 0 ? dims.w : undefined,
        dims.h > 0 ? dims.h : undefined,
      );

      uploadSuccess = 'Map uploaded successfully.';
      selectedMapId = mapId;
      await loadMaps();
      await loadAnnotations();
    } catch (e: unknown) {
      uploadError = e instanceof Error ? e.message : String(e);
    } finally {
      uploading = false;
    }
  }

  async function handlePublish() {
    if (!selectedMapId) return;
    publishing = true;
    try {
      await sstPublishMap(selectedMapId);
      await loadMaps();
    } catch {}
    publishing = false;
  }

  async function handleAddAnnotation() {
    annoError = '';
    if (!selectedMapId) return;
    if (!annoLabel.trim()) { annoError = 'Label is required.'; return; }

    addingAnno = true;
    try {
      await sstAddAnnotation({
        map_id: selectedMapId,
        label: annoLabel,
        description: annoDescription || undefined,
        x_position: annoX,
        y_position: annoY,
      });
      annoLabel = ''; annoDescription = ''; annoX = 0; annoY = 0;
      await loadAnnotations();
    } catch (e: unknown) {
      annoError = e instanceof Error ? e.message : String(e);
    } finally {
      addingAnno = false;
    }
  }

  async function handleDeleteAnnotation(id: string) {
    try {
      await sstDeleteAnnotation(id);
      await loadAnnotations();
    } catch {}
  }

  let selectedMap = $derived(maps.find((m) => m.id === selectedMapId));
</script>

<h1 class="title">Station Map Editor</h1>
<p class="subtitle">Upload station maps, annotate regions, and publish for visitors.</p>

<div class="editor-layout">
  <!-- Left: Map list + upload -->
  <div class="panel left-panel">
    <h2>Maps</h2>

    <div class="upload-section">
      <input type="file" accept="image/*" bind:this={fileInput} class="file-input" />
      <button class="btn-upload" onclick={handleUpload} disabled={uploading}>
        {uploading ? 'Uploading…' : '⬆ Upload Map'}
      </button>
      {#if uploadError}<p class="error">{uploadError}</p>{/if}
      {#if uploadSuccess}<p class="success">{uploadSuccess}</p>{/if}
    </div>

    {#if loading}
      <p class="muted">Loading maps…</p>
    {:else if maps.length === 0}
      <p class="muted">No maps uploaded yet.</p>
    {:else}
      <div class="map-list">
        {#each maps as m}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <div class="map-item" class:selected={selectedMapId === m.id} role="button" tabindex="0" onclick={() => handleSelectMap(m.id)}>
            <span class="map-name">{m.image_storage_path.split('/').pop()}</span>
            <div class="map-meta">
              {#if m.is_published}
                <span class="published-tag">Published</span>
              {:else}
                <span class="draft-tag">Draft</span>
              {/if}
              <span class="map-date">{new Date(m.created_at).toLocaleDateString()}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Right: Annotations -->
  <div class="panel right-panel">
    {#if selectedMap}
      <div class="map-actions">
        <h2>Annotations — {selectedMap.image_storage_path.split('/').pop()}</h2>
        {#if !selectedMap.is_published}
          <button class="btn-publish" onclick={handlePublish} disabled={publishing}>
            {publishing ? '…' : '🌐 Publish'}
          </button>
        {:else}
          <span class="published-badge">✓ Published</span>
        {/if}
      </div>

      {#if selectedMap.image_width && selectedMap.image_height}
        <p class="muted">Dimensions: {selectedMap.image_width} × {selectedMap.image_height} px</p>
      {/if}

      <!-- Add annotation form -->
      <div class="anno-form">
        <div class="anno-row">
          <label class="field flex2"><span class="label">Label *</span>
            <input type="text" class="input" bind:value={annoLabel} placeholder="e.g. Engine Room" />
          </label>
          <label class="field flex1"><span class="label">X</span>
            <input type="number" class="input" bind:value={annoX} step="0.1" />
          </label>
          <label class="field flex1"><span class="label">Y</span>
            <input type="number" class="input" bind:value={annoY} step="0.1" />
          </label>
        </div>
        <label class="field"><span class="label">Description</span>
          <input type="text" class="input" bind:value={annoDescription} placeholder="Shown on click…" />
        </label>
        {#if annoError}<p class="error">{annoError}</p>{/if}
        <button class="btn-primary" onclick={handleAddAnnotation} disabled={addingAnno}>
          {addingAnno ? '…' : '+ Add Annotation'}
        </button>
      </div>

      <!-- Annotation list -->
      {#if annotations.length === 0}
        <p class="muted">No annotations yet.</p>
      {:else}
        <div class="anno-list">
          {#each annotations as a}
            <div class="anno-card">
              <div class="anno-header">
                <span class="anno-label">📌 {a.label}</span>
                <span class="anno-coords">({a.x_position.toFixed(1)}, {a.y_position.toFixed(1)})</span>
              </div>
              {#if a.description}
                <p class="anno-desc">{a.description}</p>
              {/if}
              <button class="btn-delete" onclick={() => handleDeleteAnnotation(a.id)}>Delete</button>
            </div>
          {/each}
        </div>
      {/if}
    {:else}
      <p class="muted">Select or upload a map to manage annotations.</p>
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .editor-layout { display:flex;gap:1rem;flex-wrap:wrap; }
  .panel { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem; }
  .left-panel { min-width:260px;max-width:320px; }
  .right-panel { flex:1;min-width:350px; }
  .panel h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0 0 0.6rem; }
  .upload-section { display:flex;flex-direction:column;gap:0.4rem;margin-bottom:0.75rem; }
  .file-input { font-size:0.75rem;color:#94A3B8; }
  .btn-upload { background:rgba(58,190,255,0.15);border:1px solid rgba(58,190,255,0.25);color:#3ABEFF;border-radius:6px;padding:0.35rem 0.75rem;font-size:0.8rem;cursor:pointer; }
  .btn-upload:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-upload:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .map-list { display:flex;flex-direction:column;gap:0.3rem; }
  .map-item { padding:0.5rem;border:1px solid rgba(58,190,255,0.08);border-radius:6px;cursor:pointer;transition:border-color 0.15s; }
  .map-item:hover { border-color:rgba(58,190,255,0.2); }
  .map-item.selected { border-color:#3ABEFF;background:rgba(58,190,255,0.05); }
  .map-name { font-size:0.8rem;color:#E6EDF3;word-break:break-all; }
  .map-meta { display:flex;justify-content:space-between;margin-top:0.2rem; }
  .published-tag { font-size:0.65rem;color:#10B981;background:rgba(16,185,129,0.15);padding:0.05rem 0.3rem;border-radius:3px; }
  .draft-tag { font-size:0.65rem;color:#F59E0B;background:rgba(245,158,11,0.15);padding:0.05rem 0.3rem;border-radius:3px; }
  .map-date { font-size:0.65rem;color:#94A3B8; }
  .map-actions { display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem; }
  .btn-publish { background:rgba(16,185,129,0.15);border:1px solid rgba(16,185,129,0.3);color:#10B981;border-radius:6px;padding:0.3rem 0.6rem;font-size:0.75rem;cursor:pointer; }
  .btn-publish:disabled { opacity:0.5; }
  .btn-publish:hover:not(:disabled) { background:rgba(16,185,129,0.25); }
  .published-badge { font-size:0.75rem;color:#10B981; }
  .anno-form { background:rgba(14,20,40,0.4);border:1px solid rgba(58,190,255,0.06);border-radius:6px;padding:0.75rem;display:flex;flex-direction:column;gap:0.4rem;margin-bottom:0.75rem; }
  .anno-row { display:flex;gap:0.5rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .flex1 { flex:1; }
  .flex2 { flex:2; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .btn-primary { padding:0.4rem 1rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .anno-list { display:flex;flex-direction:column;gap:0.3rem; }
  .anno-card { background:rgba(14,20,40,0.4);border:1px solid rgba(58,190,255,0.06);border-radius:6px;padding:0.6rem;display:flex;flex-direction:column;gap:0.2rem; }
  .anno-header { display:flex;justify-content:space-between;align-items:center; }
  .anno-label { font-size:0.85rem;color:#E6EDF3;font-weight:500; }
  .anno-coords { font-size:0.7rem;color:#94A3B8;font-family:monospace; }
  .anno-desc { font-size:0.75rem;color:#94A3B8;margin:0; }
  .btn-delete { background:none;border:1px solid rgba(239,68,68,0.25);color:#EF4444;border-radius:4px;padding:0.15rem 0.4rem;font-size:0.7rem;cursor:pointer;align-self:flex-end; }
  .btn-delete:hover { background:rgba(239,68,68,0.15); }
</style>
