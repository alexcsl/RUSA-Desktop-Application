<!--
  UC-FA-02: Log Farm Health Check (Farmer — plant + livestock)
-->
<script lang="ts">
  import { stlLogFarmHealth } from '$lib/stores/settlers';

  let subjectType = $state<'plant' | 'livestock'>('plant');
  let subjectName = $state('');
  let logDate = $state(new Date().toISOString().slice(0, 10));
  let condition = $state('');
  let treatment = $state('');
  let notes = $state('');
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  async function handleSubmit() {
    error = ''; success = '';
    if (!subjectName.trim()) { error = `${subjectType === 'plant' ? 'Plant/Crop' : 'Animal'} name is required.`; return; }
    if (!condition.trim()) { error = 'Condition is required.'; return; }
    submitting = true;
    try {
      const id = await stlLogFarmHealth({
        log_date: logDate,
        subject_type: subjectType,
        subject_name: subjectName,
        condition,
        treatment: treatment || undefined,
        notes: notes || undefined,
      });
      success = `Farm health log submitted (ID: ${id.slice(0,8)}…).`;
      subjectName = ''; condition = ''; treatment = ''; notes = '';
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }
</script>

<h2>Farm Health Check</h2>

<form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  <div class="toggle-row">
    <button type="button" class="tog" class:active={subjectType === 'plant'} onclick={() => subjectType = 'plant'}>🌱 Plant</button>
    <button type="button" class="tog" class:active={subjectType === 'livestock'} onclick={() => subjectType = 'livestock'}>🐄 Livestock</button>
  </div>

  <div class="row">
    <label>
      {subjectType === 'plant' ? 'Plant / Crop Name' : 'Animal / Herd Name'} *
      <input type="text" bind:value={subjectName} placeholder={subjectType === 'plant' ? 'e.g. Wheat Field A' : 'e.g. Poultry Barn 2'} />
    </label>
    <label>
      Log Date *
      <input type="date" bind:value={logDate} />
    </label>
  </div>

  <label>
    Condition *
    <input type="text" bind:value={condition} placeholder="e.g. healthy, wilting, infected, recovering" />
  </label>

  <label>
    Treatment Applied
    <input type="text" bind:value={treatment} placeholder="e.g. pesticide spray, antibiotics, pruning…" />
  </label>

  <label>
    Notes / Observations
    <textarea bind:value={notes} rows="3" placeholder="Growth observations, yield estimates, concerns…"></textarea>
  </label>

  {#if error}<p class="err">{error}</p>{/if}
  {#if success}<p class="ok">{success}</p>{/if}

  <button type="submit" class="btn-primary" disabled={submitting}>
    {submitting ? 'Logging…' : 'Submit Health Log'}
  </button>
</form>

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
  .form { display:flex;flex-direction:column;gap:0.75rem;max-width:520px; }
  .form label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8; }
  .form input[type="text"], .form input[type="date"], .form textarea { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem; }
  .form textarea { resize:vertical; }
  .toggle-row { display:flex;gap:0.5rem; }
  .tog { background:#111827;border:1px solid #374151;color:#94A3B8;padding:0.4rem 1rem;border-radius:4px;cursor:pointer;font-size:0.8rem; }
  .tog.active { border-color:#3ABEFF;color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .row { display:flex;gap:1rem; }
  .row label { flex:1; }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem; }
  .btn-primary { background:#3ABEFF;color:#0B0F1A;border:none;border-radius:4px;padding:0.5rem 1.2rem;cursor:pointer;font-weight:600;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:#60CFFF; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
