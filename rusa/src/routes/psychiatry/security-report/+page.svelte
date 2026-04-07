<!-- /psychiatry/security-report — Submit security incident report (Psychiatrist + PsychiatristAssistant) -->
<script lang="ts">
  import { psySubmitSecurityReport, type PsySubmitSecurityReportPayload } from '$lib/stores/psychiatry';

  const INCIDENT_TYPES = [
    'unauthorized_access',
    'patient_safety_breach',
    'physical_threat',
    'data_breach',
    'equipment_tampering',
    'suspicious_activity',
    'staff_conduct',
    'other',
  ];

  const SEVERITY_LEVELS = ['low', 'medium', 'high', 'critical'];

  let incidentType = $state('');
  let location = $state('');
  let severity = $state('');
  let description = $state('');
  let occurredAt = $state('');
  let recommendedAction = $state('');

  let submitting = $state(false);
  let successMsg = $state('');
  let errorMsg = $state('');

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!incidentType || !location || !severity || !description) {
      errorMsg = 'Please fill in all required fields.';
      return;
    }
    submitting = true;
    errorMsg = '';
    successMsg = '';
    try {
      const payload: PsySubmitSecurityReportPayload = {
        incident_type: incidentType,
        location,
        severity,
        description,
        occurred_at: occurredAt || undefined,
        recommended_action: recommendedAction || undefined,
      };
      await psySubmitSecurityReport(payload);
      successMsg = 'Security report submitted. Galactic Security has been notified.';
      incidentType = '';
      location = '';
      severity = '';
      description = '';
      occurredAt = '';
      recommendedAction = '';
    } catch (err) {
      errorMsg = String(err);
    } finally {
      submitting = false;
    }
  }
</script>

<h1 class="title">Security Incident Report</h1>
<p class="subtitle">Reports are submitted confidentially to Galactic Security.</p>

{#if successMsg}
  <div class="banner success">{successMsg}</div>
{/if}
{#if errorMsg}
  <div class="banner error">{errorMsg}</div>
{/if}

<form class="report-form" onsubmit={handleSubmit}>
  <div class="field">
    <label for="incident-type">Incident Type <span class="req">*</span></label>
    <select id="incident-type" bind:value={incidentType} required>
      <option value="">— Select —</option>
      {#each INCIDENT_TYPES as t}
        <option value={t}>{t.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())}</option>
      {/each}
    </select>
  </div>

  <div class="field">
    <label for="location">Location <span class="req">*</span></label>
    <input id="location" type="text" bind:value={location} placeholder="e.g. Psychiatry Wing, Office 4" required />
  </div>

  <div class="field">
    <label for="severity">Severity <span class="req">*</span></label>
    <select id="severity" bind:value={severity} required>
      <option value="">— Select —</option>
      {#each SEVERITY_LEVELS as s}
        <option value={s}>{s.charAt(0).toUpperCase() + s.slice(1)}</option>
      {/each}
    </select>
  </div>

  <div class="field">
    <label for="occurred-at">Date / Time of Incident</label>
    <input id="occurred-at" type="datetime-local" bind:value={occurredAt} />
  </div>

  <div class="field">
    <label for="description">Description <span class="req">*</span></label>
    <textarea id="description" bind:value={description} rows="5" placeholder="Describe the incident in detail..." required></textarea>
  </div>

  <div class="field">
    <label for="recommended-action">Recommended Action</label>
    <textarea id="recommended-action" bind:value={recommendedAction} rows="3" placeholder="Optional: suggest a course of action..."></textarea>
  </div>

  <button type="submit" class="btn-submit" disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Report'}
  </button>
</form>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#C084FC;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1.5rem; }
  .banner { padding:0.65rem 1rem;border-radius:6px;margin-bottom:1rem;font-size:0.82rem; }
  .banner.success { background:rgba(16,185,129,0.12);border:1px solid rgba(16,185,129,0.3);color:#34D399; }
  .banner.error { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);color:#F87171; }
  .report-form { display:flex;flex-direction:column;gap:1rem;max-width:560px; }
  .field { display:flex;flex-direction:column;gap:0.3rem; }
  .field label { font-size:0.78rem;color:#94A3B8;text-transform:uppercase;letter-spacing:0.04em; }
  .req { color:#EF4444; }
  .field input, .field select, .field textarea {
    background:#0E1428;border:1px solid rgba(139,92,246,0.2);color:#E6EDF3;border-radius:6px;
    padding:0.5rem 0.65rem;font-size:0.85rem;font-family:'Inter',sans-serif;
  }
  .field input:focus, .field select:focus, .field textarea:focus {
    outline:none;border-color:rgba(139,92,246,0.5);
  }
  .field textarea { resize:vertical; }
  .btn-submit {
    align-self:flex-start;background:rgba(139,92,246,0.15);border:1px solid #8B5CF6;
    color:#C084FC;border-radius:6px;padding:0.55rem 1.5rem;cursor:pointer;
    font-size:0.85rem;font-weight:600;
  }
  .btn-submit:hover:not(:disabled) { background:rgba(139,92,246,0.28); }
  .btn-submit:disabled { opacity:0.5;cursor:not-allowed; }
</style>
