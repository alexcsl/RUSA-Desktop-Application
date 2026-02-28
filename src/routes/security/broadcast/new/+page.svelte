<!-- /security/broadcast/new — Submit Broadcast Request (UC-SH-01) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { secSubmitBroadcastRequest, getMyBroadcastRequests, type SecBroadcastRequestSummary } from '$lib/stores/security';

  let subject = $state('');
  let content = $state('');
  let urgency = $state('high');
  let rationale = $state('');

  let error = $state('');
  let success = $state('');
  let submitting = $state(false);
  let requests: SecBroadcastRequestSummary[] = $state([]);

  onMount(async () => {
    try { requests = await getMyBroadcastRequests(); } catch {}
  });

  async function handleSubmit() {
    error = ''; success = '';
    if (!subject.trim() || !content.trim()) { error = 'Subject and content are required.'; return; }

    submitting = true;
    try {
      await secSubmitBroadcastRequest({
        subject,
        content,
        urgency,
        rationale: rationale || undefined,
      });
      success = 'Broadcast request submitted for Guardian review.';
      subject = ''; content = ''; rationale = '';
      requests = await getMyBroadcastRequests();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  function statusColor(status: string): string {
    switch (status) {
      case 'pending': return '#F59E0B';
      case 'approved': return '#10B981';
      case 'rejected': return '#EF4444';
      case 'sent': return '#3ABEFF';
      default: return '#94A3B8';
    }
  }
</script>

<h1 class="title">Submit Broadcast Request</h1>
<p class="subtitle">Security broadcast requests are routed to The Guardian for approval. Non-emergency only — use the Guardian's emergency broadcast for immediate threats.</p>

<div class="form-card">
  <h2>Broadcast Request</h2>

  <label class="field"><span class="label">Subject *</span>
    <input type="text" class="input" bind:value={subject} placeholder="Broadcast subject…" />
  </label>

  <label class="field"><span class="label">Content *</span>
    <textarea class="textarea" bind:value={content} rows="5" placeholder="Detailed broadcast content…"></textarea>
  </label>

  <label class="field"><span class="label">Urgency</span>
    <select class="input" bind:value={urgency}>
      <option value="critical">Critical</option>
      <option value="high">High</option>
      <option value="normal">Normal</option>
      <option value="low">Low</option>
    </select>
  </label>

  <label class="field"><span class="label">Rationale</span>
    <textarea class="textarea" bind:value={rationale} rows="2" placeholder="Why is this broadcast needed?"></textarea>
  </label>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}

  <button class="btn-primary" onclick={handleSubmit} disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Request'}
  </button>
</div>

{#if requests.length > 0}
  <h2 class="section-title">My Broadcast Requests</h2>
  <div class="request-list">
    {#each requests as req}
      <div class="req-card">
        <div class="req-header">
          <span class="req-subject">{req.subject}</span>
          <span class="badge" style="background:rgba({statusColor(req.status) === '#F59E0B' ? '245,158,11' : statusColor(req.status) === '#10B981' ? '16,185,129' : statusColor(req.status) === '#EF4444' ? '239,68,68' : '58,190,255'},0.15);color:{statusColor(req.status)}">{req.status}</span>
        </div>
        <p class="req-content">{req.content}</p>
        <span class="req-date">{new Date(req.created_at).toLocaleString()}</span>
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:600px;display:flex;flex-direction:column;gap:0.6rem; }
  .form-card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .section-title { font-family:'Orbitron',sans-serif;font-size:0.85rem;color:#8B5CF6;margin:1.25rem 0 0.5rem; }
  .request-list { display:flex;flex-direction:column;gap:0.4rem;max-width:600px; }
  .req-card { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.65rem; }
  .req-header { display:flex;justify-content:space-between;align-items:center; }
  .req-subject { font-size:0.85rem;font-weight:500;color:#E6EDF3; }
  .badge { padding:0.1rem 0.35rem;border-radius:4px;font-size:0.65rem;font-weight:600;text-transform:uppercase; }
  .req-content { font-size:0.75rem;color:#94A3B8;margin:0.3rem 0 0.2rem; }
  .req-date { font-size:0.65rem;color:#6B7280; }
</style>
