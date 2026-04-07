<!--
  /settlers/security-report — Submit a security incident report
  Available to: Settler Commander, Civil Engineer, Farmer
-->
<script lang="ts">
  import { stlSubmitSecurityReport } from '$lib/stores/settlers';

  const INCIDENT_TYPES = [
    { value: 'settlement_breach',      label: 'Settlement Breach' },
    { value: 'structural_damage',      label: 'Structural Damage' },
    { value: 'hostile_encounter',      label: 'Hostile Encounter' },
    { value: 'theft',                  label: 'Theft' },
    { value: 'assault',                label: 'Assault' },
    { value: 'suspicious_activity',    label: 'Suspicious Activity' },
    { value: 'environmental_hazard',   label: 'Environmental Hazard' },
    { value: 'other',                  label: 'Other' },
  ];

  let incidentType = $state('');
  let location = $state('');
  let severity = $state('');
  let occurredAt = $state('');
  let description = $state('');
  let recommendedAction = $state('');
  let submitting = $state(false);
  let success = $state('');
  let error = $state('');

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = ''; success = '';
    if (!incidentType || !location.trim() || !severity || !description.trim()) {
      error = 'Please fill in all required fields.';
      return;
    }
    submitting = true;
    try {
      const id = await stlSubmitSecurityReport({
        incident_type: incidentType,
        location: location.trim(),
        description: description.trim(),
        severity,
        occurred_at: occurredAt || undefined,
        recommended_action: recommendedAction.trim() || undefined,
      });
      success = `Security report submitted (ID: ${id.slice(0,8)}…).`;
      incidentType = ''; location = ''; severity = ''; occurredAt = '';
      description = ''; recommendedAction = '';
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }
</script>

<h2>Security Report</h2>
<p class="hint">Report a security incident to the Galactic Security team.</p>

{#if success}<p class="ok">{success}</p>{/if}
{#if error}<p class="err">{error}</p>{/if}

<form class="form" onsubmit={handleSubmit}>
  <div class="row">
    <label>
      Incident Type *
      <select bind:value={incidentType} required>
        <option value="" disabled>Select type…</option>
        {#each INCIDENT_TYPES as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </label>
    <label>
      Severity *
      <select bind:value={severity} required>
        <option value="" disabled>Select…</option>
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
        <option value="critical">Critical</option>
      </select>
    </label>
  </div>
  <div class="row">
    <label style="flex:2">
      Location *
      <input type="text" bind:value={location} placeholder="e.g. Sector B-3, Farm perimeter" required />
    </label>
    <label style="flex:1">
      Occurred At (optional)
      <input type="date" bind:value={occurredAt} />
    </label>
  </div>
  <label>
    Description *
    <textarea bind:value={description} rows="5" required placeholder="Describe the incident in detail…"></textarea>
  </label>
  <label>
    Recommended Action (optional)
    <textarea bind:value={recommendedAction} rows="3" placeholder="Suggested response or action…"></textarea>
  </label>
  <button type="submit" class="btn-primary" disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Security Report'}
  </button>
</form>

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#F59E0B;font-size:1.1rem;margin:0 0 0.25rem; }
  .hint { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .ok { color:#4ADE80;font-size:0.8rem;margin:0 0 0.5rem; }
  .err { color:#EF4444;font-size:0.8rem;margin:0 0 0.5rem; }
  .form { display:flex;flex-direction:column;gap:0.75rem;max-width:600px; }
  .form label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8; }
  .form input, .form select, .form textarea { background:#111827;border:1px solid rgba(245,158,11,0.3);border-radius:4px;color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem; }
  .form textarea { resize:vertical; }
  .row { display:flex;gap:1rem; }
  .row label { flex:1; }
  .btn-primary { background:rgba(245,158,11,0.15);border:1px solid #F59E0B;color:#F59E0B;border-radius:4px;padding:0.5rem 1.2rem;cursor:pointer;font-weight:600;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover:not(:disabled) { background:rgba(245,158,11,0.28); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
