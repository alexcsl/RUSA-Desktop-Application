<!-- /security/daily-report/new — Submit Daily Security Report (UC-SH-05) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { submitDailyReport, getMyDailyReports, type DailySecurityReportSummary } from '$lib/stores/security';

  let report_date = $state(new Date().toISOString().slice(0, 10));
  let findings_summary = $state('');
  let risk_notes = $state('');
  let recommended_actions = $state('');

  let error = $state('');
  let success = $state('');
  let submitting = $state(false);
  let reports: DailySecurityReportSummary[] = $state([]);

  onMount(async () => {
    try { reports = await getMyDailyReports(); } catch {}
  });

  async function handleSubmit() {
    error = ''; success = '';
    if (!findings_summary.trim()) { error = 'Findings summary is required.'; return; }

    submitting = true;
    try {
      await submitDailyReport({
        report_date,
        findings_summary,
        risk_notes: risk_notes || undefined,
        recommended_actions: recommended_actions || undefined,
      });
      success = 'Daily report submitted and queued for Guardian delivery.';
      findings_summary = ''; risk_notes = ''; recommended_actions = '';
      report_date = new Date().toISOString().slice(0, 10);
      reports = await getMyDailyReports();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<h1 class="title">Daily Security Report</h1>
<p class="subtitle">Compile and submit a daily findings summary for The Guardian's review. This is required at the end of each operational cycle.</p>

<div class="form-card">
  <h2>New Report</h2>

  <label class="field"><span class="label">Report Date *</span>
    <input type="date" class="input" bind:value={report_date} />
  </label>

  <label class="field"><span class="label">Findings Summary *</span>
    <textarea class="textarea" bind:value={findings_summary} rows="5" placeholder="Summarise all security findings for the operational cycle…"></textarea>
  </label>

  <label class="field"><span class="label">Risk Notes</span>
    <textarea class="textarea" bind:value={risk_notes} rows="3" placeholder="Any elevated risk areas or emerging threats…"></textarea>
  </label>

  <label class="field"><span class="label">Recommended Actions</span>
    <textarea class="textarea" bind:value={recommended_actions} rows="3" placeholder="Suggested responses or preventive measures…"></textarea>
  </label>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}

  <button class="btn-primary" onclick={handleSubmit} disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Report'}
  </button>
</div>

{#if reports.length > 0}
  <h2 class="section-title">Previous Reports</h2>
  <div class="report-list">
    {#each reports as r}
      <div class="report-card">
        <div class="report-header">
          <span class="report-date-label">{r.report_date}</span>
          {#if r.delivered_to_guardian_at}
            <span class="badge delivered">Delivered</span>
          {:else}
            <span class="badge pending">Pending Delivery</span>
          {/if}
        </div>
        <p class="report-summary">{r.findings_summary}</p>
        {#if r.risk_notes}
          <p class="report-notes"><strong>Risks:</strong> {r.risk_notes}</p>
        {/if}
        {#if r.recommended_actions}
          <p class="report-notes"><strong>Actions:</strong> {r.recommended_actions}</p>
        {/if}
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
  .report-list { display:flex;flex-direction:column;gap:0.4rem;max-width:600px; }
  .report-card { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.65rem; }
  .report-header { display:flex;justify-content:space-between;align-items:center;margin-bottom:0.3rem; }
  .report-date-label { font-family:'Orbitron',sans-serif;font-size:0.8rem;color:#E6EDF3; }
  .badge { padding:0.1rem 0.35rem;border-radius:4px;font-size:0.65rem;font-weight:600;text-transform:uppercase; }
  .badge.delivered { background:rgba(16,185,129,0.15);color:#10B981; }
  .badge.pending { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .report-summary { font-size:0.78rem;color:#E6EDF3;margin:0 0 0.2rem; }
  .report-notes { font-size:0.72rem;color:#94A3B8;margin:0.15rem 0 0; }
</style>
