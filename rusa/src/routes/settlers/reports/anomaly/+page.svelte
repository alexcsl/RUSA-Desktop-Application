<!--
  UC-PS-03 / UC-TS-01: Submit Anomaly Report
-->
<script lang="ts">
  import { stlSubmitAnomalyReport } from '$lib/stores/settlers';

  let description = $state('');
  let locationInSettlement = $state('');
  let dangerLevel = $state('low');
  let severity = $state('low');
  let categories: string[] = $state([]);
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  function toggleCat(cat: string) {
    if (categories.includes(cat)) categories = categories.filter((c) => c !== cat);
    else categories = [...categories, cat];
  }

  async function handleSubmit() {
    error = ''; success = '';
    if (!description.trim()) { error = 'Description is required.'; return; }
    submitting = true;
    try {
      const id = await stlSubmitAnomalyReport({
        description,
        location_in_settlement: locationInSettlement || undefined,
        danger_level: dangerLevel,
        severity,
        category: categories.length > 0 ? categories : undefined,
      });
      success = `Anomaly report submitted (ID: ${id.slice(0,8)}…). Commander has been notified.`;
      description = ''; locationInSettlement = ''; categories = [];
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }
</script>

<h2>Anomaly Report</h2>

<form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  <label>
    Description *
    <textarea bind:value={description} rows="4" required
      placeholder="Describe the anomaly in detail…"></textarea>
  </label>

  <label>
    Location in Settlement
    <input type="text" bind:value={locationInSettlement} placeholder="e.g. Dome Section C-7" />
  </label>

  <div class="row">
    <label>
      Danger Level
      <select bind:value={dangerLevel}>
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
        <option value="critical">Critical</option>
      </select>
    </label>

    <label>
      Severity *
      <select bind:value={severity}>
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
        <option value="critical">Critical</option>
      </select>
    </label>
  </div>

  <fieldset class="cat-group">
    <legend>Category</legend>
    {#each ['structural', 'biological', 'chemical', 'other'] as cat}
      <label class="cat-label">
        <input type="checkbox" checked={categories.includes(cat)} onchange={() => toggleCat(cat)} />
        {cat.charAt(0).toUpperCase() + cat.slice(1)}
      </label>
    {/each}
  </fieldset>

  {#if error}<p class="err">{error}</p>{/if}
  {#if success}<p class="ok">{success}</p>{/if}

  <button type="submit" class="btn-primary" disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Anomaly Report'}
  </button>
</form>

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
  .form { display:flex;flex-direction:column;gap:0.75rem;max-width:560px; }
  .form label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8; }
  .form input, .form select, .form textarea { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem; }
  .form textarea { resize:vertical; }
  .row { display:flex;gap:1rem; }
  .row label { flex:1; }
  .cat-group { border:1px solid #374151;border-radius:4px;padding:0.5rem 0.75rem;display:flex;gap:1rem;flex-wrap:wrap; }
  .cat-group legend { color:#94A3B8;font-size:0.75rem;padding:0 0.3rem; }
  .cat-label { display:flex;align-items:center;gap:0.3rem;font-size:0.8rem;color:#E6EDF3;flex-direction:row; }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem; }
  .btn-primary { background:#3ABEFF;color:#0B0F1A;border:none;border-radius:4px;padding:0.5rem 1.2rem;cursor:pointer;font-weight:600;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:#60CFFF; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
