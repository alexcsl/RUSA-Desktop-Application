<!-- /sanitary/inventory/log — UC-STAS-04: Log Inventory Actions -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    sanGetInventory, sanLogInventoryAction, sanGetInventoryLogs,
    type SanitaryInventoryItem, type InventoryLogEntry,
  } from '$lib/stores/sanitary';

  let items: SanitaryInventoryItem[] = $state([]);
  let logs: InventoryLogEntry[] = $state([]);
  let error = $state('');
  let success = $state('');

  let fItemId = $state('');
  let fAction: 'add' | 'remove' = $state('remove');
  let fQuantity: number = $state(1);
  let fPurchaseNote = $state('');

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = '';
    try {
      [items, logs] = await Promise.all([sanGetInventory(), sanGetInventoryLogs()]);
    } catch (e: unknown) { error = String(e); }
  }

  async function handleLog() {
    if (!fItemId) { error = 'Select an item.'; return; }
    if (fQuantity <= 0) { error = 'Quantity must be positive.'; return; }
    error = ''; success = '';
    try {
      await sanLogInventoryAction({
        item_id: fItemId,
        action: fAction,
        quantity: fQuantity,
        purchase_note: fPurchaseNote.trim() || undefined,
      });
      success = `Logged ${fAction} of ${fQuantity}.`;
      fQuantity = 1; fPurchaseNote = '';
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }
</script>

<h1 class="title">Inventory Log</h1>
<p class="subtitle">UC-STAS-04 — Log usage or restocking of cleaning inventory items.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="form-card">
  <div class="row">
    <div class="form-group" style="flex:2;">
      <label for="item">Item</label>
      <select id="item" class="input" bind:value={fItemId}>
        <option value="">— Select item —</option>
        {#each items as it}
          <option value={it.id}>{it.item_name} ({it.quantity} {it.unit ?? ''})</option>
        {/each}
      </select>
    </div>
    <div class="form-group" style="max-width:120px;">
      <label for="action">Action</label>
      <select id="action" class="input" bind:value={fAction}>
        <option value="remove">Remove</option>
        <option value="add">Add</option>
      </select>
    </div>
    <div class="form-group" style="max-width:90px;">
      <label for="qty">Qty</label>
      <input id="qty" class="input" type="number" min="1" bind:value={fQuantity} />
    </div>
  </div>
  <div class="form-group" style="margin-top:0.4rem;">
    <label for="pnote">Purchase Note</label>
    <input id="pnote" class="input" bind:value={fPurchaseNote} placeholder="Optional note…" />
  </div>
  <button class="btn-primary" style="margin-top:0.5rem;" onclick={handleLog}>Log Action</button>
</div>

<h2 class="section-title">Recent Logs ({logs.length})</h2>
<div class="log-list">
  {#each logs as l}
    <div class="log-entry">
      <span class="log-action" class:add={l.action === 'add'} class:remove={l.action === 'remove'}>
        {l.action === 'add' ? '+' : '−'}{l.quantity}
      </span>
      <span class="log-item">{l.item_name ?? l.item_id.slice(0, 8)}</span>
      <span class="log-user">{l.logger_name ?? l.logged_by.slice(0, 8)}</span>
      <span class="log-date">{new Date(l.logged_at).toLocaleString()}</span>
      {#if l.purchase_note}<span class="log-notes">({l.purchase_note})</span>{/if}
    </div>
  {:else}
    <p class="empty">No logs yet.</p>
  {/each}
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .section-title { font-family:'Orbitron',sans-serif;font-size:0.95rem;color:#3ABEFF;margin:1.25rem 0 0.5rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-bottom:1rem; }
  .row { display:flex;gap:0.5rem;align-items:flex-end;flex-wrap:wrap; }
  .form-group { display:flex;flex-direction:column;gap:0.15rem;flex:1;min-width:80px; }
  .form-group label { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  select.input { cursor:pointer; }
  .log-list { display:flex;flex-direction:column;gap:0.3rem; }
  .log-entry { display:flex;gap:0.6rem;align-items:center;font-size:0.78rem;padding:0.4rem 0.5rem;background:rgba(14,20,40,0.4);border:1px solid rgba(58,190,255,0.04);border-radius:5px; }
  .log-action { font-weight:700;min-width:40px;text-align:center; }
  .log-action.add { color:#10B981; }
  .log-action.remove { color:#EF4444; }
  .log-item { color:#E6EDF3;font-weight:500; }
  .log-user { color:#94A3B8;font-size:0.7rem;margin-left:auto; }
  .log-date { color:#64748B;font-size:0.68rem; }
  .log-notes { color:#94A3B8;font-size:0.68rem;font-style:italic; }
  .btn-primary { padding:0.45rem 0.75rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
</style>
