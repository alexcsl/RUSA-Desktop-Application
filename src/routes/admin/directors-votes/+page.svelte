<!--
  /admin/directors-votes — Directors Voting view embedded in the Admin layout.
  Mirrors the /directors/votes page so the Administrator can see/participate
  in voting without leaving the admin shell.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getPendingVotes,
    getVoteSessions,
    getVoteSessionDetail,
    castVote,
    changeVote,
    initiateAdHocVote,
    type VoteSessionSummary,
    type VoteSessionWithRecords,
  } from '$lib/stores/directors';

  let pendingVotes: VoteSessionSummary[] = $state([]);
  let allVotes: VoteSessionSummary[] = $state([]);
  let selectedSession: VoteSessionWithRecords | null = $state(null);

  let voteChoice: 'yay' | 'nay' | 'abstain' = $state('yay');
  let voteReason = $state('');
  let voteError = $state('');

  let showAdHoc = $state(false);
  let adHocTopic = $state('');
  let adHocContext = $state('');
  let adHocError = $state('');

  let activeTab: 'pending' | 'history' = $state('pending');

  onMount(async () => { await loadData(); });

  async function loadData() {
    try {
      [pendingVotes, allVotes] = await Promise.all([getPendingVotes(), getVoteSessions()]);
    } catch (e) { console.error('Failed to load votes:', e); }
  }

  async function selectSession(id: string) {
    try {
      selectedSession = await getVoteSessionDetail(id);
      voteChoice = (selectedSession?.my_vote?.vote as typeof voteChoice) ?? 'yay';
      voteReason = selectedSession?.my_vote?.reason ?? '';
      voteError = '';
    } catch (e: unknown) { voteError = e instanceof Error ? e.message : String(e); }
  }

  async function handleCastVote() {
    if (!selectedSession) return;
    voteError = '';
    try {
      if (selectedSession.my_vote) {
        await changeVote(selectedSession.session.id, voteChoice, voteReason);
      } else {
        await castVote(selectedSession.session.id, voteChoice, voteReason);
      }
      await selectSession(selectedSession.session.id);
      await loadData();
    } catch (e: unknown) { voteError = e instanceof Error ? e.message : String(e); }
  }

  async function handleAdHocSubmit() {
    adHocError = '';
    try {
      await initiateAdHocVote(adHocTopic, adHocContext || undefined);
      showAdHoc = false;
      adHocTopic = '';
      adHocContext = '';
      await loadData();
    } catch (e: unknown) { adHocError = e instanceof Error ? e.message : String(e); }
  }

  function statusBadge(status: string) {
    const map: Record<string, string> = {
      open: 'badge-open', quorum_pending: 'badge-pending',
      decided: 'badge-decided', overridden: 'badge-override', terminated: 'badge-terminated',
    };
    return map[status] ?? '';
  }
</script>

<div class="page">
  <div class="content">
    <aside class="sidebar">
      <div class="tabs">
        <button class:active={activeTab === 'pending'} onclick={() => (activeTab = 'pending')}>
          Pending ({pendingVotes.length})
        </button>
        <button class:active={activeTab === 'history'} onclick={() => (activeTab = 'history')}>
          History
        </button>
      </div>

      <div class="vote-list">
        {#if activeTab === 'pending'}
          {#each pendingVotes as session}
            <button class="vote-card" class:selected={selectedSession?.session.id === session.id} onclick={() => selectSession(session.id)}>
              <div class="vote-topic">{session.topic}</div>
              <div class="vote-meta">
                <span class="badge {statusBadge(session.status)}">{session.status}</span>
                <span>{session.total_yay + session.total_nay + session.total_abstain}/13</span>
              </div>
            </button>
          {:else}
            <p class="empty">No pending votes.</p>
          {/each}
        {:else}
          {#each allVotes as session}
            <button class="vote-card" class:selected={selectedSession?.session.id === session.id} onclick={() => selectSession(session.id)}>
              <div class="vote-topic">{session.topic}</div>
              <div class="vote-meta">
                <span class="badge {statusBadge(session.status)}">{session.status}</span>
                {#if session.result}<span class="result-{session.result}">{session.result}</span>{/if}
              </div>
            </button>
          {:else}
            <p class="empty">No vote history.</p>
          {/each}
        {/if}
      </div>

      <button class="btn-adhoc" onclick={() => (showAdHoc = !showAdHoc)}>+ Ad-Hoc Vote</button>

      {#if showAdHoc}
        <div class="adhoc-form">
          <input type="text" placeholder="Vote topic..." bind:value={adHocTopic} class="input" />
          <textarea placeholder="Supporting context (optional)" bind:value={adHocContext} class="textarea" rows="3"></textarea>
          {#if adHocError}<p class="error">{adHocError}</p>{/if}
          <button class="btn-primary" onclick={handleAdHocSubmit}>Submit</button>
        </div>
      {/if}
    </aside>

    <main class="main-panel">
      {#if selectedSession}
        <div class="session-detail">
          <h2>{selectedSession.session.topic}</h2>
          {#if selectedSession.session.context}<p class="context">{selectedSession.session.context}</p>{/if}

          <div class="tally-grid">
            <div class="tally yay"><span class="tally-num">{selectedSession.session.total_yay}</span><span class="tally-label">Yay</span></div>
            <div class="tally nay"><span class="tally-num">{selectedSession.session.total_nay}</span><span class="tally-label">Nay</span></div>
            <div class="tally abstain"><span class="tally-num">{selectedSession.session.total_abstain}</span><span class="tally-label">Abstain</span></div>
          </div>

          <div class="meta-row">
            <span>Status: <strong class="badge {statusBadge(selectedSession.session.status)}">{selectedSession.session.status}</strong></span>
            <span>Opened: {new Date(selectedSession.session.opens_at).toLocaleString()}</span>
            {#if selectedSession.session.closes_at}<span>Closed: {new Date(selectedSession.session.closes_at).toLocaleString()}</span>{/if}
            {#if selectedSession.session.result}<span>Result: <strong class="result-{selectedSession.session.result}">{selectedSession.session.result}</strong></span>{/if}
          </div>

          {#if selectedSession.session.admin_overridden}
            <div class="override-notice">
              <strong>Administrator Override:</strong> {selectedSession.session.admin_override_decision} — {selectedSession.session.admin_override_reason}
            </div>
          {/if}

          {#if selectedSession.records.length > 0}
            <h3>Vote Records</h3>
            <table class="records-table">
              <thead><tr><th>Director</th><th>Vote</th><th>Reason</th><th>Time</th></tr></thead>
              <tbody>
                {#each selectedSession.records as record}
                  <tr>
                    <td>{record.director_name}</td>
                    <td><span class="vote-chip vote-{record.vote}">{record.vote}</span></td>
                    <td>{record.reason}</td>
                    <td>{new Date(record.created_at).toLocaleString()}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          {/if}

          {#if selectedSession.session.status === 'open' || selectedSession.session.status === 'quorum_pending'}
            <div class="vote-form">
              <h3>{selectedSession.my_vote ? 'Change Your Vote' : 'Cast Your Vote'}</h3>
              <div class="radio-group">
                <label class="radio-option" class:selected={voteChoice === 'yay'}>
                  <input type="radio" bind:group={voteChoice} value="yay" /><span class="vote-chip vote-yay">Yay</span>
                </label>
                <label class="radio-option" class:selected={voteChoice === 'nay'}>
                  <input type="radio" bind:group={voteChoice} value="nay" /><span class="vote-chip vote-nay">Nay</span>
                </label>
                <label class="radio-option" class:selected={voteChoice === 'abstain'}>
                  <input type="radio" bind:group={voteChoice} value="abstain" /><span class="vote-chip vote-abstain">Abstain</span>
                </label>
              </div>
              <textarea class="textarea" placeholder="Written reason (required)..." bind:value={voteReason} rows="3"></textarea>
              {#if voteError}<p class="error">{voteError}</p>{/if}
              <button class="btn-primary" onclick={handleCastVote}>
                {selectedSession.my_vote ? 'Change Vote' : 'Cast Vote'}
              </button>
            </div>
          {/if}
        </div>
      {:else}
        <div class="empty-state"><p>Select a voting session from the sidebar to view details.</p></div>
      {/if}
    </main>
  </div>
</div>

<style>
  .page { display:flex;flex-direction:column;height:100%;color:#E6EDF3;font-family:'Inter',sans-serif; }
  .content { display:flex;flex:1;overflow:hidden; }

  .sidebar { width:320px;min-width:280px;background:#111827;border-right:1px solid rgba(58,190,255,0.1);display:flex;flex-direction:column;overflow-y:auto; }
  .tabs { display:flex;border-bottom:1px solid rgba(58,190,255,0.1); }
  .tabs button { flex:1;padding:0.6rem;background:transparent;border:none;color:#94A3B8;cursor:pointer;font-size:0.8rem; }
  .tabs button.active { color:#3ABEFF;border-bottom:2px solid #3ABEFF; }
  .vote-list { flex:1;overflow-y:auto;padding:0.5rem; }
  .vote-card { width:100%;text-align:left;background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.75rem;margin-bottom:0.4rem;cursor:pointer;color:#E6EDF3; }
  .vote-card:hover { border-color:rgba(58,190,255,0.3); }
  .vote-card.selected { border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .vote-topic { font-size:0.85rem;font-weight:500;margin-bottom:0.3rem; }
  .vote-meta { display:flex;justify-content:space-between;font-size:0.7rem;color:#94A3B8;align-items:center; }
  .btn-adhoc { margin:0.5rem;padding:0.5rem;background:rgba(139,92,246,0.15);border:1px solid #8B5CF6;color:#C084FC;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-adhoc:hover { background:rgba(139,92,246,0.25); }
  .adhoc-form { padding:0.5rem;display:flex;flex-direction:column;gap:0.5rem; }

  .main-panel { flex:1;overflow-y:auto;padding:1.5rem; }
  .session-detail h2 { font-family:'Orbitron',sans-serif;font-size:1.2rem;color:#3ABEFF;margin:0 0 0.5rem; }
  .context { color:#94A3B8;font-size:0.9rem;margin-bottom:1rem; }
  .tally-grid { display:flex;gap:1rem;margin-bottom:1rem; }
  .tally { background:rgba(14,20,40,0.8);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem 1.5rem;text-align:center;flex:1; }
  .tally-num { display:block;font-size:2rem;font-weight:700;font-family:'Orbitron',sans-serif; }
  .tally-label { font-size:0.75rem;color:#94A3B8; }
  .tally.yay .tally-num { color:#10B981; }
  .tally.nay .tally-num { color:#EF4444; }
  .tally.abstain .tally-num { color:#F59E0B; }
  .meta-row { display:flex;gap:1.5rem;flex-wrap:wrap;font-size:0.8rem;color:#94A3B8;margin-bottom:1rem; }
  .override-notice { background:rgba(245,158,11,0.1);border:1px solid #F59E0B;border-radius:6px;padding:0.75rem;margin-bottom:1rem;font-size:0.85rem;color:#F59E0B; }
  .records-table { width:100%;border-collapse:collapse;margin-bottom:1.5rem; }
  .records-table th { text-align:left;padding:0.5rem;border-bottom:1px solid rgba(58,190,255,0.15);color:#94A3B8;font-size:0.75rem; }
  .records-table td { padding:0.5rem;border-bottom:1px solid rgba(58,190,255,0.05);font-size:0.8rem; }
  .vote-form { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem; }
  .vote-form h3 { margin:0 0 0.75rem;font-size:0.95rem;font-family:'Orbitron',sans-serif; }
  .radio-group { display:flex;gap:0.75rem;margin-bottom:0.75rem; }
  .radio-option { display:flex;align-items:center;gap:0.4rem;cursor:pointer;padding:0.4rem 0.75rem;border-radius:6px;border:1px solid transparent; }
  .radio-option.selected { border-color:rgba(58,190,255,0.3);background:rgba(58,190,255,0.05); }
  .radio-option input { display:none; }
  .vote-chip { padding:0.2rem 0.5rem;border-radius:4px;font-size:0.75rem;font-weight:600;text-transform:uppercase; }
  .vote-yay { background:rgba(16,185,129,0.15);color:#10B981; }
  .vote-nay { background:rgba(239,68,68,0.15);color:#EF4444; }
  .vote-abstain { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .badge { padding:0.15rem 0.45rem;border-radius:4px;font-size:0.7rem;font-weight:500; }
  .badge-open { background:rgba(16,185,129,0.15);color:#10B981; }
  .badge-pending { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .badge-decided { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .badge-override { background:rgba(239,68,68,0.15);color:#EF4444; }
  .badge-terminated { background:rgba(71,85,105,0.3);color:#94A3B8; }
  .result-approved { color:#10B981; }
  .result-denied { color:#EF4444; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.6rem;font-size:0.85rem;font-family:'Inter',sans-serif; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF;box-shadow:0 0 8px rgba(58,190,255,0.3); }
  .textarea { resize:vertical; }
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.85rem; }
  .btn-primary:hover { box-shadow:0 0 12px rgba(58,190,255,0.6); }
  .error { color:#EF4444;font-size:0.8rem;margin:0.25rem 0; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1.5rem; }
  .empty-state { display:flex;justify-content:center;align-items:center;height:100%;color:#475569; }
  h3 { font-size:0.9rem;color:#E6EDF3;margin:1rem 0 0.5rem; }
</style>
