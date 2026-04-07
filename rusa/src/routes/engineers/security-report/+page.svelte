<!--
  /engineers/security-report — Submit Security Incident Report (UC-ENG-SEC-01)
  Agricultural and Biological Engineers report security findings encountered
  during field experiments to the Galactic Security team.
-->
<script lang="ts">
  import { submitSecurityReport } from '$lib/stores/engineers';

  const INCIDENT_TYPES = [
    'unauthorized_access', 'suspicious_activity', 'equipment_sabotage',
    'environmental_hazard', 'biological_containment_breach',
    'unknown_entity', 'structural_damage', 'other',
  ];

  let incidentType = $state('suspicious_activity');
  let location = $state('');
  let description = $state('');
  let severity = $state('medium');
  let occurredAt = $state('');
  let recommendedAction = $state('');
  let loading = $state(false);
  let error = $state('');
  let success = $state('');
  let reportId = $state('');

  async function handleSubmit() {
    error = ''; success = '';
    if (!location.trim() || !description.trim()) {
      error = 'Location and description are required.';
      return;
    }
    loading = true;
    try {
      const id = await submitSecurityReport({
        incident_type: incidentType,
        location: location.trim(),
        description: description.trim(),
        severity,
        occurred_at: occurredAt ? new Date(occurredAt).toISOString() : undefined,
        recommended_action: recommendedAction.trim() || undefined,
      });
      reportId = id;
      success = `Report submitted. ID: ${String(id).slice(0, 8)}…`;
      location = ''; description = ''; severity = 'medium';
      occurredAt = ''; recommendedAction = ''; incidentType = 'suspicious_activity';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }
</script>

<h1 class="title">Security Incident Report</h1>
<p class="subtitle">
  Report security-relevant findings from field experiments to the Galactic Security team.
  All reports are logged and forwarded to the Security Head immediately.
</p>

<div class="alert-info">
  ⚠ Use this form for any security concern observed during or related to your experiments.
  Non-urgent observations should still be reported — the security team will triage severity.
</div>

<div class="form-card">
  <div class="form-row">
    <label class="field">
      <span class="label">Incident Type *</span>
      <select class="input" bind:value={incidentType}>
        {#each INCIDENT_TYPES as t}
          <option value={t}>{t.replace(/_/g, ' ')}</option>
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

  <label class="field">
    <span class="label">Location *</span>
    <input class="input" type="text" bind:value={location} placeholder="e.g. Lab B-4, Sector 7, Outpost Gamma" />
  </label>

  <label class="field">
    <span class="label">Date / Time Occurred</span>
    <input class="input" type="datetime-local" bind:value={occurredAt} />
  </label>

  <label class="field">
    <span class="label">Description *</span>
    <textarea class="textarea" bind:value={description} rows="5"
      placeholder="Describe what you observed in detail. Include any relevant context from your experiment…"></textarea>
  </label>

  <label class="field">
    <span class="label">Recommended Action</span>
    <textarea class="textarea" bind:value={recommendedAction} rows="2"
      placeholder="Optional — your suggested response or containment steps…"></textarea>
  </label>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}

  <button class="btn-submit" onclick={handleSubmit} disabled={loading}>
    {loading ? 'Submitting…' : 'Submit Report'}
  </button>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#F59E0B;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem;line-height:1.5; }
  .alert-info { background:rgba(245,158,11,0.07);border:1px solid rgba(245,158,11,0.2);border-radius:6px;padding:0.55rem 0.8rem;color:#FCD34D;font-size:0.78rem;margin-bottom:1.2rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(245,158,11,0.1);border-radius:8px;padding:1.5rem;max-width:580px;display:flex;flex-direction:column;gap:0.8rem; }
  .form-row { display:grid;grid-template-columns:1fr 1fr;gap:0.8rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem 0.6rem;font-size:0.82rem;font-family:'Inter',sans-serif;width:100%;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#F59E0B; }
  .textarea { resize:vertical; }
  .btn-submit { background:rgba(245,158,11,0.15);border:1px solid #F59E0B;color:#F59E0B;border-radius:6px;padding:0.55rem;cursor:pointer;font-size:0.85rem;font-weight:600;margin-top:0.25rem; }
  .btn-submit:hover:not(:disabled) { background:rgba(245,158,11,0.25); }
  .btn-submit:disabled { opacity:0.5;cursor:default; }
  .error { color:#EF4444;font-size:0.75rem; }
  .success { color:#22C55E;font-size:0.75rem; }
</style>
