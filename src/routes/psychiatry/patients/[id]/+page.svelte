<!--
  /psychiatry/patients/[id] — Patient detail page (UC-PSY-02, UC-PSY-03)
  Shows patient profile, appointments list, and recovery logs.
  Psychiatrist can log appointment findings and add recovery entries.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import {
    psyGetPatientDetail,
    psyGetAppointments,
    psyGetRecoveryLogs,
    psyLogAppointment,
    psyUpdateRecoveryLog,
    type PatientDetail,
    type AppointmentRecord,
    type RecoveryLogEntry,
  } from '$lib/stores/psychiatry';

  let patientId = $state('');
  page.subscribe((p) => (patientId = p.params.id ?? ''));

  let detail: PatientDetail | null = $state(null);
  let appointments: AppointmentRecord[] = $state([]);
  let recoveryLogs: RecoveryLogEntry[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  // Appointment form
  let showApptForm = $state(false);
  let apptDate = $state('');
  let apptLog = $state('');
  let apptError = $state('');
  let apptSaving = $state(false);

  // Recovery log form
  let showRecForm = $state(false);
  let recMilestone = $state('');
  let recStatus = $state('in_progress');
  let recDetails = $state('');
  let recError = $state('');
  let recSaving = $state(false);

  let activeTab: 'profile' | 'appointments' | 'recovery' = $state('profile');

  onMount(async () => { await loadAll(); });

  async function loadAll() {
    loading = true;
    error = '';
    try {
      [detail, appointments, recoveryLogs] = await Promise.all([
        psyGetPatientDetail(patientId),
        psyGetAppointments(patientId),
        psyGetRecoveryLogs(patientId),
      ]);
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    loading = false;
  }

  async function submitAppointment() {
    apptError = '';
    apptSaving = true;
    let logObj: Record<string, unknown> = {};
    if (apptLog.trim()) {
      try { logObj = JSON.parse(apptLog); }
      catch { apptError = 'Log must be valid JSON.'; apptSaving = false; return; }
    }
    try {
      await psyLogAppointment({ patient_id: patientId, scheduled_at: new Date(apptDate).toISOString(), appointment_log: logObj });
      showApptForm = false;
      apptDate = '';
      apptLog = '';
      appointments = await psyGetAppointments(patientId);
    } catch (e: unknown) { apptError = e instanceof Error ? e.message : String(e); }
    apptSaving = false;
  }

  async function submitRecovery() {
    recError = '';
    recSaving = true;
    try {
      await psyUpdateRecoveryLog({
        patient_id: patientId,
        milestone: recMilestone,
        status: recStatus,
        details: recDetails || undefined,
      });
      showRecForm = false;
      recMilestone = '';
      recStatus = 'in_progress';
      recDetails = '';
      recoveryLogs = await psyGetRecoveryLogs(patientId);
    } catch (e: unknown) { recError = e instanceof Error ? e.message : String(e); }
    recSaving = false;
  }

  function statusBadge(s: string) {
    const map: Record<string, string> = { in_progress: 'badge-progress', achieved: 'badge-achieved', regressed: 'badge-regressed', on_hold: 'badge-hold' };
    return map[s] ?? '';
  }
</script>

<div class="page">
  {#if loading}
    <p class="muted">Loading…</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if detail}
    <h2>{detail.patient_profile?.diagnosis || 'Patient'}</h2>
    <p class="muted">ID: {detail.id} — User: {detail.user_id}</p>

    <div class="tabs">
      <button class="tab" class:active={activeTab==='profile'} onclick={() => activeTab='profile'}>Profile</button>
      <button class="tab" class:active={activeTab==='appointments'} onclick={() => activeTab='appointments'}>Appointments ({appointments.length})</button>
      <button class="tab" class:active={activeTab==='recovery'} onclick={() => activeTab='recovery'}>Recovery ({recoveryLogs.length})</button>
    </div>

    {#if activeTab === 'profile'}
      <div class="card">
        <h3>Patient Profile</h3>
        <pre class="json">{JSON.stringify(detail.patient_profile, null, 2)}</pre>
        {#if detail.initial_notes}
          <h4>Initial Notes</h4>
          <p class="note-text">{detail.initial_notes}</p>
        {/if}
        <p class="muted">Created: {detail.created_at ? new Date(detail.created_at).toLocaleString() : '—'}</p>
      </div>
    {/if}

    {#if activeTab === 'appointments'}
      <div class="card">
        <div class="card-head">
          <h3>Appointment Records</h3>
          <button class="btn-accent-sm" onclick={() => (showApptForm = !showApptForm)}>
            {showApptForm ? 'Cancel' : '+ Log Appointment'}
          </button>
        </div>

        {#if showApptForm}
          <div class="form-box">
            {#if apptError}<p class="error">{apptError}</p>{/if}
            <label class="field-label">Date/Time
              <input class="input" type="datetime-local" bind:value={apptDate} />
            </label>
            <label class="field-label">Appointment Log (JSON)
              <textarea class="input ta mono" bind:value={apptLog} rows="4" placeholder="findings: ..., next_steps: ..."></textarea>
            </label>
            <button class="btn-primary" onclick={submitAppointment} disabled={apptSaving}>
              {apptSaving ? 'Saving…' : 'Save Appointment'}
            </button>
          </div>
        {/if}

        {#if appointments.length === 0}
          <p class="muted">No appointments yet.</p>
        {:else}
          <table class="tbl">
            <thead><tr><th>Date</th><th>Status</th><th>Findings</th></tr></thead>
            <tbody>
              {#each appointments as a}
                <tr>
                  <td>{new Date(a.scheduled_at).toLocaleString()}</td>
                  <td><span class="badge {a.status === 'completed' ? 'badge-achieved' : 'badge-progress'}">{a.status}</span></td>
                  <td class="json-cell">{a.appointment_log ? JSON.stringify(a.appointment_log) : '—'}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    {/if}

    {#if activeTab === 'recovery'}
      <div class="card">
        <div class="card-head">
          <h3>Recovery Logs</h3>
          <button class="btn-accent-sm" onclick={() => (showRecForm = !showRecForm)}>
            {showRecForm ? 'Cancel' : '+ Add Entry'}
          </button>
        </div>

        {#if showRecForm}
          <div class="form-box">
            {#if recError}<p class="error">{recError}</p>{/if}
            <label class="field-label">Milestone
              <input class="input" type="text" bind:value={recMilestone} placeholder="e.g. Completed CBT phase 1" />
            </label>
            <label class="field-label">Status
              <select class="input" bind:value={recStatus}>
                <option value="in_progress">In Progress</option>
                <option value="achieved">Achieved</option>
                <option value="regressed">Regressed</option>
                <option value="on_hold">On Hold</option>
              </select>
            </label>
            <label class="field-label">Details (optional)
              <textarea class="input ta" bind:value={recDetails} rows="3" placeholder="Clinical notes…"></textarea>
            </label>
            <button class="btn-primary" onclick={submitRecovery} disabled={recSaving}>
              {recSaving ? 'Saving…' : 'Save Entry'}
            </button>
          </div>
        {/if}

        {#if recoveryLogs.length === 0}
          <p class="muted">No recovery entries yet.</p>
        {:else}
          <table class="tbl">
            <thead><tr><th>Milestone</th><th>Status</th><th>Details</th><th>Date</th></tr></thead>
            <tbody>
              {#each recoveryLogs as r}
                <tr>
                  <td>{r.milestone}</td>
                  <td><span class="badge {statusBadge(r.status)}">{r.status}</span></td>
                  <td class="muted">{r.details ?? '—'}</td>
                  <td class="muted">{r.logged_at ? new Date(r.logged_at).toLocaleDateString() : '—'}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .page { max-width:900px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#3ABEFF;margin-bottom:0.3rem; }
  h3 { font-family:'Orbitron',sans-serif;font-size:0.85rem;color:#C084FC;margin:0; }
  h4 { font-size:0.8rem;color:#94A3B8;margin:0.6rem 0 0.3rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .error { color:#EF4444;font-size:0.8rem; }
  .card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-bottom:1rem; }
  .card-head { display:flex;justify-content:space-between;align-items:center;margin-bottom:0.8rem; }
  .tabs { display:flex;gap:0.3rem;margin-bottom:1rem; }
  .tab { padding:0.4rem 0.8rem;border:1px solid transparent;border-radius:4px;background:transparent;color:#94A3B8;cursor:pointer;font-size:0.8rem; }
  .tab.active { color:#3ABEFF;border-color:rgba(58,190,255,0.3);background:rgba(58,190,255,0.08); }
  .tab:hover:not(.active) { color:#E6EDF3; }

  .input { width:100%;padding:0.45rem 0.6rem;background:#1F2937;color:#E6EDF3;border:1px solid #374151;border-radius:4px;font-size:0.8rem;margin-top:0.2rem;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .ta { resize:vertical; }
  .mono { font-family:'Cascadia Code','Fira Mono',monospace;font-size:0.75rem; }
  .field-label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.6rem; }
  .form-box { background:rgba(31,41,55,0.5);border:1px solid rgba(139,92,246,0.15);border-radius:6px;padding:0.8rem;margin-bottom:0.8rem; }

  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.5rem 0.6rem;color:#94A3B8;border-bottom:1px solid rgba(58,190,255,0.1); }
  .tbl td { padding:0.5rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .json-cell { max-width:250px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:0.7rem;color:#94A3B8; }
  .json { background:#0B0F1A;padding:0.6rem;border-radius:4px;font-size:0.75rem;color:#94A3B8;overflow-x:auto;white-space:pre-wrap; }
  .note-text { font-size:0.8rem;color:#E6EDF3; }

  .badge { padding:0.15rem 0.5rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .badge-progress { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .badge-achieved { background:rgba(16,185,129,0.15);color:#34D399; }
  .badge-regressed { background:rgba(239,68,68,0.15);color:#EF4444; }
  .badge-hold { background:rgba(245,158,11,0.15);color:#FBBF24; }

  .btn-accent-sm { background:rgba(58,190,255,0.1);color:#3ABEFF;border:1px solid rgba(58,190,255,0.3);border-radius:4px;padding:0.25rem 0.6rem;cursor:pointer;font-size:0.7rem; }
  .btn-accent-sm:hover { background:rgba(58,190,255,0.2); }
  .btn-primary { padding:0.5rem 1.2rem;background:rgba(58,190,255,0.15);color:#3ABEFF;border:1px solid rgba(58,190,255,0.4);border-radius:6px;cursor:pointer;font-size:0.8rem;font-weight:600;margin-top:0.5rem; }
  .btn-primary:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
