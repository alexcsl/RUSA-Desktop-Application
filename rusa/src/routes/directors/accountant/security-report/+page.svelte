<!--
  Accountant Security Report — submit incident report to GalacticSecurityHead.
  Access: TheAccountant
-->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import { currentUser } from '$lib/stores/auth';
  import { dirSubmitSecurityReport, type DirSubmitSecurityReportPayload } from '$lib/stores/directors';

  let user = $state<{ full_name: string; role: string } | null>(null);
  const unsubUser = currentUser.subscribe((v) => (user = v));
  onDestroy(() => unsubUser());

  let loading = $state(false);
  let success = $state('');
  let error = $state('');

  let form = $state<DirSubmitSecurityReportPayload>({
    incident_type: '',
    location: '',
    severity: 'medium',
    description: '',
    occurred_at: '',
    recommended_action: '',
  });

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!form.incident_type || !form.location || !form.description) {
      error = 'Incident type, location, and description are required.';
      return;
    }
    loading = true;
    error = '';
    success = '';
    try {
      const payload: DirSubmitSecurityReportPayload = {
        incident_type: form.incident_type,
        location: form.location,
        severity: form.severity,
        description: form.description,
        occurred_at: form.occurred_at || undefined,
        recommended_action: form.recommended_action || undefined,
      };
      await dirSubmitSecurityReport(payload);
      success = 'Security report submitted successfully.';
      form = { incident_type: '', location: '', severity: 'medium', description: '', occurred_at: '', recommended_action: '' };
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }
</script>

<div class="page">
  <h2>Security Incident Report</h2>
  <p class="subtitle">Report security incidents to the Galactic Security Head.</p>

  {#if success}
    <div class="banner success">{success}</div>
  {/if}
  {#if error}
    <div class="banner error">{error}</div>
  {/if}

  <form onsubmit={handleSubmit} class="report-form">
    <div class="field">
      <label for="incident_type">Incident Type</label>
      <select id="incident_type" bind:value={form.incident_type} required>
        <option value="">— Select type —</option>
        <option value="financial_fraud">Financial Fraud</option>
        <option value="budget_misuse">Budget Misuse</option>
        <option value="foul_play">Foul Play / Irregularity</option>
        <option value="unauthorized_expenditure">Unauthorized Expenditure</option>
        <option value="embezzlement">Embezzlement</option>
        <option value="personnel_threat">Personnel Threat</option>
        <option value="unauthorized_access">Unauthorized Access</option>
        <option value="suspicious_activity">Suspicious Activity</option>
        <option value="sabotage">Sabotage</option>
        <option value="other">Other</option>
      </select>
    </div>

    <div class="field">
      <label for="location">Location</label>
      <input id="location" type="text" bind:value={form.location} placeholder="e.g. Finance Division, Headquarters" required />
    </div>

    <div class="row2">
      <div class="field">
        <label for="severity">Severity</label>
        <select id="severity" bind:value={form.severity}>
          <option value="low">Low</option>
          <option value="medium">Medium</option>
          <option value="high">High</option>
          <option value="critical">Critical</option>
        </select>
      </div>
      <div class="field">
        <label for="occurred_at">Occurred At (optional)</label>
        <input id="occurred_at" type="datetime-local" bind:value={form.occurred_at} />
      </div>
    </div>

    <div class="field">
      <label for="description">Description</label>
      <textarea id="description" bind:value={form.description} rows="5" placeholder="Describe the incident in detail..." required></textarea>
    </div>

    <div class="field">
      <label for="recommended_action">Recommended Action (optional)</label>
      <textarea id="recommended_action" bind:value={form.recommended_action} rows="3" placeholder="Suggested response or action..."></textarea>
    </div>

    <button type="submit" class="btn-submit" disabled={loading}>
      {loading ? 'Submitting…' : 'Submit Report'}
    </button>
  </form>
</div>

<style>
  .page { max-width:640px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.25rem; }
  .subtitle { color:#64748B;font-size:0.8rem;margin:0 0 1.25rem; }
  .banner { padding:0.6rem 1rem;border-radius:6px;font-size:0.8rem;margin-bottom:1rem; }
  .banner.success { background:rgba(16,185,129,0.15);color:#10B981;border:1px solid rgba(16,185,129,0.3); }
  .banner.error { background:rgba(239,68,68,0.15);color:#EF4444;border:1px solid rgba(239,68,68,0.3); }
  .report-form { display:flex;flex-direction:column;gap:0.9rem; }
  .field { display:flex;flex-direction:column;gap:0.3rem; }
  .row2 { display:grid;grid-template-columns:1fr 1fr;gap:0.9rem; }
  label { font-size:0.75rem;color:#94A3B8;font-weight:500; }
  input,select,textarea { background:#1F2937;border:1px solid #374151;color:#E6EDF3;border-radius:6px;padding:0.45rem 0.6rem;font-size:0.82rem;font-family:inherit; }
  input:focus,select:focus,textarea:focus { outline:none;border-color:#3ABEFF; }
  textarea { resize:vertical; }
  .btn-submit { padding:0.55rem 1.2rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.85rem;font-weight:600;align-self:flex-start; }
  .btn-submit:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-submit:disabled { opacity:0.5;cursor:not-allowed; }
</style>
