<!--
  /psychiatry/patients — Psychiatrist's patient list (UC-PSY-01)
  Shows all patients assigned to the current psychiatrist with care status.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { psyGetMyPatients, type PatientSummary } from '$lib/stores/psychiatry';

  let patients: PatientSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try {
      patients = await psyGetMyPatients();
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    loading = false;
  });

  function statusColor(s: string) {
    if (s === 'active') return 'badge-active';
    if (s === 'discharged') return 'badge-discharged';
    if (s === 'transferred') return 'badge-transferred';
    return '';
  }
</script>

<div class="page">
  <h2>My Patients</h2>

  {#if loading}
    <p class="muted">Loading patients…</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if patients.length === 0}
    <p class="muted">No patients assigned yet.</p>
  {:else}
    <table class="tbl">
      <thead>
        <tr><th>Name</th><th>Status</th><th>Notes</th><th>Created</th><th></th></tr>
      </thead>
      <tbody>
        {#each patients as p}
          <tr>
            <td>{p.patient_name}</td>
            <td><span class="badge {statusColor(p.care_status)}">{p.care_status}</span></td>
            <td class="muted">{p.initial_notes ?? '—'}</td>
            <td class="muted">{p.created_at ? new Date(p.created_at).toLocaleDateString() : '—'}</td>
            <td>
              <button class="btn-accent-sm" onclick={() => goto(`/psychiatry/patients/${p.id}`)}>View</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .page { max-width:900px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#3ABEFF;margin-bottom:1rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .error { color:#EF4444;font-size:0.8rem; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.5rem 0.6rem;color:#94A3B8;border-bottom:1px solid rgba(58,190,255,0.1); }
  .tbl td { padding:0.5rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .badge { padding:0.15rem 0.5rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .badge-active { background:rgba(16,185,129,0.15);color:#34D399; }
  .badge-discharged { background:rgba(148,163,184,0.15);color:#94A3B8; }
  .badge-transferred { background:rgba(245,158,11,0.15);color:#FBBF24; }
  .btn-accent-sm { background:rgba(58,190,255,0.1);color:#3ABEFF;border:1px solid rgba(58,190,255,0.3);border-radius:4px;padding:0.25rem 0.6rem;cursor:pointer;font-size:0.7rem; }
  .btn-accent-sm:hover { background:rgba(58,190,255,0.2); }
</style>
