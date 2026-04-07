<!--
  /me/appointments — Any authenticated user: view own psychiatric appointments.
  Shows upcoming and past sessions booked by psychiatrists or their assistants.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { psyPatientGetMyAppointments, type PatientAppointmentView } from '$lib/stores/psychiatry';

  let appointments: PatientAppointmentView[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try { appointments = await psyPatientGetMyAppointments(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    loading = false;
  });
</script>

<div class="page">
  <h2>My Appointments</h2>
  <p class="sub">Psychiatric sessions scheduled for you. Booked by your assigned psychiatrist or their assistant.</p>

  {#if error}<div class="banner error">{error}</div>{/if}

  {#if loading}
    <p class="muted">Loading…</p>
  {:else if appointments.length === 0}
    <div class="empty">
      <p class="muted">No appointments scheduled.</p>
      <p class="muted small">If you believe you should have sessions, contact the psychiatry department.</p>
    </div>
  {:else}
    <div class="list">
      {#each appointments as a}
        <div class="appt-card" class:upcoming={a.status === 'scheduled'} class:completed={a.status === 'completed'}>
          <div class="appt-date">
            <span class="day">{new Date(a.scheduled_at).toLocaleDateString(undefined, {weekday:'short',month:'short',day:'numeric'})}</span>
            <span class="time">{new Date(a.scheduled_at).toLocaleTimeString(undefined, {hour:'2-digit',minute:'2-digit'})}</span>
          </div>
          <div class="appt-info">
            <span class="doc">Dr. {a.psychiatrist_name}</span>
            <span class="badge {a.status === 'completed' ? 'badge-done' : a.status === 'cancelled' ? 'badge-cancelled' : 'badge-scheduled'}">{a.status}</span>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page { max-width:680px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .sub { font-size:0.8rem;color:#94A3B8;margin:0 0 1.25rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .small { font-size:0.75rem; }
  .banner { padding:0.6rem 0.9rem;border-radius:6px;font-size:0.8rem;margin-bottom:0.8rem; }
  .banner.error { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);color:#F87171; }
  .empty { background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:2rem;text-align:center;display:flex;flex-direction:column;gap:0.4rem; }
  .list { display:flex;flex-direction:column;gap:0.4rem; }
  .appt-card { display:flex;align-items:center;gap:1rem;padding:0.8rem 1rem;background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:8px; }
  .appt-card.upcoming { border-left:3px solid #3ABEFF; }
  .appt-card.completed { border-left:3px solid #34D399;opacity:0.75; }
  .appt-date { display:flex;flex-direction:column;align-items:center;min-width:70px;background:#0B0F1A;border-radius:6px;padding:0.4rem 0.6rem; }
  .day { font-size:0.75rem;color:#94A3B8; }
  .time { font-size:0.85rem;color:#E6EDF3;font-weight:600; }
  .appt-info { flex:1;display:flex;align-items:center;gap:0.6rem; }
  .doc { font-size:0.85rem;color:#E6EDF3; }
  .badge { padding:0.15rem 0.5rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .badge-scheduled { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .badge-done { background:rgba(16,185,129,0.15);color:#34D399; }
  .badge-cancelled { background:rgba(239,68,68,0.15);color:#EF4444; }
</style>
