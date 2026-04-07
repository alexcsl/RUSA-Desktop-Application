<!--
  UC-FA-02: Farm Health Log — Create + View (Farmer)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { stlLogFarmHealth, stlGetFarmHealthLogs, type FarmHealthRow } from '$lib/stores/settlers';

  let subjectType = $state<'plant' | 'livestock'>('plant');
  let subjectName = $state('');
  let logDate = $state(new Date().toISOString().slice(0, 10));
  let condition = $state('');
  let treatment = $state('');
  let notes = $state('');
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  let logs: FarmHealthRow[] = $state([]);
  let logsError = $state('');

  async function loadLogs() {
    try { logs = await stlGetFarmHealthLogs(); }
    catch (e: any) { logsError = e?.message ?? String(e); }
  }

  onMount(loadLogs);

  async function handleSubmit() {
    error = ''; success = '';
    if (!subjectName.trim()) { error = `${subjectType === 'plant' ? 'Plant/Crop' : 'Animal'} name is required.`; return; }
    if (!condition.trim()) { error = 'Condition is required.'; return; }
    submitting = true;
    try {
      const id = await stlLogFarmHealth({
        log_date: logDate,
        subject_type: subjectType,
        subject_name: subjectName,
        condition,
        treatment: treatment || undefined,
        notes: notes || undefined,
      });
      success = `Farm health log submitted (ID: ${id.slice(0,8)}…).`;
      subjectName = ''; condition = ''; treatment = ''; notes = '';
      await loadLogs();
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }
</script>

<h2>Farm Health Check</h2>

<form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  <div class="toggle-row">
    <button type="button" class="tog" class:active={subjectType === 'plant'} onclick={() => subjectType = 'plant'}>🌱 Plant</button>
    <button type="button" class="tog" class:active={subjectType === 'livestock'} onclick={() => subjectType = 'livestock'}>🐄 Livestock</button>
  </div>
  <div class="row">
    <label>
      {subjectType === 'plant' ? 'Plant / Crop Name' : 'Animal / Herd Name'} *
      <input type="text" bind:value={subjectName} placeholder={subjectType === 'plant' ? 'e.g. Wheat Field A' : 'e.g. Poultry Barn 2'} />
    </label>
    <label>
      Log Date *
      <input type="date" bind:value={logDate} />
    </label>
  </div>
  <label>
    Condition *
    <input type="text" bind:value={condition} placeholder="e.g. healthy, wilting, infected, recovering" />
  </label>
  <label>
    Treatment Applied
    <input type="text" bind:value={treatment} placeholder="e.g. pesticide spray, antibiotics, pruning…" />
  </label>
  <label>
    Notes / Observations
    <textarea bind:value={notes} rows="3" placeholder="Growth observations, yield estimates, concerns…"></textarea>
  </label>
  {#if error}<p class="err">{error}</p>{/if}
  {#if success}<p class="ok">{success}</p>{/if}
  <button type="submit" class="btn-primary" disabled={submitting}>
    {submitting ? 'Logging…' : 'Submit Health Log'}
  </button>
</form>

<h3>Previous Logs</h3>
{#if logsError}
  <p class="err">{logsError}</p>
{:else if logs.length === 0}
  <p class="dim">No farm health logs yet.</p>
{:else}
  <div class="table-wrap">
    <table>
      <thead><tr><th>Type</th><th>Subject</th><th>Date</th><th>Condition</th><th>Treatment</th></tr></thead>
      <tbody>
        {#each logs as l}
          <tr>
            <td><span class="badge {l.subject_type === 'plant' ? 'plant' : 'livestock'}">{l.subject_type}</span></td>
            <td>{l.subject_name}</td>
            <td class="dim">{new Date(l.log_date).toLocaleDateString()}</td>
            <td>{l.condition}</td>
            <td class="dim">{l.treatment ?? '—'}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
  h3 { font-family:'Orbitron',sans-serif;color:#E6EDF3;font-size:0.9rem;margin:1.5rem 0 0.5rem; }
  .form { display:flex;flex-direction:column;gap:0.75rem;max-width:520px; }
  .form label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8; }
  .form input[type="text"], .form input[type="date"], .form textarea { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem; }
  .form textarea { resize:vertical; }
  .toggle-row { display:flex;gap:0.5rem; }
  .tog { background:#111827;border:1px solid #374151;color:#94A3B8;padding:0.4rem 1rem;border-radius:4px;cursor:pointer;font-size:0.8rem; }
  .tog.active { border-color:#3ABEFF;color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .row { display:flex;gap:1rem; }
  .row label { flex:1; }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem; }
  .dim { color:#64748B;font-size:0.8rem; }
  .btn-primary { background:#3ABEFF;color:#0B0F1A;border:none;border-radius:4px;padding:0.5rem 1.2rem;cursor:pointer;font-weight:600;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:#60CFFF; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .table-wrap { overflow-x:auto; }
  table { width:100%;border-collapse:collapse;font-size:0.8rem; }
  th { text-align:left;color:#94A3B8;font-size:0.7rem;text-transform:uppercase;padding:0.4rem 0.6rem;border-bottom:1px solid #1E293B; }
  td { padding:0.4rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.03); }
  .badge { display:inline-block;padding:0.15rem 0.45rem;border-radius:4px;font-size:0.7rem;font-weight:600;text-transform:capitalize; }
  .plant { background:rgba(74,222,128,0.15);color:#4ADE80; }
  .livestock { background:rgba(245,158,11,0.15);color:#F59E0B; }
</style>
