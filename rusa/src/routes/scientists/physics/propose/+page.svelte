<!-- /scientists/physics/propose — Propose New Object or Phenomenon (Physicist, UC-4-06) -->
<script lang="ts">
  import { proposeNewPhysicalObject } from '$lib/stores/scientists';

  let title = $state('');
  let introduction = $state('');
  let problemStatement = $state('');
  let researchQuestions = $state('');
  let hypotheses = $state('');
  let location = $state('');
  let submitting = $state(false);
  let error = $state('');
  let success = $state('');

  async function handleSubmit() {
    error = ''; success = '';
    if (!title.trim()) { error = 'Title is required.'; return; }
    submitting = true;
    try {
      await proposeNewPhysicalObject({
        title,
        introduction: introduction || undefined,
        problem_statement: problemStatement || undefined,
        research_questions: researchQuestions || undefined,
        hypotheses: hypotheses || undefined,
        location: location || undefined,
      });
      success = 'Object/phenomenon proposal submitted — awaiting TheArtificer review.';
      title = ''; introduction = ''; problemStatement = '';
      researchQuestions = ''; hypotheses = ''; location = '';
    } catch (e: unknown) {
      error = String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<div class="page-header">
  <div class="title-row">
    <span class="icon">🔭</span>
    <div>
      <h1 class="title">Propose New Object or Phenomenon</h1>
      <p class="subtitle">Physicist-only · Reviewed by <strong>TheArtificer</strong></p>
    </div>
  </div>
</div>

<div class="form-card">
  <div class="section">
    <span class="section-label">Identification</span>
    <label class="field">
      <span class="label">Working Title <span class="req">*</span></span>
      <input class="input" bind:value={title} placeholder="e.g. Anomalous Gravitational Lensing — Sector 7" />
    </label>
    <label class="field">
      <span class="label">Location Observed</span>
      <input class="input" bind:value={location} placeholder="Where was this object or phenomenon observed?" />
    </label>
  </div>

  <div class="section">
    <span class="section-label">Research Proposal</span>
    <label class="field">
      <span class="label">Introduction</span>
      <textarea class="textarea" bind:value={introduction} rows="3" placeholder="Background and initial observations — what makes this worth investigating?"></textarea>
    </label>
    <label class="field">
      <span class="label">Problem Statement</span>
      <textarea class="textarea" bind:value={problemStatement} rows="2" placeholder="What physical properties or behaviours need investigation?"></textarea>
    </label>
    <label class="field">
      <span class="label">Research Questions</span>
      <textarea class="textarea" bind:value={researchQuestions} rows="2" placeholder="Key physical questions to explore — mass, energy, field interactions…"></textarea>
    </label>
    <label class="field">
      <span class="label">Hypotheses</span>
      <textarea class="textarea" bind:value={hypotheses} rows="2" placeholder="Expected physical properties, interactions, and classification…"></textarea>
    </label>
  </div>

  {#if error}<p class="msg msg-err">{error}</p>{/if}
  {#if success}<p class="msg msg-ok">{success}</p>{/if}

  <div class="btn-row">
    <button class="btn-primary" onclick={handleSubmit} disabled={submitting || !title.trim()}>
      {submitting ? 'Submitting…' : 'Submit Proposal'}
    </button>
  </div>
</div>

<style>
  .page-header { margin-bottom:1.25rem; }
  .title-row { display:flex;align-items:flex-start;gap:0.75rem; }
  .icon { font-size:1.8rem;line-height:1; }
  .title { font-family:'Orbitron',sans-serif;font-size:1.05rem;color:#3ABEFF;margin:0 0 0.15rem; }
  .subtitle { font-size:0.75rem;color:#64748B;margin:0; }
  .subtitle strong { color:#94A3B8; }

  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.12);border-radius:10px;padding:1.25rem;max-width:640px;display:flex;flex-direction:column;gap:1rem; }

  .section { display:flex;flex-direction:column;gap:0.5rem; }
  .section-label { font-size:0.65rem;font-weight:700;text-transform:uppercase;letter-spacing:0.08em;color:#3ABEFF;opacity:0.7; }

  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.72rem;color:#94A3B8; }
  .req { color:#EF4444; }
  .input,.textarea { width:100%;background:#0B1120;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.5rem 0.6rem;font-size:0.82rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF;box-shadow:0 0 0 2px rgba(58,190,255,0.1); }
  .textarea { resize:vertical; }

  .btn-row { display:flex;gap:0.6rem; }
  .btn-primary { padding:0.5rem 1.25rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;font-weight:600;cursor:pointer;font-size:0.82rem;transition:background 0.15s; }
  .btn-primary:hover:not(:disabled) { background:rgba(58,190,255,0.28); }
  .btn-primary:disabled { opacity:0.45;cursor:not-allowed; }

  .msg { font-size:0.78rem;margin:0;padding:0.5rem 0.75rem;border-radius:6px; }
  .msg-ok { background:rgba(16,185,129,0.08);color:#10B981;border:1px solid rgba(16,185,129,0.2); }
  .msg-err { background:rgba(239,68,68,0.08);color:#EF4444;border:1px solid rgba(239,68,68,0.2); }
</style>
