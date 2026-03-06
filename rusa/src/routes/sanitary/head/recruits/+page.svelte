<!-- /sanitary/head/recruits — UC-HS-08: Assign New Recruit to Division -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    sanAssignRecruit, sanGetStaffRoster, sanGetDivisions,
    type StaffRosterEntry, type DivisionRow,
  } from '$lib/stores/sanitary';

  let roster: StaffRosterEntry[] = $state([]);
  let divisions: DivisionRow[] = $state([]);
  let error = $state('');
  let success = $state('');

  let fUserId = $state('');
  let fDivisionId = $state('');
  let fQuarter = $state('');

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = '';
    try {
      [roster, divisions] = await Promise.all([sanGetStaffRoster(), sanGetDivisions()]);
    } catch (e: unknown) { error = String(e); }
  }

  async function handleAssign() {
    if (!fUserId || !fDivisionId || !fQuarter.trim()) { error = 'Select a staff member, division, and quarter.'; return; }
    error = ''; success = '';
    try {
      await sanAssignRecruit({ user_id: fUserId, division_id: fDivisionId, quarter: fQuarter.trim() });
      success = 'Recruit assigned to division.';
      fUserId = ''; fDivisionId = ''; fQuarter = '';
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }

  // Show unassigned or all staff
  function unassigned(): StaffRosterEntry[] {
    return roster.filter((s) => !s.division_name);
  }
</script>

<h1 class="title">Assign New Recruit</h1>
<p class="subtitle">UC-HS-08 — Place a new sanitary staff member into a division. Quota-checked.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="form-card">
  <div class="row">
    <div class="form-group">
      <label for="staff">Staff</label>
      <select id="staff" class="input" bind:value={fUserId}>
        <option value="">— Select staff —</option>
        {#each roster as s}
          <option value={s.user_id}>
            {s.full_name} ({s.role_name}){s.division_name ? ` — ${s.division_name}` : ' [unassigned]'}
          </option>
        {/each}
      </select>
    </div>
    <div class="form-group">
      <label for="div">Division</label>
      <select id="div" class="input" bind:value={fDivisionId}>
        <option value="">— Select division —</option>
        {#each divisions as d}
          <option value={d.id}>{d.name} (quota: {d.quota})</option>
        {/each}
      </select>
    </div>
    <button class="btn-primary" style="align-self:flex-end;" onclick={handleAssign}>Assign</button>
  </div>
  <div class="row" style="margin-top:0.4rem;">
    <div class="form-group">
      <label for="qtr">Quarter</label>
      <input id="qtr" class="input" bind:value={fQuarter} placeholder="Q1-2026" />
    </div>
  </div>
</div>

<h2 class="section-title">Unassigned Staff ({unassigned().length})</h2>
{#each unassigned() as s}
  <div class="roster-card">
    <span class="roster-name">{s.full_name}</span>
    <span class="roster-role">{s.role_name}</span>
  </div>
{:else}
  <p class="empty">All staff have been assigned.</p>
{/each}

<h2 class="section-title">Division Capacities</h2>
<div class="cap-grid">
  {#each divisions as d}
    <div class="cap-card">
      <span class="cap-name">{d.name}</span>
      <span class="cap-bar">
        <span class="cap-fill" style="width:50%"></span>
      </span>
      <span class="cap-text">Quota: {d.quota}</span>
    </div>
  {/each}
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .section-title { font-family:'Orbitron',sans-serif;font-size:0.95rem;color:#3ABEFF;margin:1.25rem 0 0.5rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-bottom:1rem; }
  .row { display:flex;gap:0.6rem;align-items:flex-end;flex-wrap:wrap; }
  .form-group { display:flex;flex-direction:column;gap:0.15rem;flex:1;min-width:160px; }
  .form-group label { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  select.input { cursor:pointer; }
  .roster-card { display:flex;gap:0.6rem;align-items:center;padding:0.4rem 0.6rem;background:rgba(14,20,40,0.4);border:1px solid rgba(58,190,255,0.06);border-radius:5px;margin-bottom:0.3rem;font-size:0.8rem; }
  .roster-name { font-weight:500;color:#E6EDF3; }
  .roster-role { color:#F59E0B;font-size:0.7rem; }
  .cap-grid { display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:0.5rem; }
  .cap-card { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.06);border-radius:6px;padding:0.6rem;display:flex;flex-direction:column;gap:0.3rem; }
  .cap-name { font-size:0.8rem;font-weight:500;color:#E6EDF3; }
  .cap-bar { height:6px;background:rgba(58,190,255,0.1);border-radius:3px;overflow:hidden; }
  .cap-fill { height:100%;background:#3ABEFF;border-radius:3px;display:block;transition:width .3s; }
  .cap-text { font-size:0.7rem;color:#94A3B8; }
  .btn-primary { padding:0.4rem 0.7rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
</style>
