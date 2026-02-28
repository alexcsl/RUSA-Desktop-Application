<!--
  /directors/artificer/math-results — TheArtificer: View Submitted Math Results
  Shows all math results submitted by Mathematicians for tasks assigned by this director.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getMathResultsForDirector, type MathResultItem } from '$lib/stores/directors';
  import LatexPreview from '$lib/components/LatexPreview.svelte';

  let results: MathResultItem[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let expanded: string | null = $state(null);

  onMount(async () => {
    try {
      results = await getMathResultsForDirector();
    } catch (e: any) {
      error = e?.toString() ?? 'Failed to load math results.';
    } finally {
      loading = false;
    }
  });

  function fmtDate(iso: string): string {
    return new Date(iso).toLocaleString();
  }

  function toggle(id: string) {
    expanded = expanded === id ? null : id;
  }
</script>

<h2>Math Results</h2>
<p class="hint">Review calculation results submitted by Mathematicians for your assigned tasks.</p>

{#if error}<p class="msg-err">{error}</p>{/if}

{#if loading}
  <p class="muted">Loading results…</p>
{:else if results.length === 0}
  <p class="muted">No math results have been submitted yet.</p>
{:else}
  <div class="results-list">
    {#each results as r}
      <div class="result-card">
        <div class="result-header" role="button" tabindex="0"
             onclick={() => toggle(r.id)}
             onkeydown={(e) => { if (e.key === 'Enter') toggle(r.id); }}>
          <div class="result-title">
            <span class="math-icon">📐</span>
            <strong>Result {r.id.slice(0,8)}…</strong>
            <span class="by">by {r.submitted_by_name}</span>
          </div>
          <div class="result-meta">
            <span class="date">{fmtDate(r.created_at)}</span>
            <span class="expand-icon">{expanded === r.id ? '▼' : '▶'}</span>
          </div>
        </div>
        {#if expanded === r.id}
          <div class="result-body">
            {#if r.content?.content_latex}
              <div class="section">
                <h4>LaTeX Content</h4>
                <LatexPreview source={r.content.content_latex} displayMode={true} />
                <details class="raw-toggle">
                  <summary>View raw LaTeX</summary>
                  <pre class="raw-latex">{r.content.content_latex}</pre>
                </details>
              </div>
            {/if}
            {#if r.content?.workings}
              <div class="section">
                <h4>Workings</h4>
                <pre class="workings">{r.content.workings}</pre>
              </div>
            {/if}
            {#if r.content?.calculations_area}
              <div class="section">
                <h4>Calculations</h4>
                <pre class="workings">{r.content.calculations_area}</pre>
              </div>
            {/if}
            {#if r.pdf_storage_path}
              <p class="pdf-note">📄 PDF available: <code>{r.pdf_storage_path}</code></p>
            {/if}
            <p class="task-ref">Task ID: <code>{r.task_id}</code></p>
          </div>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.3rem; }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:1rem; }
  .muted { color:#64748B;font-size:0.8rem; }
  .msg-err { color:#EF4444;font-size:0.8rem;margin-bottom:0.5rem; }

  .results-list { display:flex;flex-direction:column;gap:0.5rem; }
  .result-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;overflow:hidden; }
  .result-header { display:flex;justify-content:space-between;align-items:center;padding:0.75rem 1rem;cursor:pointer; }
  .result-header:hover { background:rgba(58,190,255,0.04); }
  .result-title { display:flex;align-items:center;gap:0.5rem;font-size:0.85rem; }
  .math-icon { font-size:1rem; }
  .by { color:#94A3B8;font-size:0.75rem; }
  .result-meta { display:flex;align-items:center;gap:0.75rem; }
  .date { color:#64748B;font-size:0.7rem; }
  .expand-icon { color:#3ABEFF;font-size:0.7rem; }

  .result-body { padding:0 1rem 1rem;border-top:1px solid rgba(58,190,255,0.06); }
  .section { margin-top:0.75rem; }
  .section h4 { font-size:0.8rem;color:#8B5CF6;margin-bottom:0.3rem; }
  .raw-toggle { margin-top:0.4rem; }
  .raw-toggle summary { color:#64748B;font-size:0.7rem;cursor:pointer; }
  .raw-latex { background:#0B0F1A;border:1px solid #334155;border-radius:4px;padding:0.5rem;font-size:0.75rem;color:#94A3B8;white-space:pre-wrap;font-family:'Fira Code',monospace; }
  .workings { background:#0B0F1A;border:1px solid #334155;border-radius:6px;padding:0.5rem;font-size:0.78rem;color:#CBD5E1;white-space:pre-wrap;font-family:'Fira Code',monospace; }
  .pdf-note { color:#F59E0B;font-size:0.75rem;margin-top:0.5rem; }
  .pdf-note code { color:#94A3B8;font-size:0.7rem; }
  .task-ref { color:#64748B;font-size:0.7rem;margin-top:0.3rem; }
  .task-ref code { color:#94A3B8;font-size:0.7rem; }

</style>
