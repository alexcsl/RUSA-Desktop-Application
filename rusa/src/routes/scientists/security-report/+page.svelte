<!-- /scientists/security-report — Security Report submission for Science Division -->
<script lang="ts">
  import { sciSubmitSecurityReport } from '$lib/stores/scientists';

  const INCIDENT_TYPES = [
    'unauthorized_access', 'equipment_theft', 'data_breach',
    'hazardous_material_incident', 'physical_threat', 'suspicious_activity',
    'laboratory_incident', 'specimen_containment_breach', 'other',
  ];

  let incidentType = $state('suspicious_activity');
  let location = $state('');
  let description = $state('');
  let severity = $state('medium');
  let occurredAt = $state('');
  let recommendedAction = $state('');
  let submitting = $state(false);
  let error = $state('');
  let success = $state('');

  async function handleSubmit() {
    error = ''; success = '';
    if (!location.trim()) { error = 'Location is required.'; return; }
    if (!description.trim()) { error = 'Description is required.'; return; }

    submitting = true;
    try {
      await sciSubmitSecurityReport({
        incident_type: incidentType,
        location,
        description,
        severity,
        occurred_at: occurredAt || undefined,
        recommended_action: recommendedAction || undefined,
      });
      success = 'Security report submitted to Galactic Security Head.';
      location = ''; description = ''; occurredAt = ''; recommendedAction = '';
      severity = 'medium'; incidentType = 'suspicious_activity';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<h1 class="title">Security Report</h1>
<p class="subtitle">Report a security incident from the Science Division. Submitted directly to Galactic Security Head.</p>

<div class="warning-banner">
  <span class="warn-icon">⚠</span>
  <span>For immediate threats, contact Galactic Security directly. This form is for documented incident reporting.</span>
</div>

<div class="form-card">
  <div class="form-row">
    <label class="field">
      <span class="label">Incident Type *</span>
      <select class="input" bind:value={incidentType}>
        {#each INCIDENT_TYPES as t}
          <option value={t}>{t.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())}</option>
        {/each}
      </select>
    </label>
    <label class="field">
      <span class="label">Severity *</span>
      <select class="input" bind:value={severity}>
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
        <option value="critical">Critical</option>
      </select>
    </label>
  </div>

  <label class="field"><span class="label">Location *</span>
    <input class="input" type="text" bind:value={location} placeholder="Lab section, room, or area where incident occurred…" />
  </label>

  <label class="field"><span class="label">Date/Time of Occurrence</span>
    <input class="input" type="datetime-local" bind:value={occurredAt} />
  </label>

  <label class="field"><span class="label">Description *</span>
    <textarea class="textarea" bind:value={description} rows="5"
      placeholder="Describe what happened, who was involved, and any evidence observed…"></textarea>
  </label>

  <label class="field"><span class="label">Recommended Action</span>
    <textarea class="textarea" bind:value={recommendedAction} rows="3"
      placeholder="What action should be taken in response?"></textarea>
  </label>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}

  <button class="btn-primary" onclick={handleSubmit} disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Security Report'}
  </button>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#F59E0B;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.78rem;margin:0 0 0.75rem;line-height:1.5; }
  .warning-banner { display:flex;align-items:center;gap:0.5rem;background:rgba(245,158,11,0.08);border:1px solid rgba(245,158,11,0.25);border-radius:6px;padding:0.6rem 0.75rem;margin-bottom:0.75rem;font-size:0.78rem;color:#FCD34D;max-width:600px; }
  .warn-icon { font-size:1rem;flex-shrink:0; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(245,158,11,0.15);border-radius:8px;padding:1.25rem;max-width:600px;display:flex;flex-direction:column;gap:0.6rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(245,158,11,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#F59E0B; }
  .textarea { resize:vertical; }
  .form-row { display:grid;grid-template-columns:1fr 1fr;gap:0.6rem; }
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#F59E0B 0%,#EF4444 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
</style>
