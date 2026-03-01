<!--
  /psychiatry/assistant/schedule — Book Appointment (UC-PA-01)
  Assistant views psychiatrist schedule and books appointments.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    psyAssistantGetPatients,
    psyAssistantGetSchedule,
    psyScheduleAppointment,
    type PatientListEntry,
    type ScheduleSlot,
  } from '$lib/stores/psychiatry';

  let patients: PatientListEntry[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let success = $state('');

  // We need a way to discover psychiatrist IDs.
  // In real app, an endpoint would list psychiatrists.
  // For now, once a patient is selected, we get psychiatrist from their record.
  let psychiatristId = $state('');
  let schedule: ScheduleSlot[] = $state([]);
  let scheduleLoading = $state(false);

  // Booking form
  let selectedPatientId = $state('');
  let selectedSlotTime = $state('');
  let booking = $state(false);

  onMount(async () => {
    try { patients = await psyAssistantGetPatients(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    loading = false;
  });

  async function loadPsychiatristSchedule() {
    if (!psychiatristId) return;
    scheduleLoading = true;
    try { schedule = await psyAssistantGetSchedule(psychiatristId); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    scheduleLoading = false;
  }

  async function bookAppointment(slotStart: string) {
    if (!selectedPatientId || !psychiatristId) { error = 'Select a patient and enter psychiatrist ID.'; return; }
    booking = true;
    error = '';
    success = '';
    try {
      await psyScheduleAppointment({
        patient_id: selectedPatientId,
        psychiatrist_id: psychiatristId,
        scheduled_at: slotStart,
      });
      success = 'Appointment booked successfully!';
      await loadPsychiatristSchedule();
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    booking = false;
  }
</script>

<div class="page">
  <h2>Book Appointment</h2>

  {#if success}<div class="toast-success">{success}</div>{/if}
  {#if error}<div class="toast-error">{error}</div>{/if}

  <section class="card">
    <h3>1. Select Patient</h3>
    {#if loading}
      <p class="muted">Loading…</p>
    {:else}
      <select class="input" bind:value={selectedPatientId}>
        <option value="">— Pick a patient —</option>
        {#each patients as p}
          <option value={p.id}>{p.patient_name} ({p.care_status})</option>
        {/each}
      </select>
    {/if}
  </section>

  <section class="card">
    <h3>2. Load Psychiatrist Schedule</h3>
    <label class="field-label">Psychiatrist ID
      <input class="input" type="text" bind:value={psychiatristId} placeholder="UUID of the psychiatrist" />
    </label>
    <button class="btn-accent-sm" onclick={loadPsychiatristSchedule} disabled={!psychiatristId || scheduleLoading}>
      {scheduleLoading ? 'Loading…' : 'Load Schedule'}
    </button>
  </section>

  {#if schedule.length > 0}
    <section class="card">
      <h3>3. Pick Available Slot</h3>
      <table class="tbl">
        <thead><tr><th>Start</th><th>End</th><th>Status</th><th></th></tr></thead>
        <tbody>
          {#each schedule as s}
            <tr>
              <td>{new Date(s.slot_start).toLocaleString()}</td>
              <td>{new Date(s.slot_end).toLocaleString()}</td>
              <td>
                {#if s.is_available}
                  <span class="badge badge-avail">Open</span>
                {:else}
                  <span class="badge badge-blocked">Blocked</span>
                {/if}
              </td>
              <td>
                {#if s.is_available}
                  <button class="btn-primary-sm" onclick={() => bookAppointment(s.slot_start)} disabled={booking || !selectedPatientId}>
                    Book
                  </button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>
  {/if}
</div>

<style>
  .page { max-width:800px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#3ABEFF;margin-bottom:1rem; }
  h3 { font-family:'Orbitron',sans-serif;font-size:0.85rem;color:#C084FC;margin-bottom:0.6rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-bottom:1rem; }
  .input { width:100%;padding:0.45rem 0.6rem;background:#1F2937;color:#E6EDF3;border:1px solid #374151;border-radius:4px;font-size:0.8rem;margin-top:0.2rem;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .field-label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.6rem; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.5rem 0.6rem;color:#94A3B8;border-bottom:1px solid rgba(58,190,255,0.1); }
  .tbl td { padding:0.5rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .badge { padding:0.15rem 0.5rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .badge-avail { background:rgba(16,185,129,0.15);color:#34D399; }
  .badge-blocked { background:rgba(245,158,11,0.15);color:#FBBF24; }
  .btn-accent-sm { background:rgba(58,190,255,0.1);color:#3ABEFF;border:1px solid rgba(58,190,255,0.3);border-radius:4px;padding:0.25rem 0.6rem;cursor:pointer;font-size:0.7rem;margin-top:0.3rem; }
  .btn-accent-sm:hover { background:rgba(58,190,255,0.2); }
  .btn-primary-sm { padding:0.25rem 0.6rem;background:rgba(58,190,255,0.15);color:#3ABEFF;border:1px solid rgba(58,190,255,0.4);border-radius:4px;cursor:pointer;font-size:0.7rem; }
  .btn-primary-sm:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-primary-sm:disabled { opacity:0.5;cursor:not-allowed; }
  .toast-success { background:rgba(16,185,129,0.1);border:1px solid rgba(16,185,129,0.3);color:#34D399;padding:0.5rem 0.8rem;border-radius:6px;font-size:0.8rem;margin-bottom:0.8rem; }
  .toast-error { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);color:#EF4444;padding:0.5rem 0.8rem;border-radius:6px;font-size:0.8rem;margin-bottom:0.8rem; }
</style>
