<!--
  /directors/guardian/broadcast/new — Send Emergency Broadcast (UC-GUA-03)
  TheGuardian sends an emergency broadcast bypassing vote.
-->
<script lang="ts">
  import { sendEmergencyBroadcast } from '$lib/stores/directors';

  let subject = $state('');
  let content = $state('');
  let urgency = $state('high');
  let rationale = $state('');
  let loading = $state(false);
  let error = $state('');
  let success = $state('');

  async function handleSubmit() {
    error = ''; success = '';
    if (!subject.trim() || !content.trim() || !rationale.trim()) {
      error = 'Subject, content, and rationale are all required.';
      return;
    }
    loading = true;
    try {
      await sendEmergencyBroadcast({
        subject: subject.trim(),
        content: content.trim(),
        urgency,
        rationale: rationale.trim(),
      });
      success = 'Emergency broadcast sent.';
      subject = ''; content = ''; urgency = 'high'; rationale = '';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }
</script>

<h1 class="title">Emergency Broadcast</h1>
<p class="subtitle">Send an emergency broadcast to the entire organization. This bypasses the vote queue.</p>

<div class="alert-banner">⚡ This action is immediate and audited. Use only when warranted.</div>

<div class="form-card">
  <label class="field">
    <span class="label">Subject *</span>
    <input class="input" type="text" bind:value={subject} placeholder="Emergency subject…" />
  </label>
  <label class="field">
    <span class="label">Content *</span>
    <textarea class="textarea" bind:value={content} rows="5" placeholder="Broadcast message…"></textarea>
  </label>
  <label class="field">
    <span class="label">Urgency *</span>
    <select class="input" bind:value={urgency}>
      <option value="critical">Critical</option>
      <option value="high">High</option>
      <option value="medium">Medium</option>
    </select>
  </label>
  <label class="field">
    <span class="label">Rationale *</span>
    <textarea class="textarea" bind:value={rationale} rows="3" placeholder="Why is this broadcast necessary?"></textarea>
  </label>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}
  <button class="btn-warn" onclick={handleSubmit} disabled={loading}>
    {loading ? 'Sending…' : 'Send Emergency Broadcast'}
  </button>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#F59E0B;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .alert-banner { background:rgba(245,158,11,0.08);border:1px solid rgba(245,158,11,0.25);border-radius:6px;padding:0.5rem 0.75rem;color:#FCD34D;font-size:0.78rem;margin-bottom:1rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(245,158,11,0.12);border-radius:8px;padding:1.5rem;max-width:520px;display:flex;flex-direction:column;gap:0.75rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem 0.6rem;font-size:0.82rem;font-family:'Inter',sans-serif;width:100%;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#F59E0B; }
  .textarea { resize:vertical; }
  .btn-warn { background:rgba(245,158,11,0.15);border:1px solid #F59E0B;color:#F59E0B;border-radius:6px;padding:0.55rem;cursor:pointer;font-size:0.85rem;font-weight:600;margin-top:0.25rem; }
  .btn-warn:hover:not(:disabled) { background:rgba(245,158,11,0.25); }
  .btn-warn:disabled { opacity:0.5;cursor:default; }
  .error { color:#EF4444;font-size:0.75rem; }
  .success { color:#22C55E;font-size:0.75rem; }
</style>
