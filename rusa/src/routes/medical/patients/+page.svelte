<!-- /medical/patients — Patient Registry & Record Viewer (UC-MED-02) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    medGetPatients, medGetPatientRecord,
    type PatientSummary, type TreatmentLog,
  } from '$lib/stores/medical';

  let patients: PatientSummary[] = $state([]);
  let selected: PatientSummary | null = $state(null);
  let logs: TreatmentLog[] = $state([]);
  let loading = $state(false);
  let error = $state('');

  onMount(async () => {
    try { patients = await medGetPatients(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  });

  async function selectPatient(p: PatientSummary) {
    selected = p;
    loading = true;
    error = '';
    try { logs = await medGetPatientRecord(p.id); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  }

  function fmtDate(iso: string): string {
    return new Date(iso).toLocaleDateString('en-GB', { year: 'numeric', month: 'short', day: 'numeric' });
  }
</script>

<h1 class="title">Patient Records</h1>
<p class="subtitle">Select a patient to view their treatment history.</p>

{#if error}<p class="error">{error}</p>{/if}

<div class="grid">
  <div class="list-panel">
    {#each patients as p}
      <button class="card" class:selected={selected?.id === p.id} onclick={() => selectPatient(p)}>
        <div class="card-title">{p.full_name}</div>
        <div class="card-meta">
          <span>Registered {fmtDate(p.created_at)}</span>
        </div>
      </button>
    {:else}
      <p class="empty">No patients registered.</p>
    {/each}
  </div>

  <div class="detail-panel">
    {#if selected}
      <h2>{selected.full_name}</h2>
      <div class="kv"><span class="k">Patient ID:</span><span class="mono">{selected.id.slice(0, 8)}</span></div>
      <div class="kv"><span class="k">User ID:</span><span class="mono">{selected.user_id.slice(0, 8)}</span></div>
      <div class="kv"><span class="k">Registered:</span><span>{fmtDate(selected.created_at)}</span></div>

      <h3>Treatment History</h3>
      {#if loading}
        <p class="hint">Loading…</p>
      {:else if logs.length === 0}
        <p class="hint">No treatment logs recorded.</p>
      {:else}
        {#each logs as log}
          <div class="log-card">
            <div class="log-header">
              <span class="log-date">{fmtDate(log.treatment_date)}</span>
              <span class="log-by">by {log.treater_name}</span>
            </div>
            <div class="kv"><span class="k">Diagnosis:</span><span>{log.diagnosis}</span></div>
            <div class="kv"><span class="k">Treatment:</span><span>{log.treatment_provided}</span></div>
            {#if log.medications}
              <div class="kv"><span class="k">Medications:</span><span>{log.medications}</span></div>
            {/if}
            {#if log.follow_up_notes}
              <div class="kv"><span class="k">Follow-up:</span><span>{log.follow_up_notes}</span></div>
            {/if}
          </div>
        {/each}
      {/if}
    {:else}
      <div class="empty-state"><p>Select a patient to view records.</p></div>
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .grid { display:flex;gap:1rem;flex:1;overflow:hidden; }
  .list-panel { width:320px;overflow-y:auto; }
  .detail-panel { flex:1;overflow-y:auto; }
  .card { width:100%;text-align:left;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.7rem;margin-bottom:0.4rem;cursor:pointer;color:#E6EDF3; }
  .card:hover { border-color:rgba(58,190,255,0.3); }
  .card.selected { border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .card-title { font-size:0.85rem;font-weight:500;margin-bottom:0.2rem; }
  .card-meta { font-size:0.7rem;color:#94A3B8; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1rem;color:#3ABEFF;margin:0 0 0.75rem; }
  h3 { font-size:0.85rem;color:#E6EDF3;margin:0.75rem 0 0.4rem; }
  .kv { display:flex;gap:0.5rem;font-size:0.8rem;margin-bottom:0.3rem; }
  .k { color:#94A3B8;min-width:90px; }
  .mono { font-family:monospace;font-size:0.75rem;color:#8B5CF6; }
  .log-card { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.75rem;margin-bottom:0.5rem; }
  .log-header { display:flex;justify-content:space-between;margin-bottom:0.4rem; }
  .log-date { font-size:0.8rem;color:#3ABEFF;font-weight:600; }
  .log-by { font-size:0.7rem;color:#94A3B8; }
  .hint { color:#94A3B8;font-size:0.8rem; }
  .error { color:#EF4444;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1.5rem; }
  .empty-state { display:flex;justify-content:center;align-items:center;height:100%;color:#475569; }
</style>
