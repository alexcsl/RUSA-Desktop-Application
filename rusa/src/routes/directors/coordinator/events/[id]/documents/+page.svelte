<!--
  /directors/coordinator/events/[id]/documents — Event Documents (UC-CORD-03)
  TheCoordinator uploads and views documents attached to an event.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { getEventDocuments, uploadEventDocument, type EventDocSummary } from '$lib/stores/directors';

  let eventId = $derived($page.params.id);
  let docs: EventDocSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let uploading = $state(false);
  let fileName = $state('');
  let fileUrl = $state('');
  let uploadMsg = $state('');

  onMount(async () => { await loadDocs(); });

  async function loadDocs() {
    loading = true; error = '';
    try { docs = await getEventDocuments(eventId); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  }

  async function handleUpload() {
    uploadMsg = '';
    if (!fileName.trim() || !fileUrl.trim()) { uploadMsg = 'File name and URL are required.'; return; }
    uploading = true;
    try {
      await uploadEventDocument({ event_id: eventId, file_name: fileName.trim(), file_url: fileUrl.trim() });
      uploadMsg = 'Document uploaded.';
      fileName = ''; fileUrl = '';
      await loadDocs();
    } catch (e: unknown) { uploadMsg = e instanceof Error ? e.message : String(e); }
    finally { uploading = false; }
  }

  function formatDate(d: string) { return new Date(d).toLocaleString(); }
</script>

<h1 class="title">Event Documents</h1>
<p class="subtitle">Documents attached to event <code>{eventId.slice(0,8)}…</code></p>

<div class="layout">
  <div class="upload-card">
    <h2>Upload Document</h2>
    <label class="field">
      <span class="label">File Name *</span>
      <input class="input" type="text" bind:value={fileName} placeholder="e.g. agenda.pdf" />
    </label>
    <label class="field">
      <span class="label">File URL *</span>
      <input class="input" type="text" bind:value={fileUrl} placeholder="https://…" />
    </label>
    {#if uploadMsg}<p class="upload-msg">{uploadMsg}</p>{/if}
    <button class="btn-primary" onclick={handleUpload} disabled={uploading}>
      {uploading ? 'Uploading…' : 'Upload'}
    </button>
  </div>

  <div class="docs-panel">
    <h2>Attached Documents</h2>
    {#if loading}<p class="muted">Loading…</p>
    {:else if error}<p class="error">{error}</p>
    {:else if docs.length === 0}<p class="muted">No documents yet.</p>
    {:else}
      {#each docs as doc}
        <div class="doc-row">
          <span class="doc-name">{doc.file_name}</span>
          <span class="doc-date">{formatDate(doc.uploaded_at)}</span>
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  code { background:rgba(58,190,255,0.1);color:#3ABEFF;padding:0.1rem 0.3rem;border-radius:3px;font-size:0.8rem; }
  .layout { display:flex;gap:1.5rem;flex-wrap:wrap; }
  .upload-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;width:360px;display:flex;flex-direction:column;gap:0.6rem; }
  .upload-card h2,.docs-panel h2 { font-family:'Orbitron',sans-serif;font-size:0.85rem;color:#8B5CF6;margin:0 0 0.4rem; }
  .docs-panel { flex:1;min-width:260px; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif;width:100%;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;padding:0.45rem;cursor:pointer;font-size:0.82rem;font-weight:600; }
  .btn-primary:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:default; }
  .upload-msg { font-size:0.75rem;color:#22C55E; }
  .muted { color:#475569;font-size:0.8rem;font-style:italic; }
  .doc-row { display:flex;justify-content:space-between;align-items:center;padding:0.45rem 0;border-bottom:1px solid rgba(255,255,255,0.04); }
  .doc-name { font-size:0.82rem;color:#E6EDF3; }
  .doc-date { font-size:0.68rem;color:#475569; }
  .error { color:#EF4444;font-size:0.75rem; }
</style>
