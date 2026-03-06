<!--
  UC-PS-06: View Settlement Inventory (read-only for general settlers)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { stlGetSettlementInventory, type InventoryItem } from '$lib/stores/settlers';

  let items: InventoryItem[] = $state([]);
  let search = $state('');
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try { items = await stlGetSettlementInventory(); }
    catch (e: any) { error = e?.message ?? String(e); }
    finally { loading = false; }
  });

  let filtered = $derived(
    items.filter((i) => {
      const q = search.toLowerCase();
      return !q || i.item_name.toLowerCase().includes(q) || (i.category ?? '').toLowerCase().includes(q);
    })
  );
</script>

<h2>Settlement Inventory</h2>

<div class="bar">
  <input type="text" class="search" placeholder="Search by name or category…" bind:value={search} />
  <span class="dim">{filtered.length} item(s)</span>
</div>

{#if loading}
  <p class="dim">Loading…</p>
{:else if error}
  <p class="err">{error}</p>
{:else if filtered.length === 0}
  <p class="dim">No inventory items found.</p>
{:else}
  <table class="tbl">
    <thead>
      <tr><th>Item</th><th>Category</th><th>Qty</th><th>Unit</th><th>Min Threshold</th><th>Updated</th></tr>
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
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
  .bar { display:flex;align-items:center;gap:1rem;margin-bottom:0.75rem; }
  .search { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.35rem 0.5rem;font-size:0.8rem;width:260px; }
  .dim { color:#64748B;font-size:0.8rem; }
  .err { color:#EF4444;font-size:0.8rem; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.4rem 0.6rem;border-bottom:1px solid #374151;color:#94A3B8;font-weight:500; }
  .tbl td { padding:0.4rem 0.6rem;border-bottom:1px solid rgba(55,65,81,0.4); }
  .badge { display:inline-block;padding:0.15rem 0.55rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .cat { background:rgba(139,92,246,0.15);color:#A78BFA; }
  .low { color:#EF4444;font-weight:600; }
</style>
