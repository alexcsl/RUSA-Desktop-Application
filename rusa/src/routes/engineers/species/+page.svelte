<!--
  /engineers/species — UC-AGE-05 / UC-BE-01: Species Archive Viewer
  Displays species from science_archive scoped by engineer role.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getSpeciesArchive, type SpeciesArchiveItem } from '$lib/stores/engineers';

  let items = $state<SpeciesArchiveItem[]>([]);
  let loading = $state(true);
  let error = $state('');
  let selected = $state<SpeciesArchiveItem | null>(null);
  let search = $state('');

  onMount(load);

  async function load() {
    loading = true;
    error = '';
    try {
      items = await getSpeciesArchive(search || undefined);
    } catch (e: any) {
      error = e?.toString() ?? 'Failed to load species archive.';
    } finally {
      loading = false;
    }
  }

  interface TaxonomyInfo {
    kingdom?: string;
    phylum?: string;
    family?: string;
    genus?: string;
    species?: string;
  }

  interface SpeciesDetail {
    taxonomy?: TaxonomyInfo;
    habitat?: string;
    description?: string;
    discovery_date?: string;
    [key: string]: unknown;
  }

  function getDetail(d: unknown): SpeciesDetail {
    if (!d || typeof d !== 'object') return {};
    return d as SpeciesDetail;
  }

  function extraFields(d: SpeciesDetail): [string, unknown][] {
    const skip = new Set(['taxonomy', 'habitat', 'description', 'discovery_date']);
    return Object.entries(d).filter(([k]) => !skip.has(k));
  }
</script>

<h2>Species Archive</h2>

<div class="search-bar">
  <input type="text" bind:value={search} placeholder="Search species by name…" />
  <button class="btn-primary" onclick={load}>Search</button>
</div>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if items.length === 0}
  <p class="muted">No species entries found{search ? ` matching "${search}"` : ''}.</p>
{:else}
  <div class="grid">
    {#each items as item}
      <button class="card" class:active={selected?.id === item.id} onclick={() => (selected = selected?.id === item.id ? null : item)}>
        <span class="card-name">{item.name}</span>
        <span class="card-class">{item.classification ?? 'Unclassified'}</span>
        <span class="card-date">{new Date(item.created_at).toLocaleDateString()}</span>
      </button>
    {/each}
  </div>

  {#if selected}
    {@const info = getDetail(selected.detail)}
    <div class="detail-panel">
      <h3>{selected.name}</h3>
      <p class="meta">
        Type: {selected.archive_type}
        · Classification: {selected.classification ?? '—'}
        · Created {new Date(selected.created_at).toLocaleString()}
      </p>

      {#if info.description}
        <p class="species-desc">{info.description}</p>
      {/if}

      <div class="detail-grid">
        {#if info.taxonomy}
          <div class="detail-section">
            <h4>Taxonomy</h4>
            <dl>
              {#if info.taxonomy.kingdom}<dt>Kingdom</dt><dd>{info.taxonomy.kingdom}</dd>{/if}
              {#if info.taxonomy.phylum}<dt>Phylum</dt><dd>{info.taxonomy.phylum}</dd>{/if}
              {#if info.taxonomy.family}<dt>Family</dt><dd>{info.taxonomy.family}</dd>{/if}
              {#if info.taxonomy.genus}<dt>Genus</dt><dd>{info.taxonomy.genus}</dd>{/if}
              {#if info.taxonomy.species}<dt>Species</dt><dd>{info.taxonomy.species}</dd>{/if}
            </dl>
          </div>
        {/if}

        <div class="detail-section">
          <h4>Details</h4>
          <dl>
            {#if info.habitat}<dt>Habitat</dt><dd>{info.habitat}</dd>{/if}
            {#if info.discovery_date}<dt>Discovered</dt><dd>{info.discovery_date}</dd>{/if}
            {#if selected.experiment_id}<dt>Experiment</dt><dd class="uuid">{selected.experiment_id}</dd>{/if}
          </dl>
        </div>
      </div>

      {#if extraFields(info).length > 0}
        <div class="detail-section extra">
          <h4>Additional Data</h4>
          <dl>
            {#each extraFields(info) as [key, val]}
              <dt>{key.replace(/_/g, ' ')}</dt>
              <dd>{typeof val === 'object' ? JSON.stringify(val) : String(val)}</dd>
            {/each}
          </dl>
        </div>
      {/if}
    </div>
  {/if}
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.75rem; }
  .search-bar { display:flex;gap:0.5rem;margin-bottom:1rem;max-width:500px; }
  .search-bar input { flex:1;background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem;font-size:0.8rem; }
  .grid { display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:0.5rem;margin-bottom:1rem; }
  .card { background:#111827;border:1px solid #1E293B;border-radius:8px;padding:0.75rem;text-align:left;cursor:pointer;color:#E6EDF3;display:flex;flex-direction:column;gap:0.2rem;transition:border-color 0.2s; }
  .card:hover, .card.active { border-color:#3ABEFF; }
  .card-name { font-size:0.85rem;font-weight:600; }
  .card-class { font-size:0.7rem;color:#A5B4FC; }
  .card-date { font-size:0.7rem;color:#64748B; }
  .detail-panel { background:#111827;border:1px solid rgba(58,190,255,0.15);border-radius:8px;padding:1rem;max-width:700px; }
  .detail-panel h3 { font-size:0.95rem;color:#E6EDF3;margin-bottom:0.25rem; }
  .meta { font-size:0.7rem;color:#64748B;margin-bottom:0.5rem; }
  .species-desc { font-size:0.8rem;color:#CBD5E1;margin-bottom:0.75rem;line-height:1.5; }
  .detail-grid { display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;margin-bottom:0.75rem; }
  .detail-section { background:#0B0F1A;border:1px solid #1E293B;border-radius:6px;padding:0.75rem; }
  .detail-section.extra { grid-column:1/-1; }
  .detail-section h4 { font-family:'Orbitron',sans-serif;font-size:0.7rem;color:#8B5CF6;margin-bottom:0.5rem;text-transform:uppercase;letter-spacing:0.5px; }
  dl { display:grid;grid-template-columns:auto 1fr;gap:0.15rem 0.75rem; }
  dt { font-size:0.7rem;color:#64748B;white-space:nowrap; }
  dd { font-size:0.75rem;color:#E6EDF3;margin:0; }
  .uuid { font-family:'Fira Code',monospace;font-size:0.65rem;color:#A5B4FC; }
  .muted { color:#64748B;font-size:0.85rem; }
  .error { color:#EF4444;font-size:0.85rem; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
</style>
