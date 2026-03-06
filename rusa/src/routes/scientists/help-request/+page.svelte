<!--
  /scientists/help-request — UC-GS-02: Cross-Department Help Request
  Available to: Physicist, Chemist, Biologist (not Mathematician per spec)
-->
<script lang="ts">
  import { submitHelpRequest } from '$lib/stores/scientists';

  let title = $state('');
  let description = $state('');
  let urgency = $state('medium');
  let calculationsArea = $state('');
  let submitting = $state(false);
  let message = $state('');
  let isError = $state(false);

  async function handleSubmit() {
    if (!title.trim() || !description.trim()) return;
    submitting = true;
    message = '';
    try {
      await submitHelpRequest({
        title,
        description,
        urgency,
        calculations_area: calculationsArea || undefined,
      });
      message = 'Help request submitted successfully. Your proxy Director has been notified.';
      isError = false;
      title = ''; description = ''; calculationsArea = '';
    } catch (e: any) {
      message = e?.toString() ?? 'Failed to submit.';
      isError = true;
    } finally {
      submitting = false;
    }
  }
</script>

<h2>Cross-Department Help Request</h2>
<p class="hint">Submit a help request to your proxy Director (Artificer for Physics, Observer for Chemistry/Biology).</p>

<div class="form-card">
  <div class="form-group">
    <label for="title">Title *</label>
    <input id="title" type="text" bind:value={title} placeholder="Brief request title" />
  </div>
  <div class="form-group">
    <label for="desc">Description *</label>
    <textarea id="desc" bind:value={description} rows="4" placeholder="Describe the help needed…"></textarea>
  </div>
  <div class="form-group">
    <label for="urgency">Urgency</label>
    <select id="urgency" bind:value={urgency}>
      <option value="low">Low</option>
      <option value="medium">Medium</option>
      <option value="high">High</option>
      <option value="critical">Critical</option>
    </select>
  </div>
  <div class="form-group">
    <label for="calc">Calculations / Data Area (LaTeX supported)</label>
    <textarea id="calc" bind:value={calculationsArea} rows="3" placeholder="\\LaTeX content…"></textarea>
  </div>
  <button class="btn-primary" onclick={handleSubmit} disabled={submitting || !title.trim() || !description.trim()}>
    {submitting ? 'Submitting…' : 'Submit Help Request'}
  </button>
  {#if message}<p class:msg-ok={!isError} class:msg-err={isError}>{message}</p>{/if}
</div>

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.5rem; }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:1rem; }
  .form-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:600px; }
  .form-group { margin-bottom:0.75rem; }
  .form-group label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.25rem; }
  .form-group input, .form-group textarea, .form-group select { width:100%;background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem;font-size:0.8rem; }
  .form-group textarea { resize:vertical;font-family:'Fira Code',monospace; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .msg-ok { color:#22C55E;font-size:0.8rem;margin-top:0.5rem; }
  .msg-err { color:#EF4444;font-size:0.8rem;margin-top:0.5rem; }
</style>
