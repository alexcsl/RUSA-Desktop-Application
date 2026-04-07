<!--
  /directors/guardian/security-report — TheGuardian submits a security report.
  Uses dir_submit_security_report (backend auth guard includes TheGuardian).
  Theme: red #EF4444 (Guardian = security enforcement).
-->
<script lang="ts">
  import { dirSubmitSecurityReport, type DirSubmitSecurityReportPayload } from '$lib/stores/directors';

  const INCIDENT_TYPES = [
    { value: 'security_breach', label: 'Security Breach' },
    { value: 'unauthorized_access', label: 'Unauthorized Access' },
    { value: 'physical_threat', label: 'Physical Threat' },
    { value: 'personnel_misconduct', label: 'Personnel Misconduct' },
    { value: 'broadcast_misuse', label: 'Broadcast Misuse' },
    { value: 'missing_person', label: 'Missing Person' },
    { value: 'other', label: 'Other' },
  ];

  let incidentType = $state('security_breach');
  let title = $state('');
  let description = $state('');
  let location = $state('');
  let submitting = $state(false);
  let result: string | null = $state(null);
  let error: string | null = $state(null);

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!title.trim() || !description.trim()) return;
    submitting = true;
    error = null;
    result = null;
    try {
      const payload: DirSubmitSecurityReportPayload = {
        incident_type: incidentType,
        title: title.trim(),
        description: description.trim(),
        location: location.trim() || undefined,
      };
      const id = await dirSubmitSecurityReport(payload);
      result = id;
      title = '';
      description = '';
      location = '';
      incidentType = 'security_breach';
    } catch (e: unknown) {
      error = String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<div class="page">
  <h2>Security Report</h2>
  <p class="subtitle">Submit a security incident report to the Galactic Security Head.</p>

  {#if result}
    <div class="success">Report submitted. Reference ID: <code>{result}</code></div>
  {/if}
  {#if error}
    <div class="error">{error}</div>
  {/if}

  <form class="report-form" onsubmit={handleSubmit}>
    <label>
      Incident Type
      <select bind:value={incidentType}>
        {#each INCIDENT_TYPES as t}
          <option value={t.value}>{t.label}</option>
        {/each}
      </select>
    </label>

    <label>
      Title <span class="req">*</span>
      <input type="text" bind:value={title} placeholder="Brief incident title" required />
    </label>

    <label>
      Location (optional)
      <input type="text" bind:value={location} placeholder="Base, sector, or area" />
    </label>

    <label>
      Description <span class="req">*</span>
      <textarea bind:value={description} rows={6} placeholder="Describe the incident in detail..." required></textarea>
    </label>

    <button type="submit" class="btn-submit" disabled={submitting}>
      {submitting ? 'Submitting…' : 'Submit Report'}
    </button>
  </form>
</div>

<style>
  .page { max-width:600px;margin:0 auto;padding:2rem; }
  h2 { color:#EF4444;font-family:'Orbitron',sans-serif;margin-bottom:0.25rem; }
  .subtitle { color:#94A3B8;font-size:0.85rem;margin-bottom:1.5rem; }
  .success { background:rgba(16,185,129,0.1);border:1px solid #10B981;color:#10B981;padding:0.75rem 1rem;border-radius:6px;margin-bottom:1rem;font-size:0.85rem; }
  .error { background:rgba(239,68,68,0.1);border:1px solid #EF4444;color:#EF4444;padding:0.75rem 1rem;border-radius:6px;margin-bottom:1rem;font-size:0.85rem; }
  .report-form { display:flex;flex-direction:column;gap:1rem; }
  label { display:flex;flex-direction:column;gap:0.3rem;font-size:0.85rem;color:#94A3B8; }
  input,select,textarea { background:#1F2937;border:1px solid #374151;border-radius:6px;color:#E6EDF3;padding:0.5rem 0.75rem;font-size:0.9rem; }
  input:focus,select:focus,textarea:focus { outline:none;border-color:#EF4444; }
  textarea { resize:vertical;font-family:inherit; }
  .req { color:#EF4444; }
  .btn-submit { background:#EF4444;color:#fff;border:none;border-radius:6px;padding:0.65rem 1.5rem;font-weight:600;cursor:pointer;font-size:0.9rem;align-self:flex-start; }
  .btn-submit:hover:not(:disabled) { background:#DC2626; }
  .btn-submit:disabled { opacity:0.5;cursor:not-allowed; }
  code { font-family:monospace;font-size:0.8rem;background:rgba(255,255,255,0.05);padding:0.1rem 0.4rem;border-radius:3px; }
</style>
