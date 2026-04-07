<!--
  /psychiatry/assistant/schedule — Book Appointment (UC-PA-01) + View Psychiatrist Schedule
  Fixed: auto-derives psychiatrist from assigned relationship — no manual UUID input required.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    psyAssistantGetPatients,
    psyGetAssignedPsychiatrists,
    psyAssistantGetSchedule,
    psyAssistantGetAppointments,
    psyScheduleAppointment,
    type PatientListEntry,
    type AssignedPsychiatrist,
    type ScheduleSlot,
    type AppointmentSummary,
  } from '$lib/stores/psychiatry';

  let patients: PatientListEntry[] = $state([]);
  let assignedPsychiatrists: AssignedPsychiatrist[] = $state([]);
  let patientAppointments: AppointmentSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let success = $state('');

  // Derived from selected patient
  let selectedPatientId = $state('');
  let selectedPsychiatristId = $state('');
  let selectedPsychiatristName = $state('');

  let schedule: ScheduleSlot[] = $state([]);
  let scheduleLoading = $state(false);
  let booking = $state(false);

  onMount(async () => {
    try {
      [patients, assignedPsychiatrists] = await Promise.all([
        psyAssistantGetPatients(),
        psyGetAssignedPsychiatrists(),
      ]);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
    loading = false;
  });

  async function handlePatientSelect(e: Event) {
    const val = (e.target as HTMLSelectElement).value;
    selectedPatientId = val;
    schedule = [];
    patientAppointments = [];
    selectedPsychiatristId = '';
    selectedPsychiatristName = '';
    if (!val) return;

    const patient = patients.find((p) => p.id === val);
    if (!patient) return;

    selectedPsychiatristId = patient.psychiatrist_id;
    selectedPsychiatristName = patient.psychiatrist_name;
    await loadSchedule(patient.psychiatrist_id);

    try { patientAppointments = await psyAssistantGetAppointments(val); }
    catch { patientAppointments = []; }
  }

  async function loadSchedule(psychiatristId: string) {
    scheduleLoading = true;
    error = '';
    try {
      schedule = await psyAssistantGetSchedule(psychiatristId);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
    scheduleLoading = false;
  }

  async function bookAppointment(slotStart: string) {
    if (!selectedPatientId || !selectedPsychiatristId) return;
    booking = true;
    error = '';
    success = '';
    try {
      await psyScheduleAppointment({
        patient_id: selectedPatientId,
        psychiatrist_id: selectedPsychiatristId,
        scheduled_at: slotStart,
      });
      success = 'Appointment booked successfully! The psychiatrist and patient have been notified.';
      await loadSchedule(selectedPsychiatristId);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
    booking = false;
  }
</script>

<div class="page">
  <h2>Book Appointment</h2>

  {#if success}<div class="banner success">{success}</div>{/if}
  {#if error}<div class="banner error">{error}</div>{/if}

  {#if !loading && assignedPsychiatrists.length === 0}
    <div class="banner warn">You are not assigned to any psychiatrist. Contact your administrator.</div>
  {/if}

  {#if assignedPsychiatrists.length > 0}
    <section class="card">
      <h3>Your Assigned Psychiatrist(s)</h3>
      <div class="doc-list">
        {#each assignedPsychiatrists as doc}
          <span class="doc-tag">Dr. {doc.full_name}</span>
        {/each}
      </div>
    </section>
  {/if}

  <section class="card">
    <h3>1. Select Patient</h3>
    {#if loading}
      <p class="muted">Loading…</p>
    {:else if patients.length === 0}
      <p class="muted">No patients available.</p>
    {:else}
      <select class="input" onchange={handlePatientSelect} value={selectedPatientId}>
        <option value="">— Pick a patient —</option>
        {#each patients as p}
          <option value={p.id}>{p.patient_name} ({p.care_status}) — Dr. {p.psychiatrist_name}</option>
        {/each}
      </select>
      {#if selectedPsychiatristName}
        <p class="sub-info">Psychiatrist: <strong>{selectedPsychiatristName}</strong></p>
      {/if}
    {/if}
  </section>

  {#if selectedPatientId}
    <section class="card">
      <h3>Patient's Existing Appointments <span class="note">(requires schedule access)</span></h3>
      {#if patientAppointments.length === 0}
        <p class="muted">No existing appointments or schedule access not granted.</p>
      {:else}
        <table class="tbl">
          <thead><tr><th>Date / Time</th><th>Status</th></tr></thead>
          <tbody>
            {#each patientAppointments as a}
              <tr>
                <td>{new Date(a.scheduled_at).toLocaleString()}</td>
                <td>
                  <span class="badge {a.status === 'scheduled' ? 'badge-avail' : a.status === 'cancelled' ? 'badge-cancelled' : 'badge-blocked'}">{a.status}</span>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>

    <section class="card">
      <h3>2. Available Slots — Dr. {selectedPsychiatristName}</h3>
      {#if scheduleLoading}
        <p class="muted">Loading schedule…</p>
      {:else if schedule.length === 0}
        <p class="muted">No schedule slots available. Ask the psychiatrist to add slots first.</p>
      {:else}
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
                    <button
                      class="btn-book"
                      onclick={() => bookAppointment(s.slot_start)}
                      disabled={booking}
                    >
                      {booking ? 'Booking…' : 'Book'}
                    </button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </section>
  {/if}
</div>

<style>
  .page { max-width:800px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#3ABEFF;margin-bottom:1rem; }
  h3 { font-family:'Orbitron',sans-serif;font-size:0.85rem;color:#C084FC;margin-bottom:0.6rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .sub-info { font-size:0.78rem;color:#94A3B8;margin-top:0.4rem; }
  .sub-info strong { color:#C084FC; }
  .card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-bottom:1rem; }
  .banner { padding:0.65rem 1rem;border-radius:6px;margin-bottom:1rem;font-size:0.82rem; }
  .banner.success { background:rgba(16,185,129,0.12);border:1px solid rgba(16,185,129,0.3);color:#34D399; }
  .banner.error { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);color:#F87171; }
  .banner.warn { background:rgba(245,158,11,0.1);border:1px solid rgba(245,158,11,0.3);color:#FBBF24; }
  .doc-list { display:flex;flex-wrap:wrap;gap:0.4rem; }
  .doc-tag { background:rgba(139,92,246,0.15);color:#C084FC;border:1px solid rgba(139,92,246,0.3);border-radius:12px;padding:0.2rem 0.7rem;font-size:0.78rem; }
  .input { width:100%;padding:0.45rem 0.6rem;background:#1F2937;color:#E6EDF3;border:1px solid #374151;border-radius:4px;font-size:0.8rem;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.5rem 0.6rem;color:#94A3B8;border-bottom:1px solid rgba(58,190,255,0.1); }
  .tbl td { padding:0.5rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .badge { padding:0.15rem 0.5rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .badge-avail { background:rgba(16,185,129,0.15);color:#34D399; }
  .badge-blocked { background:rgba(245,158,11,0.15);color:#FBBF24; }
  .badge-cancelled { background:rgba(239,68,68,0.15);color:#EF4444; }
  .note { font-size:0.7rem;color:#94A3B8;font-family:'Inter',sans-serif;font-weight:400; }
  .btn-book { padding:0.25rem 0.7rem;background:rgba(139,92,246,0.15);color:#C084FC;border:1px solid rgba(139,92,246,0.4);border-radius:4px;cursor:pointer;font-size:0.75rem; }
  .btn-book:hover:not(:disabled) { background:rgba(139,92,246,0.3); }
  .btn-book:disabled { opacity:0.5;cursor:not-allowed; }
</style>
