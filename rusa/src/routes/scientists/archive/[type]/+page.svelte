<!--
  /scientists/archive/[type] — UC-PH-03 / UC-CH-01 / UC-BIO-01: Science Archive Viewer
  type param: "matter" | "physical_object" | "species"
  Renders each archive entry's JSONB detail as structured, human-readable fields.
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
  let search = $state('');

  onMount(load);

  async function load() {
    loading = true; error = '';
    try { items = await getArchive(archiveType); }
    catch (e: unknown) { error = String(e); }
    finally { loading = false; }
  }

  let filtered = $derived(
    search.trim()
      ? items.filter(i => i.name.toLowerCase().includes(search.toLowerCase()))
      : items
  );

  function pageTitle(t: string): string {
    if (t === 'matter') return 'Matter Archive';
    if (t === 'physical_object') return 'Object & Phenomenon Archive';
    if (t === 'species') return 'Species Archive';
    return t.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase()) + ' Archive';
  }

  function pageIcon(t: string): string {
    if (t === 'matter') return '⚗';
    if (t === 'physical_object') return '🔭';
    if (t === 'species') return '🧬';
    return '📁';
  }

  /* ── Type-safe detail accessors ── */
  function d(key: string): string {
    if (!selected) return '—';
    const v = selected.detail[key];
    if (v == null || v === '') return '—';
    return String(v);
  }

  function tax(key: string): string {
    if (!selected) return '—';
    const taxonomy = selected.detail['taxonomy'] as Record<string, unknown> | undefined;
    if (!taxonomy) return '—';
    const v = taxonomy[key];
    if (v == null || v === '') return '—';
    return String(v);
  }

  function fmt(iso: string | undefined | null): string {
    if (!iso || iso === '—') return '—';
    const d = new Date(iso);
    if (isNaN(d.getTime())) return iso;
    return d.toLocaleDateString('en-GB', { day: 'numeric', month: 'long', year: 'numeric' });
  }
</script>

<div class="page">
  <!-- ── Left sidebar ── -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <span class="icon">{pageIcon(archiveType)}</span>
      <h2>{pageTitle(archiveType)}</h2>
    </div>

    <div class="search-wrap">
      <input class="search" bind:value={search} placeholder="Search by name…" />
    </div>

    {#if loading}
      <div class="empty-msg">Loading…</div>
    {:else if error}
      <div class="err-msg">{error}</div>
    {:else if filtered.length === 0}
      <div class="empty-msg">No entries found.</div>
    {:else}
      <div class="item-list">
        {#each filtered as item}
          <button
            class="item-row"
            class:selected={selected?.id === item.id}
            onclick={() => selected = selected?.id === item.id ? null : item}
          >
            <span class="item-name">{item.name}</span>
            {#if item.classification}
              <span class="item-class">{item.classification}</span>
            {/if}
            <span class="item-date">{new Date(item.created_at).toLocaleDateString()}</span>
          </button>
        {/each}
      </div>
    {/if}
  </aside>

  <!-- ── Right detail panel ── -->
  <main class="detail">
    {#if !selected}
      <div class="no-selection">
        <span class="no-sel-icon">{pageIcon(archiveType)}</span>
        <p>Select an entry from the list to view its details.</p>
      </div>

    {:else if archiveType === 'species'}
      <!-- ═══════════════ SPECIES ═══════════════ -->
      <div class="entry-header">
        <h3>{selected.name}</h3>
        {#if selected.classification}<span class="classification-badge">{selected.classification}</span>{/if}
        <span class="date-badge">Added {fmt(selected.created_at)}</span>
      </div>

      <section class="section">
        <h4 class="section-title">🔬 Taxonomy</h4>
        <div class="taxonomy-grid">
          {#each [
            ['Domain',  tax('domain')],
            ['Kingdom', tax('kingdom')],
            ['Phylum',  tax('phylum')],
            ['Class',   tax('class')],
            ['Order',   tax('order')],
            ['Family',  tax('family')],
            ['Genus',   tax('genus')],
            ['Species', tax('species')],
          ] as [rank, val]}
            <div class="tax-row">
              <span class="tax-rank">{rank}</span>
              <span class="tax-val" class:muted={val === '—'}>{val}</span>
            </div>
          {/each}
        </div>
      </section>

      <section class="section">
        <h4 class="section-title">🌍 Discovery</h4>
        <div class="field-group">
          <div class="field-row">
            <span class="field-label">Location of Discovery</span>
            <span class="field-value">{d('location_of_discovery')}</span>
          </div>
          <div class="field-row">
            <span class="field-label">Date of Discovery</span>
            <span class="field-value">{fmt(d('date_of_discovery'))}</span>
          </div>
          <div class="field-row">
            <span class="field-label">Natural Habitat</span>
            <span class="field-value">{d('natural_habitat')}</span>
          </div>
        </div>
      </section>

      <section class="section">
        <h4 class="section-title">🧪 Characteristics</h4>
        <div class="text-block-group">
          <div class="text-block">
            <span class="text-block-label">Physical Characteristics</span>
            <p class="text-block-body">{d('physical_characteristics')}</p>
          </div>
          <div class="text-block">
            <span class="text-block-label">Behavioral Characteristics</span>
            <p class="text-block-body">{d('behavioral_characteristics')}</p>
          </div>
          <div class="text-block">
            <span class="text-block-label">Description</span>
            <p class="text-block-body">{d('description')}</p>
          </div>
        </div>
      </section>

    {:else if archiveType === 'matter'}
      <!-- ═══════════════ MATTER ═══════════════ -->
      <div class="entry-header">
        <h3>{selected.name}</h3>
        {#if selected.classification}<span class="classification-badge">{selected.classification}</span>{/if}
        <span class="date-badge">Added {fmt(selected.created_at)}</span>
      </div>

      <section class="section">
        <h4 class="section-title">⚗ Identification</h4>
        <div class="field-group">
          <div class="field-row">
            <span class="field-label">Category</span>
            <span class="field-value">{d('item_category')}</span>
          </div>
          <div class="field-row">
            <span class="field-label">Chemical Formula</span>
            <span class="field-value formula">{d('chemical_formula')}</span>
          </div>
          <div class="field-row">
            <span class="field-label">Status</span>
            <span class="field-value">
              <span class="status-chip status-{d('status').toLowerCase()}">{d('status')}</span>
            </span>
          </div>
        </div>
      </section>

      <section class="section">
        <h4 class="section-title">📍 Origin</h4>
        <div class="field-group">
          <div class="field-row">
            <span class="field-label">Date of Origin</span>
            <span class="field-value">{fmt(d('date_of_origin'))}</span>
          </div>
          <div class="field-row">
            <span class="field-label">Location of Origin</span>
            <span class="field-value">{d('location_of_origin')}</span>
          </div>
          <div class="field-row">
            <span class="field-label">Acceptance Date</span>
            <span class="field-value">{fmt(d('acceptance_date'))}</span>
          </div>
        </div>
      </section>

      <section class="section">
        <h4 class="section-title">📋 Description</h4>
        <div class="text-block-group">
          <div class="text-block">
            <p class="text-block-body">{d('description')}</p>
          </div>
        </div>
      </section>

    {:else if archiveType === 'physical_object'}
      <!-- ═══════════════ PHYSICAL OBJECT ═══════════════ -->
      <div class="entry-header">
        <h3>{selected.name}</h3>
        {#if selected.classification}<span class="classification-badge">{selected.classification}</span>{/if}
        <span class="date-badge">Added {fmt(selected.created_at)}</span>
      </div>

      <section class="section">
        <h4 class="section-title">🔭 Classification</h4>
        <div class="field-group">
          <div class="field-row">
            <span class="field-label">Classification</span>
            <span class="field-value">{d('classification')}</span>
          </div>
          <div class="field-row">
            <span class="field-label">Date of Origin</span>
            <span class="field-value">{fmt(d('date_of_origin'))}</span>
          </div>
          <div class="field-row">
            <span class="field-label">Location of Origin</span>
            <span class="field-value">{d('location_of_origin')}</span>
          </div>
        </div>
      </section>

      <section class="section">
        <h4 class="section-title">📐 Physical Properties</h4>
        <div class="text-block-group">
          <div class="text-block">
            <span class="text-block-label">Key Physical Characteristics</span>
            <p class="text-block-body">{d('key_physical_characteristics')}</p>
          </div>
          <div class="text-block">
            <span class="text-block-label">Description</span>
            <p class="text-block-body">{d('description')}</p>
          </div>
        </div>
      </section>

    {/if}

    {#if selected && selected.experiment_id}
      <div class="exp-ref">
        <span class="exp-ref-label">Linked Experiment</span>
        <code class="exp-ref-id">{selected.experiment_id}</code>
      </div>
    {/if}
  </main>
</div>

<style>
  .page { display:flex;height:100%;overflow:hidden;color:#E6EDF3;font-family:'Inter',sans-serif; }

  /* ── Sidebar ── */
  .sidebar { width:280px;min-width:220px;display:flex;flex-direction:column;background:#0D1117;border-right:1px solid rgba(58,190,255,0.1);overflow:hidden; }
  .sidebar-header { display:flex;align-items:center;gap:0.5rem;padding:1rem 1rem 0.5rem;border-bottom:1px solid rgba(58,190,255,0.08); }
  .icon { font-size:1.2rem; }
  .sidebar-header h2 { font-family:'Orbitron',sans-serif;font-size:0.85rem;color:#3ABEFF;margin:0; }
  .search-wrap { padding:0.6rem 0.75rem; }
  .search { width:100%;background:#111827;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem 0.6rem;font-size:0.78rem;box-sizing:border-box; }
  .search:focus { outline:none;border-color:#3ABEFF; }
  .item-list { flex:1;overflow-y:auto;padding:0 0.4rem 0.5rem; }
  .item-row { width:100%;text-align:left;background:transparent;border:1px solid transparent;border-radius:6px;padding:0.55rem 0.65rem;cursor:pointer;color:#E6EDF3;display:flex;flex-direction:column;gap:0.1rem;transition:all 0.15s;margin-bottom:0.2rem; }
  .item-row:hover { background:rgba(58,190,255,0.05);border-color:rgba(58,190,255,0.15); }
  .item-row.selected { background:rgba(58,190,255,0.1);border-color:#3ABEFF; }
  .item-name { font-size:0.82rem;font-weight:600;white-space:nowrap;overflow:hidden;text-overflow:ellipsis; }
  .item-class { font-size:0.68rem;color:#8B5CF6;font-style:italic; }
  .item-date { font-size:0.65rem;color:#475569; }
  .empty-msg { padding:1.5rem 1rem;font-size:0.8rem;color:#475569;text-align:center; }
  .err-msg { padding:1rem;font-size:0.78rem;color:#EF4444; }

  /* ── Detail panel ── */
  .detail { flex:1;overflow-y:auto;padding:1.5rem; }

  .no-selection { display:flex;flex-direction:column;align-items:center;justify-content:center;height:100%;gap:0.75rem;color:#475569; }
  .no-sel-icon { font-size:3rem;opacity:0.3; }
  .no-selection p { font-size:0.85rem; }

  /* Entry header */
  .entry-header { display:flex;align-items:baseline;flex-wrap:wrap;gap:0.5rem;margin-bottom:1.25rem;padding-bottom:0.75rem;border-bottom:1px solid rgba(58,190,255,0.1); }
  .entry-header h3 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#E6EDF3;margin:0;flex:1;min-width:200px; }
  .classification-badge { background:rgba(139,92,246,0.15);color:#A78BFA;border:1px solid rgba(139,92,246,0.3);border-radius:20px;padding:0.15rem 0.6rem;font-size:0.7rem;font-style:italic; }
  .date-badge { background:rgba(58,190,255,0.08);color:#64748B;border-radius:20px;padding:0.15rem 0.6rem;font-size:0.68rem; }

  /* Sections */
  .section { margin-bottom:1.25rem; }
  .section-title { font-size:0.72rem;font-weight:700;text-transform:uppercase;letter-spacing:0.08em;color:#64748B;margin:0 0 0.6rem;display:flex;align-items:center;gap:0.4rem; }

  /* Field rows */
  .field-group { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:8px;overflow:hidden; }
  .field-row { display:flex;align-items:baseline;gap:1rem;padding:0.55rem 0.85rem;border-bottom:1px solid rgba(255,255,255,0.03); }
  .field-row:last-child { border-bottom:none; }
  .field-label { font-size:0.72rem;color:#64748B;min-width:160px;flex-shrink:0; }
  .field-value { font-size:0.82rem;color:#CBD5E1; }
  .field-value.muted { color:#475569; }
  .formula { font-family:'Fira Code',monospace;color:#A5B4FC;letter-spacing:0.05em; }

  /* Taxonomy */
  .taxonomy-grid { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:8px;overflow:hidden;display:grid;grid-template-columns:1fr 1fr; }
  .tax-row { display:flex;align-items:center;gap:0.6rem;padding:0.45rem 0.85rem;border-bottom:1px solid rgba(255,255,255,0.03);border-right:1px solid rgba(255,255,255,0.03); }
  .tax-row:nth-child(even) { border-right:none; }
  .tax-row:nth-last-child(-n+2) { border-bottom:none; }
  .tax-rank { font-size:0.68rem;color:#64748B;min-width:60px; }
  .tax-val { font-size:0.82rem;color:#CBD5E1;font-style:italic; }
  .tax-val.muted { color:#475569;font-style:normal; }

  /* Text blocks */
  .text-block-group { display:flex;flex-direction:column;gap:0.6rem; }
  .text-block { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.75rem 0.9rem; }
  .text-block-label { display:block;font-size:0.68rem;text-transform:uppercase;letter-spacing:0.06em;color:#64748B;margin-bottom:0.35rem; }
  .text-block-body { margin:0;font-size:0.82rem;color:#CBD5E1;line-height:1.6; }

  /* Status chip */
  .status-chip { padding:0.15rem 0.5rem;border-radius:4px;font-size:0.7rem;font-weight:600;text-transform:capitalize; }
  .status-active { background:rgba(16,185,129,0.15);color:#10B981; }
  .status-archived { background:rgba(71,85,105,0.3);color:#94A3B8; }
  .status-pending { background:rgba(245,158,11,0.15);color:#F59E0B; }

  /* Linked experiment */
  .exp-ref { display:flex;align-items:center;gap:0.6rem;margin-top:1rem;padding:0.6rem 0.9rem;background:rgba(14,20,40,0.4);border:1px solid rgba(58,190,255,0.08);border-radius:8px; }
  .exp-ref-label { font-size:0.7rem;color:#64748B; }
  .exp-ref-id { font-size:0.72rem;color:#A5B4FC;font-family:'Fira Code',monospace; }
</style>
