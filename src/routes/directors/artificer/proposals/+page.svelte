<!--
  /directors/artificer/proposals — TheArtificer: Experiment Proposal Queue
  View and approve/deny experiment proposals routed from Physicist & Mathematician.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getExperimentProposalQueue,
    decideExperimentProposal,
    type ExperimentProposalSummary,
  } from '$lib/stores/directors';

  let proposals: ExperimentProposalSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let success = $state('');

  // Decision modal
  let deciding: ExperimentProposalSummary | null = $state(null);
  let decision: 'approved' | 'denied' = $state('approved');
  let reason = $state('');
  let submitting = $state(false);

  onMount(async () => {
    await loadQueue();
  });

  async function loadQueue() {
    loading = true;
    error = '';
    try {
      proposals = await getExperimentProposalQueue();
    } catch (e: any) {
      error = e?.toString() ?? 'Failed to load proposals.';
    } finally {
      loading = false;
    }
  }

  function openDecision(p: ExperimentProposalSummary) {
    deciding = p;
    decision = 'approved';
    reason = '';
  }

  async function handleDecision() {
    if (!deciding) return;
    submitting = true;
    error = '';
    success = '';
    try {
      await decideExperimentProposal(deciding.request_id, decision, reason || undefined);
      success = `Experiment "${deciding.title}" ${decision}.`;
      deciding = null;
      await loadQueue();
    } catch (e: any) {
      error = e?.toString() ?? 'Decision failed.';
    } finally {
      submitting = false;
    }
  }

  function fmtDate(iso: string): string {
    return new Date(iso).toLocaleString();
  }

  function typeBadge(t: string): string {
    const m: Record<string, string> = { physical: '⚛️ Physical', chemical: '🧪 Chemical', biology_observation: '🧬 Biology' };
    return m[t] ?? t;
  }

  function statusColor(s: string): string {
    if (s === 'pending') return '#F59E0B';
    if (s === 'approved') return '#22C55E';
    if (s === 'denied') return '#EF4444';
    return '#94A3B8';
  }
</script>

<h2>Experiment Proposals</h2>
<p class="hint">Review proposals from Physicists and Mathematicians routed to you for bypass approval.</p>

{#if success}<p class="msg-ok">{success}</p>{/if}
{#if error}<p class="msg-err">{error}</p>{/if}

{#if loading}
  <p class="muted">Loading proposals…</p>
{:else if proposals.length === 0}
  <p class="muted">No experiment proposals in your queue.</p>
{:else}
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>Title</th>
          <th>Type</th>
          <th>Proposed By</th>
          <th>Status</th>
          <th>Submitted</th>
          <th>Action</th>
        </tr>
      </thead>
      <tbody>
        {#each proposals as p}
          <tr>
            <td class="title-col">{p.title}</td>
            <td>{typeBadge(p.experiment_type)}</td>
            <td>{p.requester_name}</td>
            <td><span class="status-badge" style="color:{statusColor(p.status)}">{p.status}</span></td>
            <td class="date-col">{fmtDate(p.created_at)}</td>
            <td>
              {#if p.status === 'pending'}
                <button class="btn-sm" onclick={() => openDecision(p)}>Review</button>
              {:else}
                <span class="muted">—</span>
              {/if}
            </td>
          </tr>
          {#if p.description}
            <tr class="desc-row">
              <td colspan="6"><span class="desc-label">Description:</span> {p.description}</td>
            </tr>
          {/if}
        {/each}
      </tbody>
    </table>
  </div>
{/if}

{#if deciding}
  <div class="modal-overlay" role="dialog" aria-modal="true">
    <div class="modal-card">
      <h3>Review: {deciding.title}</h3>
      <p class="muted">{typeBadge(deciding.experiment_type)} — proposed by {deciding.requester_name}</p>
      {#if deciding.description}<p class="desc">{deciding.description}</p>{/if}

      <div class="form-group">
        <label for="dec">Decision</label>
        <select id="dec" bind:value={decision}>
          <option value="approved">✅ Approve</option>
          <option value="denied">❌ Deny</option>
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
  .status-badge { font-weight:600;text-transform:uppercase;font-size:0.7rem; }
  .desc-row td { color:#94A3B8;font-size:0.75rem;padding:0.25rem 0.5rem 0.5rem;border-bottom:1px solid rgba(255,255,255,0.06); }
  .desc-label { color:#8B5CF6;font-weight:600; }

  .btn-sm { background:rgba(58,190,255,0.1);border:1px solid #3ABEFF;color:#3ABEFF;padding:0.2rem 0.5rem;border-radius:4px;cursor:pointer;font-size:0.75rem; }
  .btn-sm:hover { background:rgba(58,190,255,0.25); }

  .modal-overlay { position:fixed;inset:0;background:rgba(0,0,0,0.7);display:flex;justify-content:center;align-items:center;z-index:50; }
  .modal-card { background:#111827;border:1px solid rgba(58,190,255,0.2);border-radius:10px;padding:1.25rem;width:480px;max-width:95vw; }
  .modal-card h3 { font-family:'Orbitron',sans-serif;font-size:0.95rem;color:#3ABEFF;margin-bottom:0.4rem; }
  .desc { color:#94A3B8;font-size:0.8rem;margin:0.5rem 0; }
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
