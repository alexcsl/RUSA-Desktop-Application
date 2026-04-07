<!-- /astronauts/security-report — Submit a security incident report -->
<script lang="ts">
  import { astSubmitSecurityReport } from '$lib/stores/astronauts';

  const INCIDENT_TYPES = [
    { value: 'mission_anomaly',             label: 'Mission Anomaly' },
    { value: 'hostile_contact',             label: 'Hostile Contact' },
    { value: 'unauthorized_access',         label: 'Unauthorized Access' },
    { value: 'equipment_theft',             label: 'Equipment Theft' },
    { value: 'hazardous_material_incident', label: 'Hazardous Material Incident' },
    { value: 'physical_threat',             label: 'Physical Threat' },
    { value: 'suspicious_activity',         label: 'Suspicious Activity' },
    { value: 'other',                       label: 'Other' },
  ];

  const SEVERITY_OPTIONS = [
    { value: 'low',      label: 'Low' },
    { value: 'medium',   label: 'Medium' },
    { value: 'high',     label: 'High' },
    { value: 'critical', label: 'Critical' },
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
    error = '';
    success = '';

    if (!incidentType || !location.trim() || !severity || !description.trim()) {
      error = 'Please fill in all required fields.';
      return;
    }

    submitting = true;
    try {
      const id = await astSubmitSecurityReport({
        incident_type: incidentType,
        location: location.trim(),
        description: description.trim(),
        severity,
        occurred_at: occurredAt || undefined,
        recommended_action: recommendedAction.trim() || undefined,
      });
      success = `Security report submitted successfully. Report ID: ${id}`;
      incidentType = '';
      location = '';
      severity = '';
      occurredAt = '';
      description = '';
      recommendedAction = '';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  function severityClass(v: string): string {
    switch (v) {
      case 'critical': return 'sev-critical';
      case 'high':     return 'sev-high';
      case 'medium':   return 'sev-medium';
      default:         return 'sev-low';
    }
  }
</script>

<h1 class="title">Security Report</h1>
<p class="subtitle">Report a security incident to the Galactic Security team.</p>

{#if success}
  <div class="banner success">{success}</div>
{/if}
{#if error}
  <div class="banner error">{error}</div>
{/if}

<form class="form-card" onsubmit={handleSubmit}>
  <div class="form-row two-col">
    <div class="field">
      <label for="incident-type">Incident Type <span class="req">*</span></label>
      <select id="incident-type" bind:value={incidentType} required>
        <option value="" disabled>Select type…</option>
        {#each INCIDENT_TYPES as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>
    <div class="field">
      <label for="severity">Severity <span class="req">*</span></label>
      <select id="severity" bind:value={severity} class={severity ? severityClass(severity) : ''} required>
        <option value="" disabled>Select severity…</option>
        {#each SEVERITY_OPTIONS as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>
  </div>

  <div class="form-row two-col">
    <div class="field">
      <label for="location">Location <span class="req">*</span></label>
      <input id="location" type="text" bind:value={location} placeholder="e.g. Sector 7, Outpost Gamma" required />
    </div>
    <div class="field">
      <label for="occurred-at">Occurred At (optional)</label>
      <input id="occurred-at" type="date" bind:value={occurredAt} />
    </div>
  </div>

  <div class="field">
    <label for="description">Description <span class="req">*</span></label>
    <textarea id="description" bind:value={description} rows="5" placeholder="Describe the incident in detail…" required></textarea>
  </div>

  <div class="field">
    <label for="recommended-action">Recommended Action (optional)</label>
    <textarea id="recommended-action" bind:value={recommendedAction} rows="3" placeholder="Any recommended response or action…"></textarea>
  </div>

  <div class="form-footer">
    <button type="submit" class="btn-submit" disabled={submitting}>
      {submitting ? 'Submitting…' : 'Submit Report'}
    </button>
  </div>
</form>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#F59E0B;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1.25rem; }

  .banner { border-radius:6px;padding:0.65rem 1rem;font-size:0.8rem;margin-bottom:1rem; }
  .banner.success { background:rgba(16,185,129,0.12);border:1px solid rgba(16,185,129,0.3);color:#10B981; }
  .banner.error   { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);color:#EF4444; }

  .form-card { background:rgba(14,20,40,0.7);border:1px solid rgba(245,158,11,0.15);border-radius:10px;padding:1.5rem;max-width:680px; }

  .form-row { display:flex;gap:1rem; }
  .two-col .field { flex:1; }
  .field { display:flex;flex-direction:column;gap:0.3rem;margin-bottom:1rem; }
  label { font-size:0.75rem;color:#94A3B8;font-weight:600;text-transform:uppercase;letter-spacing:0.03em; }
  .req { color:#EF4444; }

  input, select, textarea {
    background:#0B0F1A;
    border:1px solid rgba(245,158,11,0.2);
    border-radius:6px;
    color:#E6EDF3;
    font-size:0.8rem;
    padding:0.45rem 0.75rem;
    outline:none;
    font-family:inherit;
    width:100%;
    box-sizing:border-box;
  }
  input:focus, select:focus, textarea:focus { border-color:#F59E0B; }
  input::placeholder, textarea::placeholder { color:#475569; }
  textarea { resize:vertical;min-height:80px; }

  select.sev-low      { color:#10B981; }
  select.sev-medium   { color:#F59E0B; }
  select.sev-high     { color:#F97316; }
  select.sev-critical { color:#EF4444; }

  .form-footer { display:flex;justify-content:flex-end;margin-top:0.5rem; }
  .btn-submit {
    background:rgba(245,158,11,0.15);
    border:1px solid #F59E0B;
    color:#F59E0B;
    border-radius:6px;
    padding:0.5rem 1.5rem;
    font-size:0.8rem;
    font-weight:600;
    cursor:pointer;
  }
  .btn-submit:hover:not(:disabled) { background:rgba(245,158,11,0.28); }
  .btn-submit:disabled { opacity:0.5;cursor:not-allowed; }
</style>
