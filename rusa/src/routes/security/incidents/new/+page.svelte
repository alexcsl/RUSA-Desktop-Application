<!-- /security/incidents/new — Create Incident Report (UC-SH-02 / UC-SS-01) -->
<script lang="ts">
  import { createIncidentReport } from '$lib/stores/security';

  let source = $state('direct_observation');
  let incidentType = $state('');
  let location = $state('');
  let occurredAt = $state('');
  let description = $state('');
  let severity = $state('medium');
  let recommendedAction = $state('');
  let sectorOrBase = $state('');
  let compromisedParty = $state('');
  let responseProcedure = $state('');
  let incidentStart = $state('');
  let incidentEnd = $state('');
  let categories: string[] = $state([]);

  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  const categoryOptions = [
    'unauthorized_access', 'physical_security', 'equipment_malfunction',
    'surveillance', 'perimeter_breach', 'structural_integrity',
    'personnel_safety', 'data_breach', 'environmental_hazard', 'other',
  ];

  function toggleCategory(cat: string) {
    if (categories.includes(cat)) {
      categories = categories.filter((c) => c !== cat);
    } else {
      categories = [...categories, cat];
    }
  }

  async function handleSubmit() {
    error = ''; success = '';
    if (!incidentType.trim()) { error = 'Incident type is required.'; return; }
    if (!location.trim()) { error = 'Location is required.'; return; }
    if (!description.trim()) { error = 'Description is required.'; return; }

    submitting = true;
    try {
      const meta: Record<string, unknown> = {};
      if (compromisedParty.trim()) meta.compromised_party = compromisedParty;
      if (categories.length > 0) meta.category = categories;
      if (responseProcedure.trim()) meta.response_procedure = responseProcedure;
      if (incidentStart.trim()) meta.incident_start = incidentStart;
      if (incidentEnd.trim()) meta.incident_end = incidentEnd;

      await createIncidentReport({
        source,
        incident_type: incidentType,
        location,
        occurred_at: occurredAt ? new Date(occurredAt).toISOString() : undefined,
        description,
        severity,
        recommended_action: recommendedAction || undefined,
        sector_or_base: sectorOrBase || undefined,
        incident_meta: Object.keys(meta).length > 0 ? meta : undefined,
      });

      success = 'Incident report created successfully.';
      // Reset form
      incidentType = ''; location = ''; occurredAt = ''; description = '';
      severity = 'medium'; recommendedAction = ''; sectorOrBase = '';
      compromisedParty = ''; responseProcedure = ''; incidentStart = '';
      incidentEnd = ''; categories = []; source = 'direct_observation';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<h1 class="title">Create Incident Report</h1>
<p class="subtitle">Document a security incident from direct observation, external report, or assignment.</p>

<div class="form-card">
  <h2>Incident Details</h2>

  <label class="field"><span class="label">Source *</span>
    <select class="input" bind:value={source}>
      <option value="direct_observation">Direct Observation</option>
      <option value="external_report">External Report</option>
      <option value="assignment">Assignment</option>
    </select>
  </label>

  <label class="field"><span class="label">Incident Type *</span>
    <input type="text" class="input" bind:value={incidentType} placeholder="e.g. Unauthorized Access, Equipment Malfunction…" />
  </label>

  <label class="field"><span class="label">Location *</span>
    <input type="text" class="input" bind:value={location} placeholder="e.g. Sector Alpha — Docking Bay 3" />
  </label>

  <label class="field"><span class="label">Occurred At</span>
    <input type="datetime-local" class="input" bind:value={occurredAt} />
  </label>

  <label class="field"><span class="label">Severity *</span>
    <select class="input" bind:value={severity}>
      <option value="low">Low</option>
      <option value="medium">Medium</option>
      <option value="high">High</option>
      <option value="critical">Critical</option>
    </select>
  </label>

  <label class="field"><span class="label">Sector / Base</span>
    <input type="text" class="input" bind:value={sectorOrBase} placeholder="e.g. Sector Alpha" />
  </label>

  <label class="field"><span class="label">Description *</span>
    <textarea class="textarea" bind:value={description} rows="5" placeholder="Detailed incident description…"></textarea>
  </label>

  <label class="field"><span class="label">Recommended Action</span>
    <textarea class="textarea" bind:value={recommendedAction} rows="3" placeholder="Recommended response or follow-up actions…"></textarea>
  </label>

  <div class="field">
    <span class="label">Category</span>
    <div class="checkbox-group">
      {#each categoryOptions as cat}
        <label class="checkbox-item">
          <input type="checkbox" checked={categories.includes(cat)} onchange={() => toggleCategory(cat)} />
          <span>{cat.replace(/_/g, ' ')}</span>
        </label>
      {/each}
    </div>
  </div>

  <label class="field"><span class="label">Compromised Party</span>
    <input type="text" class="input" bind:value={compromisedParty} placeholder="e.g. Docking Bay 3 Access Control" />
  </label>

  <label class="field"><span class="label">Response Procedure</span>
    <textarea class="textarea" bind:value={responseProcedure} rows="2" placeholder="Response steps taken or planned…"></textarea>
  </label>

  <div class="row-fields">
    <label class="field half"><span class="label">Incident Start</span>
      <input type="datetime-local" class="input" bind:value={incidentStart} />
    </label>
    <label class="field half"><span class="label">Incident End</span>
      <input type="datetime-local" class="input" bind:value={incidentEnd} />
    </label>
  </div>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}

  <button class="btn-primary" onclick={handleSubmit} disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Report'}
  </button>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:650px;display:flex;flex-direction:column;gap:0.6rem; }
  .form-card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .checkbox-group { display:flex;flex-wrap:wrap;gap:0.4rem; }
  .checkbox-item { display:flex;align-items:center;gap:0.3rem;font-size:0.75rem;color:#94A3B8;cursor:pointer; }
  .checkbox-item input { accent-color:#3ABEFF; }
  .checkbox-item span { text-transform:capitalize; }
  .row-fields { display:flex;gap:0.75rem; }
  .half { flex:1; }
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
</style>
