<!-- /station/overview — Space Station Settler landing dashboard -->
<script lang="ts">
  import { getContext, onMount } from 'svelte';
  import type { Writable } from 'svelte/store';
  import { sstGetInventory, sstGetPersonnel, sstGetSupplyRequests, type InventoryItem, type PersonnelLogEntry, type SupplyRequestSummary } from '$lib/stores/space_station';

  const stationIdStore = getContext<Writable<string>>('selectedStationId');

  let currentStationId = $state('');
  let inventory: InventoryItem[] = $state([]);
  let personnel: PersonnelLogEntry[] = $state([]);
  let supplyRequests: SupplyRequestSummary[] = $state([]);
  let loading = $state(true);

  const unsub = stationIdStore.subscribe((v) => {
    currentStationId = v;
    if (v) loadAll();
  });

  async function loadAll() {
    if (!currentStationId) return;
    loading = true;
    try {
      [inventory, personnel, supplyRequests] = await Promise.all([
        sstGetInventory(currentStationId),
        sstGetPersonnel(currentStationId, false),
        sstGetSupplyRequests(currentStationId),
      ]);
    } catch {}
    loading = false;
  }

  onMount(() => { if (currentStationId) loadAll(); });

  let onBoardCount = $derived(personnel.filter((p) => !p.departed_at).length);
  let lowStockCount = $derived(inventory.filter((i) => i.quantity <= 5).length);
  let pendingSupply = $derived(supplyRequests.filter((r) => r.status === 'pending_approval').length);
</script>

<h1 class="title">Station Overview</h1>
<p class="subtitle">Welcome back. Here's a snapshot of your station's current status.</p>

{#if loading}
  <p class="muted">Loading dashboard…</p>
{:else}
  <div class="stat-grid">
    <div class="stat-card">
      <div class="stat-value">{onBoardCount}</div>
      <div class="stat-label">Personnel On Board</div>
    </div>
    <div class="stat-card" class:warn={lowStockCount > 0}>
      <div class="stat-value">{lowStockCount}</div>
      <div class="stat-label">Low Stock Items</div>
    </div>
    <div class="stat-card">
      <div class="stat-value">{inventory.length}</div>
      <div class="stat-label">Inventory Items</div>
    </div>
    <div class="stat-card" class:warn={pendingSupply > 0}>
      <div class="stat-value">{pendingSupply}</div>
      <div class="stat-label">Pending Supply Requests</div>
    </div>
  </div>

  {#if personnel.length > 0}
    <div class="section-card">
      <h2>Currently On Board</h2>
      <table class="mini-table">
        <thead><tr><th>Name</th><th>Role</th><th>Arrived</th></tr></thead>
        <tbody>
          {#each personnel.slice(0, 5) as p}
            <tr>
              <td>{p.person_name}</td>
              <td class="muted">{p.role ?? '—'}</td>
              <td class="muted small">{new Date(p.arrived_at).toLocaleDateString()}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      {#if personnel.length > 5}<p class="muted small">+{personnel.length - 5} more — see Personnel On-Board.</p>{/if}
    </div>
  {/if}
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1.5rem; }
  .muted { color:#94A3B8; }
  .small { font-size:0.75rem; }
  .stat-grid { display:grid;grid-template-columns:repeat(auto-fill,minmax(160px,1fr));gap:1rem;margin-bottom:1.5rem; }
  .stat-card { background:rgba(14,20,40,0.7);border:1px solid rgba(58,190,255,0.12);border-radius:8px;padding:1rem 1.25rem;text-align:center; }
  .stat-card.warn { border-color:rgba(245,158,11,0.3);background:rgba(245,158,11,0.05); }
  .stat-value { font-family:'Orbitron',sans-serif;font-size:1.8rem;color:#3ABEFF;font-weight:700; }
  .stat-card.warn .stat-value { color:#F59E0B; }
  .stat-label { font-size:0.72rem;color:#94A3B8;margin-top:0.25rem;text-transform:uppercase;letter-spacing:0.04em; }
  .section-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem 1.25rem; }
  .section-card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#3ABEFF;margin:0 0 0.75rem; }
  .mini-table { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .mini-table th { text-align:left;color:#94A3B8;font-size:0.7rem;text-transform:uppercase;padding:0.3rem 0.5rem;border-bottom:1px solid rgba(255,255,255,0.06); }
  .mini-table td { padding:0.35rem 0.5rem;border-bottom:1px solid rgba(255,255,255,0.03);color:#E6EDF3; }
</style>
