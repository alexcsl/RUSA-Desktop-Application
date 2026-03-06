<!-- /sanitary/head/scheduling — UC-HS-04: Quarterly Staff Scheduling -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    sanGetStaffRoster, sanGetAllShifts, sanAllocateShift, sanDeleteShift, sanGetDivisions,
    type StaffRosterEntry, type SanitaryShift, type DivisionRow,
  } from '$lib/stores/sanitary';

  let roster: StaffRosterEntry[] = $state([]);
  let shifts: SanitaryShift[] = $state([]);
  let divisions: DivisionRow[] = $state([]);
  let error = $state('');
  let success = $state('');

  // Form
  let fUserId = $state('');
  let fShiftStart = $state('');
  let fShiftEnd = $state('');
  let fQuarter = $state('Q1-2026');

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = ''; success = '';
    try {
      [roster, shifts, divisions] = await Promise.all([
        sanGetStaffRoster(),
        sanGetAllShifts(),
        sanGetDivisions(),
      ]);
    } catch (e: unknown) { error = String(e); }
  }

  async function handleAllocate() {
    if (!fUserId || !fShiftStart || !fShiftEnd) { error = 'Staff, start, and end times are required.'; return; }
    error = ''; success = '';
    try {
      await sanAllocateShift({ user_id: fUserId, shift_start: new Date(fShiftStart).toISOString(), shift_end: new Date(fShiftEnd).toISOString(), quarter: fQuarter });
      success = 'Shift allocated.';
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }

  async function handleDeleteShift(id: string) {
    error = ''; success = '';
    try {
      await sanDeleteShift({ shift_id: id });
      success = 'Shift removed.';
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }
</script>

<h1 class="title">Staff Scheduling</h1>
<p class="subtitle">UC-HS-04 — Allocate quarterly shifts across sanitary divisions.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="form-card">
  <h2>Allocate Shift</h2>
  <div class="row">
    <div class="form-group">
      <label for="staff">Staff</label>
      <select id="staff" class="input" bind:value={fUserId}>
        <option value="">— Select —</option>
        {#each roster as s}
          <option value={s.user_id}>{s.full_name} ({s.role_name})</option>
        {/each}
      </select>
    </div>
    <div class="form-group">
      <label for="quarter">Quarter</label>
      <input id="quarter" class="input" bind:value={fQuarter} placeholder="Q1-2026" />
    </div>
  </div>
  <div class="row" style="margin-top:0.4rem;">
    <div class="form-group">
      <label for="ss">Shift Start</label>
      <input id="ss" class="input" type="datetime-local" bind:value={fShiftStart} />
    </div>
    <div class="form-group">
      <label for="se">Shift End</label>
      <input id="se" class="input" type="datetime-local" bind:value={fShiftEnd} />
    </div>
    <button class="btn-primary" style="align-self:flex-end;" onclick={handleAllocate}>Allocate</button>
  </div>
</div>

<h2 class="section-title">Current Shifts ({shifts.length})</h2>
<div class="table-wrap">
  <table class="data-table">
    <thead>
      <tr><th>Staff</th><th>Quarter</th><th>Start</th><th>End</th><th></th></tr>
    </thead>
    <tbody>
      {#each shifts as sh}
        <tr>
          <td>{sh.staff_name}</td>
          <td>{sh.quarter}</td>
          <td>{new Date(sh.shift_start).toLocaleString()}</td>
          <td>{new Date(sh.shift_end).toLocaleString()}</td>
          <td><button class="btn-icon" onclick={() => handleDeleteShift(sh.id)}>✕</button></td>
        </tr>
      {:else}
        <tr><td colspan="5" class="empty">No shifts scheduled.</td></tr>
      {/each}
    </tbody>
  </table>
</div>

<h2 class="section-title">Staff Roster</h2>
{#each roster as s}
  <div class="roster-card">
    <span class="roster-name">{s.full_name}</span>
    <span class="roster-role">{s.role_name}</span>
    {#if s.division_name}<span class="roster-div">{s.division_name}</span>{/if}
  </div>
{:else}
  <p class="empty">No roster data.</p>
{/each}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .section-title { font-family:'Orbitron',sans-serif;font-size:0.95rem;color:#3ABEFF;margin:1.25rem 0 0.5rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-bottom:1rem; }
  .form-card h2 { font-size:0.9rem;color:#E6EDF3;margin:0 0 0.5rem; }
  .row { display:flex;gap:0.6rem;align-items:flex-end;flex-wrap:wrap; }
  .form-group { display:flex;flex-direction:column;gap:0.15rem;flex:1;min-width:130px; }
  .form-group label { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  select.input { cursor:pointer; }
  .table-wrap { overflow-x:auto; }
  .data-table { width:100%;border-collapse:collapse;font-size:0.78rem; }
  .data-table th { text-align:left;color:#94A3B8;padding:0.35rem 0.5rem;border-bottom:1px solid rgba(58,190,255,0.1); }
  .data-table td { padding:0.35rem 0.5rem;border-bottom:1px solid rgba(58,190,255,0.04); }
  .roster-card { display:flex;gap:0.6rem;align-items:center;padding:0.4rem 0.6rem;background:rgba(14,20,40,0.4);border:1px solid rgba(58,190,255,0.06);border-radius:5px;margin-bottom:0.3rem;font-size:0.8rem; }
  .roster-name { font-weight:500; }
  .roster-role { color:#F59E0B;font-size:0.7rem; }
  .roster-div { color:#94A3B8;font-size:0.7rem;margin-left:auto; }
  .btn-primary { padding:0.4rem 0.7rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-icon { background:none;border:none;color:#EF4444;cursor:pointer;font-size:0.85rem; }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
</style>
