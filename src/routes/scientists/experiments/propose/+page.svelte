<!--
  /scientists/experiments/propose — UC-PH-01 / UC-CH-04 / UC-BIO-02: Experiment Proposal
-->
<script lang="ts">
  import { proposeExperiment } from '$lib/stores/scientists';
  import { currentUser } from '$lib/stores/auth';
  import type { SessionUser } from '$lib/stores/auth';

  let user: SessionUser | null = $state(null);
  currentUser.subscribe((v) => (user = v));

  let title = $state('');
  let experimentType = $state('');
  let introduction = $state('');
  let problemStatement = $state('');
  let researchQuestions = $state('');
  let hypotheses = $state('');
  let methodology = $state('');
  let expectedOutcomes = $state('');
  let location = $state('');
  let submitting = $state(false);
  let message = $state('');
  let isError = $state(false);

  // Auto-set experiment type based on role
  $effect(() => {
    if (user?.role === 'Physicist') experimentType = 'physical';
    else if (user?.role === 'Chemist') experimentType = 'chemical';
    else if (user?.role === 'Biologist') experimentType = 'biology_observation';
  });

  function typeLabel(t: string): string {
    const m: Record<string, string> = {
      physical: 'Physical Experiment',
      chemical: 'Chemical Experiment',
      biology_observation: 'Biology Observation',
    };
    return m[t] ?? t;
  }

  async function handleSubmit() {
    if (!title.trim() || !experimentType) return;
    submitting = true;
    message = '';
    try {
      const id = await proposeExperiment({
        title,
        experiment_type: experimentType,
        introduction: introduction || undefined,
        problem_statement: problemStatement || undefined,
        research_questions: researchQuestions || undefined,
        hypotheses: hypotheses || undefined,
        methodology: methodology || undefined,
        expected_outcomes: expectedOutcomes || undefined,
        location: location || undefined,
      });
      message = `Experiment proposal submitted (ID: ${id}). Your supervisor has been notified.`;
      isError = false;
      title = ''; introduction = ''; problemStatement = ''; researchQuestions = '';
      hypotheses = ''; methodology = ''; expectedOutcomes = ''; location = '';
    } catch (e: any) {
      message = e?.toString() ?? 'Submission failed.';
      isError = true;
    } finally {
      submitting = false;
    }
  }
</script>

<h2>Propose Experiment</h2>
<p class="hint">Propose a new experiment. It will enter the standard approval workflow via your supervisor Director.</p>

<div class="form-card">
  <div class="form-group">
    <label for="title">Title *</label>
    <input id="title" type="text" bind:value={title} placeholder="Experiment title" />
  </div>
  <div class="form-group">
    <label for="type">Type</label>
    <input id="type" type="text" value={typeLabel(experimentType)} readonly />
  </div>
  <div class="form-group">
    <label for="intro">Introduction</label>
    <textarea id="intro" bind:value={introduction} rows="3" placeholder="Project introduction…"></textarea>
  </div>
  <div class="form-group">
    <label for="problem">Problem Statement</label>
    <textarea id="problem" bind:value={problemStatement} rows="3"></textarea>
  </div>
  <div class="form-group">
    <label for="rq">Research Questions</label>
    <textarea id="rq" bind:value={researchQuestions} rows="3"></textarea>
  </div>
  <div class="form-group">
    <label for="hypo">Hypotheses</label>
    <textarea id="hypo" bind:value={hypotheses} rows="3"></textarea>
  </div>
  <div class="form-group">
    <label for="method">Methodology</label>
    <textarea id="method" bind:value={methodology} rows="3"></textarea>
  </div>
  <div class="form-group">
    <label for="outcomes">Expected Outcomes</label>
    <textarea id="outcomes" bind:value={expectedOutcomes} rows="2"></textarea>
  </div>
  <div class="form-group">
    <label for="loc">Location</label>
    <input id="loc" type="text" bind:value={location} placeholder="Experiment location" />
  </div>
  <button class="btn-primary" onclick={handleSubmit} disabled={submitting || !title.trim()}>
    {submitting ? 'Submitting…' : 'Submit Proposal'}
  </button>
  {#if message}<p class:msg-ok={!isError} class:msg-err={isError}>{message}</p>{/if}
</div>

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.5rem; }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:1rem; }
  .form-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:650px; }
  .form-group { margin-bottom:0.75rem; }
  .form-group label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.25rem; }
  .form-group input, .form-group textarea { width:100%;background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem;font-size:0.8rem; }
  .form-group input[readonly] { opacity:0.7; }
  .form-group textarea { resize:vertical; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .msg-ok { color:#22C55E;font-size:0.8rem;margin-top:0.5rem; }
  .msg-err { color:#EF4444;font-size:0.8rem;margin-top:0.5rem; }
</style>
