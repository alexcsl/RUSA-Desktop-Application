<!--
  /scientists/experiments/[id]/conclusion — UC-PH-07 / UC-CH-06 / UC-BIO-07: Conclusion Request
-->
<script lang="ts">
  import { page } from '$app/stores';
  import { submitExperimentConclusion } from '$lib/stores/scientists';
  import { goto } from '$app/navigation';

  let experimentId = $state('');
  page.subscribe((p) => (experimentId = p.params.id ?? ''));

  let reason = $state('');
  let submitting = $state(false);
  let message = $state('');
  let isError = $state(false);

  async function handleSubmit() {
    if (!reason.trim()) return;
    submitting = true;
    message = '';
    try {
      await submitExperimentConclusion({ experiment_id: experimentId, reason });
      message = 'Conclusion request submitted. Awaiting Taskmaster review.';
      isError = false;
      reason = '';
    } catch (e: any) {
      message = e?.toString() ?? 'Failed to submit conclusion request.';
      isError = true;
    } finally {
      submitting = false;
    }
  }
</script>

<h2>Request Experiment Conclusion</h2>
<p class="hint">
  Submit a closure request for experiment <code>{experimentId.slice(0, 8)}…</code>
  <br />This will route to <strong>TheTaskmaster</strong> (Director) for review.
</p>

<div class="form-card">
  <div class="form-group">
    <label for="reason">Reason for conclusion *</label>
    <textarea id="reason" bind:value={reason} rows="6"
      placeholder="Describe why this experiment should be concluded and summarise findings…"
    ></textarea>
  </div>
  <div class="btn-row">
    <button class="btn-primary" onclick={handleSubmit} disabled={submitting || !reason.trim()}>
      {submitting ? 'Submitting…' : 'Submit Conclusion Request'}
    </button>
    <button class="btn-secondary" onclick={() => goto(`/scientists/experiments/${experimentId}`)}>Cancel</button>
  </div>
  {#if message}<p class:msg-ok={!isError} class:msg-err={isError}>{message}</p>{/if}
</div>

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.5rem; }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:1rem; }
  .hint code { color:#A5B4FC; }
  .hint strong { color:#E6EDF3; }
  .form-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:600px; }
  .form-group { margin-bottom:0.75rem; }
  .form-group label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.25rem; }
  .form-group textarea { width:100%;background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem;font-size:0.8rem;resize:vertical;font-family:'Fira Code',monospace; }
  .btn-row { display:flex;gap:0.75rem; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-secondary { background:transparent;border:1px solid #475569;color:#94A3B8;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-secondary:hover { color:#E6EDF3;border-color:#3ABEFF; }
  .msg-ok { color:#22C55E;font-size:0.8rem;margin-top:0.5rem; }
  .msg-err { color:#EF4444;font-size:0.8rem;margin-top:0.5rem; }
</style>
