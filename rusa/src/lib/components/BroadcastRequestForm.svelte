<!--
  BroadcastRequestForm — reusable component for submitting broadcast requests.
  Used across subsystem layouts (Engineers, Scientists, etc.)
  Source of truth: TODO_DIRECTORS.md — "shared BroadcastRequestForm"
-->
<script lang="ts">
  import { submitBroadcastRequest } from '$lib/stores/directors';

  let type_ = $state('informational');
  let subject = $state('');
  let content = $state('');
  let targetScope = $state('all');
  let urgency = $state('medium');
  let rationale = $state('');
  let submitting = $state(false);
  let message = $state('');
  let isError = $state(false);

  async function handleSubmit() {
    if (!subject.trim() || !content.trim()) return;
    submitting = true;
    message = '';
    try {
      await submitBroadcastRequest({
        type_,
        subject,
        content,
        target_scope: targetScope || undefined,
        urgency: urgency || undefined,
        rationale: rationale || undefined,
      });
      message = 'Broadcast request submitted for Director review.';
      isError = false;
      subject = ''; content = ''; rationale = '';
    } catch (e: any) {
      message = e?.toString() ?? 'Failed to submit broadcast request.';
      isError = true;
    } finally {
      submitting = false;
    }
  }
</script>

<h2>Broadcast Request</h2>
<p class="hint">Submit a broadcast request. It will be reviewed and approved by a Director before being sent.</p>

<div class="form-card">
  <div class="row">
    <div class="form-group" style="flex:1">
      <label for="btype">Broadcast Type</label>
      <select id="btype" bind:value={type_}>
        <option value="informational">Informational</option>
        <option value="emergency">Emergency</option>
        <option value="security">Security</option>
      </select>
    </div>
    <div class="form-group" style="flex:1">
      <label for="urgency">Urgency</label>
      <select id="urgency" bind:value={urgency}>
        <option value="low">Low</option>
        <option value="medium">Medium</option>
        <option value="high">High</option>
        <option value="critical">Critical</option>
      </select>
    </div>
  </div>
  <div class="form-group">
    <label for="subject">Subject *</label>
    <input id="subject" type="text" bind:value={subject} placeholder="Broadcast subject line" />
  </div>
  <div class="form-group">
    <label for="content">Content *</label>
    <textarea id="content" bind:value={content} rows="5" placeholder="Broadcast message body…"></textarea>
  </div>
  <div class="form-group">
    <label for="scope">Target Scope</label>
    <select id="scope" bind:value={targetScope}>
      <option value="all">All Personnel</option>
      <option value="engineers">Engineers Only</option>
      <option value="scientists">Scientists</option>
      <option value="medical">Medical</option>
      <option value="security">Security</option>
    </select>
  </div>
  <div class="form-group">
    <label for="rationale">Rationale</label>
    <textarea id="rationale" bind:value={rationale} rows="2" placeholder="Why is this broadcast necessary?"></textarea>
  </div>
  <button class="btn-primary" onclick={handleSubmit} disabled={submitting || !subject.trim() || !content.trim()}>
    {submitting ? 'Submitting…' : 'Submit Broadcast Request'}
  </button>
  {#if message}<p class:msg-ok={!isError} class:msg-err={isError}>{message}</p>{/if}
</div>

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.5rem; }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:1rem; }
  .form-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:600px; }
  .row { display:flex;gap:1rem; }
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
