<!-- /security/incidents/[id]/assign — Assign Staff to Incident (UC-SH-03) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import {
    assignStaffToIncident,
    getSecurityPersonnel,
    getIncidentArchive,
    type IncidentReportSummary,
    type SecurityPersonnelItem,
  } from '$lib/stores/security';

  let incidentId = $state('');
  page.subscribe((p) => (incidentId = p.params.id ?? ''));

  let incident: IncidentReportSummary | null = $state(null);
  let staff: SecurityPersonnelItem[] = $state([]);
  let selectedStaff = $state('');
  let loading = $state(true);
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  onMount(async () => {
    try {
      const [incidents, personnel] = await Promise.all([
        getIncidentArchive(),
        getSecurityPersonnel(),
      ]);
      incident = incidents.find((i) => i.id === incidentId) ?? null;
      // Filter to only staff (not Heads or Guardian)
      staff = personnel.filter((p) => p.role_name === 'GalacticSecurityStaff');
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
    loading = false;
  });

  async function handleAssign() {
    error = ''; success = '';
    if (!selectedStaff) { error = 'Select a staff member.'; return; }

    submitting = true;
    try {
      await assignStaffToIncident({ incident_id: incidentId, user_id: selectedStaff });
      success = 'Staff assigned successfully.';
      setTimeout(() => goto('/security/incidents'), 1200);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }

  function severityColor(sev: string): string {
    switch (sev) {
      case 'critical': return '#EF4444';
      case 'high': return '#F59E0B';
      case 'medium': return '#3ABEFF';
      case 'low': return '#10B981';
      default: return '#94A3B8';
    }
  }
</script>

<h1 class="title">Assign Staff to Incident</h1>
<p class="subtitle">Select a security staff member to handle this incident.</p>

{#if loading}
  <p class="muted">Loading…</p>
{:else if !incident}
  <p class="error">Incident not found.</p>
{:else}
  <div class="incident-preview">
    <div class="preview-header">
      <span class="inc-type">{incident.incident_type}</span>
      <span class="badge" style="color:{severityColor(incident.severity)}">{incident.severity}</span>
    </div>
    <p class="preview-loc">📍 {incident.location}{incident.sector_or_base ? ` — ${incident.sector_or_base}` : ''}</p>
    <p class="preview-desc">{incident.description}</p>
    {#if incident.assigned_to_name}
      <p class="already-assigned">Currently assigned to: <strong>{incident.assigned_to_name}</strong></p>
    {/if}
  </div>

  <div class="form-card">
    <h2>Select Staff</h2>

    {#if staff.length === 0}
      <p class="muted">No security staff available.</p>
    {:else}
      <label class="field"><span class="label">Staff Member</span>
        <select class="input" bind:value={selectedStaff}>
          <option value="">— Select —</option>
          {#each staff as s}
            <option value={s.id}>{s.full_name}</option>
          {/each}
        </select>
      </label>
    {/if}

    {#if error}<p class="error">{error}</p>{/if}
    {#if success}<p class="success">{success}</p>{/if}

    <button class="btn-primary" onclick={handleAssign} disabled={submitting || staff.length === 0}>
      {submitting ? 'Assigning…' : 'Assign to Incident'}
    </button>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .incident-preview { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.85rem;max-width:600px;margin-bottom:1rem; }
  .preview-header { display:flex;align-items:center;gap:0.5rem;margin-bottom:0.3rem; }
  .inc-type { font-size:0.9rem;font-weight:600;color:#E6EDF3; }
  .badge { font-size:0.7rem;font-weight:600;text-transform:uppercase; }
  .preview-loc { font-size:0.75rem;color:#94A3B8;margin:0 0 0.3rem; }
  .preview-desc { font-size:0.8rem;color:#C9D1D9;margin:0;line-height:1.4; }
  .already-assigned { font-size:0.75rem;color:#F59E0B;margin:0.4rem 0 0; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:600px;display:flex;flex-direction:column;gap:0.6rem; }
  .form-card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
</style>
