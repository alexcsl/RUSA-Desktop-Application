<!--
  /directors/librarian/security-report — Security Incident Report (UC-LIB-03)
  TheLibrarian submits a security incident report to the Galactic Security team.
-->
<script lang="ts">
  import { dirSubmitSecurityReport } from '$lib/stores/directors';

  let incidentType = $state('unauthorized_access');
  let location = $state('');
  let description = $state('');
  let severity = $state('medium');
  let occurredAt = $state('');
  let recommendedAction = $state('');
  let loading = $state(false);
  let error = $state('');
  let success = $state('');

  const INCIDENT_TYPES = [
    'unauthorized_access',
    'data_leak',
    'restricted_access_violation',
    'information_misuse',
    'archive_tampering',
    'other',
  ];

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!location.trim() || !description.trim()) {
      error = 'Location and description are required.';
      return;
    }
    loading = true; error = ''; success = '';
    try {
      await dirSubmitSecurityReport({
        incident_type: incidentType,
        location: location.trim(),
        description: description.trim(),
        severity,
        occurred_at: occurredAt || undefined,
        recommended_action: recommendedAction.trim() || undefined,
      });
      success = 'Security report submitted to the Galactic Security team.';
      location = ''; description = ''; severity = 'medium';
      occurredAt = ''; recommendedAction = ''; incidentType = 'unauthorized_access';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="page">
  <h2>Security Report</h2>
  <p class="subtitle">Report a security incident related to archive access or information management.</p>

  {#if success}
    <div class="banner success">{success}</div>
  {/if}
  {#if error}
    <div class="banner error">{error}</div>
  {/if}

  <form onsubmit={handleSubmit} class="form">
    <div class="row2">
      <div class="field">
        <label for="incidentType">Incident Type</label>
        <select id="incidentType" bind:value={incidentType}>
          {#each INCIDENT_TYPES as t}
            <option value={t}>{t.replace(/_/g, ' ')}</option>
          {/each}
        </select>
      </div>
      <div class="field">
        <label for="severity">Severity</label>
        <select id="severity" bind:value={severity}>
          <option value="low">Low</option>
          <option value="medium">Medium</option>
          <option value="high">High</option>
          <option value="critical">Critical</option>
        </select>
      </div>
    </div>

    <div class="field">
      <label for="location">Location *</label>
      <input id="location" type="text" bind:value={location} placeholder="e.g. Archive Level 3, Document Vault B" required />
    </div>

    <div class="field">
      <label for="description">Description *</label>
      <textarea id="description" bind:value={description} rows="4" placeholder="Describe what occurred, what information was affected, and who was involved…" required></textarea>
    </div>

    <div class="row2">
      <div class="field">
        <label for="occurredAt">Date/Time of Incident (optional)</label>
        <input id="occurredAt" type="datetime-local" bind:value={occurredAt} />
      </div>
      <div class="field">
        <label for="recommendedAction">Recommended Action (optional)</label>
        <input id="recommendedAction" type="text" bind:value={recommendedAction} placeholder="e.g. Restrict access, Investigate personnel" />
      </div>
    </div>

    <div class="actions">
      <button type="submit" class="btn-primary" disabled={loading}>
        {loading ? 'Submitting…' : 'Submit Report'}
      </button>
    </div>
  </form>
</div>

<style>
  .page { max-width:600px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#8B5CF6;margin:0 0 0.25rem; }
  .subtitle { color:#64748B;font-size:0.8rem;margin:0 0 1.25rem; }
  .banner { padding:0.6rem 1rem;border-radius:6px;font-size:0.8rem;margin-bottom:1rem; }
  .banner.success { background:rgba(16,185,129,0.15);color:#10B981;border:1px solid rgba(16,185,129,0.3); }
  .banner.error { background:rgba(239,68,68,0.15);color:#EF4444;border:1px solid rgba(239,68,68,0.3); }
  .form { display:flex;flex-direction:column;gap:0.9rem; }
  .field { display:flex;flex-direction:column;gap:0.3rem; }
  .row2 { display:grid;grid-template-columns:1fr 1fr;gap:0.9rem; }
  label { font-size:0.75rem;color:#94A3B8;font-weight:500; }
  input,select,textarea { background:#1F2937;border:1px solid #374151;color:#E6EDF3;border-radius:6px;padding:0.45rem 0.6rem;font-size:0.82rem;font-family:inherit; }
  input:focus,select:focus,textarea:focus { outline:none;border-color:#8B5CF6; }
  textarea { resize:vertical; }
  .actions { margin-top:0.25rem; }
  .btn-primary { padding:0.55rem 1.2rem;background:rgba(139,92,246,0.15);border:1px solid #8B5CF6;color:#8B5CF6;border-radius:6px;cursor:pointer;font-size:0.85rem;font-weight:600; }
  .btn-primary:hover:not(:disabled) { background:rgba(139,92,246,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
