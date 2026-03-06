<!-- /station/personnel — UC-SSS-06: Log Personnel On Board -->
<script lang="ts">
  import { getContext, onMount } from 'svelte';
  import type { Writable } from 'svelte/store';
  import { sstLogArrival, sstLogDeparture, sstGetPersonnel, type PersonnelLogEntry } from '$lib/stores/space_station';

  const stationId = getContext<Writable<string>>('selectedStationId');

  let entries: PersonnelLogEntry[] = $state([]);
  let loading = $state(true);
  let showAll = $state(false);

  // Arrival form
  let personName = $state('');
  let personRole = $state('');
  let arrivedAt = $state('');
  let formError = $state('');
  let formSuccess = $state('');
  let submitting = $state(false);

  let currentStationId = $state('');
  const unsub = stationId.subscribe((v) => {
    currentStationId = v;
    if (v) loadPersonnel();
  });

  async function loadPersonnel() {
    if (!currentStationId) return;
    loading = true;
    try {
      entries = await sstGetPersonnel(currentStationId, showAll);
    } catch {}
    loading = false;
  }

  onMount(() => { if (currentStationId) loadPersonnel(); });

  $effect(() => {
    showAll; // track
    if (currentStationId) loadPersonnel();
  });

  async function handleLogArrival() {
    formError = ''; formSuccess = '';
    if (!currentStationId) { formError = 'No station selected.'; return; }
    if (!personName.trim()) { formError = 'Person name is required.'; return; }
    if (!arrivedAt) { formError = 'Arrival date is required.'; return; }

    submitting = true;
    try {
      await sstLogArrival({
        station_id: currentStationId,
        person_name: personName,
        role: personRole || undefined,
        arrived_at: new Date(arrivedAt).toISOString(),
      });
      formSuccess = `${personName} logged as arrived.`;
      personName = ''; personRole = ''; arrivedAt = '';
      await loadPersonnel();
    } catch (e: unknown) {
      formError = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  async function handleDeparture(id: string) {
    try {
      await sstLogDeparture(id, new Date().toISOString());
      await loadPersonnel();
    } catch {}
  }

  let activeCount = $derived(entries.filter((e) => !e.departed_at).length);
</script>

<h1 class="title">Personnel Log</h1>
<p class="subtitle">Track who is currently on board. Log arrivals and departures.</p>

<div class="two-col">
  <!-- Left: Log arrival -->
  <div class="form-card">
    <h2>Log Arrival</h2>

    <label class="field"><span class="label">Person Name *</span>
      <input type="text" class="input" bind:value={personName} placeholder="e.g. Dr. Alice Carter" />
    </label>

    <label class="field"><span class="label">Role</span>
      <input type="text" class="input" bind:value={personRole} placeholder="e.g. Engineer, Scientist…" />
    </label>

    <label class="field"><span class="label">Arrived At *</span>
      <input type="datetime-local" class="input" bind:value={arrivedAt} />
    </label>

    {#if formError}<p class="error">{formError}</p>{/if}
    {#if formSuccess}<p class="success">{formSuccess}</p>{/if}

    <button class="btn-primary" onclick={handleLogArrival} disabled={submitting}>
      {submitting ? 'Logging…' : 'Log Arrival'}
    </button>
  </div>

  <!-- Right: Personnel list -->
  <div class="list-section">
    <div class="list-controls">
      <span class="active-count">On Board: {activeCount}</span>
      <label class="toggle">
        <input type="checkbox" bind:checked={showAll} />
        <span>Show departed</span>
      </label>
    </div>

    {#if loading}
      <p class="muted">Loading personnel…</p>
    {:else if entries.length === 0}
      <p class="muted">No personnel records.</p>
    {:else}
      <div class="pers-table">
        <div class="pers-header-row">
          <span class="col-name">Name</span>
          <span class="col-role">Role</span>
          <span class="col-arrived">Arrived</span>
          <span class="col-status">Status</span>
          <span class="col-action"></span>
        </div>
        {#each entries as p}
          <div class="pers-row" class:departed={!!p.departed_at}>
            <span class="col-name">{p.person_name}</span>
            <span class="col-role">{p.role ?? '—'}</span>
            <span class="col-arrived">{new Date(p.arrived_at).toLocaleDateString()}</span>
            <span class="col-status">
              {#if p.departed_at}
                <span class="status-departed">Departed {new Date(p.departed_at).toLocaleDateString()}</span>
              {:else}
                <span class="status-aboard">On Board</span>
              {/if}
            </span>
            <span class="col-action">
              {#if !p.departed_at}
                <button class="btn-depart" onclick={() => handleDeparture(p.id)}>Mark Departed</button>
              {/if}
            </span>
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
  .list-section { flex:1;min-width:400px; }
  .list-controls { display:flex;justify-content:space-between;align-items:center;margin-bottom:0.6rem; }
  .active-count { font-size:0.85rem;color:#10B981;font-weight:600; }
  .toggle { display:flex;align-items:center;gap:0.3rem;font-size:0.75rem;color:#94A3B8;cursor:pointer; }
  .toggle input { accent-color:#3ABEFF; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .pers-table { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;overflow:hidden; }
  .pers-header-row { display:flex;padding:0.5rem 0.75rem;background:rgba(58,190,255,0.05);font-size:0.7rem;color:#94A3B8;font-weight:600;text-transform:uppercase;letter-spacing:0.5px; }
  .pers-row { display:flex;padding:0.5rem 0.75rem;border-top:1px solid rgba(255,255,255,0.04);font-size:0.8rem;align-items:center; }
  .pers-row:hover { background:rgba(58,190,255,0.03); }
  .pers-row.departed { opacity:0.6; }
  .col-name { flex:2; }
  .col-role { flex:1.5;color:#94A3B8; }
  .col-arrived { flex:1;color:#94A3B8;font-size:0.75rem; }
  .col-status { flex:1.5; }
  .col-action { flex:1;text-align:right; }
  .status-aboard { color:#10B981;font-size:0.75rem;font-weight:600; }
  .status-departed { color:#94A3B8;font-size:0.75rem; }
  .btn-depart { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.25);color:#EF4444;border-radius:4px;padding:0.2rem 0.5rem;font-size:0.7rem;cursor:pointer; }
  .btn-depart:hover { background:rgba(239,68,68,0.2); }
</style>
