<!-- /station/inventory — UC-SSS-03: Manage Station Inventory -->
<script lang="ts">
  import { getContext, onMount } from 'svelte';
  import type { Writable } from 'svelte/store';
  import { sstGetInventory, sstManageInventory, type InventoryItem } from '$lib/stores/space_station';

  const stationId = getContext<Writable<string>>('selectedStationId');

  let items: InventoryItem[] = $state([]);
  let loading = $state(true);
  let searchTerm = $state('');
  let filterCategory = $state('all');

  // Add/Remove form
  let itemName = $state('');
  let category = $state('');
  let quantityChange = $state(1);
  let unit = $state('');
  let formError = $state('');
  let formSuccess = $state('');
  let submitting = $state(false);

  let currentStationId = $state('');
  const unsub = stationId.subscribe((v) => {
    currentStationId = v;
    if (v) loadInventory();
  });

  async function loadInventory() {
    if (!currentStationId) return;
    loading = true;
    try {
      items = await sstGetInventory(currentStationId);
    } catch {}
    loading = false;
  }

  onMount(() => { if (currentStationId) loadInventory(); });

  let categories = $derived([...new Set(items.map((i) => i.category).filter(Boolean))]);

  let filtered = $derived(
    items.filter((item) => {
      if (filterCategory !== 'all' && item.category !== filterCategory) return false;
      if (searchTerm.trim()) {
        const s = searchTerm.toLowerCase();
        return item.item_name.toLowerCase().includes(s) ||
          (item.category ?? '').toLowerCase().includes(s);
      }
      return true;
    })
  );

  async function handleAdd(isRemove = false) {
    formError = ''; formSuccess = '';
    if (!currentStationId) { formError = 'No station selected.'; return; }
    if (!itemName.trim()) { formError = 'Item name is required.'; return; }
    if (quantityChange <= 0) { formError = 'Quantity must be positive.'; return; }

    submitting = true;
    try {
      await sstManageInventory({
        station_id: currentStationId,
        item_name: itemName,
        category: category || undefined,
        quantity_change: isRemove ? -quantityChange : quantityChange,
        unit: unit || undefined,
      });
      formSuccess = isRemove ? `Removed ${quantityChange} of ${itemName}.` : `Added ${quantityChange} of ${itemName}.`;
      itemName = ''; category = ''; quantityChange = 1; unit = '';
      await loadInventory();
    } catch (e: unknown) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  function quantityColor(qty: number): string {
    if (qty <= 0) return '#EF4444';
    if (qty <= 5) return '#F59E0B';
    return '#10B981';
  }
</script>

<h1 class="title">Station Inventory</h1>
<p class="subtitle">Track food, clothing, bedsheets, and all station supplies.</p>

<div class="two-col">
  <!-- Left: form -->
  <div class="form-card">
    <h2>Add / Remove Item</h2>

    <label class="field"><span class="label">Item Name *</span>
      <input type="text" class="input" bind:value={itemName} placeholder="e.g. Oxygen Canister, Meal Pack…" />
    </label>

    <label class="field"><span class="label">Category</span>
      <input type="text" class="input" bind:value={category} placeholder="e.g. Food, Medical, Equipment…" />
    </label>

    <div class="row-fields">
      <label class="field half"><span class="label">Quantity *</span>
        <input type="number" class="input" bind:value={quantityChange} min="1" />
      </label>
      <label class="field half"><span class="label">Unit</span>
        <input type="text" class="input" bind:value={unit} placeholder="e.g. pcs, kg, L…" />
      </label>
    </div>

    {#if formError}<p class="error">{formError}</p>{/if}
    {#if formSuccess}<p class="success">{formSuccess}</p>{/if}

    <div class="btn-row">
      <button class="btn-primary" onclick={() => handleAdd(false)} disabled={submitting}>
        {submitting ? '…' : '+ Add'}
      </button>
      <button class="btn-danger" onclick={() => handleAdd(true)} disabled={submitting}>
        {submitting ? '…' : '− Remove'}
      </button>
    </div>
  </div>

  <!-- Right: inventory list -->
  <div class="list-section">
    <div class="controls">
      <input type="text" class="search-input" bind:value={searchTerm} placeholder="Search inventory…" />
      <select class="filter-select" bind:value={filterCategory}>
        <option value="all">All Categories</option>
        {#each categories as cat}
          <option value={cat}>{cat}</option>
        {/each}
      </select>
    </div>

    {#if loading}
      <p class="muted">Loading inventory…</p>
    {:else if filtered.length === 0}
      <p class="muted">No items found.</p>
    {:else}
      <div class="inv-table">
        <div class="inv-header-row">
          <span class="col-name">Item</span>
          <span class="col-cat">Category</span>
          <span class="col-qty">Qty</span>
          <span class="col-unit">Unit</span>
          <span class="col-date">Updated</span>
        </div>
        {#each filtered as item}
          <div class="inv-row">
            <span class="col-name">{item.item_name}</span>
            <span class="col-cat">{item.category ?? '—'}</span>
            <span class="col-qty" style="color:{quantityColor(item.quantity)}">{item.quantity}</span>
            <span class="col-unit">{item.unit ?? '—'}</span>
            <span class="col-date">{new Date(item.updated_at).toLocaleDateString()}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .two-col { display:flex;gap:1.25rem;flex-wrap:wrap; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;min-width:280px;max-width:350px;display:flex;flex-direction:column;gap:0.6rem; }
  .form-card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0; }
  .list-section { flex:1;min-width:350px; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .row-fields { display:flex;gap:0.75rem; }
  .half { flex:1; }
  .btn-row { display:flex;gap:0.5rem; }
  .btn-primary { padding:0.4rem 1rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-danger { padding:0.4rem 1rem;background:rgba(239,68,68,0.15);border:1px solid rgba(239,68,68,0.3);color:#EF4444;border-radius:6px;cursor:pointer;font-size:0.8rem;font-weight:600; }
  .btn-danger:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-danger:hover { background:rgba(239,68,68,0.25); }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .controls { display:flex;gap:0.5rem;margin-bottom:0.75rem; }
  .search-input { flex:1;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .search-input:focus { outline:none;border-color:#3ABEFF; }
  .filter-select { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .inv-table { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;overflow:hidden; }
  .inv-header-row { display:flex;padding:0.5rem 0.75rem;background:rgba(58,190,255,0.05);font-size:0.7rem;color:#94A3B8;font-weight:600;text-transform:uppercase;letter-spacing:0.5px; }
  .inv-row { display:flex;padding:0.5rem 0.75rem;border-top:1px solid rgba(255,255,255,0.04);font-size:0.8rem; }
  .inv-row:hover { background:rgba(58,190,255,0.03); }
  .col-name { flex:2; }
  .col-cat { flex:1.5;color:#94A3B8; }
  .col-qty { flex:0.5;text-align:center;font-weight:600; }
  .col-unit { flex:0.5;color:#94A3B8;text-align:center; }
  .col-date { flex:1;color:#94A3B8;text-align:right;font-size:0.75rem; }
</style>
