<!--
  UC-SC-08: Manage Settlement Inventory (Commander — CRUD)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    stlGetSettlementInventory,
    stlManageInventory,
    type InventoryItem,
  } from '$lib/stores/settlers';

  let items: InventoryItem[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let success = $state('');
  let search = $state('');

  /* form state */
  let editId: string | null = $state(null);
  let fItemName = $state('');
  let fCategory = $state('materials');
  let fQuantity = $state(0);
  let fUnit = $state('unit');
  let fMinThreshold = $state(5);
  let submitting = $state(false);

  async function load() {
    loading = true; error = '';
    try { items = await stlGetSettlementInventory(); }
    catch (e: any) { error = e?.message ?? String(e); }
    finally { loading = false; }
  }
  onMount(load);

  let filtered = $derived(
    items.filter((i) => {
      const q = search.toLowerCase();
      return !q || i.item_name.toLowerCase().includes(q) || (i.category ?? '').toLowerCase().includes(q);
    })
  );

  function startEdit(it: InventoryItem) {
    editId = it.id;
    fItemName = it.item_name;
    fCategory = it.category ?? 'materials';
    fQuantity = it.quantity;
    fUnit = it.unit ?? 'unit';
    fMinThreshold = it.min_threshold;
  }
  function startNew() {
    editId = null;
    fItemName = ''; fCategory = 'materials'; fQuantity = 0; fUnit = 'unit'; fMinThreshold = 5;
  }

  async function save() {
    error = ''; success = '';
    if (!fItemName.trim()) { error = 'Item name required.'; return; }
    submitting = true;
    try {
      const action = editId ? 'update' : 'add';
      await stlManageInventory({
        action,
        id: editId ?? undefined,
        item_name: fItemName,
        category: fCategory,
        quantity: fQuantity,
        unit: fUnit,
      });
      success = editId ? 'Item updated.' : 'Item added.';
      startNew();
      await load();
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }

  async function del(it: InventoryItem) {
    if (!confirm(`Delete "${it.item_name}"?`)) return;
    error = ''; success = '';
    try {
      await stlManageInventory({
        action: 'delete',
        id: it.id,
      });
      success = 'Item deleted.';
      await load();
    } catch (e: any) { error = e?.message ?? String(e); }
  }
</script>

<h2>Manage Inventory</h2>

<!-- Inline form -->
<section class="card">
  <h3>{editId ? 'Edit Item' : 'Add Item'}</h3>
  <form class="form" onsubmit={(e) => { e.preventDefault(); save(); }}>
    <div class="row">
      <label>
        Name *
        <input type="text" bind:value={fItemName} placeholder="Item name" />
      </label>
      <label>
        Category
        <select bind:value={fCategory}>
          <option value="materials">Materials</option>
          <option value="food">Food</option>
          <option value="medical">Medical</option>
          <option value="tools">Tools</option>
          <option value="seeds">Seeds</option>
          <option value="equipment">Equipment</option>
          <option value="other">Other</option>
        </select>
      </label>
    </div>
    <div class="row">
      <label>
        Qty
        <input type="number" bind:value={fQuantity} min="0" />
      </label>
      <label>
        Unit
        <input type="text" bind:value={fUnit} placeholder="kg, unit, litre…" />
      </label>
      <label>
        Min Threshold
        <input type="number" bind:value={fMinThreshold} min="0" />
      </label>
    </div>
    <div class="row">
      <button type="submit" class="btn-primary" disabled={submitting}>
        {submitting ? 'Saving…' : editId ? 'Update' : 'Add'}
      </button>
      {#if editId}
        <button type="button" class="btn-ghost" onclick={startNew}>Cancel</button>
      {/if}
    </div>
  </form>
</section>

{#if error}<p class="err">{error}</p>{/if}
{#if success}<p class="ok">{success}</p>{/if}

<!-- Filter bar -->
<div class="bar">
  <input type="text" class="search" placeholder="Search…" bind:value={search} />
  <span class="dim">{filtered.length} item(s)</span>
</div>

{#if loading}
  <p class="dim">Loading…</p>
{:else if filtered.length === 0}
  <p class="dim">No items.</p>
{:else}
  <table class="tbl">
    <thead>
      <tr><th>Item</th><th>Category</th><th>Qty</th><th>Unit</th><th>Min</th><th>Updated</th><th></th></tr>
    </thead>
    <tbody>
      {#each filtered as it}
        <tr>
          <td>{it.item_name}</td>
          <td><span class="badge cat">{it.category}</span></td>
          <td class:low={it.quantity <= it.min_threshold}>{it.quantity}</td>
          <td>{it.unit}</td>
          <td>{it.min_threshold}</td>
          <td class="dim">{new Date(it.updated_at).toLocaleDateString()}</td>
          <td class="actions">
            <button class="btn-sm edit" onclick={() => startEdit(it)}>✎</button>
            <button class="btn-sm del-btn" onclick={() => del(it)}>✕</button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
  h3 { font-family:'Orbitron',sans-serif;color:#E6EDF3;font-size:0.9rem;margin:0 0 0.5rem; }
  .card { background:#111827;border:1px solid rgba(58,190,255,0.15);border-radius:6px;padding:1rem;margin-bottom:1rem; }
  .form { display:flex;flex-direction:column;gap:0.6rem; }
  .form label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8;flex:1; }
  .form input, .form select { background:#0B0F1A;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.35rem 0.5rem;font-size:0.8rem; }
  .row { display:flex;gap:0.75rem;align-items:flex-end; }
  .bar { display:flex;align-items:center;gap:1rem;margin-bottom:0.75rem; }
  .search { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.35rem 0.5rem;font-size:0.8rem;width:220px; }
  .dim { color:#64748B;font-size:0.8rem; }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.4rem 0.6rem;border-bottom:1px solid #374151;color:#94A3B8;font-weight:500; }
  .tbl td { padding:0.4rem 0.6rem;border-bottom:1px solid rgba(55,65,81,0.4); }
  .badge { display:inline-block;padding:0.15rem 0.55rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .cat { background:rgba(139,92,246,0.15);color:#A78BFA; }
  .low { color:#EF4444;font-weight:600; }
  .actions { display:flex;gap:0.4rem; }
  .btn-sm { padding:0.2rem 0.5rem;border-radius:4px;font-size:0.75rem;border:none;cursor:pointer; }
  .edit { background:rgba(58,190,255,0.12);color:#3ABEFF; }
  .edit:hover { background:rgba(58,190,255,0.25); }
  .del-btn { background:rgba(239,68,68,0.12);color:#EF4444; }
  .del-btn:hover { background:rgba(239,68,68,0.25); }
  .btn-primary { background:#3ABEFF;color:#0B0F1A;border:none;border-radius:4px;padding:0.45rem 1rem;cursor:pointer;font-weight:600;font-size:0.8rem; }
  .btn-primary:hover { background:#60CFFF; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-ghost { background:none;border:1px solid #374151;color:#94A3B8;padding:0.45rem 0.9rem;border-radius:4px;cursor:pointer;font-size:0.8rem; }
</style>
