<!--
  /scientists/tasks/[id] — Task detail view with math results submission for Mathematician
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { getMyTasks, getMathResults, submitMathResults, type ScientistTask, type MathResult } from '$lib/stores/scientists';
  import { currentUser } from '$lib/stores/auth';
  import type { SessionUser } from '$lib/stores/auth';

  let taskId = $state('');
  page.subscribe((p) => (taskId = p.params.id ?? ''));

  let user: SessionUser | null = $state(null);
  currentUser.subscribe((v) => (user = v));

  let task: ScientistTask | null = $state(null);
  let mathResults: MathResult[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  // Math results form
  let contentLatex = $state('');
  let workings = $state('');
  let calculationsArea = $state('');
  let submitting = $state(false);
  let submitMsg = $state('');

  onMount(async () => {
    try {
      const tasks = await getMyTasks();
      task = tasks.find((t) => t.id === taskId) ?? null;
      if (task) {
        try { mathResults = await getMathResults(taskId); } catch {}
      }
    } catch (e: any) {
      error = e?.toString() ?? 'Failed to load task.';
    } finally {
      loading = false;
    }
  });

  async function handleSubmitResults() {
    submitting = true;
    submitMsg = '';
    try {
      await submitMathResults({
        task_id: taskId,
        content_latex: contentLatex,
        workings: workings || undefined,
        calculations_area: calculationsArea || undefined,
      });
      submitMsg = 'Results submitted successfully.';
      contentLatex = '';
      workings = '';
      calculationsArea = '';
      mathResults = await getMathResults(taskId);
    } catch (e: any) {
      submitMsg = e?.toString() ?? 'Submission failed.';
    } finally {
      submitting = false;
    }
  }
</script>

<h2>Task Detail</h2>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if !task}
  <p class="muted">Task not found.</p>
{:else}
  <div class="card">
    <h3>{task.title}</h3>
    <div class="meta">
      <span>Type: <strong>{task.task_type}</strong></span>
      <span>Status: <strong>{task.status}</strong></span>
      <span>Assigned by: <strong>{task.assigner_name}</strong></span>
      {#if task.due_date}<span>Due: <strong>{task.due_date}</strong></span>{/if}
    </div>
    {#if task.description}
      <p class="desc">{task.description}</p>
    {/if}
  </div>

  <!-- Mathematician: submit results -->
  {#if user?.role === 'Mathematician' && task.status !== 'completed'}
    <div class="card" style="margin-top:1rem;">
      <h3>Submit Results (LaTeX)</h3>
      <div class="form-group">
        <label for="latex">LaTeX Content *</label>
        <textarea id="latex" bind:value={contentLatex} rows="6" placeholder="Enter LaTeX expressions e.g. \\frac a b = c"></textarea>
      </div>
      <div class="form-group">
        <label for="workings">Workings</label>
        <textarea id="workings" bind:value={workings} rows="4" placeholder="Step-by-step workings…"></textarea>
      </div>
      <div class="form-group">
        <label for="calc">Calculations Area</label>
        <textarea id="calc" bind:value={calculationsArea} rows="3"></textarea>
      </div>
      <button class="btn-primary" onclick={handleSubmitResults} disabled={submitting || !contentLatex.trim()}>
        {submitting ? 'Submitting…' : 'Submit Results'}
      </button>
      {#if submitMsg}<p class="msg">{submitMsg}</p>{/if}
    </div>
  {/if}

  <!-- Math results history -->
  {#if mathResults.length > 0}
    <div class="card" style="margin-top:1rem;">
      <h3>Math Results</h3>
      {#each mathResults as r}
        <div class="result-entry">
          <pre class="latex-block">{(r.content as any).content_latex ?? ''}</pre>
          {#if (r.content as any).workings}
            <p class="small"><strong>Workings:</strong> {(r.content as any).workings}</p>
          {/if}
          <p class="small muted">Submitted: {new Date(r.created_at).toLocaleString()}</p>
        </div>
      {/each}
    </div>
  {/if}
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:1rem; }
  h3 { font-size:0.95rem;color:#E6EDF3;margin:0 0 0.5rem; }
  .muted { color:#64748B;font-size:0.85rem; }
  .error { color:#EF4444;font-size:0.85rem; }
  .card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem; }
  .meta { display:flex;gap:1rem;flex-wrap:wrap;font-size:0.8rem;color:#94A3B8;margin-bottom:0.5rem; }
  .desc { font-size:0.85rem;color:#CBD5E1;line-height:1.5; }
  .form-group { margin-bottom:0.75rem; }
  .form-group label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.25rem; }
  .form-group textarea { width:100%;background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem;font-family:'Fira Code',monospace;font-size:0.8rem;resize:vertical; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .msg { font-size:0.8rem;color:#22C55E;margin-top:0.5rem; }
  .result-entry { border-bottom:1px solid rgba(255,255,255,0.05);padding:0.5rem 0; }
  .latex-block { background:#0B0F1A;padding:0.5rem;border-radius:4px;font-size:0.8rem;overflow-x:auto;color:#A5B4FC; }
  .small { font-size:0.75rem;color:#94A3B8; }
</style>
