<!--
  /psychiatry/assistant/patients — Assistant patient list (UC-PA-03, UC-PA-04)
  Shows restricted patient list (names only). Click to view recovery logs (GDPR-checked).
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    psyAssistantGetPatients,
    psyAssistantGetRecoveryLog,
    psyAssistantGetAppointments,
    type PatientListEntry,
    type RecoveryLogEntry,
    type AppointmentSummary,
  } from '$lib/stores/psychiatry';

  let patients: PatientListEntry[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  let selectedId = $state('');
  let recoveryLogs: RecoveryLogEntry[] = $state([]);
  let appointments: AppointmentSummary[] = $state([]);
  let detailLoading = $state(false);
  let detailError = $state('');
  let detailTab: 'recovery' | 'appointments' = $state('recovery');

  onMount(async () => {
    try { patients = await psyAssistantGetPatients(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    loading = false;
  });

  async function selectPatient(id: string) {
    selectedId = id;
    detailLoading = true;
    detailError = '';
    try {
      [recoveryLogs, appointments] = await Promise.all([
        psyAssistantGetRecoveryLog(id),
        psyAssistantGetAppointments(id),
      ]);
    } catch (e: unknown) { detailError = e instanceof Error ? e.message : String(e); }
    detailLoading = false;
  }

  function statusBadge(s: string) {
    const map: Record<string, string> = { in_progress: 'badge-progress', achieved: 'badge-achieved', regressed: 'badge-regressed', on_hold: 'badge-hold' };
    return map[s] ?? '';
  }
</script>

<div class="page">
  <h2>Patient List</h2>
  <p class="muted sub">Restricted view — clinical details hidden per GDPR.</p>

  <div class="split">
    <div class="list-col">
      {#if loading}
        <p class="muted">Loading…</p>
      {:else if error}
        <p class="error">{error}</p>
      {:else if patients.length === 0}
        <p class="muted">No patients visible.</p>
      {:else}
        {#each patients as p}
          <button class="patient-row" class:active={selectedId === p.id} onclick={() => selectPatient(p.id)}>
            <span class="pname">{p.patient_name}</span>
            <span class="badge {p.care_status === 'active' ? 'badge-active' : ''}">{p.care_status}</span>
          </button>
        {/each}
      {/if}
    </div>

    <div class="detail-col">
      {#if !selectedId}
        <p class="muted center">Select a patient to view details.</p>
      {:else if detailLoading}
        <p class="muted">Loading…</p>
      {:else if detailError}
        <p class="error">{detailError}</p>
      {:else}
        <div class="tabs">
          <button class="tab" class:active={detailTab==='recovery'} onclick={() => detailTab='recovery'}>
            Recovery ({recoveryLogs.length})
          </button>
          <button class="tab" class:active={detailTab==='appointments'} onclick={() => detailTab='appointments'}>
            Appointments ({appointments.length})
          </button>
        </div>

        {#if detailTab === 'recovery'}
          {#if recoveryLogs.length === 0}
            <p class="muted">No recovery entries.</p>
          {:else}
            <table class="tbl">
              <thead><tr><th>Milestone</th><th>Status</th><th>Details</th><th>Date</th></tr></thead>
              <tbody>
                {#each recoveryLogs as r}
                  <tr>
                    <td>{r.milestone}</td>
                    <td><span class="badge {statusBadge(r.status)}">{r.status}</span></td>
                    <td class="muted">{r.details ?? '🔒 Restricted'}</td>
                    <td class="muted">{r.logged_at ? new Date(r.logged_at).toLocaleDateString() : '—'}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        {/if}

        {#if detailTab === 'appointments'}
          {#if appointments.length === 0}
            <p class="muted">No appointments.</p>
          {:else}
            <table class="tbl">
              <thead><tr><th>Date</th><th>Status</th></tr></thead>
              <tbody>
                {#each appointments as a}
                  <tr>
                    <td>{new Date(a.scheduled_at).toLocaleString()}</td>
                    <td><span class="badge {a.status === 'completed' ? 'badge-achieved' : 'badge-progress'}">{a.status}</span></td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}
        {/if}
      {/if}
    </div>
  </div>
</div>

<style>
  .page { max-width:1000px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#3ABEFF;margin-bottom:0.2rem; }
  .sub { margin-bottom:1rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .center { text-align:center;margin-top:2rem; }
  .error { color:#EF4444;font-size:0.8rem; }

  .split { display:flex;gap:1rem; }
  .list-col { width:260px;min-width:200px;display:flex;flex-direction:column;gap:0.2rem; }
  .detail-col { flex:1;background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;min-height:300px; }

  .patient-row { display:flex;justify-content:space-between;align-items:center;padding:0.5rem 0.7rem;background:transparent;border:1px solid rgba(255,255,255,0.05);border-radius:6px;cursor:pointer;color:#E6EDF3;font-size:0.8rem;text-align:left;width:100%; }
  .patient-row:hover { background:rgba(58,190,255,0.05); }
  .patient-row.active { background:rgba(58,190,255,0.1);border-color:rgba(58,190,255,0.3); }
  .pname { flex:1; }

  .tabs { display:flex;gap:0.3rem;margin-bottom:0.8rem; }
  .tab { padding:0.4rem 0.8rem;border:1px solid transparent;border-radius:4px;background:transparent;color:#94A3B8;cursor:pointer;font-size:0.8rem; }
  .tab.active { color:#3ABEFF;border-color:rgba(58,190,255,0.3);background:rgba(58,190,255,0.08); }

  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.5rem 0.6rem;color:#94A3B8;border-bottom:1px solid rgba(58,190,255,0.1); }
  .tbl td { padding:0.5rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .badge { padding:0.15rem 0.5rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .badge-active { background:rgba(16,185,129,0.15);color:#34D399; }
  .badge-progress { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .badge-achieved { background:rgba(16,185,129,0.15);color:#34D399; }
  .badge-regressed { background:rgba(239,68,68,0.15);color:#EF4444; }
  .badge-hold { background:rgba(245,158,11,0.15);color:#FBBF24; }
</style>
