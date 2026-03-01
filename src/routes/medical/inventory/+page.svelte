<!-- /medical/inventory — Medical Inventory Management (UC-MED-03) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    medGetInventory, medAddInventory, medUpdateInventory, medRemoveInventory,
    type InventoryItem, type AddInventoryPayload, type UpdateInventoryPayload,
  } from '$lib/stores/medical';
  import { currentUser } from '$lib/stores/auth';

  let items: InventoryItem[] = $state([]);
  let selected: InventoryItem | null = $state(null);
  let error = $state('');
  let success = $state('');
  let isHead = $state(false);

  // Add form
  let showAdd = $state(false);
  let addName = $state('');
  let addType = $state('medicine');
  let addQty = $state(1);
  let addUnit = $state('');

  // Update form
  let editQty = $state(0);

  currentUser.subscribe((u) => { isHead = u?.role === 'HeadOfMedicine'; });

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = '';
    try { items = await medGetInventory(); } catch (e: unknown) { error = String(e); }
  }

  async function handleAdd() {
    if (!addName.trim()) { error = 'Item name is required.'; return; }
    error = ''; success = '';
    try {
      const payload: AddInventoryPayload = {
        item_name: addName,
        item_type: addType,
        quantity: addQty,
        unit: addUnit.trim() || undefined,
      };
      await medAddInventory(payload);
      success = `Added "${addName}".`;
      addName = ''; addQty = 1; addUnit = ''; showAdd = false;
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }

  async function handleUpdate() {
    if (!selected) return;
    error = ''; success = '';
    try {
      const payload: UpdateInventoryPayload = { item_id: selected.id, quantity: editQty };
      await medUpdateInventory(payload);
      success = `Updated quantity of "${selected.item_name}".`;
      selected = null;
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }

  async function handleRemove(item: InventoryItem) {
    if (!confirm(`Remove "${item.item_name}" from inventory?`)) return;
    error = ''; success = '';
    try {
      await medRemoveInventory({ item_id: item.id });
      success = `Removed "${item.item_name}".`;
      if (selected?.id === item.id) selected = null;
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }

  function selectItem(item: InventoryItem) {
    selected = item;
    editQty = item.quantity;
    error = ''; success = '';
  }

  function badgeType(t: string): string {
    const m: Record<string, string> = { medicine: 'med', equipment: 'equip', supply: 'sup' };
    return m[t] ?? 'default';
  }
</script>

<h1 class="title">Medical Inventory</h1>
<p class="subtitle">Track medicines, equipment, and supplies.</p>

<div class="toolbar">
  <button class="btn-primary" onclick={() => (showAdd = !showAdd)}>
    {showAdd ? 'Cancel' : '+ Add Item'}
  </button>
</div>

{#if showAdd}
  <div class="form-card">
    <div class="row">
      <div class="form-group" style="flex:2">
        <label for="aname">Item Name</label>
        <input id="aname" class="input" bind:value={addName} placeholder="e.g. Ibuprofen 400 mg" />
      </div>
      <div class="form-group" style="flex:1">
        <label for="atype">Type</label>
        <select id="atype" class="input" bind:value={addType}>
          <option value="medicine">Medicine</option>
          <option value="equipment">Equipment</option>
          <option value="supply">Supply</option>
        </select>
      </div>
      <div class="form-group" style="flex:0.5">
        <label for="aqty">Qty</label>
        <input id="aqty" type="number" class="input" min="0" bind:value={addQty} />
      </div>
      <div class="form-group" style="flex:0.5">
        <label for="aunit">Unit</label>
        <input id="aunit" class="input" bind:value={addUnit} placeholder="box" />
      </div>
    </div>
    <button class="btn-primary" onclick={handleAdd}>Add to Inventory</button>
  </div>
{/if}

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="grid">
  <div class="list-panel">
    {#each items as item}
      <button class="card" class:selected={selected?.id === item.id} onclick={() => selectItem(item)}>
        <div class="card-title">{item.item_name}</div>
        <div class="card-meta">
          <span class="badge badge-{badgeType(item.item_type)}">{item.item_type}</span>
          <span>Qty: {item.quantity}{item.unit ? ` ${item.unit}` : ''}</span>
        </div>
      </button>
    {:else}
      <p class="empty">Inventory is empty.</p>
    {/each}
  </div>

  <div class="detail-panel">
    {#if selected}
      <h2>{selected.item_name}</h2>
      <div class="kv"><span class="k">Type:</span><span class="badge badge-{badgeType(selected.item_type)}">{selected.item_type}</span></div>
      <div class="kv"><span class="k">Quantity:</span><span>{selected.quantity}{selected.unit ? ` ${selected.unit}` : ''}</span></div>
      <div class="kv"><span class="k">Updated:</span><span>{new Date(selected.updated_at).toLocaleString()}</span></div>

      <div class="update-form">
        <h3>Update Quantity</h3>
        <div class="row">
          <input type="number" class="input" min="0" bind:value={editQty} style="width:100px" />
          <button class="btn-primary" onclick={handleUpdate}>Update</button>
          <button class="btn-danger" onclick={() => handleRemove(selected!)}>Remove</button>
        </div>
      </div>
    {:else}
      <div class="empty-state"><p>Select an item to view/edit.</p></div>
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .toolbar { margin-bottom:0.75rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;display:flex;flex-direction:column;gap:0.6rem;margin-bottom:0.75rem; }
  .row { display:flex;gap:0.6rem;align-items:flex-end; }
  .form-group { display:flex;flex-direction:column;gap:0.2rem; }
  .form-group label { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .grid { display:flex;gap:1rem;flex:1;overflow:hidden; }
  .list-panel { width:320px;overflow-y:auto; }
  .detail-panel { flex:1;overflow-y:auto; }
  .card { width:100%;text-align:left;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.7rem;margin-bottom:0.4rem;cursor:pointer;color:#E6EDF3; }
  .card:hover { border-color:rgba(58,190,255,0.3); }
  .card.selected { border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .card-title { font-size:0.85rem;font-weight:500;margin-bottom:0.2rem; }
  .card-meta { display:flex;justify-content:space-between;font-size:0.7rem;color:#94A3B8; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1rem;color:#3ABEFF;margin:0 0 0.75rem; }
  h3 { font-size:0.85rem;color:#E6EDF3;margin:0.75rem 0 0.4rem; }
  .kv { display:flex;gap:0.5rem;font-size:0.8rem;margin-bottom:0.3rem; }
  .k { color:#94A3B8;min-width:80px; }
  .update-form { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.75rem;margin-top:0.75rem; }
  .badge { padding:0.12rem 0.4rem;border-radius:4px;font-size:0.65rem; }
  .badge-med { background:rgba(16,185,129,0.15);color:#10B981; }
  .badge-equip { background:rgba(139,92,246,0.15);color:#C084FC; }
  .badge-sup { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .btn-primary { padding:0.45rem 0.75rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-danger { padding:0.45rem 0.75rem;background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.4);color:#EF4444;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-danger:hover { background:rgba(239,68,68,0.25); }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1.5rem; }
  .empty-state { display:flex;justify-content:center;align-items:center;height:100%;color:#475569; }
</style>
