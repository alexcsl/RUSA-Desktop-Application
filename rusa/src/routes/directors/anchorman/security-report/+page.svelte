<!--
  Anchorman Security Report — submit incident report + view own submitted reports.
  Access: TheAnchorman (and any Director role via `require_auth_director!` for viewing)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    dirSubmitSecurityReport, dirGetMySecurityReports,
    type DirSubmitSecurityReportPayload, type DirIncidentReportRow,
  } from '$lib/stores/directors';

  type Tab = 'submit' | 'my-reports';
  let activeTab = $state<Tab>('submit');

  // Submit form
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

  // My Reports
  let reports: DirIncidentReportRow[] = $state([]);
  let reportsLoading = $state(false);
  let reportsError = $state('');

  async function loadReports() {
    if (reports.length > 0) return;
    reportsLoading = true;
    reportsError = '';
    try {
      reports = await dirGetMySecurityReports();
    } catch (e) {
      reportsError = String(e);
    } finally {
      reportsLoading = false;
    }
  }

  function switchTab(tab: Tab) {
    activeTab = tab;
    if (tab === 'my-reports') loadReports();
  }

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
      // Invalidate cache so My Reports reloads
      reports = [];
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  }

  function fmtDate(d: string | null) {
    if (!d) return '—';
    return new Date(d).toLocaleString();
  }
</script>

<div class="page">
  <h2>Security Incident Report</h2>

  <div class="tabs">
    <button class="tab" class:active={activeTab === 'submit'} onclick={() => switchTab('submit')}>Submit Report</button>
    <button class="tab" class:active={activeTab === 'my-reports'} onclick={() => switchTab('my-reports')}>My Reports</button>
  </div>

  {#if activeTab === 'submit'}
    <p class="subtitle">Report security incidents to the Galactic Security Head.</p>

    {#if success}<div class="banner success">{success}</div>{/if}
    {#if error}<div class="banner error">{error}</div>{/if}

    <form onsubmit={handleSubmit} class="report-form">
      <div class="field">
        <label for="incident_type">Incident Type</label>
        <select id="incident_type" bind:value={form.incident_type} required>
          <option value="">— Select type —</option>
          <option value="unauthorized_broadcast">Unauthorized Broadcast</option>
          <option value="information_leak">Information Leak</option>
          <option value="personnel_threat">Personnel Threat</option>
          <option value="impersonation">Impersonation</option>
          <option value="system_breach">System Breach</option>
          <option value="sabotage">Sabotage</option>
          <option value="civil_unrest">Civil Unrest</option>
          <option value="missing_personnel">Missing Personnel</option>
          <option value="suspicious_activity">Suspicious Activity</option>
          <option value="unauthorized_access">Unauthorized Access</option>
          <option value="violence">Violence</option>
          <option value="other">Other</option>
        </select>
      </div>

      <div class="field">
        <label for="location">Location</label>
        <input id="location" type="text" bind:value={form.location} placeholder="e.g. Command Deck, Sector 7" required />
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

  {:else}
    <p class="subtitle">Security reports you have submitted.</p>

    {#if reportsLoading}
      <p class="loading">Loading reports…</p>
    {:else if reportsError}
      <div class="banner error">{reportsError}</div>
    {:else}
      <table class="tbl">
        <thead><tr><th>Type</th><th>Location</th><th>Severity</th><th>Occurred</th><th>Submitted</th></tr></thead>
        <tbody>
          {#each reports as r}
            <tr>
              <td>{r.incident_type.replace(/_/g, ' ')}</td>
              <td>{r.location}</td>
              <td><span class="badge sev-{r.severity}">{r.severity}</span></td>
              <td>{fmtDate(r.occurred_at)}</td>
              <td>{fmtDate(r.created_at)}</td>
            </tr>
          {/each}
          {#if reports.length === 0}
            <tr><td colspan="5" class="empty">No reports submitted yet.</td></tr>
          {/if}
        </tbody>
      </table>
    {/if}
  {/if}
</div>

<style>
  .page { max-width:700px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.75rem; }
  .subtitle { color:#64748B;font-size:0.8rem;margin:0 0 1.25rem; }
  .tabs { display:flex;gap:0.4rem;margin-bottom:1rem; }
  .tab { padding:0.35rem 0.75rem;border:1px solid #374151;background:#1F2937;color:#94A3B8;border-radius:4px;cursor:pointer;font-size:0.78rem; }
  .tab.active { border-color:#3ABEFF;color:#3ABEFF;background:rgba(58,190,255,0.1); }
  .banner { padding:0.6rem 1rem;border-radius:6px;font-size:0.8rem;margin-bottom:1rem; }
  .banner.success { background:rgba(16,185,129,0.15);color:#10B981;border:1px solid rgba(16,185,129,0.3); }
  .banner.error { background:rgba(239,68,68,0.15);color:#EF4444;border:1px solid rgba(239,68,68,0.3); }
  .report-form { display:flex;flex-direction:column;gap:0.9rem; }
  .field { display:flex;flex-direction:column;gap:0.3rem; }
  .row2 { display:grid;grid-template-columns:1fr 1fr;gap:0.9rem; }
  label { font-size:0.75rem;color:#94A3B8;font-weight:500; }
  input,select,textarea { background:#1F2937;border:1px solid #374151;color:#E6EDF3;border-radius:6px;padding:0.45rem 0.6rem;font-size:0.82rem;font-family:inherit;width:100%;box-sizing:border-box; }
  input:focus,select:focus,textarea:focus { outline:none;border-color:#3ABEFF; }
  textarea { resize:vertical; }
  .btn-submit { padding:0.55rem 1.2rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.85rem;font-weight:600;align-self:flex-start; }
  .btn-submit:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-submit:disabled { opacity:0.5;cursor:not-allowed; }
  .loading { color:#94A3B8;font-size:0.85rem; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { color:#64748B;font-weight:500;text-align:left;padding:0.4rem 0.6rem;border-bottom:1px solid #1F2937; }
  .tbl td { color:#CBD5E1;padding:0.45rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .empty { color:#4B5563;font-style:italic;text-align:center; }
  .badge { padding:0.15rem 0.4rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .sev-low { background:rgba(16,185,129,0.15);color:#10B981; }
  .sev-medium { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .sev-high { background:rgba(239,68,68,0.15);color:#EF4444; }
  .sev-critical { background:rgba(239,68,68,0.3);color:#EF4444; }
</style>
