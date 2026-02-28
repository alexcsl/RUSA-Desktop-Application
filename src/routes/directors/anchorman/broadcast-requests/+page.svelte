<!-- /directors/anchorman/broadcast-requests — TheAnchorman Broadcast + Termination (UC-ANC-01/02/03) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getBroadcastRequestQueue, decideBroadcastRequest, sendInformationalBroadcast,
    terminatePersonnel, getPersonnelList,
    type BroadcastRequestSummary, type PersonnelListItem,
  } from '$lib/stores/directors';

  let queue: BroadcastRequestSummary[] = $state([]);
  let selected: BroadcastRequestSummary | null = $state(null);
  let activeTab: 'queue' | 'send' | 'terminate' = $state('queue');

  let decision: 'approved' | 'rejected' = $state('approved');
  let decisionReason = $state('');
  let error = $state('');
  let success = $state('');

  // Send informational
  let sendSubject = $state('');
  let sendContent = $state('');
  let sendScope = $state('company_wide');
  let sendUrgency = $state('normal');
  let sendError = $state('');
  let sendSuccess = $state('');

  // Terminate personnel (UC-ANC-03)
  let personnel: PersonnelListItem[] = $state([]);
  let termTarget = $state('');
  let termReason = $state('');
  let termDate = $state('');
  let termError = $state('');
  let termSuccess = $state('');
  let termConfirm = $state(false);

  // Exclude Directors and Administrator from termination targets
  const DIRECTOR_ROLES = [
    'TheDirector','GeneralDirector','TheAccountant','TheLibrarian','TheNomad',
    'TheArtificer','TheObserver','TheWanderer','TheTaskmaster','TheGuardian',
    'TheStatistician','TheCoordinator','TheOverseer','TheAnchorman','Administrator',
  ];

  function terminablePersonnel() {
    return personnel.filter(p => !DIRECTOR_ROLES.includes(p.role_name));
  }

  onMount(async () => {
    [queue, personnel] = await Promise.all([getBroadcastRequestQueue(), getPersonnelList()]);
  });

  async function handleDecide() {
    if (!selected) return;
    error = ''; success = '';
    try {
      await decideBroadcastRequest(selected.id, decision, decisionReason || undefined);
      success = `Request ${decision}.`;
      selected = null;
      queue = await getBroadcastRequestQueue();
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }

  async function handleSend() {
    sendError = ''; sendSuccess = '';
    if (!sendSubject.trim() || !sendContent.trim()) { sendError = 'Subject and content required.'; return; }
    try {
      await sendInformationalBroadcast({
        subject: sendSubject, content: sendContent,
        target_scope: sendScope, urgency: sendUrgency,
      });
      sendSuccess = 'Informational broadcast sent.';
      sendSubject = ''; sendContent = '';
    } catch (e: unknown) { sendError = e instanceof Error ? e.message : String(e); }
  }

  async function handleTerminate() {
    termError = ''; termSuccess = '';
    if (!termTarget || !termReason.trim() || !termDate) { termError = 'All fields required.'; return; }
    if (!termConfirm) { termError = 'You must confirm the termination.'; return; }
    try {
      await terminatePersonnel({ target_user_id: termTarget, reason: termReason, effective_date: termDate });
      const p = terminablePersonnel().find(x => x.id === termTarget);
      termSuccess = `${p?.full_name ?? 'Personnel'} terminated effective ${termDate}.`;
      termTarget = ''; termReason = ''; termDate = ''; termConfirm = false;
      personnel = await getPersonnelList();
    } catch (e: unknown) { termError = e instanceof Error ? e.message : String(e); }
  }

  function statusColor(s: string) {
    const m: Record<string,string> = { pending:'st-pending', approved:'st-approved', rejected:'st-rejected', sent:'st-sent' };
    return m[s] ?? '';
  }
</script>

<h1 class="title">Anchorman — Broadcast & Personnel</h1>
<p class="subtitle">Approve broadcast requests, send informational broadcasts, and manage personnel termination.</p>

<div class="tabs">
  <button class:active={activeTab === 'queue'} onclick={() => (activeTab = 'queue')}>
    Approval Queue ({queue.filter(q => q.status === 'pending').length})
  </button>
  <button class:active={activeTab === 'send'} onclick={() => (activeTab = 'send')}>
    Send Broadcast
  </button>
  <button class:active={activeTab === 'terminate'} onclick={() => (activeTab = 'terminate')}>
    Terminate Personnel
  </button>
</div>

{#if activeTab === 'queue'}
  <div class="grid">
    <div class="list-panel">
      {#each queue as req}
        <button class="card" class:selected={selected?.id === req.id} onclick={() => { selected = req; error = ''; success = ''; }}>
          <div class="card-title">{req.subject}</div>
          <div class="card-meta">
            <span class="badge {statusColor(req.status)}">{req.status}</span>
            <span>{req.type_}</span>
            <span>{req.requester_name}</span>
          </div>
        </button>
      {:else}
        <p class="empty">No requests in queue.</p>
      {/each}
    </div>

    <div class="detail-panel">
      {#if selected}
        <h2>{selected.subject}</h2>
        <div class="kv"><span class="k">From:</span><span>{selected.requester_name}</span></div>
        <div class="kv"><span class="k">Type:</span><span>{selected.type_}</span></div>
        <div class="kv"><span class="k">Urgency:</span><span>{selected.urgency ?? 'N/A'}</span></div>
        <div class="kv"><span class="k">Scope:</span><span>{selected.target_scope}</span></div>
        <div class="kv"><span class="k">Status:</span><span class="badge {statusColor(selected.status)}">{selected.status}</span></div>

        <h3>Content</h3>
        <div class="content-block">{selected.content}</div>

        {#if selected.rationale}
          <h3>Rationale</h3>
          <div class="content-block">{selected.rationale}</div>
        {/if}

        {#if selected.status === 'pending'}
          <div class="action-form">
            <h3>Decision</h3>
            <select class="input" bind:value={decision}>
              <option value="approved">Approve</option>
              <option value="rejected">Reject</option>
            </select>
            <textarea class="textarea" placeholder="Reason (optional)…" bind:value={decisionReason} rows="2"></textarea>
            {#if error}<p class="error">{error}</p>{/if}
            {#if success}<p class="success">{success}</p>{/if}
            <button class="btn-primary" onclick={handleDecide}>Submit Decision</button>
          </div>
        {/if}
      {:else}
        <div class="empty-state"><p>Select a request to review.</p></div>
      {/if}
    </div>
  </div>
{:else if activeTab === 'send'}
  <div class="form-card">
    <h2>Send Informational Broadcast</h2>
    <label class="field"><span class="label">Subject</span>
      <input type="text" class="input" bind:value={sendSubject} placeholder="Broadcast subject…" />
    </label>
    <label class="field"><span class="label">Content</span>
      <textarea class="textarea" bind:value={sendContent} rows="5" placeholder="Broadcast content…"></textarea>
    </label>
    <label class="field"><span class="label">Target Scope</span>
      <select class="input" bind:value={sendScope}>
        <option value="all">All Personnel</option>
        <option value="directors">Directors Only</option>
        <option value="settlers">Settlers</option>
        <option value="scientists">Scientists</option>
      </select>
    </label>
    <label class="field"><span class="label">Urgency</span>
      <select class="input" bind:value={sendUrgency}>
        <option value="normal">Normal</option>
        <option value="high">High</option>
        <option value="low">Low</option>
      </select>
    </label>
    {#if sendError}<p class="error">{sendError}</p>{/if}
    {#if sendSuccess}<p class="success">{sendSuccess}</p>{/if}
    <button class="btn-primary" onclick={handleSend}>Send Broadcast</button>
  </div>
{:else if activeTab === 'terminate'}
  <div class="form-card">
    <h2 class="term-title">Terminate Personnel</h2>
    <p class="term-warn">This action is irreversible. The account will be soft-deleted. Directors and The Administrator cannot be terminated.</p>

    <label class="field"><span class="label">Personnel</span>
      <select class="input" bind:value={termTarget}>
        <option value="">— Select personnel —</option>
        {#each terminablePersonnel() as p}
          <option value={p.id}>{p.full_name} ({p.role_name})</option>
        {/each}
      </select>
    </label>

    <label class="field"><span class="label">Reason</span>
      <textarea class="textarea" bind:value={termReason} rows="3" placeholder="Reason for termination…"></textarea>
    </label>

    <label class="field"><span class="label">Effective Date</span>
      <input type="date" class="input" bind:value={termDate} />
    </label>

    <label class="confirm-row">
      <input type="checkbox" bind:checked={termConfirm} />
      <span>I confirm this termination is authorized and final.</span>
    </label>

    {#if termError}<p class="error">{termError}</p>{/if}
    {#if termSuccess}<p class="success">{termSuccess}</p>{/if}
    <button class="btn-danger" onclick={handleTerminate}>Terminate</button>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .tabs { display:flex;gap:0.25rem;margin-bottom:1rem; }
  .tabs button { padding:0.45rem 0.9rem;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);color:#94A3B8;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .tabs button.active { color:#3ABEFF;border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .grid { display:flex;gap:1rem;overflow:hidden; }
  .list-panel { width:320px;overflow-y:auto; }
  .detail-panel { flex:1;overflow-y:auto; }
  .card { width:100%;text-align:left;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.65rem;margin-bottom:0.35rem;cursor:pointer;color:#E6EDF3; }
  .card:hover { border-color:rgba(58,190,255,0.3); }
  .card.selected { border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .card-title { font-size:0.8rem;font-weight:500;margin-bottom:0.2rem; }
  .card-meta { display:flex;gap:0.5rem;font-size:0.7rem;color:#94A3B8; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1rem;color:#3ABEFF;margin:0 0 0.5rem; }
  h3 { font-size:0.85rem;color:#E6EDF3;margin:0.75rem 0 0.4rem; }
  .kv { display:flex;gap:0.5rem;font-size:0.8rem;margin-bottom:0.3rem; }
  .k { color:#94A3B8;min-width:70px; }
  .content-block { background:#0E1428;border:1px solid rgba(58,190,255,0.1);border-radius:6px;padding:0.75rem;font-size:0.8rem;color:#94A3B8;white-space:pre-wrap; }
  .action-form { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-top:0.75rem;display:flex;flex-direction:column;gap:0.5rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:600px;display:flex;flex-direction:column;gap:0.6rem; }
  .form-card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .btn-primary { padding:0.45rem 1rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .badge { padding:0.1rem 0.35rem;border-radius:4px;font-size:0.65rem; }
  .st-pending { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .st-approved { background:rgba(16,185,129,0.15);color:#10B981; }
  .st-rejected { background:rgba(239,68,68,0.15);color:#EF4444; }
  .st-sent { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
  .empty-state { display:flex;justify-content:center;align-items:center;height:100%;color:#475569; }
  .term-title { color:#EF4444!important; }
  .term-warn { color:#F59E0B;font-size:0.75rem;margin:0 0 0.5rem;line-height:1.4; }
  .confirm-row { display:flex;align-items:center;gap:0.5rem;font-size:0.8rem;color:#94A3B8;cursor:pointer; }
  .confirm-row input[type=checkbox] { accent-color:#EF4444; }
  .btn-danger { padding:0.45rem 1rem;background:#EF4444;border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-danger:hover { background:#DC2626; }
</style>
