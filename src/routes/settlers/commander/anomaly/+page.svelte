<!--
  UC-SC-02A: Commander Anomaly Report (submitted directly to Directors)
-->
<script lang="ts">
  import { stlSubmitCommanderAnomaly } from '$lib/stores/settlers';

  let description = $state('');
  let location = $state('');
  let dangerLevel = $state('medium');
  let severity = $state('medium');
  let category: string[] = $state([]);
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  function toggleCat(c: string) {
    category = category.includes(c) ? category.filter(x => x !== c) : [...category, c];
  }

  async function handleSubmit() {
    error = ''; success = '';
    if (!description.trim()) { error = 'Description is required.'; return; }
    if (!location.trim()) { error = 'Location is required.'; return; }
    if (category.length === 0) { error = 'Select at least one category.'; return; }
    submitting = true;
    try {
      const id = await stlSubmitCommanderAnomaly({
        description,
        location_in_settlement: location,
        danger_level: dangerLevel,
        severity,
        category,
      });
      success = `Commander anomaly report submitted (ID: ${id.slice(0,8)}…). Directors notified.`;
      description = ''; location = ''; category = [];
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }
</script>

<h2>Commander Anomaly Report</h2>
<p class="hint">This report is submitted directly to the Directors board (bypasses the queue).</p>

<form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  <label>
    Description *
    <textarea bind:value={description} rows="4" required
      placeholder="Describe the anomaly in detail…"></textarea>
  </label>

  <label>
    Location *
    <input type="text" bind:value={location} placeholder="e.g. Sector 4B – East Greenhouse" />
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
      Severity
      <select bind:value={severity}>
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
        <option value="critical">Critical</option>
      </select>
    </label>
  </div>

  <fieldset class="cats">
    <legend>Category *</legend>
    {#each ['structural','biological','chemical','other'] as c}
      <label class="chk">
        <input type="checkbox" checked={category.includes(c)} onchange={() => toggleCat(c)} />
        {c}
      </label>
    {/each}
  </fieldset>

  {#if error}<p class="err">{error}</p>{/if}
  {#if success}<p class="ok">{success}</p>{/if}

  <button type="submit" class="btn-primary" disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit to Directors'}
  </button>
</form>

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 0.3rem; }
  .hint { color:#64748B;font-size:0.75rem;margin:0 0 1rem; }
  .form { display:flex;flex-direction:column;gap:0.75rem;max-width:560px; }
  .form label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8; }
  .form input, .form select, .form textarea { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem; }
  .form textarea { resize:vertical; }
  .row { display:flex;gap:1rem; }
  .row label { flex:1; }
  .cats { border:1px solid #374151;border-radius:4px;padding:0.5rem;display:flex;gap:1rem; }
  .cats legend { color:#94A3B8;font-size:0.75rem;padding:0 0.3rem; }
  .chk { flex-direction:row!important;align-items:center;gap:0.3rem;text-transform:capitalize; }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem; }
  .btn-primary { background:#3ABEFF;color:#0B0F1A;border:none;border-radius:4px;padding:0.5rem 1.2rem;cursor:pointer;font-weight:600;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:#60CFFF; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
