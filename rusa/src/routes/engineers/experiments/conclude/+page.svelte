<!--
  /engineers/experiments/conclude — UC-AGE-04 / UC-BE-05: Conclude an Experiment
  Lists active experiments belonging to the engineer's discipline and lets them
  submit a conclusion request. Routes to The Taskmaster for review.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getExperimentArchive,
    submitExperimentConclusion,
    type ExperimentSummary,
  } from '$lib/stores/engineers';

  let experiments: ExperimentSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  // Form state
  let selectedId = $state('');
  let summary = $state('');
  let finalOutcomes = $state('');
  let submitting = $state(false);
  let success = $state('');

  let selected = $derived(experiments.find((e) => e.id === selectedId) ?? null);

  onMount(async () => {
    try {
      // Fetch only active experiments — only these can be concluded
      experiments = await getExperimentArchive('active');
    } catch (e: unknown) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  async function handleSubmit() {
    if (!selectedId || !summary.trim()) return;
    submitting = true;
    error = '';
    success = '';
    try {
      await submitExperimentConclusion({
        experiment_id: selectedId,
        summary,
        final_outcomes: finalOutcomes || undefined,
      });
      success = `Conclusion request submitted for "${selected?.title}". Awaiting Taskmaster review.`;
      selectedId = '';
      summary = '';
      finalOutcomes = '';
      // Refresh list — the concluded experiment will no longer be 'active'
      experiments = await getExperimentArchive('active');
    } catch (e: unknown) {
      error = String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<div class="page">
  <h2>Conclude Experiment</h2>
  <p class="hint">Submit a conclusion request for an active experiment. The Taskmaster reviews and approves closure.</p>

  {#if success}<div class="banner success">{success}</div>{/if}
  {#if error}<div class="banner error">{error}</div>{/if}

  {#if loading}
    <p class="muted">Loading…</p>
  {:else if experiments.length === 0}
    <div class="empty">
      <p class="muted">No active experiments to conclude.</p>
      <p class="muted small">Only experiments with status <strong>active</strong> can be concluded.</p>
    </div>
  {:else}
    <form class="form-card" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
      <label class="field">
        <span class="label">Select Experiment <span class="req">*</span></span>
        <select class="input" bind:value={selectedId}>
          <option value="">— Pick an active experiment —</option>
          {#each experiments as exp}
            <option value={exp.id}>{exp.title} ({exp.experiment_type})</option>
          {/each}
        </select>
      </label>

      {#if selected}
        <div class="exp-meta">
          <span>Type: <strong>{selected.experiment_type}</strong></span>
          <span>Proposed by: <strong>{selected.proposer_name}</strong></span>
          <span>Status: <strong class="status-active">{selected.status}</strong></span>
        </div>
      {/if}

      <label class="field">
        <span class="label">Summary <span class="req">*</span></span>
        <textarea
          class="input"
          rows="5"
          bind:value={summary}
          placeholder="Summarise findings and why the experiment should be concluded…"
        ></textarea>
      </label>

      <label class="field">
        <span class="label">Final Outcomes</span>
        <textarea
          class="input"
          rows="3"
          bind:value={finalOutcomes}
          placeholder="Key results, data summary, recommendations… (optional)"
        ></textarea>
      </label>

      <button
        type="submit"
        class="btn-primary"
        disabled={submitting || !selectedId || !summary.trim()}
      >
        {submitting ? 'Submitting…' : 'Submit Conclusion Request'}
      </button>
    </form>
  {/if}
</div>

<style>
  .page { max-width:640px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .hint { color:#64748B;font-size:0.8rem;margin:0 0 1.25rem; }
  .muted { color:#94A3B8;font-size:0.85rem; }
  .small { font-size:0.75rem;margin-top:0.25rem; }
  .small strong { color:#3ABEFF; }
  .banner { padding:0.6rem 0.9rem;border-radius:6px;font-size:0.8rem;margin-bottom:1rem; }
  .banner.success { background:rgba(16,185,129,0.1);border:1px solid rgba(16,185,129,0.3);color:#34D399; }
  .banner.error { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);color:#F87171; }
  .empty { background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:2rem;display:flex;flex-direction:column;gap:0.4rem; }
  .form-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;display:flex;flex-direction:column;gap:0.75rem; }
  .field { display:flex;flex-direction:column;gap:0.25rem; }
  .label { font-size:0.72rem;color:#94A3B8; }
  .req { color:#EF4444; }
  .input { width:100%;background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem 0.6rem;font-size:0.8rem;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  textarea.input { resize:vertical;font-family:'Fira Code','Courier New',monospace; }
  .exp-meta { display:flex;flex-wrap:wrap;gap:0.75rem;background:rgba(58,190,255,0.04);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.6rem 0.8rem;font-size:0.78rem;color:#94A3B8; }
  .exp-meta strong { color:#E6EDF3; }
  .status-active { color:#3ABEFF; }
  .btn-primary { padding:0.5rem 1.25rem;background:rgba(58,190,255,0.15);border:1px solid rgba(58,190,255,0.4);color:#3ABEFF;border-radius:6px;font-weight:600;cursor:pointer;font-size:0.85rem;align-self:flex-start; }
  .btn-primary:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.4;cursor:default; }
</style>
