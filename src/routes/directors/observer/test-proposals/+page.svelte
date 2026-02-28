<!--
  /directors/observer/test-proposals — TheObserver: Test Proposal Review Queue
  Approve/reject new test methods proposed by Chemists and Biologists.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getTestProposalQueue,
    decideTestProposal,
    type TestProposalQueueItem,
  } from '$lib/stores/directors';

  let proposals: TestProposalQueueItem[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let success = $state('');

  let deciding: TestProposalQueueItem | null = $state(null);
  let decision: 'approved' | 'rejected' = $state('approved');
  let reason = $state('');
  let submitting = $state(false);

  onMount(async () => { await loadQueue(); });

  async function loadQueue() {
    loading = true; error = '';
    try { proposals = await getTestProposalQueue(); }
    catch (e: any) { error = e?.toString() ?? 'Failed to load test proposals.'; }
    finally { loading = false; }
  }

  function openDecision(p: TestProposalQueueItem) {
    deciding = p; decision = 'approved'; reason = '';
  }

  async function handleDecision() {
    if (!deciding) return;
    submitting = true; error = ''; success = '';
    try {
      await decideTestProposal(deciding.id, decision, reason || undefined);
      success = `Test "${deciding.proposal_data.name ?? 'Unnamed'}" ${decision}.`;
      deciding = null;
      await loadQueue();
    } catch (e: any) { error = e?.toString() ?? 'Decision failed.'; }
    finally { submitting = false; }
  }

  function fmtDate(iso: string): string { return new Date(iso).toLocaleString(); }
</script>

<h2>Test Proposal Review</h2>
<p class="hint">Approve or reject new test methods proposed by Chemists and Biologists.</p>

{#if success}<p class="msg-ok">{success}</p>{/if}
{#if error}<p class="msg-err">{error}</p>{/if}

{#if loading}
  <p class="muted">Loading proposals…</p>
{:else if proposals.length === 0}
  <p class="muted">No pending test proposals in your queue.</p>
{:else}
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>Test Name</th>
          <th>Category</th>
          <th>Scope</th>
          <th>Proposed By</th>
          <th>Submitted</th>
          <th>Action</th>
        </tr>
      </thead>
      <tbody>
        {#each proposals as p}
          <tr>
            <td class="title-col">{p.proposal_data.name ?? '—'}</td>
            <td>{p.proposal_data.category ?? '—'}</td>
            <td>{p.proposal_data.species_scope ?? '—'}</td>
            <td>{p.proposer_name}</td>
            <td class="date-col">{fmtDate(p.created_at)}</td>
            <td>
              <button class="btn-sm" onclick={() => openDecision(p)}>Review</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

{#if deciding}
  <div class="modal-overlay" role="dialog" aria-modal="true">
    <div class="modal-card">
      <h3>Review: {deciding.proposal_data.name ?? 'Unnamed Test'}</h3>
      <p class="muted">Proposed by {deciding.proposer_name}</p>

      <div class="detail-grid">
        {#if deciding.proposal_data.goal}
          <div class="detail-item"><span class="lbl">Goal:</span> {deciding.proposal_data.goal}</div>
        {/if}
        {#if deciding.proposal_data.procedure}
          <div class="detail-item"><span class="lbl">Procedure:</span> {deciding.proposal_data.procedure}</div>
        {/if}
        {#if deciding.proposal_data.apparatuses}
          <div class="detail-item"><span class="lbl">Apparatuses:</span> {deciding.proposal_data.apparatuses}</div>
        {/if}
        {#if deciding.proposal_data.required_data}
          <div class="detail-item"><span class="lbl">Required Data:</span> {deciding.proposal_data.required_data}</div>
        {/if}
        {#if deciding.proposal_data.justification}
          <div class="detail-item"><span class="lbl">Justification:</span> {deciding.proposal_data.justification}</div>
        {/if}
      </div>

      <div class="form-group">
        <label for="dec">Decision</label>
        <select id="dec" bind:value={decision}>
          <option value="approved">✅ Approve</option>
          <option value="rejected">❌ Reject</option>
        </select>
      </div>
      <div class="form-group">
        <label for="rsn">Reason (optional)</label>
        <textarea id="rsn" bind:value={reason} rows="3" placeholder="Provide rationale…"></textarea>
      </div>
      <div class="modal-actions">
        <button class="btn-primary" onclick={handleDecision} disabled={submitting}>
          {submitting ? 'Submitting…' : 'Confirm'}
        </button>
        <button class="btn-cancel" onclick={() => (deciding = null)} disabled={submitting}>Cancel</button>
      </div>
    </div>
  </div>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.3rem; }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:1rem; }
  .muted { color:#64748B;font-size:0.8rem; }
  .msg-ok { color:#22C55E;font-size:0.8rem;margin-bottom:0.5rem; }
  .msg-err { color:#EF4444;font-size:0.8rem;margin-bottom:0.5rem; }

  .table-wrap { overflow-x:auto; }
  table { width:100%;border-collapse:collapse;font-size:0.8rem; }
  th { text-align:left;padding:0.5rem;border-bottom:1px solid rgba(58,190,255,0.15);color:#94A3B8;font-weight:600;white-space:nowrap; }
  td { padding:0.5rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .title-col { color:#E6EDF3;font-weight:500;max-width:200px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .date-col { color:#64748B;font-size:0.75rem;white-space:nowrap; }

  .btn-sm { background:rgba(58,190,255,0.1);border:1px solid #3ABEFF;color:#3ABEFF;padding:0.2rem 0.5rem;border-radius:4px;cursor:pointer;font-size:0.75rem; }
  .btn-sm:hover { background:rgba(58,190,255,0.25); }

  .modal-overlay { position:fixed;inset:0;background:rgba(0,0,0,0.7);display:flex;justify-content:center;align-items:center;z-index:50; }
  .modal-card { background:#111827;border:1px solid rgba(58,190,255,0.2);border-radius:10px;padding:1.25rem;width:520px;max-width:95vw; }
  .modal-card h3 { font-family:'Orbitron',sans-serif;font-size:0.95rem;color:#3ABEFF;margin-bottom:0.4rem; }

  .detail-grid { display:flex;flex-direction:column;gap:0.4rem;margin:0.75rem 0;padding:0.75rem;background:rgba(255,255,255,0.02);border-radius:6px;border:1px solid rgba(255,255,255,0.06); }
  .detail-item { font-size:0.8rem;color:#CBD5E1; }
  .lbl { color:#8B5CF6;font-weight:600; }

  .form-group { margin-bottom:0.75rem; }
  .form-group label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.2rem; }
  .form-group select, .form-group textarea { width:100%;background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem;font-size:0.8rem; }
  .form-group textarea { resize:vertical; }
  .modal-actions { display:flex;gap:0.5rem;margin-top:0.75rem; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;padding:0.4rem 0.85rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-cancel { background:transparent;border:1px solid #475569;color:#94A3B8;padding:0.4rem 0.85rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-cancel:hover { border-color:#EF4444;color:#EF4444; }
</style>
