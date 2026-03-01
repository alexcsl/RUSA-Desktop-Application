<!-- /medical/treatment-log — Log Patient Treatment (UC-MED-01) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    medGetPatients, medLogTreatment, medRegisterPatient, medGetUserLookup,
    type PatientSummary, type UserLookupItem, type LogTreatmentPayload,
  } from '$lib/stores/medical';

  // State
  let patients: PatientSummary[] = $state([]);
  let userLookup: UserLookupItem[] = $state([]);
  let selectedPatientId = $state('');
  let treatmentDate = $state(todayISO());
  let diagnosis = $state('');
  let treatmentProvided = $state('');
  let medications = $state('');
  let followUpNotes = $state('');
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  // Registration form
  let showRegister = $state(false);
  let registerUserId = $state('');
  let registerError = $state('');

  function todayISO(): string {
    return new Date().toISOString().slice(0, 16);
  }

  onMount(async () => {
    try {
      [patients, userLookup] = await Promise.all([medGetPatients(), medGetUserLookup()]);
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  });

  async function handleRegister() {
    if (!registerUserId) { registerError = 'Select a user.'; return; }
    registerError = '';
    try {
      await medRegisterPatient({ user_id: registerUserId });
      patients = await medGetPatients();
      showRegister = false;
      registerUserId = '';
    } catch (e: unknown) { registerError = e instanceof Error ? e.message : String(e); }
  }

  async function handleSubmit() {
    if (!selectedPatientId || !diagnosis.trim() || !treatmentProvided.trim()) {
      error = 'Patient, diagnosis, and treatment are required.';
      return;
    }
    error = '';
    success = '';
    submitting = true;
    try {
      const payload: LogTreatmentPayload = {
        patient_id: selectedPatientId,
        treatment_date: new Date(treatmentDate).toISOString(),
        diagnosis,
        treatment_provided: treatmentProvided,
        medications: medications.trim() || undefined,
        follow_up_notes: followUpNotes.trim() || undefined,
      };
      await medLogTreatment(payload);
      success = 'Treatment logged successfully.';
      diagnosis = '';
      treatmentProvided = '';
      medications = '';
      followUpNotes = '';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<h1 class="title">Log Patient Treatment</h1>
<p class="subtitle">Record a treatment encounter for a registered patient.</p>

<div class="form-card">
  <div class="row">
    <div class="form-group" style="flex:2">
      <label for="patient">Patient</label>
      <select id="patient" class="input" bind:value={selectedPatientId}>
        <option value="">— Select patient —</option>
        {#each patients as p}
          <option value={p.id}>{p.full_name}</option>
        {/each}
      </select>
    </div>
    <div class="form-group" style="flex:1">
      <label>&nbsp;</label><!-- spacer -->
      <button class="btn-secondary" onclick={() => (showRegister = !showRegister)}>
        + Register Patient
      </button>
    </div>
    <div class="form-group" style="flex:1">
      <label for="tdate">Treatment Date</label>
      <input id="tdate" type="datetime-local" class="input" bind:value={treatmentDate} />
    </div>
  </div>

  {#if showRegister}
    <div class="register-form">
      <label for="reguser">Select user to register as patient:</label>
      <select id="reguser" class="input" bind:value={registerUserId}>
        <option value="">— Select user —</option>
        {#each userLookup as u}
          <option value={u.id}>{u.full_name} ({u.role_name})</option>
        {/each}
      </select>
      {#if registerError}<p class="error">{registerError}</p>{/if}
      <button class="btn-primary" onclick={handleRegister}>Register</button>
    </div>
  {/if}

  <div class="form-group">
    <label for="diagnosis">Diagnosis</label>
    <textarea id="diagnosis" class="textarea" rows="2" placeholder="Primary diagnosis…" bind:value={diagnosis}></textarea>
  </div>

  <div class="form-group">
    <label for="treatment">Treatment Provided</label>
    <textarea id="treatment" class="textarea" rows="3" placeholder="Procedures, interventions, actions taken…" bind:value={treatmentProvided}></textarea>
  </div>

  <div class="row">
    <div class="form-group" style="flex:1">
      <label for="meds">Medications <span class="optional">(optional)</span></label>
      <textarea id="meds" class="textarea" rows="2" placeholder="Prescribed medications…" bind:value={medications}></textarea>
    </div>
    <div class="form-group" style="flex:1">
      <label for="follow">Follow-up Notes <span class="optional">(optional)</span></label>
      <textarea id="follow" class="textarea" rows="2" placeholder="Schedule follow-up, notes…" bind:value={followUpNotes}></textarea>
    </div>
  </div>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}

  <button class="btn-primary" onclick={handleSubmit} disabled={submitting}>
    {submitting ? 'Saving…' : 'Log Treatment'}
  </button>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;display:flex;flex-direction:column;gap:0.75rem; }
  .row { display:flex;gap:0.75rem; }
  .form-group { display:flex;flex-direction:column;gap:0.25rem; }
  .form-group label { font-size:0.75rem;color:#94A3B8; }
  .optional { color:#475569; }
  .input, .textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus, .textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  select.input { cursor:pointer; }
  .register-form { background:rgba(16,185,129,0.05);border:1px solid rgba(16,185,129,0.2);border-radius:6px;padding:0.75rem;display:flex;flex-direction:column;gap:0.5rem; }
  .register-form label { font-size:0.75rem;color:#94A3B8; }
  .btn-primary { padding:0.5rem 1rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-secondary { padding:0.5rem 0.75rem;background:rgba(139,92,246,0.1);border:1px solid rgba(139,92,246,0.3);color:#C084FC;border-radius:6px;cursor:pointer;font-size:0.75rem; }
  .btn-secondary:hover { background:rgba(139,92,246,0.2); }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
</style>
