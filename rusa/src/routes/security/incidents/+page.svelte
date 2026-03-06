<!-- /security/incidents — Incident Archive (view all incidents) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { currentUser } from '$lib/stores/auth';
  import type { SessionUser } from '$lib/stores/auth';
  import { getIncidentArchive, type IncidentReportSummary } from '$lib/stores/security';

  let incidents: IncidentReportSummary[] = $state([]);
  let loading = $state(true);
  let filterSeverity = $state('all');
  let searchTerm = $state('');
  let user: SessionUser | null = $state(null);
  const unsub = currentUser.subscribe((v) => (user = v));

  onMount(async () => {
    try {
      incidents = await getIncidentArchive();
    } catch {}
    loading = false;
  });

  let filtered = $derived(
    incidents.filter((inc) => {
      if (filterSeverity !== 'all' && inc.severity !== filterSeverity) return false;
      if (searchTerm.trim()) {
        const s = searchTerm.toLowerCase();
        return (
          inc.incident_type.toLowerCase().includes(s) ||
          inc.location.toLowerCase().includes(s) ||
          inc.description.toLowerCase().includes(s) ||
          inc.reporter_name.toLowerCase().includes(s)
        );
      }
      return true;
    })
  );

  function severityColor(sev: string): string {
    switch (sev) {
      case 'critical': return '#EF4444';
      case 'high': return '#F59E0B';
      case 'medium': return '#3ABEFF';
      case 'low': return '#10B981';
      default: return '#94A3B8';
    }
  }

  function isHead(): boolean {
    return user?.role === 'GalacticSecurityHead';
  }
</script>

<h1 class="title">Incident Archive</h1>
<p class="subtitle">All recorded security incidents. {isHead() ? 'Click an incident to assign staff.' : ''}</p>

<div class="controls">
  <input type="text" class="search-input" bind:value={searchTerm} placeholder="Search incidents…" />
  <select class="filter-select" bind:value={filterSeverity}>
    <option value="all">All Severities</option>
    <option value="critical">Critical</option>
    <option value="high">High</option>
    <option value="medium">Medium</option>
    <option value="low">Low</option>
  </select>
</div>

{#if loading}
  <p class="muted">Loading incidents…</p>
{:else if filtered.length === 0}
  <p class="muted">No incidents found.</p>
{:else}
  <div class="incident-list">
    {#each filtered as inc}
      <div class="incident-card">
        <div class="inc-header">
          <div class="inc-title-row">
            <span class="inc-type">{inc.incident_type}</span>
            <span class="badge" style="background:rgba({severityColor(inc.severity) === '#EF4444' ? '239,68,68' : severityColor(inc.severity) === '#F59E0B' ? '245,158,11' : severityColor(inc.severity) === '#3ABEFF' ? '58,190,255' : '16,185,129'},0.15);color:{severityColor(inc.severity)}">{inc.severity}</span>
          </div>
          <span class="inc-date">{new Date(inc.created_at).toLocaleString()}</span>
        </div>

        <div class="inc-meta">
          <span>📍 {inc.location}</span>
          {#if inc.sector_or_base}<span>🏛 {inc.sector_or_base}</span>{/if}
          <span>👤 {inc.reporter_name}</span>
          <span class="source-tag">{inc.source.replace(/_/g, ' ')}</span>
        </div>

        <p class="inc-desc">{inc.description}</p>

        {#if inc.recommended_action}
          <p class="inc-action"><strong>Recommended:</strong> {inc.recommended_action}</p>
        {/if}

        <div class="inc-footer">
          {#if inc.assigned_to_name}
            <span class="assigned-badge">Assigned to: {inc.assigned_to_name}</span>
          {:else if isHead()}
            <a href="/security/incidents/{inc.id}/assign" class="btn-assign">Assign Staff</a>
          {:else}
            <span class="unassigned-tag">Unassigned</span>
          {/if}
        </div>
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .controls { display:flex;gap:0.5rem;margin-bottom:1rem;max-width:600px; }
  .search-input { flex:1;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .search-input:focus { outline:none;border-color:#3ABEFF; }
  .filter-select { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .incident-list { display:flex;flex-direction:column;gap:0.5rem;max-width:750px; }
  .incident-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.85rem; }
  .inc-header { display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:0.4rem; }
  .inc-title-row { display:flex;align-items:center;gap:0.5rem; }
  .inc-type { font-size:0.9rem;font-weight:600;color:#E6EDF3; }
  .badge { padding:0.1rem 0.4rem;border-radius:4px;font-size:0.65rem;font-weight:600;text-transform:uppercase; }
  .inc-date { font-size:0.7rem;color:#94A3B8; }
  .inc-meta { display:flex;flex-wrap:wrap;gap:0.5rem;font-size:0.75rem;color:#94A3B8;margin-bottom:0.4rem; }
  .source-tag { background:rgba(139,92,246,0.15);color:#C084FC;padding:0.05rem 0.3rem;border-radius:3px;text-transform:capitalize; }
  .inc-desc { font-size:0.8rem;color:#C9D1D9;margin:0.3rem 0;line-height:1.4; }
  .inc-action { font-size:0.75rem;color:#94A3B8;margin:0.2rem 0; }
  .inc-footer { display:flex;align-items:center;gap:0.5rem;margin-top:0.4rem; }
  .assigned-badge { font-size:0.75rem;color:#10B981;background:rgba(16,185,129,0.1);padding:0.15rem 0.4rem;border-radius:4px; }
  .unassigned-tag { font-size:0.75rem;color:#F59E0B;background:rgba(245,158,11,0.1);padding:0.15rem 0.4rem;border-radius:4px; }
  .btn-assign { font-size:0.75rem;color:#3ABEFF;background:rgba(58,190,255,0.1);padding:0.2rem 0.5rem;border-radius:4px;text-decoration:none;border:1px solid rgba(58,190,255,0.2); }
  .btn-assign:hover { background:rgba(58,190,255,0.2); }
</style>
