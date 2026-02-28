<!-- /directors/guardian/security-reports — TheGuardian Security (UC-DIR-15, UC-DIR-16) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    sendEmergencyBroadcast, submitBroadcastRequest, getBroadcastRequestQueue,
    type BroadcastRequestSummary,
  } from '$lib/stores/directors';

  let queue: BroadcastRequestSummary[] = $state([]);
  let activeTab: 'emergency' | 'requests' = $state('emergency');

  // Emergency broadcast form
  let emSubject = $state('');
  let emContent = $state('');
  let emScope = $state('company_wide');
  let emError = $state('');
  let emSuccess = $state('');

  // Security broadcast request
  let reqSubject = $state('');
  let reqContent = $state('');
  let reqUrgency = $state('high');
  let reqRationale = $state('');
  let reqError = $state('');
  let reqSuccess = $state('');

  onMount(async () => {
    try { queue = await getBroadcastRequestQueue(); } catch {}
  });

  async function handleEmergency() {
    emError = ''; emSuccess = '';
    if (!emSubject.trim() || !emContent.trim()) { emError = 'Subject and content required.'; return; }
    try {
      await sendEmergencyBroadcast({ subject: emSubject, content: emContent, target_scope: emScope });
      emSuccess = 'Emergency broadcast sent immediately.';
      emSubject = ''; emContent = '';
    } catch (e: unknown) { emError = e instanceof Error ? e.message : String(e); }
  }

  async function handleRequest() {
    reqError = ''; reqSuccess = '';
    if (!reqSubject.trim() || !reqContent.trim()) { reqError = 'Subject and content required.'; return; }
    try {
      await submitBroadcastRequest({
        type_: 'security', subject: reqSubject, content: reqContent,
        urgency: reqUrgency, rationale: reqRationale || undefined,
      });
      reqSuccess = 'Security broadcast request submitted for approval.';
      reqSubject = ''; reqContent = ''; reqRationale = '';
      queue = await getBroadcastRequestQueue();
    } catch (e: unknown) { reqError = e instanceof Error ? e.message : String(e); }
  }
</script>

<h1 class="title">Guardian — Security Control</h1>
<p class="subtitle">Emergency broadcasts bypass voting. Non-emergency security broadcasts require approval.</p>

<div class="tabs">
  <button class:active={activeTab === 'emergency'} onclick={() => (activeTab = 'emergency')}>Emergency Broadcast</button>
  <button class:active={activeTab === 'requests'} onclick={() => (activeTab = 'requests')}>Request Broadcast</button>
</div>

{#if activeTab === 'emergency'}
  <div class="form-card">
    <h2>Send Emergency Broadcast</h2>
    <p class="warn-text">This bypasses the normal voting process and sends immediately.</p>
    <label class="field"><span class="label">Subject</span>
      <input type="text" class="input" bind:value={emSubject} placeholder="Security alert subject…" />
    </label>
    <label class="field"><span class="label">Content</span>
      <textarea class="textarea" bind:value={emContent} rows="5" placeholder="Detailed broadcast content…"></textarea>
    </label>
    <label class="field"><span class="label">Target Scope</span>
      <select class="input" bind:value={emScope}>
        <option value="all">All Personnel</option>
        <option value="directors">Directors Only</option>
        <option value="security">Security Teams</option>
      </select>
    </label>
    {#if emError}<p class="error">{emError}</p>{/if}
    {#if emSuccess}<p class="success">{emSuccess}</p>{/if}
    <button class="btn-danger" onclick={handleEmergency}>Send Emergency Broadcast</button>
  </div>
{:else}
  <div class="form-card">
    <h2>Submit Security Broadcast Request</h2>
    <label class="field"><span class="label">Subject</span>
      <input type="text" class="input" bind:value={reqSubject} placeholder="Broadcast subject…" />
    </label>
    <label class="field"><span class="label">Content</span>
      <textarea class="textarea" bind:value={reqContent} rows="4" placeholder="Broadcast content…"></textarea>
    </label>
    <label class="field"><span class="label">Urgency</span>
      <select class="input" bind:value={reqUrgency}>
        <option value="high">High</option>
        <option value="medium">Medium</option>
        <option value="low">Low</option>
      </select>
    </label>
    <label class="field"><span class="label">Rationale</span>
      <textarea class="textarea" bind:value={reqRationale} rows="2" placeholder="Why is this broadcast needed?"></textarea>
    </label>
    {#if reqError}<p class="error">{reqError}</p>{/if}
    {#if reqSuccess}<p class="success">{reqSuccess}</p>{/if}
    <button class="btn-primary" onclick={handleRequest}>Submit Request</button>
  </div>

  {#if queue.length > 0}
    <h2 class="section-title">My Requests</h2>
    {#each queue.filter(q => q.type_ === 'security') as req}
      <div class="req-card">
        <div class="req-header">
          <span class="req-subject">{req.subject}</span>
          <span class="badge badge-{req.status}">{req.status}</span>
        </div>
        <p class="req-content">{req.content}</p>
      </div>
    {/each}
  {/if}
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .tabs { display:flex;gap:0.25rem;margin-bottom:1rem; }
  .tabs button { padding:0.45rem 0.9rem;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);color:#94A3B8;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .tabs button.active { color:#3ABEFF;border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:600px;display:flex;flex-direction:column;gap:0.6rem; }
  .form-card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0; }
  .warn-text { color:#F59E0B;font-size:0.75rem;margin:0;background:rgba(245,158,11,0.08);padding:0.4rem 0.6rem;border-radius:4px;border:1px solid rgba(245,158,11,0.2); }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-danger { padding:0.5rem 1.25rem;background:rgba(239,68,68,0.15);border:1px solid #EF4444;color:#EF4444;border-radius:6px;cursor:pointer;font-size:0.8rem;font-weight:600;align-self:flex-start; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .section-title { font-family:'Orbitron',sans-serif;font-size:0.85rem;color:#8B5CF6;margin:1.25rem 0 0.5rem; }
  .req-card { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.65rem;margin-bottom:0.4rem; }
  .req-header { display:flex;justify-content:space-between;align-items:center; }
  .req-subject { font-size:0.85rem;font-weight:500; }
  .req-content { font-size:0.75rem;color:#94A3B8;margin:0.3rem 0 0; }
  .badge { padding:0.1rem 0.35rem;border-radius:4px;font-size:0.65rem; }
  .badge-pending { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .badge-approved { background:rgba(16,185,129,0.15);color:#10B981; }
  .badge-rejected { background:rgba(239,68,68,0.15);color:#EF4444; }
  .badge-sent { background:rgba(58,190,255,0.15);color:#3ABEFF; }
</style>
