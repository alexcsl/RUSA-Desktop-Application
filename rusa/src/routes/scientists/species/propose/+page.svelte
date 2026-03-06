<!--
  /scientists/species/propose — UC-BIO-02: Propose New Species (Biologist-only)
  Rust backend: ProposeNewSpeciesPayload { title, introduction, problem_statement,
                research_questions, hypotheses, location }
  Creates a biology_observation experiment routed to TheObserver.
-->
<script lang="ts">
  import { proposeNewSpecies } from '$lib/stores/scientists';
  import { goto } from '$app/navigation';

  let title = $state('');
  let introduction = $state('');
  let problemStatement = $state('');
  let researchQuestions = $state('');
  let hypotheses = $state('');
  let location = $state('');
  let submitting = $state(false);
  let message = $state('');
  let isError = $state(false);

  async function handleSubmit() {
    if (!title.trim()) return;
    submitting = true;
    message = '';
    try {
      await proposeNewSpecies({
        title,
        introduction: introduction || undefined,
        problem_statement: problemStatement || undefined,
        research_questions: researchQuestions || undefined,
        hypotheses: hypotheses || undefined,
        location: location || undefined,
      });
      message = 'Species proposal submitted for TheObserver review.';
      isError = false;
      title = ''; introduction = ''; problemStatement = '';
      researchQuestions = ''; hypotheses = ''; location = '';
    } catch (e: any) {
      message = e?.toString() ?? 'Failed to submit species proposal.';
      isError = true;
    } finally {
      submitting = false;
    }
  }
</script>

<h2>Propose New Species</h2>
<p class="hint">Only Biologists can propose new species. The proposal will be reviewed by TheObserver.</p>

<div class="form-card">
  <div class="form-group">
    <label for="title">Proposal Title *</label>
    <input id="title" bind:value={title} placeholder="e.g. Discovery: Blue-wing Moth (Lepidoptera azurea)" />
  </div>
  <div class="form-group">
    <label for="intro">Introduction</label>
    <textarea id="intro" bind:value={introduction} rows="3" placeholder="Context and background on this species…"></textarea>
  </div>
  <div class="form-group">
    <label for="problem">Problem Statement</label>
    <textarea id="problem" bind:value={problemStatement} rows="2" placeholder="What knowledge gap does this address?"></textarea>
  </div>
  <div class="form-group">
    <label for="rq">Research Questions</label>
    <textarea id="rq" bind:value={researchQuestions} rows="2" placeholder="Key questions to investigate…"></textarea>
  </div>
  <div class="form-group">
    <label for="hyp">Hypotheses</label>
    <textarea id="hyp" bind:value={hypotheses} rows="2" placeholder="Testable predictions…"></textarea>
  </div>
  <div class="form-group">
    <label for="loc">Location</label>
    <input id="loc" bind:value={location} placeholder="Habitat / sector where observed" />
  </div>
  <div class="btn-row">
    <button class="btn-primary" onclick={handleSubmit} disabled={submitting || !title.trim()}>
      {submitting ? 'Submitting…' : 'Submit Species Proposal'}
    </button>
    <button class="btn-secondary" onclick={() => goto('/scientists/experiments')}>Cancel</button>
  </div>
  {#if message}<p class:msg-ok={!isError} class:msg-err={isError}>{message}</p>{/if}
</div>

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.5rem; }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:1rem; }
  .form-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:650px; }
  .form-group { margin-bottom:0.75rem; }
  .form-group label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.25rem; }
  .form-group input, .form-group textarea { width:100%;background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem;font-size:0.8rem; }
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
