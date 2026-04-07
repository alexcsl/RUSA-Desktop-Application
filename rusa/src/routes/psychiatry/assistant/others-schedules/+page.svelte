<!--
  /psychiatry/assistant/others-schedules — View appointments of crew members
  who have granted this assistant schedule access (UC-PA-02 follow-up).
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    psyAssistantGetGrantedAppointments,
    type GrantedScheduleAppointment,
  } from '$lib/stores/psychiatry';

  let appointments: GrantedScheduleAppointment[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try {
      appointments = await psyAssistantGetGrantedAppointments();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  // Group by patient for clearer display
  interface PatientGroup {
    patient_user_id: string;
    patient_name: string;
    appointments: GrantedScheduleAppointment[];
  }

  let grouped = $derived((): PatientGroup[] => {
    const map = new Map<string, PatientGroup>();
    for (const a of appointments) {
      if (!map.has(a.patient_user_id)) {
        map.set(a.patient_user_id, { patient_user_id: a.patient_user_id, patient_name: a.patient_name, appointments: [] });
      }
      map.get(a.patient_user_id)!.appointments.push(a);
    }
    return [...map.values()];
  });

  function statusClass(status: string): string {
    if (status === 'scheduled') return 'badge-ok';
    if (status === 'cancelled') return 'badge-cancel';
    return 'badge-other';
  }
</script>

<div class="page">
  <h2>Others' Schedules</h2>
  <p class="hint">
    Showing appointments for crew members who have granted you schedule access.
    To request access from additional patients, use <a href="/psychiatry/assistant/access-request" class="link">Schedule Request</a>.
  </p>

  {#if error}<div class="banner error">{error}</div>{/if}

  {#if loading}
    <p class="muted">Loading…</p>
  {:else if appointments.length === 0}
    <div class="empty-state">
      <p class="muted">No schedule data available.</p>
      <p class="muted small">No patients have granted you schedule access yet, or none have upcoming appointments.</p>
    </div>
  {:else}
    {#each grouped() as group}
      <div class="patient-block">
        <div class="patient-header">
          <span class="patient-name">{group.patient_name}</span>
          <span class="appt-count">{group.appointments.length} appointment{group.appointments.length !== 1 ? 's' : ''}</span>
        </div>
        <table class="tbl">
          <thead>
            <tr>
              <th>Date &amp; Time</th>
              <th>Status</th>
              <th>Psychiatrist</th>
            </tr>
          </thead>
          <tbody>
            {#each group.appointments as a}
              <tr>
                <td class="dt">{new Date(a.scheduled_at).toLocaleString()}</td>
                <td><span class="badge {statusClass(a.status)}">{a.status}</span></td>
                <td class="psych">Dr. {a.psychiatrist_name}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/each}
  {/if}
</div>

<style>
  .page { max-width:760px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .hint { color:#64748B;font-size:0.8rem;margin:0 0 1.25rem;line-height:1.5; }
  .link { color:#C084FC;text-decoration:none; }
  .link:hover { text-decoration:underline; }
  .muted { color:#94A3B8;font-size:0.85rem; }
  .small { font-size:0.75rem;margin-top:0.25rem; }
  .banner { padding:0.6rem 0.9rem;border-radius:6px;font-size:0.8rem;margin-bottom:1rem; }
  .banner.error { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);color:#F87171; }
  .empty-state { background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:2rem;display:flex;flex-direction:column;gap:0.3rem; }

  .patient-block { margin-bottom:1.2rem; }
  .patient-header { display:flex;align-items:center;justify-content:space-between;margin-bottom:0.4rem;padding:0.4rem 0.6rem;background:rgba(139,92,246,0.08);border:1px solid rgba(139,92,246,0.15);border-radius:6px; }
  .patient-name { font-size:0.88rem;color:#E6EDF3;font-weight:600; }
  .appt-count { font-size:0.72rem;color:#94A3B8; }

  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.4rem 0.6rem;color:#64748B;border-bottom:1px solid rgba(58,190,255,0.08);font-size:0.7rem;text-transform:uppercase; }
  .tbl td { padding:0.45rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .dt { color:#E6EDF3; }
  .psych { color:#94A3B8;font-size:0.78rem; }

  .badge { padding:0.15rem 0.5rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .badge-ok { background:rgba(16,185,129,0.15);color:#34D399; }
  .badge-cancel { background:rgba(239,68,68,0.12);color:#F87171; }
  .badge-other { background:rgba(245,158,11,0.12);color:#FBBF24; }
</style>
