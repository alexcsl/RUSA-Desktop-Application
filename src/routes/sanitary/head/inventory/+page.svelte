<!-- /sanitary/head/inventory — UC-HS-03: Manage Cleaning Inventory -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    sanGetInventory, sanAddInventory, sanUpdateInventory, sanRemoveInventory,
    type SanitaryInventoryItem,
  } from '$lib/stores/sanitary';

  let items: SanitaryInventoryItem[] = $state([]);
  let selected: SanitaryInventoryItem | null = $state(null);
  let error = $state('');
  let success = $state('');
  let mode: 'view' | 'add' | 'edit' = $state('view');

  // Form state
  let fName = $state('');
  let fQuantity: number = $state(0);
  let fUnit = $state('units');
  let fCategory = $state('');

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = ''; success = '';
    try { items = await sanGetInventory(); } catch (e: unknown) { error = String(e); }
  }

  function select(item: SanitaryInventoryItem) {
    selected = item;
    mode = 'view';
  }

  function startAdd() {
    fName = ''; fQuantity = 0; fUnit = 'units'; fCategory = '';
    mode = 'add'; selected = null;
  }

  function startEdit() {
    if (!selected) return;
    fName = selected.item_name;
    fQuantity = selected.quantity;
    fUnit = selected.unit ?? 'units';
    fCategory = selected.category ?? '';
    mode = 'edit';
  }

  async function handleSave() {
    error = ''; success = '';
    try {
      if (mode === 'add') {
        await sanAddInventory({ item_name: fName, quantity: fQuantity, unit: fUnit || undefined, category: fCategory.trim() || undefined });
        success = 'Item added.';
      } else if (mode === 'edit' && selected) {
        await sanUpdateInventory({ item_id: selected.id, quantity: fQuantity });
        success = 'Item updated.';
      }
      mode = 'view'; selected = null;
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }

  async function handleRemove() {
    if (!selected) return;
    error = ''; success = '';
    try {
      await sanRemoveInventory({ item_id: selected.id });
      success = 'Item removed.';
      selected = null; mode = 'view';
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }
</script>

<h1 class="title">Cleaning Inventory</h1>
<p class="subtitle">UC-HS-03 — Add, update, and remove cleaning inventory items.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="grid">
  <div class="list-panel">
    <div class="panel-header">
      <span>Items ({items.length})</span>
      <button class="btn-primary btn-sm" onclick={startAdd}>+ Add</button>
    </div>
    {#each items as item}
      <button class="card" class:card-selected={selected?.id === item.id} onclick={() => select(item)}>
        <div class="card-row">
          <span class="card-name">{item.item_name}</span>
          <span class="card-qty">{item.quantity} {item.unit ?? 'units'}</span>
        </div>
        {#if item.category}<span class="badge-cat">{item.category}</span>{/if}
      </button>
    {:else}
      <p class="empty">No items.</p>
    {/each}
  </div>

  <div class="detail-panel">
    {#if mode === 'view' && selected}
      <h2 class="detail-title">{selected.item_name}</h2>
      <div class="detail-row"><span class="label">Quantity</span><span>{selected.quantity} {selected.unit ?? 'units'}</span></div>
      {#if selected.category}<div class="detail-row"><span class="label">Category</span><span>{selected.category}</span></div>{/if}
      <div class="detail-row"><span class="label">Last Updated</span><span>{new Date(selected.updated_at).toLocaleString()}</span></div>
      <div class="detail-actions">
        <button class="btn-primary" onclick={startEdit}>Edit</button>
        <button class="btn-danger" onclick={handleRemove}>Remove</button>
      </div>
    {:else if mode === 'add' || mode === 'edit'}
      <h2 class="detail-title">{mode === 'add' ? 'New Item' : 'Edit Item'}</h2>
      <div class="form-group"><label for="fn">Name</label><input id="fn" class="input" bind:value={fName} /></div>
      <div class="form-group"><label for="fq">Quantity</label><input id="fq" class="input" type="number" min="0" bind:value={fQuantity} /></div>
      <div class="form-group"><label for="fu">Unit</label><input id="fu" class="input" bind:value={fUnit} placeholder="e.g. liters, packs" /></div>
      <div class="form-group"><label for="fc">Category</label><input id="fc" class="input" bind:value={fCategory} placeholder="e.g. disinfectant" /></div>
      <div class="detail-actions">
        <button class="btn-primary" onclick={handleSave}>Save</button>
        <button class="btn-secondary" onclick={() => { mode='view'; }}>Cancel</button>
      </div>
    {:else}
      <p class="placeholder-text">Select an item or click <strong>+ Add</strong></p>
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .grid { display:grid;grid-template-columns:1fr 1.3fr;gap:0.75rem; }
  .list-panel { display:flex;flex-direction:column;gap:0.35rem;max-height:70vh;overflow-y:auto; }
  .panel-header { display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem;color:#94A3B8;font-size:0.8rem; }
  .detail-panel { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.6rem;cursor:pointer;text-align:left;width:100%;color:#E6EDF3;transition:border-color .15s; }
  .card:hover { border-color:rgba(58,190,255,0.3); }
  .card-selected { border-color:#3ABEFF;background:rgba(58,190,255,0.06); }
  .card-row { display:flex;justify-content:space-between;align-items:center; }
  .card-name { font-size:0.82rem;font-weight:500; }
  .card-qty { font-size:0.75rem;color:#94A3B8; }

  .detail-title { font-size:0.95rem;font-family:'Orbitron',sans-serif;color:#3ABEFF;margin:0 0 0.7rem; }
  .detail-row { display:flex;justify-content:space-between;padding:0.3rem 0;border-bottom:1px solid rgba(58,190,255,0.06);font-size:0.8rem; }
  .label { color:#94A3B8; }
  .detail-actions { display:flex;gap:0.5rem;margin-top:0.8rem; }
  .form-group { display:flex;flex-direction:column;gap:0.15rem;margin-bottom:0.45rem; }
  .form-group label { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .btn-sm { padding:0.25rem 0.5rem;font-size:0.7rem; }
  .btn-primary { padding:0.4rem 0.65rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-secondary { padding:0.4rem 0.65rem;background:transparent;border:1px solid #475569;color:#94A3B8;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-danger { padding:0.4rem 0.65rem;background:rgba(239,68,68,0.15);border:1px solid #EF4444;color:#EF4444;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .placeholder-text { color:#475569;font-size:0.85rem;text-align:center;padding:2rem 0; }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
</style>
