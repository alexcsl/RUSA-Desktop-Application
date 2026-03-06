<!--
  /scientists/archive/[type] — UC-PH-03 / UC-CH-01 / UC-BIO-01: Science Archive Viewer
  type param: "matter" | "physical_object" | "species"
-->
<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { getArchive, type ArchiveItem } from '$lib/stores/scientists';

  let archiveType = $state('');
  page.subscribe((p) => (archiveType = (p.params as Record<string, string>).type ?? ''));

  let items = $state<ArchiveItem[]>([]);
  let loading = $state(true);
  let error = $state('');
  let selected = $state<ArchiveItem | null>(null);

  onMount(load);

  async function load() {
    loading = true;
    error = '';
    try {
      items = await getArchive(archiveType);
    } catch (e: any) {
      error = e?.toString() ?? 'Failed to load archive.';
    } finally {
      loading = false;
    }
  }

  /* pretty-print JSONB detail */
  function fmtDetail(d: unknown): string {
    if (!d) return '—';
    if (typeof d === 'string') return d;
    return JSON.stringify(d, null, 2);
  }

  function friendlyType(t: string): string {
    return t.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
  }
</script>

<h2>{friendlyType(archiveType)} Archive</h2>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if items.length === 0}
  <p class="muted">No entries in the {friendlyType(archiveType)} archive yet.</p>
{:else}
  <div class="grid">
    {#each items as item}
      <button class="card" class:active={selected?.id === item.id} onclick={() => (selected = selected?.id === item.id ? null : item)}>
        <span class="card-name">{item.name}</span>
        <span class="card-date">{new Date(item.created_at).toLocaleDateString()}</span>
      </button>
    {/each}
  </div>

  {#if selected}
    <div class="detail-panel">
      <h3>{selected.name}</h3>
      <p class="meta">Type: {selected.archive_type} · Created {new Date(selected.created_at).toLocaleString()}</p>
      <pre>{fmtDetail(selected.detail)}</pre>
      {#if selected.storage_path}
        <p class="storage">Storage: <code>{selected.storage_path}</code></p>
      {/if}
    </div>
  {/if}
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.75rem; }
  .grid { display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:0.5rem;margin-bottom:1rem; }
  .card { background:#111827;border:1px solid #1E293B;border-radius:8px;padding:0.75rem;text-align:left;cursor:pointer;color:#E6EDF3;display:flex;flex-direction:column;gap:0.2rem;transition:border-color 0.2s; }
  .card:hover, .card.active { border-color:#3ABEFF; }
  .card-name { font-size:0.85rem;font-weight:600; }
  .card-date { font-size:0.7rem;color:#64748B; }
  .detail-panel { background:#111827;border:1px solid rgba(58,190,255,0.15);border-radius:8px;padding:1rem;max-width:700px; }
  .detail-panel h3 { font-size:0.95rem;color:#E6EDF3;margin-bottom:0.25rem; }
  .meta { font-size:0.7rem;color:#64748B;margin-bottom:0.5rem; }
  pre { background:#0B0F1A;border:1px solid #1E293B;border-radius:6px;padding:0.75rem;color:#A5B4FC;font-size:0.75rem;overflow-x:auto;max-height:300px; }
  .storage { font-size:0.75rem;color:#94A3B8;margin-top:0.5rem; }
  .storage code { color:#A5B4FC; }
  .muted { color:#64748B;font-size:0.85rem; }
  .error { color:#EF4444;font-size:0.85rem; }
</style>
