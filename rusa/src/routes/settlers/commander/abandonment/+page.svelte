<!--
  UC-SC-03: Request Settlement Abandonment (Commander → Directors via vote)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    stlRequestAbandonment,
    stlListSettlementAnomalies,
    type AnomalyReportSummary,
  } from '$lib/stores/settlers';

  let anomalies: AnomalyReportSummary[] = $state([]);
  let anomalyReportId = $state('');
  let reason = $state('');
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);
  let loading = $state(true);

  onMount(async () => {
    try {
      anomalies = await stlListSettlementAnomalies();
    } catch (e: any) {
      error = `Could not load anomaly reports: ${e?.message ?? String(e)}`;
    } finally {
      loading = false;
    }
  });

  function selectedAnomaly(): AnomalyReportSummary | undefined {
    return anomalies.find((a) => a.id === anomalyReportId);
  }

  async function handleSubmit() {
    error = ''; success = '';
    if (!anomalyReportId) { error = 'Select an anomaly report first.'; return; }
    if (!reason.trim()) { error = 'Reason is required.'; return; }
    submitting = true;
    try {
      const id = await stlRequestAbandonment({ anomaly_report_id: anomalyReportId, reason });
      success = `Abandonment request submitted (ID: ${id.slice(0, 8)}…). Directors will vote on it.`;
      anomalyReportId = ''; reason = '';
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }
</script>

<h2>Request Settlement Abandonment</h2>
<p class="hint">Select a filed anomaly report and provide justification. Directors must vote to approve.</p>

{#if loading}
  <p class="muted">Loading anomaly reports…</p>
{:else}
  <form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    <label>
      Anomaly Report *
      <select bind:value={anomalyReportId} required>
        <option value="">— Select an anomaly report —</option>
        {#each anomalies as a}
          <option value={a.id}>
            [{a.danger_level ?? 'N/A'}] {a.category ? `${a.category} — ` : ''}{a.description.slice(0, 80)}{a.description.length > 80 ? '…' : ''} ({a.status})
          </option>
        {/each}
      </select>
      {#if anomalies.length === 0}
        <span class="field-hint">No anomaly reports found. File a commander anomaly first.</span>
      {/if}
    </label>

    {#if selectedAnomaly()}
      {@const a = selectedAnomaly()!}
      <div class="preview-card">
        <div class="preview-row"><span class="lbl">Category</span><span>{a.category ?? '—'}</span></div>
        <div class="preview-row"><span class="lbl">Danger</span><span class="badge" class:badge-high={a.danger_level === 'High' || a.danger_level === 'Critical'}>{a.danger_level ?? '—'}</span></div>
        <div class="preview-row"><span class="lbl">Status</span><span>{a.status}</span></div>
        <div class="preview-row full"><span class="lbl">Description</span><span>{a.description}</span></div>
      </div>
    {/if}

    <label>
      Reason *
      <textarea bind:value={reason} rows="5" required
        placeholder="Explain in detail why the settlement must be abandoned…"></textarea>
    </label>

    {#if error}<p class="err">{error}</p>{/if}
    {#if success}<p class="ok">{success}</p>{/if}

    <button type="submit" class="btn-primary" disabled={submitting || !anomalyReportId}>
      {submitting ? 'Submitting…' : 'Submit Abandonment Request'}
    </button>
  </form>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 0.3rem; }
  .hint { color:#64748B;font-size:0.75rem;margin:0 0 1.2rem; }
  .muted { color:#64748B;font-size:0.8rem; }
  .form { display:flex;flex-direction:column;gap:0.85rem;max-width:580px; }
  .form label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8; }
  .form select, .form textarea {
    background:#111827;border:1px solid #374151;border-radius:4px;
    color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem;
  }
  .form select { cursor:pointer; }
  .form textarea { resize:vertical; }
  .field-hint { color:#F59E0B;font-size:0.72rem;margin-top:0.2rem; }
  .preview-card {
    background:#0B0F1A;border:1px solid rgba(58,190,255,0.15);border-radius:6px;
    padding:0.75rem 1rem;display:grid;grid-template-columns:1fr 1fr;gap:0.4rem 1rem;
  }
  .preview-row { display:flex;flex-direction:column;gap:0.1rem;font-size:0.78rem; }
  .preview-row.full { grid-column:1/-1; }
  .lbl { color:#64748B;font-size:0.7rem;text-transform:uppercase;letter-spacing:0.04em; }
  .badge { display:inline-block;padding:0.1rem 0.4rem;border-radius:3px;font-size:0.7rem;background:rgba(100,116,139,0.2);color:#94A3B8; }
  .badge-high { background:rgba(239,68,68,0.15);color:#EF4444; }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem; }
  .btn-primary { background:#3ABEFF;color:#0B0F1A;border:none;border-radius:4px;padding:0.5rem 1.2rem;cursor:pointer;font-weight:600;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:#60CFFF; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
