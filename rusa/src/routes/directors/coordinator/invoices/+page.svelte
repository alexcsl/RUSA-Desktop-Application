<!--
  /directors/coordinator/invoices — Invoices & Receipts (UC-CORD-03)
  TheCoordinator views all event financial documents with file preview.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getEvents, getEventDocuments,
    type EventSummary, type EventDocSummary,
  } from '$lib/stores/directors';
  import { dirGetFileSignedUrl } from '$lib/stores/directors';

  interface EventWithDocs {
    event: EventSummary;
    docs: EventDocSummary[];
  }

  const RELEVANT_TYPES = ['invoice', 'ticket', 'receipt', 'contract'];

  let groups: EventWithDocs[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  let previewUrl = $state<string | null>(null);
  let previewFilename = $state('');
  let previewContentType = $state('');
  let previewLoading = $state(false);

  onMount(async () => {
    try {
      const events = await getEvents();
      const results = await Promise.all(
        events.map(async (event) => {
          const docs = await getEventDocuments(event.id);
          const filtered = docs.filter(
            (d) => d.document_type && RELEVANT_TYPES.includes(d.document_type.toLowerCase())
          );
          return { event, docs: filtered };
        })
      );
      groups = results.filter((g) => g.docs.length > 0);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  async function openPreview(doc: EventDocSummary) {
    previewLoading = true;
    previewFilename = doc.original_filename;
    previewContentType = doc.content_type;
    previewUrl = null;
    try {
      const result = await dirGetFileSignedUrl(doc.storage_path);
      previewUrl = result.signed_url;
    } catch (e: unknown) {
      previewUrl = null;
      error = String(e);
    } finally {
      previewLoading = false;
    }
  }

  function closePreview() {
    previewUrl = null;
    previewFilename = '';
  }

  function isImage(ct: string) {
    return ct.startsWith('image/');
  }

  function isPdf(ct: string) {
    return ct === 'application/pdf';
  }

  function formatDate(d: string): string { return new Date(d).toLocaleDateString(); }

  function docTypeColor(t: string | null): string {
    const m: Record<string, string> = {
      invoice: '#3ABEFF', receipt: '#22C55E', ticket: '#8B5CF6', contract: '#F59E0B',
    };
    return t ? (m[t.toLowerCase()] ?? '#94A3B8') : '#94A3B8';
  }
</script>

<h1 class="title">Invoices &amp; Receipts</h1>
<p class="subtitle">Archived event documents grouped by event. Click "View" to preview a file.</p>

{#if loading}<p class="muted">Loading…</p>
{:else if error}<p class="error">{error}</p>
{:else if groups.length === 0}
  <p class="muted">No archived documents yet.</p>
{:else}
  <div class="groups">
    {#each groups as group}
      <div class="event-group">
        <h3 class="event-title">{group.event.title}</h3>
        {#if group.event.event_date}
          <p class="event-meta">Event date: {formatDate(group.event.event_date)}</p>
        {/if}
        <div class="doc-list">
          {#each group.docs as doc}
            <div class="doc-card">
              <div class="doc-header">
                <span class="doc-name">{doc.original_filename}</span>
                {#if doc.document_type}
                  <span class="doc-type" style="color:{docTypeColor(doc.document_type)}">
                    {doc.document_type}
                  </span>
                {/if}
                <button class="btn-view" onclick={() => openPreview(doc)}>View</button>
              </div>
              <p class="doc-meta">Uploaded: {formatDate(doc.uploaded_at)}</p>
            </div>
          {/each}
        </div>
      </div>
    {/each}
  </div>
{/if}

<!-- Preview modal -->
{#if previewLoading}
  <div class="overlay">
    <div class="modal-box">
      <p class="muted">Loading preview…</p>
    </div>
  </div>
{:else if previewUrl}
  <div class="overlay" onclick={closePreview}>
    <div class="modal-box" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <span class="modal-filename">{previewFilename}</span>
        <button class="btn-close" onclick={closePreview}>✕</button>
      </div>
      {#if isImage(previewContentType)}
        <img src={previewUrl} alt={previewFilename} class="preview-img" />
      {:else if isPdf(previewContentType)}
        <iframe src={previewUrl} title={previewFilename} class="preview-pdf"></iframe>
      {:else}
        <div class="preview-fallback">
          <p>Preview not available for this file type.</p>
          <a href={previewUrl} target="_blank" rel="noopener" class="btn-download">Open / Download</a>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .error { color:#EF4444;font-size:0.75rem; }
  .groups { display:flex;flex-direction:column;gap:1.25rem; }
  .event-group { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.1);border-radius:10px;padding:1rem; }
  .event-title { font-family:'Orbitron',sans-serif;font-size:0.88rem;color:#3ABEFF;margin:0 0 0.2rem; }
  .event-meta { font-size:0.72rem;color:#64748B;margin:0 0 0.75rem; }
  .doc-list { display:flex;flex-direction:column;gap:0.4rem; }
  .doc-card { background:rgba(255,255,255,0.03);border:1px solid rgba(255,255,255,0.06);border-radius:6px;padding:0.6rem 0.8rem; }
  .doc-header { display:flex;align-items:center;gap:0.5rem;flex-wrap:wrap; }
  .doc-name { font-size:0.82rem;color:#E6EDF3;font-weight:500;flex:1; }
  .doc-type { font-size:0.67rem;font-weight:700;text-transform:capitalize;border:1px solid currentColor;border-radius:4px;padding:0.1rem 0.35rem;opacity:0.9; }
  .doc-meta { font-size:0.7rem;color:#475569;margin:0.2rem 0 0; }
  .btn-view { padding:0.2rem 0.55rem;background:rgba(58,190,255,0.1);border:1px solid rgba(58,190,255,0.3);color:#3ABEFF;border-radius:4px;cursor:pointer;font-size:0.72rem;white-space:nowrap; }
  .btn-view:hover { background:rgba(58,190,255,0.2); }

  /* Preview modal */
  .overlay { position:fixed;inset:0;background:rgba(0,0,0,0.75);display:flex;align-items:center;justify-content:center;z-index:100; }
  .modal-box { background:#111827;border:1px solid rgba(58,190,255,0.2);border-radius:10px;padding:1.25rem;max-width:800px;width:90%;max-height:90vh;display:flex;flex-direction:column;gap:0.75rem; }
  .modal-header { display:flex;justify-content:space-between;align-items:center; }
  .modal-filename { font-size:0.82rem;color:#94A3B8;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;max-width:90%; }
  .btn-close { background:none;border:none;color:#64748B;cursor:pointer;font-size:1.1rem;padding:0.1rem 0.3rem; }
  .btn-close:hover { color:#E6EDF3; }
  .preview-img { max-width:100%;max-height:60vh;object-fit:contain;border-radius:6px; }
  .preview-pdf { width:100%;height:60vh;border:none;border-radius:6px; }
  .preview-fallback { text-align:center;padding:2rem;color:#64748B;font-size:0.82rem;display:flex;flex-direction:column;gap:0.75rem;align-items:center; }
  .btn-download { padding:0.4rem 1rem;background:rgba(58,190,255,0.15);border:1px solid rgba(58,190,255,0.4);color:#3ABEFF;border-radius:6px;font-size:0.8rem;text-decoration:none; }
</style>
