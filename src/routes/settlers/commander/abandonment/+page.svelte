<!--
  UC-SC-03: Request Settlement Abandonment (Commander → Directors via vote)
-->
<script lang="ts">
  import { stlRequestAbandonment } from '$lib/stores/settlers';

  let anomalyReportId = $state('');
  let reason = $state('');
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  async function handleSubmit() {
    error = ''; success = '';
    if (!anomalyReportId.trim()) { error = 'Anomaly report ID is required. File a commander anomaly first.'; return; }
    if (!reason.trim()) { error = 'Reason is required.'; return; }
    submitting = true;
    try {
      const id = await stlRequestAbandonment({ anomaly_report_id: anomalyReportId, reason });
      success = `Abandonment request submitted (ID: ${id.slice(0,8)}…). Directors will vote on it.`;
      anomalyReportId = ''; reason = '';
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }
</script>

<h2>Request Settlement Abandonment</h2>
<p class="hint">Requires a supporting anomaly report. Directors must vote to approve.</p>

<form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  <label>
    Anomaly Report ID *
    <input type="text" bind:value={anomalyReportId} placeholder="UUID of the anomaly report" />
  </label>

  <label>
    Reason *
    <textarea bind:value={reason} rows="5" required
      placeholder="Explain why the settlement must be abandoned…"></textarea>
  </label>

  {#if error}<p class="err">{error}</p>{/if}
  {#if success}<p class="ok">{success}</p>{/if}

  <button type="submit" class="btn-primary" disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Abandonment Request'}
  </button>
</form>

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 0.3rem; }
  .hint { color:#64748B;font-size:0.75rem;margin:0 0 1rem; }
  .form { display:flex;flex-direction:column;gap:0.75rem;max-width:560px; }
  .form label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8; }
  .form input, .form textarea { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem; }
  .form textarea { resize:vertical; }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem; }
  .btn-primary { background:#3ABEFF;color:#0B0F1A;border:none;border-radius:4px;padding:0.5rem 1.2rem;cursor:pointer;font-weight:600;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:#60CFFF; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
