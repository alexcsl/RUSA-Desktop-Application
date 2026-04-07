<!--
  UC-CE-03: Building Health Log — Create + View (Civil Engineer)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { stlLogBuildingHealth, stlGetBuildingLogs, type BuildingLogRow } from '$lib/stores/settlers';

  let buildingName = $state('');
  let checkDate = $state(new Date().toISOString().slice(0, 10));
  let status = $state<'pass' | 'fail' | 'needs_repair'>('pass');
  let findings = $state('');
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  let logs: BuildingLogRow[] = $state([]);
  let logsError = $state('');

  async function loadLogs() {
    try { logs = await stlGetBuildingLogs(); }
    catch (e: any) { logsError = e?.message ?? String(e); }
  }

  onMount(loadLogs);

  async function handleSubmit() {
    error = ''; success = '';
    if (!buildingName.trim()) { error = 'Building name is required.'; return; }
    if (!checkDate) { error = 'Check date is required.'; return; }
    submitting = true;
    try {
      const id = await stlLogBuildingHealth({
        building_name: buildingName,
        check_date: checkDate,
        findings: findings || undefined,
        status,
      });
      success = `Building health log submitted (ID: ${id.slice(0,8)}…).`;
      buildingName = ''; findings = '';
      await loadLogs();
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }

  function statusClass(s: string): string {
    return s === 'pass' ? 'st-pass' : s === 'fail' ? 'st-fail' : 'st-repair';
  }
</script>

<h2>Building Health Check</h2>

<form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  <label>
    Building Name *
    <input type="text" bind:value={buildingName} placeholder="e.g. Greenhouse Alpha" />
  </label>
  <div class="row">
    <label>
      Check Date *
      <input type="date" bind:value={checkDate} />
    </label>
    <label>
      Status
      <select bind:value={status}>
        <option value="pass">Pass</option>
        <option value="fail">Fail</option>
        <option value="needs_repair">Needs Repair</option>
      </select>
    </label>
  </div>
  <label>
    Findings / Observations
    <textarea bind:value={findings} rows="4" placeholder="Cracks, leaks, wear observations…"></textarea>
  </label>
  {#if error}<p class="err">{error}</p>{/if}
  {#if success}<p class="ok">{success}</p>{/if}
  <button type="submit" class="btn-primary" disabled={submitting}>
    {submitting ? 'Logging…' : 'Submit Health Check'}
  </button>
</form>

<h3>Previous Logs</h3>
{#if logsError}
  <p class="err">{logsError}</p>
{:else if logs.length === 0}
  <p class="dim">No building health logs yet.</p>
{:else}
  <div class="table-wrap">
    <table>
      <thead><tr><th>Building</th><th>Date</th><th>Status</th><th>Findings</th></tr></thead>
      <tbody>
        {#each logs as l}
          <tr>
            <td>{l.building_name}</td>
            <td class="dim">{new Date(l.check_date).toLocaleDateString()}</td>
            <td><span class="badge {statusClass(l.status)}">{l.status.replace(/_/g,' ')}</span></td>
            <td class="dim cell-find">{l.findings ?? '—'}</td>
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
  .form input, .form select, .form textarea { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem; }
  .form textarea { resize:vertical; }
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
  .cell-find { max-width:260px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .badge { display:inline-block;padding:0.15rem 0.45rem;border-radius:4px;font-size:0.7rem;font-weight:700;text-transform:capitalize; }
  .st-pass { background:rgba(74,222,128,0.15);color:#4ADE80; }
  .st-fail { background:rgba(239,68,68,0.15);color:#EF4444; }
  .st-repair { background:rgba(245,158,11,0.15);color:#F59E0B; }
</style>
