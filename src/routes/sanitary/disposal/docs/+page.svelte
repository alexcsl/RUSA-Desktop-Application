<!-- /sanitary/disposal/docs — UC-DC-01: Manage Disposal Documentation -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    sanCreateDisposalDoc, sanUpdateDisposalDoc, sanGetDisposalDocs,
    type DisposalDoc,
  } from '$lib/stores/sanitary';

  let docs: DisposalDoc[] = $state([]);
  let selected: DisposalDoc | null = $state(null);
  let error = $state('');
  let success = $state('');
  let mode: 'view' | 'add' | 'edit' = $state('view');

  // Form
  let fWasteCategory = $state('');
  let fProcedure = $state('');
  let fSafetyReqs = $state('');
  let fComplianceNotes = $state('');

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = ''; success = '';
    try { docs = await sanGetDisposalDocs(); } catch (e: unknown) { error = String(e); }
  }

  function startAdd() {
    fWasteCategory = ''; fProcedure = ''; fSafetyReqs = ''; fComplianceNotes = '';
    mode = 'add'; selected = null;
  }

  function startEdit() {
    if (!selected) return;
    fWasteCategory = selected.waste_category;
    fProcedure = selected.procedure;
    fSafetyReqs = selected.safety_requirements ?? '';
    fComplianceNotes = selected.compliance_notes ?? '';
    mode = 'edit';
  }

  async function handleSave() {
    error = ''; success = '';
    try {
      if (mode === 'add') {
        if (!fWasteCategory.trim() || !fProcedure.trim()) { error = 'Waste category and procedure are required.'; return; }
        await sanCreateDisposalDoc({ waste_category: fWasteCategory.trim(), procedure: fProcedure.trim(), safety_requirements: fSafetyReqs.trim() || undefined, compliance_notes: fComplianceNotes.trim() || undefined });
        success = 'Document created.';
      } else if (mode === 'edit' && selected) {
        await sanUpdateDisposalDoc({
          doc_id: selected.id,
          procedure: fProcedure.trim() || undefined,
          safety_requirements: fSafetyReqs.trim() || undefined,
          compliance_notes: fComplianceNotes.trim() || undefined,
          change_summary: 'Updated via UI',
        });
        success = 'Document updated (revision logged).';
      }
      mode = 'view'; selected = null;
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }
</script>

<h1 class="title">Disposal Documentation</h1>
<p class="subtitle">UC-DC-01 — Maintain disposal handling documentation with revision history.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="grid">
  <div class="list-panel">
    <div class="panel-header">
      <span>Documents ({docs.length})</span>
      <button class="btn-primary btn-sm" onclick={startAdd}>+ New</button>
    </div>
    {#each docs as d}
      <button class="card" class:card-selected={selected?.id === d.id} onclick={() => { selected = d; mode = 'view'; }}>
        <span class="card-title">{d.waste_category}</span>
        <span class="card-type">{d.procedure.slice(0, 60)}…</span>
      </button>
    {:else}
      <p class="empty">No disposal docs.</p>
    {/each}
  </div>

  <div class="detail-panel">
    {#if mode === 'view' && selected}
      <h2 class="detail-title">{selected.waste_category}</h2>
      <div class="detail-row"><span class="label">Author</span><span>{selected.author_name ?? selected.authored_by.slice(0, 8)}</span></div>
      <div class="detail-row"><span class="label">Created</span><span>{new Date(selected.created_at).toLocaleString()}</span></div>
      {#if selected.safety_requirements}<div class="detail-row"><span class="label">Safety Requirements</span><span>{selected.safety_requirements}</span></div>{/if}
      {#if selected.compliance_notes}<div class="detail-row"><span class="label">Compliance Notes</span><span>{selected.compliance_notes}</span></div>{/if}
      <div class="procedure-section">
        <span class="label">Procedure</span>
        <p class="procedure-text">{selected.procedure}</p>
      </div>

      {#if selected.revision_history && selected.revision_history.length > 0}
        <h3 class="rev-title">Revision History</h3>
        {#each selected.revision_history as rev}
          <div class="rev-entry">
            <span class="rev-num">Rev {rev.revision}</span>
            <span class="rev-summary">{rev.summary}</span>
            <span class="rev-date">{String(rev.changed_at ?? '').slice(0, 10)}</span>
          </div>
        {/each}
      {/if}

      <div class="detail-actions">
        <button class="btn-primary" onclick={startEdit}>Edit</button>
      </div>
    {:else if mode === 'add' || mode === 'edit'}
      <h2 class="detail-title">{mode === 'add' ? 'New Document' : 'Edit Document'}</h2>
      <div class="form-group"><label for="wc">Waste Category</label><input id="wc" class="input" bind:value={fWasteCategory} placeholder="e.g. Biohazard, Chemical…" /></div>
      <div class="form-group"><label for="proc">Procedure</label><textarea id="proc" class="input" rows="5" bind:value={fProcedure}></textarea></div>
      <div class="form-group"><label for="safety">Safety Requirements</label><textarea id="safety" class="input" rows="2" bind:value={fSafetyReqs} placeholder="Optional…"></textarea></div>
      <div class="form-group"><label for="comp">Compliance Notes</label><textarea id="comp" class="input" rows="2" bind:value={fComplianceNotes} placeholder="Optional…"></textarea></div>
      <div class="detail-actions">
        <button class="btn-primary" onclick={handleSave}>Save</button>
        <button class="btn-secondary" onclick={() => { mode='view'; }}>Cancel</button>
      </div>
    {:else}
      <p class="placeholder-text">Select a document or click <strong>+ New</strong></p>
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .grid { display:grid;grid-template-columns:0.8fr 1.2fr;gap:0.75rem; }
  .list-panel { display:flex;flex-direction:column;gap:0.35rem;max-height:70vh;overflow-y:auto; }
  .panel-header { display:flex;justify-content:space-between;align-items:center;margin-bottom:0.5rem;color:#94A3B8;font-size:0.8rem; }
  .detail-panel { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.55rem;cursor:pointer;text-align:left;width:100%;color:#E6EDF3;transition:border-color .15s;display:flex;flex-direction:column;gap:0.15rem; }
  .card:hover { border-color:rgba(58,190,255,0.3); }
  .card-selected { border-color:#3ABEFF;background:rgba(58,190,255,0.06); }
  .card-title { font-size:0.82rem;font-weight:500; }
  .card-type { font-size:0.7rem;color:#F59E0B; }
  .detail-title { font-size:0.95rem;font-family:'Orbitron',sans-serif;color:#3ABEFF;margin:0 0 0.7rem; }
  .detail-row { display:flex;justify-content:space-between;padding:0.3rem 0;border-bottom:1px solid rgba(58,190,255,0.06);font-size:0.8rem; }
  .label { color:#94A3B8; }
  .procedure-section { margin-top:0.5rem; }
  .procedure-text { font-size:0.8rem;color:#CBD5E1;margin:0.2rem 0 0;white-space:pre-wrap; }
  .rev-title { font-size:0.8rem;color:#94A3B8;margin:0.8rem 0 0.3rem; }
  .rev-entry { display:flex;gap:0.5rem;align-items:center;font-size:0.72rem;padding:0.2rem 0;border-bottom:1px solid rgba(58,190,255,0.03); }
  .rev-num { color:#3ABEFF;font-weight:600;min-width:40px; }
  .rev-summary { color:#CBD5E1;flex:1; }
  .rev-date { color:#64748B;font-size:0.65rem; }
  .detail-actions { display:flex;gap:0.5rem;margin-top:0.8rem; }
  .form-group { display:flex;flex-direction:column;gap:0.15rem;margin-bottom:0.45rem; }
  .form-group label { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  textarea.input { resize:vertical; }
  .btn-sm { padding:0.25rem 0.5rem;font-size:0.7rem; }
  .btn-primary { padding:0.4rem 0.65rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-secondary { padding:0.4rem 0.65rem;background:transparent;border:1px solid #475569;color:#94A3B8;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .placeholder-text { color:#475569;font-size:0.85rem;text-align:center;padding:2rem 0; }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
</style>
