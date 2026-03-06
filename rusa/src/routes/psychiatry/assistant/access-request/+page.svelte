<!--
  /psychiatry/assistant/access-request — Request Schedule Access (UC-PA-02)
  Assistant requests access to a patient's schedule. Patient must grant it.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    psyAssistantGetPatients,
    psyRequestScheduleAccess,
    type PatientListEntry,
  } from '$lib/stores/psychiatry';

  let patients: PatientListEntry[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let success = $state('');
  let requesting = $state(false);

  onMount(async () => {
    try { patients = await psyAssistantGetPatients(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    loading = false;
  });

  async function requestAccess(patientUserId: string, name: string) {
    requesting = true;
    error = '';
    success = '';
    try {
      await psyRequestScheduleAccess(patientUserId);
      success = `Access request sent to ${name}. They will review it.`;
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    requesting = false;
  }
</script>

<div class="page">
  <h2>Request Schedule Access</h2>
  <p class="muted sub">Send access requests to patients. Once granted, you can book appointments on their behalf.</p>

  {#if success}<div class="toast-success">{success}</div>{/if}
  {#if error}<div class="toast-error">{error}</div>{/if}

  {#if loading}
    <p class="muted">Loading patients…</p>
  {:else if patients.length === 0}
    <p class="muted">No patients available.</p>
  {:else}
    <div class="list">
      {#each patients as p}
        <div class="row">
          <div class="info">
            <span class="pname">{p.patient_name}</span>
            <span class="badge {p.care_status === 'active' ? 'badge-active' : ''}">{p.care_status}</span>
          </div>
          <button class="btn-accent-sm" onclick={() => requestAccess(p.user_id, p.patient_name)} disabled={requesting}>
            Request Access
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .page { max-width:700px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#3ABEFF;margin-bottom:0.2rem; }
  .sub { margin-bottom:1rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .list { display:flex;flex-direction:column;gap:0.3rem; }
  .row { display:flex;justify-content:space-between;align-items:center;padding:0.6rem 0.8rem;background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:6px; }
  .info { display:flex;align-items:center;gap:0.5rem; }
  .pname { font-size:0.85rem;color:#E6EDF3; }
  .badge { padding:0.15rem 0.5rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .badge-active { background:rgba(16,185,129,0.15);color:#34D399; }
  .btn-accent-sm { background:rgba(139,92,246,0.1);color:#C084FC;border:1px solid rgba(139,92,246,0.3);border-radius:4px;padding:0.3rem 0.7rem;cursor:pointer;font-size:0.75rem; }
  .btn-accent-sm:hover:not(:disabled) { background:rgba(139,92,246,0.2); }
  .btn-accent-sm:disabled { opacity:0.5;cursor:not-allowed; }
  .toast-success { background:rgba(16,185,129,0.1);border:1px solid rgba(16,185,129,0.3);color:#34D399;padding:0.5rem 0.8rem;border-radius:6px;font-size:0.8rem;margin-bottom:0.8rem; }
  .toast-error { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);color:#EF4444;padding:0.5rem 0.8rem;border-radius:6px;font-size:0.8rem;margin-bottom:0.8rem; }
</style>
