<!--
  UC-CE-01: Submit Construction Progress Report (Civil Engineer)
-->
<script lang="ts">
  import { stlSubmitConstructionReport } from '$lib/stores/settlers';

  let taskId = $state('');
  let constructionProgress = $state('');
  let week = $state('');
  let ragStatus = $state('green');
  let issues = $state('');
  let materials = $state<{ material: string; quantity: number; unit: string }[]>([]);
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  function addMaterial() { materials = [...materials, { material: '', quantity: 0, unit: '' }]; }
  function removeMaterial(i: number) { materials = materials.filter((_, idx) => idx !== i); }

  async function handleSubmit() {
    error = ''; success = '';
    if (!taskId.trim()) { error = 'Task ID is required.'; return; }
    if (!constructionProgress.trim()) { error = 'Construction progress is required.'; return; }
    submitting = true;
    try {
      const id = await stlSubmitConstructionReport({
        task_id: taskId,
        construction_progress: constructionProgress,
        week: week || undefined,
        rag_status: ragStatus || undefined,
        materials_used: materials.length ? materials : undefined,
        issues: issues || undefined,
      });
      success = `Construction report submitted (ID: ${id.slice(0,8)}…).`;
      taskId = ''; constructionProgress = ''; week = ''; issues = ''; materials = [];
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }
</script>

<h2>Construction Progress Report</h2>

<form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  <label>
    Task ID *
    <input type="text" bind:value={taskId} placeholder="Paste the assigned task UUID" />
  </label>

  <label>
    Construction Progress *
    <textarea bind:value={constructionProgress} rows="4" placeholder="Describe what was built, progress details…"></textarea>
  </label>

  <div class="row">
    <label>
      Week
      <input type="text" bind:value={week} placeholder="e.g. W12" />
    </label>
    <label>
      RAG Status
      <select bind:value={ragStatus}>
        <option value="green">🟢 Green</option>
        <option value="amber">🟡 Amber</option>
        <option value="red">🔴 Red</option>
      </select>
    </label>
  </div>

  <label>
    Issues / Blockers
    <textarea bind:value={issues} rows="2" placeholder="Any issues encountered…"></textarea>
  </label>

  <div class="materials-section">
    <div class="mat-header">
      <span class="lbl">Materials Used</span>
      <button type="button" class="btn-sm" onclick={addMaterial}>+ Add Material</button>
    </div>
    {#each materials as mat, i}
      <div class="mat-row">
        <input type="text" bind:value={mat.material} placeholder="Material" />
        <input type="number" bind:value={mat.quantity} placeholder="Qty" min="0" />
        <input type="text" bind:value={mat.unit} placeholder="Unit" />
        <button type="button" class="btn-del" onclick={() => removeMaterial(i)}>✕</button>
      </div>
    {/each}
  </div>

  {#if error}<p class="err">{error}</p>{/if}
  {#if success}<p class="ok">{success}</p>{/if}

  <button type="submit" class="btn-primary" disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Report'}
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
  .lbl { font-size:0.8rem;color:#94A3B8; }
  .materials-section { display:flex;flex-direction:column;gap:0.4rem; }
  .mat-header { display:flex;justify-content:space-between;align-items:center; }
  .mat-row { display:flex;gap:0.4rem;align-items:center; }
  .mat-row input { flex:1;background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.35rem 0.45rem;font-size:0.78rem; }
  .mat-row input[type="number"] { max-width:70px; }
  .btn-sm { background:rgba(58,190,255,0.1);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:4px;padding:0.25rem 0.6rem;cursor:pointer;font-size:0.75rem; }
  .btn-sm:hover { background:rgba(58,190,255,0.2); }
  .btn-del { background:transparent;border:1px solid #374151;color:#EF4444;border-radius:4px;padding:0.2rem 0.4rem;cursor:pointer;font-size:0.75rem; }
  .btn-del:hover { background:rgba(239,68,68,0.1); }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem; }
  .btn-primary { background:#3ABEFF;color:#0B0F1A;border:none;border-radius:4px;padding:0.5rem 1.2rem;cursor:pointer;font-weight:600;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:#60CFFF; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
