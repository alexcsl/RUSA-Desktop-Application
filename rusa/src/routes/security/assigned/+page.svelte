<!--
  /security/assigned — UC-SS-03: View Assigned Incidents (Staff)
  Shows all incident reports currently assigned to the logged-in security staff member.
  Auth guard enforced by backend (GalacticSecurityStaff | GalacticSecurityHead).
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getMyAssignedIncidents, type IncidentReportSummary } from '$lib/stores/security';

  let incidents: IncidentReportSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let search = $state('');

  let filtered = $derived(
    incidents.filter((inc) => {
      const q = search.toLowerCase();
      return (
        inc.incident_type.toLowerCase().includes(q) ||
        inc.location.toLowerCase().includes(q) ||
        inc.description.toLowerCase().includes(q) ||
        inc.severity.toLowerCase().includes(q)
      );
    })
  );

  onMount(async () => {
    try {
      incidents = await getMyAssignedIncidents();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  function severityColor(s: string): string {
    switch (s) {
      case 'critical': return '#EF4444';
      case 'high': return '#F59E0B';
      case 'medium': return '#3ABEFF';
      case 'low': return '#10B981';
      default: return '#94A3B8';
    }
  }
</script>

<h1 class="title">My Assigned Incidents</h1>
<p class="subtitle">Incident reports currently assigned to you for handling and resolution.</p>

<div class="toolbar">
  <input class="search" type="text" placeholder="Search incidents…" bind:value={search} />
  <span class="count">{filtered.length} incident{filtered.length !== 1 ? 's' : ''}</span>
</div>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if filtered.length === 0 && incidents.length === 0}
  <div class="empty">No incidents are currently assigned to you.</div>
{:else if filtered.length === 0}
  <div class="empty">No incidents match your search.</div>
{:else}
  <div class="incident-list">
    {#each filtered as inc}
      <div class="incident-card">
        <div class="card-header">
          <div class="type-row">
            <span class="incident-type">{inc.incident_type.replace(/_/g, ' ')}</span>
            <span class="severity-badge"
              style="background:rgba(0,0,0,0.2);color:{severityColor(inc.severity)};border:1px solid {severityColor(inc.severity)}40">
              {inc.severity}
            </span>
          </div>
          <span class="date">{new Date(inc.created_at).toLocaleString()}</span>
        </div>

        <div class="card-body">
          <div class="meta-row">
            <span class="meta-label">Location</span>
            <span class="meta-value">{inc.location}</span>
          </div>
          {#if inc.sector_or_base}
            <div class="meta-row">
              <span class="meta-label">Sector / Base</span>
              <span class="meta-value">{inc.sector_or_base}</span>
            </div>
          {/if}
          <div class="meta-row">
            <span class="meta-label">Source</span>
            <span class="meta-value">{inc.source.replace(/_/g, ' ')}</span>
          </div>
          {#if inc.occurred_at}
            <div class="meta-row">
              <span class="meta-label">Occurred At</span>
              <span class="meta-value">{new Date(inc.occurred_at).toLocaleString()}</span>
            </div>
          {/if}
          <div class="meta-row">
            <span class="meta-label">Reported By</span>
            <span class="meta-value">{inc.reporter_name}</span>
          </div>
        </div>

        <p class="description">{inc.description}</p>

        {#if inc.recommended_action}
          <div class="action-box">
            <span class="action-label">Recommended Action</span>
            <p class="action-text">{inc.recommended_action}</p>
          </div>
        {/if}

        <div class="card-footer">
          <span class="id-tag">ID: {inc.id.slice(0, 8)}…</span>
        </div>
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.78rem;margin:0 0 0.75rem;line-height:1.5; }

  .toolbar { display:flex;align-items:center;gap:0.75rem;margin-bottom:0.75rem; }
  .search { flex:1;max-width:380px;background:#0E1428;border:1px solid rgba(58,190,255,0.15);border-radius:6px;color:#E6EDF3;padding:0.4rem 0.6rem;font-size:0.8rem; }
  .search:focus { outline:none;border-color:#3ABEFF; }
  .count { font-size:0.75rem;color:#6B7280; }

  .muted { color:#94A3B8;font-size:0.8rem; }
  .error { color:#EF4444;font-size:0.8rem; }
  .empty { background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:1.25rem;text-align:center;color:#94A3B8;font-size:0.85rem; }

  .incident-list { display:flex;flex-direction:column;gap:0.6rem;max-width:720px; }
  .incident-card { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:0.85rem;display:flex;flex-direction:column;gap:0.5rem; }

  .card-header { display:flex;justify-content:space-between;align-items:flex-start; }
  .type-row { display:flex;align-items:center;gap:0.5rem; }
  .incident-type { font-size:0.88rem;font-weight:600;color:#E6EDF3;text-transform:capitalize; }
  .severity-badge { padding:0.1rem 0.4rem;border-radius:4px;font-size:0.65rem;font-weight:700;text-transform:uppercase; }
  .date { font-size:0.65rem;color:#6B7280;white-space:nowrap; }

  .card-body { display:flex;flex-direction:column;gap:0.2rem; }
  .meta-row { display:flex;gap:0.5rem;font-size:0.75rem; }
  .meta-label { color:#94A3B8;min-width:100px;flex-shrink:0; }
  .meta-value { color:#E6EDF3; }

  .description { font-size:0.8rem;color:#CBD5E1;line-height:1.5;margin:0;background:rgba(255,255,255,0.02);border-left:2px solid rgba(58,190,255,0.2);padding:0.4rem 0.6rem;border-radius:0 4px 4px 0; }

  .action-box { background:rgba(245,158,11,0.06);border:1px solid rgba(245,158,11,0.15);border-radius:6px;padding:0.5rem 0.65rem; }
  .action-label { font-size:0.65rem;color:#F59E0B;display:block;margin-bottom:0.15rem; }
  .action-text { font-size:0.78rem;color:#FCD34D;margin:0;line-height:1.4; }

  .card-footer { display:flex;justify-content:flex-end; }
  .id-tag { font-size:0.65rem;color:#374151;font-family:monospace; }
</style>
