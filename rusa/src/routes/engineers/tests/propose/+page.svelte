<!--
  /engineers/tests/propose — UC-AGE-02 / UC-BE-03: Propose New Test
  Routes to The Observer for approval.
-->
<script lang="ts">
  import { proposeNewTest } from '$lib/stores/engineers';
  import { currentUser } from '$lib/stores/auth';
  import type { SessionUser } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { onDestroy } from 'svelte';

  let user: SessionUser | null = $state(null);
  const unsubUser = currentUser.subscribe((v) => (user = v));
  onDestroy(unsubUser);

  let name = $state('');
  let goal = $state('');
  let procedure = $state('');
  let speciesScope = $state('');
  let category = $state('agricultural');
  let apparatuses = $state('');
  let requiredData = $state('');
  let justification = $state('');
  let submitting = $state(false);
  let message = $state('');
  let isError = $state(false);

  /* Auto-set category from role */
  $effect(() => {
    if (user?.role === 'AgriculturalEngineer') category = 'agricultural';
    if (user?.role === 'BiologicalEngineer')   category = 'biological';
  });

  async function handleSubmit() {
    if (!name.trim() || !goal.trim()) return;
    submitting = true;
    message = '';
    try {
      await proposeNewTest({
        name,
        goal,
        procedure: procedure || undefined,
        species_scope: speciesScope || undefined,
        category: category ? [category] : [],
        apparatuses: apparatuses || undefined,
        required_data: requiredData || undefined,
        justification: justification || undefined,
      });
      message = 'Test proposal submitted for Observer review.';
      isError = false;
      name = ''; goal = ''; procedure = ''; speciesScope = '';
      apparatuses = ''; requiredData = ''; justification = '';
    } catch (e: any) {
      message = e?.toString() ?? 'Failed to submit test proposal.';
      isError = true;
    } finally {
      submitting = false;
    }
  }
</script>

<h2>Propose New Test</h2>
<p class="hint">Proposals are reviewed and approved by The Observer.</p>

<div class="form-card">
  <div class="form-group">
    <label for="name">Test Name *</label>
    <input id="name" bind:value={name} placeholder="e.g. Soil pH Analysis — Sector 7" />
  </div>
  <div class="form-group">
    <label for="goal">Goal *</label>
    <textarea id="goal" bind:value={goal} rows="3" placeholder="What the test aims to measure or discover…"></textarea>
  </div>
  <div class="row">
    <div class="form-group" style="flex:1">
      <label for="cat">Category</label>
      <select id="cat" bind:value={category}>
        <option value="agricultural">Agricultural</option>
        <option value="biological">Biological</option>
      </select>
    </div>
    <div class="form-group" style="flex:1">
      <label for="species">Species Scope</label>
      <input id="species" bind:value={speciesScope} placeholder="Leave blank if N/A" />
    </div>
  </div>
  <div class="form-group">
    <label for="proc">Procedure</label>
    <textarea id="proc" bind:value={procedure} rows="4" placeholder="Step-by-step protocol…"></textarea>
  </div>
  <div class="form-group">
    <label for="app">Apparatuses</label>
    <input id="app" bind:value={apparatuses} placeholder="e.g. Spectrophotometer, Soil sampler" />
  </div>
  <div class="form-group">
    <label for="data">Required Data</label>
    <input id="data" bind:value={requiredData} placeholder="Data sets needed" />
  </div>
  <div class="form-group">
    <label for="just">Justification</label>
    <textarea id="just" bind:value={justification} rows="2" placeholder="Why this test is needed…"></textarea>
  </div>
  <div class="btn-row">
    <button class="btn-primary" onclick={handleSubmit} disabled={submitting || !name.trim() || !goal.trim()}>
      {submitting ? 'Submitting…' : 'Submit Proposal'}
    </button>
    <button class="btn-secondary" onclick={() => goto('/engineers/tests')}>Cancel</button>
  </div>
  {#if message}<p class:msg-ok={!isError} class:msg-err={isError}>{message}</p>{/if}
</div>

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.5rem; }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:1rem; }
  .form-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:650px; }
  .row { display:flex;gap:1rem; }
  .form-group { margin-bottom:0.75rem; }
  .form-group label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.25rem; }
  .form-group input, .form-group textarea, .form-group select { width:100%;background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem;font-size:0.8rem; }
  .form-group textarea { resize:vertical;font-family:'Fira Code',monospace; }
  .btn-row { display:flex;gap:0.75rem; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-secondary { background:transparent;border:1px solid #475569;color:#94A3B8;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-secondary:hover { color:#E6EDF3;border-color:#3ABEFF; }
  .msg-ok { color:#22C55E;font-size:0.8rem;margin-top:0.5rem; }
  .msg-err { color:#EF4444;font-size:0.8rem;margin-top:0.5rem; }
</style>
