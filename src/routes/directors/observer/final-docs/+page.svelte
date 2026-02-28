<!--
  /directors/observer/final-docs — TheObserver: Final Document Review Queue
  Approve/reject final object documents (species) → science_archive.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getFinalDocumentQueue,
    decideFinalDocument,
    type FinalDocQueueItem,
  } from '$lib/stores/directors';

  let docs: FinalDocQueueItem[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let success = $state('');

  let deciding: FinalDocQueueItem | null = $state(null);
  let decision: 'approved' | 'rejected' = $state('approved');
  let reason = $state('');
  let submitting = $state(false);

  onMount(async () => { await loadQueue(); });

  async function loadQueue() {
    loading = true; error = '';
    try { docs = await getFinalDocumentQueue(); }
    catch (e: any) { error = e?.toString() ?? 'Failed to load final documents.'; }
    finally { loading = false; }
  }

  function openDecision(d: FinalDocQueueItem) {
    deciding = d; decision = 'approved'; reason = '';
  }

  async function handleDecision() {
    if (!deciding) return;
    submitting = true; error = ''; success = '';
    try {
      await decideFinalDocument(deciding.id, decision, reason || undefined);
      const name = (deciding.document_data as any).name ?? deciding.doc_type;
      success = `Document "${name}" ${decision}. ${decision === 'approved' ? 'Archived to science_archive.' : ''}`;
      deciding = null;
      await loadQueue();
    } catch (e: any) { error = e?.toString() ?? 'Decision failed.'; }
    finally { submitting = false; }
  }

  function fmtDate(iso: string): string { return new Date(iso).toLocaleString(); }

  function typeBadge(t: string): string {
    const m: Record<string, string> = { matter: '🧱 Matter', physical_object: '⚛️ Physical Object', species: '🧬 Species' };
    return m[t] ?? t;
  }
</script>

<h2>Final Document Review</h2>
<p class="hint">Review final species documents for archiving.</p>

{#if success}<p class="msg-ok">{success}</p>{/if}
{#if error}<p class="msg-err">{error}</p>{/if}

{#if loading}
  <p class="muted">Loading documents…</p>
{:else if docs.length === 0}
  <p class="muted">No pending final documents in your queue.</p>
{:else}
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>Name</th>
          <th>Type</th>
          <th>Experiment</th>
          <th>Submitted By</th>
          <th>Date</th>
          <th>Action</th>
        </tr>
      </thead>
      <tbody>
        {#each docs as d}
          <tr>
            <td class="title-col">{(d.document_data as any).name ?? '—'}</td>
            <td>{typeBadge(d.doc_type)}</td>
            <td>{d.experiment_title}</td>
            <td>{d.submitter_name}</td>
            <td class="date-col">{fmtDate(d.created_at)}</td>
            <td>
              <button class="btn-sm" onclick={() => openDecision(d)}>Review</button>
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
      <h3>Review: {(deciding.document_data as any).name ?? deciding.doc_type}</h3>
      <p class="muted">{typeBadge(deciding.doc_type)} — from {deciding.submitter_name}</p>
      <p class="muted">Experiment: {deciding.experiment_title}</p>

      <div class="detail-grid">
        {#each Object.entries(deciding.document_data) as [key, val]}
          <div class="detail-item">
            <span class="lbl">{key.replace(/_/g, ' ')}:</span>
            {typeof val === 'object' ? JSON.stringify(val) : String(val)}
          </div>
        {/each}
      </div>

      <div class="form-group">
        <label for="dec">Decision</label>
        <select id="dec" bind:value={decision}>
          <option value="approved">✅ Approve → Archive</option>
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
  .modal-card { background:#111827;border:1px solid rgba(58,190,255,0.2);border-radius:10px;padding:1.25rem;width:560px;max-width:95vw; }
  .modal-card h3 { font-family:'Orbitron',sans-serif;font-size:0.95rem;color:#3ABEFF;margin-bottom:0.4rem; }

  .detail-grid { display:flex;flex-direction:column;gap:0.4rem;margin:0.75rem 0;padding:0.75rem;background:rgba(255,255,255,0.02);border-radius:6px;border:1px solid rgba(255,255,255,0.06);max-height:250px;overflow-y:auto; }
  .detail-item { font-size:0.8rem;color:#CBD5E1;word-break:break-word; }
  .lbl { color:#8B5CF6;font-weight:600;text-transform:capitalize; }

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
