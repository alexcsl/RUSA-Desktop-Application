<!--
  /scientists/math/results/new — UC-MA-01: Submit Calculation / Results Response
  Standalone math results page (for convenience — can also submit from task detail)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getMyTasks, submitMathResults, type ScientistTask } from '$lib/stores/scientists';
  import LatexPreview from '$lib/components/LatexPreview.svelte';

  let tasks: ScientistTask[] = $state([]);
  let selectedTask = $state('');
  let contentLatex = $state('');
  let workings = $state('');
  let calculationsArea = $state('');
  let submitting = $state(false);
  let message = $state('');
  let isError = $state(false);
  let loading = $state(true);

  onMount(async () => {
    try {
      const all = await getMyTasks();
      tasks = all.filter((t) => t.status !== 'completed' && t.status !== 'cancelled');
    } catch {}
    loading = false;
  });

  async function handleSubmit() {
    if (!selectedTask || !contentLatex.trim()) return;
    submitting = true;
    message = '';
    try {
      await submitMathResults({
        task_id: selectedTask,
        content_latex: contentLatex,
        workings: workings || undefined,
        calculations_area: calculationsArea || undefined,
      });
      message = 'Math results submitted and task marked as completed.';
      isError = false;
      contentLatex = ''; workings = ''; calculationsArea = '';
      // Refresh tasks
      const all = await getMyTasks();
      tasks = all.filter((t) => t.status !== 'completed' && t.status !== 'cancelled');
      selectedTask = '';
    } catch (e: any) {
      message = e?.toString() ?? 'Submission failed.';
      isError = true;
    } finally {
      submitting = false;
    }
  }
</script>

<h2>Submit Math Results</h2>
<p class="hint">Submit calculations and LaTeX results for an assigned task. Results are routed back through The Artificer.</p>

{#if loading}
  <p class="muted">Loading tasks…</p>
{:else}
  <div class="form-card">
    <div class="form-group">
      <label for="task">Select Task *</label>
      <select id="task" bind:value={selectedTask}>
        <option value="">— select a task —</option>
        {#each tasks as t}
          <option value={t.id}>{t.title} ({t.status})</option>
        {/each}
      </select>
      {#if tasks.length === 0}
        <p class="muted" style="margin-top:0.25rem;">No pending tasks.</p>
      {/if}
    </div>
    <div class="form-group">
      <label for="latex">LaTeX Content *</label>
      <textarea id="latex" bind:value={contentLatex} rows="8" placeholder="Enter LaTeX expressions e.g. \\frac a b = c"></textarea>
    </div>
    <div class="form-group">
      <span class="preview-label">Live Preview</span>
      <LatexPreview source={contentLatex} displayMode={true} />
    </div>
    <div class="form-group">
      <label for="work">Workings (step-by-step)</label>
      <textarea id="work" bind:value={workings} rows="5" placeholder="Step 1: …"></textarea>
    </div>
    <div class="form-group">
      <label for="calc">Calculations Area</label>
      <textarea id="calc" bind:value={calculationsArea} rows="3"></textarea>
    </div>
    <button class="btn-primary" onclick={handleSubmit} disabled={submitting || !selectedTask || !contentLatex.trim()}>
      {submitting ? 'Submitting…' : 'Submit Results'}
    </button>
    {#if message}<p class:msg-ok={!isError} class:msg-err={isError}>{message}</p>{/if}
  </div>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.5rem; }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:1rem; }
  .muted { color:#64748B;font-size:0.8rem; }
  .form-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:650px; }
  .form-group { margin-bottom:0.75rem; }
  .form-group label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.25rem; }
  .preview-label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.25rem; }
  .form-group select, .form-group textarea { width:100%;background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem;font-size:0.8rem; }
  .form-group textarea { resize:vertical;font-family:'Fira Code',monospace; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .msg-ok { color:#22C55E;font-size:0.8rem;margin-top:0.5rem; }
  .msg-err { color:#EF4444;font-size:0.8rem;margin-top:0.5rem; }
</style>
