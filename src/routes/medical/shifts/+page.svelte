<!-- /medical/shifts — Staff Roster & Shift Allocation (UC-HOM-01)
     HeadOfMedicine only -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    medGetStaffList, medGetAllShifts, medAllocateShift, medDeleteShift,
    type MedicalStaffMember, type ShiftEntry,
  } from '$lib/stores/medical';

  let staff: MedicalStaffMember[] = $state([]);
  let shifts: ShiftEntry[] = $state([]);
  let selectedStaff: MedicalStaffMember | null = $state(null);
  let error = $state('');
  let success = $state('');

  // Allocation form
  let allocUserId = $state('');
  let allocStart = $state('');
  let allocEnd = $state('');

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = '';
    try {
      [staff, shifts] = await Promise.all([medGetStaffList(), medGetAllShifts()]);
    } catch (e: unknown) { error = String(e); }
  }

  function selectStaffMember(m: MedicalStaffMember) {
    selectedStaff = m;
    allocUserId = m.id;
    error = ''; success = '';
  }

  function staffShifts(userId: string): ShiftEntry[] {
    return shifts.filter((s) => s.user_id === userId);
  }

  function fmtDateTime(iso: string): string {
    return new Date(iso).toLocaleString('en-GB', {
      month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
    });
  }

  async function handleAllocate() {
    if (!allocUserId || !allocStart || !allocEnd) {
      error = 'Staff member, start, and end are required.';
      return;
    }
    error = ''; success = '';
    try {
      await medAllocateShift({
        user_id: allocUserId,
        shift_start: new Date(allocStart).toISOString(),
        shift_end: new Date(allocEnd).toISOString(),
      });
      success = 'Shift allocated & staff notified.';
      allocStart = ''; allocEnd = '';
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }

  async function handleDelete(shiftId: string) {
    if (!confirm('Delete this shift?')) return;
    error = ''; success = '';
    try {
      await medDeleteShift({ shift_id: shiftId });
      success = 'Shift deleted.';
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }
</script>

<h1 class="title">Staff Roster & Shifts</h1>
<p class="subtitle">View medical staff and allocate/manage shifts.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="grid">
  <!-- Staff list -->
  <div class="list-panel">
    <h3>Staff</h3>
    {#each staff as m}
      <button class="card" class:selected={selectedStaff?.id === m.id} onclick={() => selectStaffMember(m)}>
        <div class="card-title">{m.full_name}</div>
        <div class="card-meta">
          {#if m.specialty}<span class="badge badge-spec">{m.specialty}</span>{/if}
          {#if m.base_name}<span>{m.base_name}</span>{/if}
        </div>
      </button>
    {:else}
      <p class="empty">No medical staff found.</p>
    {/each}
  </div>

  <!-- Detail & allocation -->
  <div class="detail-panel">
    {#if selectedStaff}
      <h2>{selectedStaff.full_name}</h2>
      <div class="kv"><span class="k">Specialty:</span><span>{selectedStaff.specialty ?? '—'}</span></div>
      <div class="kv"><span class="k">Base:</span><span>{selectedStaff.base_name ?? '—'}</span></div>

      <!-- Allocate shift form -->
      <div class="form-card">
        <h3>Allocate New Shift</h3>
        <div class="row">
          <div class="form-group" style="flex:1">
            <label for="start">Start</label>
            <input id="start" type="datetime-local" class="input" bind:value={allocStart} />
          </div>
          <div class="form-group" style="flex:1">
            <label for="end">End</label>
            <input id="end" type="datetime-local" class="input" bind:value={allocEnd} />
          </div>
          <div class="form-group" style="flex:0.5;justify-content:flex-end">
            <button class="btn-primary" onclick={handleAllocate}>Allocate</button>
          </div>
        </div>
      </div>

      <!-- Existing shifts -->
      <h3>Assigned Shifts</h3>
      {#each staffShifts(selectedStaff.id) as s}
        <div class="shift-row">
          <span class="shift-time">{fmtDateTime(s.shift_start)} → {fmtDateTime(s.shift_end)}</span>
          <button class="btn-xs-danger" onclick={() => handleDelete(s.id)}>×</button>
        </div>
      {:else}
        <p class="hint">No shifts assigned yet.</p>
      {/each}
    {:else}
      <div class="empty-state"><p>Select a staff member to manage shifts.</p></div>
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .grid { display:flex;gap:1rem;flex:1;overflow:hidden; }
  .list-panel { width:280px;overflow-y:auto; }
  .detail-panel { flex:1;overflow-y:auto; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1rem;color:#3ABEFF;margin:0 0 0.5rem; }
  h3 { font-size:0.85rem;color:#E6EDF3;margin:0.75rem 0 0.4rem; }
  .card { width:100%;text-align:left;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.7rem;margin-bottom:0.4rem;cursor:pointer;color:#E6EDF3; }
  .card:hover { border-color:rgba(58,190,255,0.3); }
  .card.selected { border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .card-title { font-size:0.85rem;font-weight:500;margin-bottom:0.2rem; }
  .card-meta { display:flex;gap:0.5rem;font-size:0.7rem;color:#94A3B8; }
  .kv { display:flex;gap:0.5rem;font-size:0.8rem;margin-bottom:0.3rem; }
  .k { color:#94A3B8;min-width:80px; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-top:0.5rem;display:flex;flex-direction:column;gap:0.5rem; }
  .row { display:flex;gap:0.6rem;align-items:flex-end; }
  .form-group { display:flex;flex-direction:column;gap:0.2rem; }
  .form-group label { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .btn-primary { padding:0.45rem 0.75rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .shift-row { display:flex;justify-content:space-between;align-items:center;background:rgba(14,20,40,0.4);border:1px solid rgba(58,190,255,0.06);border-radius:4px;padding:0.45rem 0.6rem;margin-bottom:0.3rem; }
  .shift-time { font-size:0.8rem;color:#E6EDF3; }
  .btn-xs-danger { background:none;border:none;color:#EF4444;cursor:pointer;font-size:1rem;padding:0 0.3rem; }
  .btn-xs-danger:hover { color:#F87171; }
  .badge { padding:0.12rem 0.4rem;border-radius:4px;font-size:0.65rem; }
  .badge-spec { background:rgba(16,185,129,0.15);color:#10B981; }
  .hint { color:#94A3B8;font-size:0.8rem; }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1.5rem; }
  .empty-state { display:flex;justify-content:center;align-items:center;height:100%;color:#475569; }
</style>
