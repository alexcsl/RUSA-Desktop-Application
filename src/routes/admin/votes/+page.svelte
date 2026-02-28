<!--
  /admin/votes — Administrator Vote Management (UC-DIR-08 override, UC-DIR-06 terminate)
  Only accessible by Administrator role. Shows all sessions with override/terminate actions.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { currentUser } from '$lib/stores/auth';
  import type { SessionUser } from '$lib/stores/auth';
  import {
    getVoteSessions,
    getVoteSessionDetail,
    adminOverrideVote,
    adminTerminateVote,
    type VoteSessionSummary,
    type VoteSessionWithRecords,
  } from '$lib/stores/directors';

  let user: SessionUser | null = $state(null);
  currentUser.subscribe((v) => (user = v));

  let sessions: VoteSessionSummary[] = $state([]);
  let selected: VoteSessionWithRecords | null = $state(null);

  let overrideDecision: 'approved' | 'denied' = $state('approved');
  let overrideReason = $state('');
  let terminateReason = $state('');
  let error = $state('');
  let showOverride = $state(false);
  let showTerminate = $state(false);

  onMount(async () => {
    sessions = await getVoteSessions();
  });

  async function selectSession(id: string) {
    selected = await getVoteSessionDetail(id);
    showOverride = false;
    showTerminate = false;
    error = '';
  }

  async function handleOverride() {
    if (!selected || !overrideReason.trim()) {
      error = 'Reason is required for override.';
      return;
    }
    error = '';
    try {
      await adminOverrideVote(selected.session.id, overrideDecision, overrideReason);
      await selectSession(selected.session.id);
      sessions = await getVoteSessions();
      showOverride = false;
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }

  async function handleTerminate() {
    if (!selected || !terminateReason.trim()) {
      error = 'Reason is required for termination.';
      return;
    }
    error = '';
    try {
      await adminTerminateVote(selected.session.id, terminateReason);
      await selectSession(selected.session.id);
      sessions = await getVoteSessions();
      showTerminate = false;
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }

  function isOpen(s: VoteSessionSummary) {
    return s.status === 'open' || s.status === 'quorum_pending';
  }
</script>

<div class="page">
  <div class="content">
    <aside class="sidebar">
      <h2 class="sidebar-title">All Sessions</h2>
      {#each sessions as s}
        <button class="vote-card" class:selected={selected?.session.id === s.id} onclick={() => selectSession(s.id)}>
          <div class="topic">{s.topic}</div>
          <div class="meta">
            <span class="badge badge-{s.status}">{s.status}</span>
            {#if s.result}<span class="result-{s.result}">{s.result}</span>{/if}
          </div>
        </button>
      {:else}
        <p class="empty">No sessions.</p>
      {/each}
    </aside>

    <main class="main-panel">
      {#if selected}
        <h2>{selected.session.topic}</h2>
        {#if selected.session.context}<p class="context">{selected.session.context}</p>{/if}

        <div class="tally-grid">
          <div class="t yay"><span class="n">{selected.session.total_yay}</span><span class="l">Yay</span></div>
          <div class="t nay"><span class="n">{selected.session.total_nay}</span><span class="l">Nay</span></div>
          <div class="t abs"><span class="n">{selected.session.total_abstain}</span><span class="l">Abstain</span></div>
        </div>

        {#if selected.session.admin_overridden}
          <div class="override-notice">
            Override: {selected.session.admin_override_decision} — {selected.session.admin_override_reason}
          </div>
        {/if}

        {#if selected.records.length > 0}
          <h3>Records</h3>
          <table class="tbl">
            <thead><tr><th>Director</th><th>Vote</th><th>Reason</th></tr></thead>
            <tbody>
              {#each selected.records as r}
                <tr><td>{r.director_name}</td><td class="vote-{r.vote}">{r.vote}</td><td>{r.reason}</td></tr>
              {/each}
            </tbody>
          </table>
        {/if}

        <!-- Admin actions only on open sessions -->
        {#if isOpen({ ...selected.session, total_yay: selected.session.total_yay, total_nay: selected.session.total_nay, total_abstain: selected.session.total_abstain } as VoteSessionSummary)}
          <div class="actions">
            <button class="btn-warn" onclick={() => { showOverride = !showOverride; showTerminate = false; }}>Override</button>
            <button class="btn-danger" onclick={() => { showTerminate = !showTerminate; showOverride = false; }}>Terminate</button>
          </div>

          {#if showOverride}
            <div class="action-form">
              <h3>Override Decision</h3>
              <select bind:value={overrideDecision} class="input">
                <option value="approved">Approved</option>
                <option value="denied">Denied</option>
              </select>
              <textarea class="textarea" placeholder="Reason..." bind:value={overrideReason} rows="3"></textarea>
              <button class="btn-primary" onclick={handleOverride}>Confirm Override</button>
            </div>
          {/if}

          {#if showTerminate}
            <div class="action-form">
              <h3>Terminate Session</h3>
              <textarea class="textarea" placeholder="Reason..." bind:value={terminateReason} rows="3"></textarea>
              <button class="btn-danger" onclick={handleTerminate}>Confirm Termination</button>
            </div>
          {/if}

          {#if error}<p class="error">{error}</p>{/if}
        {/if}
      {:else}
        <div class="empty-state"><p>Select a session to manage.</p></div>
      {/if}
    </main>
  </div>
</div>

<style>
  .page { display:flex;flex-direction:column;height:100%;color:#E6EDF3;font-family:'Inter',sans-serif; }
  .content { display:flex;flex:1;overflow:hidden; }
  .sidebar { width:300px;background:#111827;border-right:1px solid rgba(58,190,255,0.1);overflow-y:auto;padding:0.5rem; }
  .sidebar-title { font-family:'Orbitron',sans-serif;font-size:0.85rem;color:#8B5CF6;padding:0.5rem;margin:0; }
  .vote-card { width:100%;text-align:left;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.65rem;margin-bottom:0.35rem;cursor:pointer;color:#E6EDF3; }
  .vote-card.selected { border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .topic { font-size:0.8rem;font-weight:500;margin-bottom:0.2rem; }
  .meta { display:flex;justify-content:space-between;font-size:0.7rem;color:#94A3B8; }
  .main-panel { flex:1;overflow-y:auto;padding:1.5rem; }
  .main-panel h2 { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#3ABEFF;margin:0 0 0.5rem; }
  .context { color:#94A3B8;font-size:0.85rem;margin-bottom:1rem; }
  .tally-grid { display:flex;gap:0.75rem;margin-bottom:1rem; }
  .t { background:rgba(14,20,40,0.8);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:0.8rem 1.2rem;text-align:center;flex:1; }
  .n { display:block;font-size:1.6rem;font-weight:700;font-family:'Orbitron',sans-serif; }
  .l { font-size:0.7rem;color:#94A3B8; }
  .yay .n { color:#10B981; } .nay .n { color:#EF4444; } .abs .n { color:#F59E0B; }
  .override-notice { background:rgba(245,158,11,0.1);border:1px solid #F59E0B;border-radius:6px;padding:0.65rem;margin-bottom:1rem;font-size:0.8rem;color:#F59E0B; }
  .tbl { width:100%;border-collapse:collapse;margin-bottom:1rem; }
  .tbl th { text-align:left;padding:0.4rem;border-bottom:1px solid rgba(58,190,255,0.15);color:#94A3B8;font-size:0.7rem; }
  .tbl td { padding:0.4rem;border-bottom:1px solid rgba(58,190,255,0.05);font-size:0.8rem; }
  .vote-yay { color:#10B981; } .vote-nay { color:#EF4444; } .vote-abstain { color:#F59E0B; }
  .actions { display:flex;gap:0.75rem;margin:1rem 0; }
  .action-form { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-bottom:1rem;display:flex;flex-direction:column;gap:0.5rem; }
  .action-form h3 { margin:0;font-size:0.9rem;font-family:'Orbitron',sans-serif; }
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.85rem; }
  .btn-warn { padding:0.5rem 1rem;background:rgba(245,158,11,0.15);border:1px solid #F59E0B;color:#F59E0B;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-danger { padding:0.5rem 1rem;background:rgba(239,68,68,0.15);border:1px solid #EF4444;color:#EF4444;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .error { color:#EF4444;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
  .empty-state { display:flex;justify-content:center;align-items:center;height:100%;color:#475569; }
  .badge { padding:0.12rem 0.4rem;border-radius:4px;font-size:0.65rem; }
  .badge-open { background:rgba(16,185,129,0.15);color:#10B981; }
  .badge-quorum_pending { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .badge-decided { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .badge-overridden { background:rgba(239,68,68,0.15);color:#EF4444; }
  .badge-terminated { background:rgba(71,85,105,0.3);color:#94A3B8; }
  .result-approved { color:#10B981; } .result-denied { color:#EF4444; }
  h3 { font-size:0.85rem;margin:0.75rem 0 0.4rem; }
</style>
