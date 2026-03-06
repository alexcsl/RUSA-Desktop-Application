<!--
  UC-CE-03: Log Building Health Check (Civil Engineer)
-->
<script lang="ts">
  import { stlLogBuildingHealth } from '$lib/stores/settlers';

  let buildingName = $state('');
  let checkDate = $state(new Date().toISOString().slice(0, 10));
  let status = $state<'pass' | 'fail' | 'needs_repair'>('pass');
  let findings = $state('');
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

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
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
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

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
  .form { display:flex;flex-direction:column;gap:0.75rem;max-width:520px; }
  .form label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8; }
  .form input, .form select, .form textarea { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem; }
  .form textarea { resize:vertical; }
  .row { display:flex;gap:1rem; }
  .row label { flex:1; }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem; }
  .btn-primary { background:#3ABEFF;color:#0B0F1A;border:none;border-radius:4px;padding:0.5rem 1.2rem;cursor:pointer;font-weight:600;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:#60CFFF; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
