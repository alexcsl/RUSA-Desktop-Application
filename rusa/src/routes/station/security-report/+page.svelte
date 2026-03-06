<!-- /station/security-report — UC-SSS-01: Report Finding to Galactic Security -->
<script lang="ts">
  import { getContext } from 'svelte';
  import type { Writable } from 'svelte/store';
  import { sstReportFinding } from '$lib/stores/space_station';

  const stationId = getContext<Writable<string>>('selectedStationId');

  let incidentType = $state('');
  let location = $state('');
  let occurredAt = $state('');
  let description = $state('');
  let severity = $state('medium');
  let recommendedAction = $state('');
  let sectorOrBase = $state('');

  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  let currentStationId = $state('');
  const unsub = stationId.subscribe((v) => (currentStationId = v));

  async function handleSubmit() {
    error = ''; success = '';
    if (!currentStationId) { error = 'No station selected.'; return; }
    if (!incidentType.trim()) { error = 'Incident type is required.'; return; }
    if (!location.trim()) { error = 'Location is required.'; return; }
    if (!description.trim()) { error = 'Description is required.'; return; }

    submitting = true;
    try {
      await sstReportFinding({
        station_id: currentStationId,
        incident_type: incidentType,
        location,
        occurred_at: occurredAt ? new Date(occurredAt).toISOString() : undefined,
        description,
        severity,
        recommended_action: recommendedAction || undefined,
        sector_or_base: sectorOrBase || undefined,
      });
      success = 'Finding reported to Galactic Security successfully.';
      incidentType = ''; location = ''; occurredAt = ''; description = '';
      severity = 'medium'; recommendedAction = ''; sectorOrBase = '';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<h1 class="title">Report Finding to Galactic Security</h1>
<p class="subtitle">Submit a finding to the nearest Galactic Security team. Reports are delivered in real-time.</p>

<div class="form-card">
  <h2>Finding Details</h2>

  <label class="field"><span class="label">Incident Type *</span>
    <input type="text" class="input" bind:value={incidentType} placeholder="e.g. Unauthorized Access, Structural Anomaly…" />
  </label>

  <label class="field"><span class="label">Location *</span>
    <input type="text" class="input" bind:value={location} placeholder="e.g. Deck 3 — Storage Bay" />
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
    <input type="text" class="input" bind:value={sectorOrBase} placeholder="e.g. Sector 1" />
  </label>

  <label class="field"><span class="label">Description *</span>
    <textarea class="textarea" bind:value={description} rows="5" placeholder="Detailed description of the finding…"></textarea>
  </label>

  <label class="field"><span class="label">Recommended Action</span>
    <textarea class="textarea" bind:value={recommendedAction} rows="3" placeholder="Recommended response or follow-up…"></textarea>
  </label>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}

  <button class="btn-primary" onclick={handleSubmit} disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Finding'}
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
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
</style>
