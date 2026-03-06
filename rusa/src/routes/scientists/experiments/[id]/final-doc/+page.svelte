<!--
  /scientists/experiments/[id]/final-doc — UC-PH-08 / UC-CH-07 / UC-BIO-04: Submit Final Document
  document_data JSONB shape varies by type (matter / physical_object / species).
-->
<script lang="ts">
  import { page } from '$app/stores';
  import { submitFinalDocument } from '$lib/stores/scientists';
  import { goto } from '$app/navigation';

  let experimentId = $state('');
  page.subscribe((p) => (experimentId = p.params.id ?? ''));

  /* Auto-determine doc type from user role (injected by layout) */
  let docType = $state('matter'); // default — overridden below
  let role: string | null = null;
  try { role = localStorage.getItem('user_role'); } catch {}
  if (role === 'Physicist')  docType = 'physical_object';
  if (role === 'Chemist')    docType = 'matter';
  if (role === 'Biologist')  docType = 'species';

  /* -- Common fields -- */
  let name = $state('');
  let description = $state('');

  /* -- Matter (Chemist) -- */
  let chemicalFormula = $state('');
  let state_of_matter = $state('solid');
  let hazard = $state('');

  /* -- Physical Object (Physicist) -- */
  let mass = $state('');
  let dimensions = $state('');
  let materialComposition = $state('');

  /* -- Species (Biologist) -- */
  let classification = $state('');
  let habitat = $state('');
  let diet = $state('');

  let submitting = $state(false);
  let message = $state('');
  let isError = $state(false);

  function buildDocData(): Record<string, unknown> {
    const base: Record<string, unknown> = { name, description };
    if (docType === 'matter') {
      return { ...base, chemical_formula: chemicalFormula, state_of_matter, hazard: hazard || undefined };
    }
    if (docType === 'physical_object') {
      return { ...base, mass: mass ? parseFloat(mass) : undefined, dimensions, material_composition: materialComposition };
    }
    /* species */
    return { ...base, classification, habitat, diet: diet || undefined };
  }

  async function handleSubmit() {
    if (!name.trim()) return;
    submitting = true;
    message = '';
    try {
      await submitFinalDocument({
        experiment_id: experimentId,
        doc_type: docType,
        document_data: buildDocData(),
      });
      message = 'Final document submitted.';
      isError = false;
    } catch (e: any) {
      message = e?.toString() ?? 'Failed to submit final document.';
      isError = true;
    } finally {
      submitting = false;
    }
  }
</script>

<h2>Submit Final Document</h2>
<p class="hint">
  Type: <strong>{docType.replace(/_/g, ' ')}</strong> — Experiment <code>{experimentId.slice(0, 8)}…</code>
</p>

<div class="form-card">
  <!-- Common -->
  <div class="form-group">
    <label for="name">Name *</label>
    <input id="name" bind:value={name} placeholder="Object / substance / species name" />
  </div>
  <div class="form-group">
    <label for="desc">Description</label>
    <textarea id="desc" bind:value={description} rows="3" placeholder="General description…"></textarea>
  </div>

  {#if docType === 'matter'}
    <fieldset class="section">
      <legend>Matter Details</legend>
      <div class="row">
        <div class="form-group" style="flex:1">
          <label for="formula">Chemical Formula</label>
          <input id="formula" bind:value={chemicalFormula} placeholder="e.g. H₂O" />
        </div>
        <div class="form-group" style="flex:1">
          <label for="som">State of Matter</label>
          <select id="som" bind:value={state_of_matter}>
            <option value="solid">Solid</option>
            <option value="liquid">Liquid</option>
            <option value="gas">Gas</option>
            <option value="plasma">Plasma</option>
          </select>
        </div>
      </div>
      <div class="form-group">
        <label for="hazard">Hazard Notes</label>
        <input id="hazard" bind:value={hazard} placeholder="Any safety concerns" />
      </div>
    </fieldset>
  {:else if docType === 'physical_object'}
    <fieldset class="section">
      <legend>Physical Object Details</legend>
      <div class="row">
        <div class="form-group" style="flex:1">
          <label for="mass">Mass (kg)</label>
          <input id="mass" type="number" step="any" bind:value={mass} />
        </div>
        <div class="form-group" style="flex:1">
          <label for="dim">Dimensions</label>
          <input id="dim" bind:value={dimensions} placeholder="e.g. 10×5×3 cm" />
        </div>
      </div>
      <div class="form-group">
        <label for="mat">Material Composition</label>
        <input id="mat" bind:value={materialComposition} placeholder="e.g. aluminium alloy" />
      </div>
    </fieldset>
  {:else}
    <fieldset class="section">
      <legend>Species Details</legend>
      <div class="form-group">
        <label for="class">Classification</label>
        <input id="class" bind:value={classification} placeholder="Kingdom > Phylum > …" />
      </div>
      <div class="row">
        <div class="form-group" style="flex:1">
          <label for="hab">Habitat</label>
          <input id="hab" bind:value={habitat} placeholder="Terrestrial / Aquatic / …" />
        </div>
        <div class="form-group" style="flex:1">
          <label for="diet">Diet</label>
          <input id="diet" bind:value={diet} placeholder="Herbivore / Omnivore / …" />
        </div>
      </div>
    </fieldset>
  {/if}

  <div class="btn-row">
    <button class="btn-primary" onclick={handleSubmit} disabled={submitting || !name.trim()}>
      {submitting ? 'Submitting…' : 'Submit Final Document'}
    </button>
    <button class="btn-secondary" onclick={() => goto(`/scientists/experiments/${experimentId}`)}>Back</button>
  </div>
  {#if message}<p class:msg-ok={!isError} class:msg-err={isError}>{message}</p>{/if}
</div>

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.5rem; }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:1rem; }
  .hint code { color:#A5B4FC; }
  .hint strong { color:#E6EDF3;text-transform:capitalize; }
  .form-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:650px; }
  .row { display:flex;gap:1rem; }
  .section { border:1px solid #1E293B;border-radius:6px;padding:0.75rem;margin-bottom:0.75rem; }
  .section legend { color:#3ABEFF;font-size:0.75rem;padding:0 0.25rem; }
  .form-group { margin-bottom:0.75rem; }
  .form-group label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.25rem; }
  .form-group input, .form-group textarea, .form-group select { width:100%;background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem;font-size:0.8rem; }
  .form-group textarea { resize:vertical;font-family:'Fira Code',monospace; }
  .btn-row { display:flex;gap:0.75rem;margin-top:0.25rem; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-secondary { background:transparent;border:1px solid #475569;color:#94A3B8;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-secondary:hover { color:#E6EDF3;border-color:#3ABEFF; }
  .msg-ok { color:#22C55E;font-size:0.8rem;margin-top:0.5rem; }
  .msg-err { color:#EF4444;font-size:0.8rem;margin-top:0.5rem; }
</style>
