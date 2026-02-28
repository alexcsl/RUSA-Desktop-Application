<!-- /station/archive/new — UC-SSS-02: Add Finding to Private Station Archive -->
<script lang="ts">
  import { getContext } from 'svelte';
  import type { Writable } from 'svelte/store';
  import { sstAddToArchive } from '$lib/stores/space_station';

  const stationId = getContext<Writable<string>>('selectedStationId');

  let findingType = $state('');
  let archiveDescription = $state('');
  let severityLevel = $state('medium');
  let isReportedToSecurity = $state(false);
  let locationOnStation = $state('');
  let responseProcedure = $state('');
  let categories: string[] = $state([]);

  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  let currentStationId = $state('');
  const unsub = stationId.subscribe((v) => (currentStationId = v));

  const categoryOptions = [
    'structural', 'environmental', 'biological', 'equipment',
    'security', 'navigation', 'communication', 'power_systems', 'other',
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
    if (!currentStationId) { error = 'No station selected.'; return; }
    if (!findingType.trim()) { error = 'Finding type is required.'; return; }
    if (!archiveDescription.trim()) { error = 'Description is required.'; return; }

    submitting = true;
    try {
      const archiveData: Record<string, unknown> = {
        description: archiveDescription,
        severity: severityLevel,
        is_reported_to_security: isReportedToSecurity,
      };
      if (locationOnStation.trim()) archiveData.location_on_station = locationOnStation;
      if (responseProcedure.trim()) archiveData.response_procedure = responseProcedure;
      if (categories.length > 0) archiveData.category = categories;

      await sstAddToArchive({
        station_id: currentStationId,
        finding_type: findingType,
        archive_data: archiveData,
      });

      success = 'Finding added to the private archive.';
      findingType = ''; archiveDescription = ''; severityLevel = 'medium';
      isReportedToSecurity = false; locationOnStation = ''; responseProcedure = '';
      categories = [];
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<h1 class="title">Add to Private Archive</h1>
<p class="subtitle">Record an internal finding to the station's private archive. Only station settlers can access these.</p>

<div class="form-card">
  <h2>Archive Entry</h2>

  <label class="field"><span class="label">Finding Type *</span>
    <input type="text" class="input" bind:value={findingType} placeholder="e.g. Structural Anomaly, Equipment Malfunction…" />
  </label>

  <label class="field"><span class="label">Severity</span>
    <select class="input" bind:value={severityLevel}>
      <option value="low">Low</option>
      <option value="medium">Medium</option>
      <option value="high">High</option>
      <option value="critical">Critical</option>
    </select>
  </label>

  <label class="field"><span class="label">Location on Station</span>
    <input type="text" class="input" bind:value={locationOnStation} placeholder="e.g. Deck 2 — Engine Room" />
  </label>

  <label class="field"><span class="label">Description *</span>
    <textarea class="textarea" bind:value={archiveDescription} rows="5" placeholder="Detailed description of the finding…"></textarea>
  </label>

  <label class="field"><span class="label">Response Procedure</span>
    <textarea class="textarea" bind:value={responseProcedure} rows="3" placeholder="Steps taken or recommended…"></textarea>
  </label>

  <div class="field">
    <span class="label">Categories</span>
    <div class="checkbox-group">
      {#each categoryOptions as cat}
        <label class="checkbox-item">
          <input type="checkbox" checked={categories.includes(cat)} onchange={() => toggleCategory(cat)} />
          <span>{cat.replace(/_/g, ' ')}</span>
        </label>
      {/each}
    </div>
  </div>

  <label class="checkbox-field">
    <input type="checkbox" bind:checked={isReportedToSecurity} />
    <span>Also reported to Galactic Security</span>
  </label>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}

  <button class="btn-primary" onclick={handleSubmit} disabled={submitting}>
    {submitting ? 'Saving…' : 'Save to Archive'}
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
  .checkbox-field { display:flex;align-items:center;gap:0.4rem;font-size:0.8rem;color:#94A3B8;cursor:pointer; }
  .checkbox-field input { accent-color:#3ABEFF; }
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
</style>
