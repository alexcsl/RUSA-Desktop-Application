<!--
  Guardian / Overseer: Incident Reports view
  Shows all incident reports categorised by source: Earth, Galactic, External.
  Access: TheGuardian, TheOverseer
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { dirGuardianGetIncidentReports, type DirIncidentReportRow } from '$lib/stores/directors';

  let reports: DirIncidentReportRow[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let activeTab: 'earth' | 'galactic' | 'external' = $state('earth');

  onMount(async () => {
    loading = true; error = '';
    try { reports = await dirGuardianGetIncidentReports(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  });

  let filtered = $derived(reports.filter((r) => {
    if (activeTab === 'external') return r.source === 'external_report';
    if (activeTab === 'earth') {
      return r.source !== 'external_report' &&
        (r.sector_or_base?.toLowerCase().includes('earth') || r.incident_type?.toLowerCase().includes('earth'));
    }
    return r.source !== 'external_report' &&
      !r.sector_or_base?.toLowerCase().includes('earth') &&
      !r.incident_type?.toLowerCase().includes('earth');
  }));

  function severityColor(s: string): string {
    const m: Record<string, string> = { low: '#22C55E', medium: '#F59E0B', high: '#EF4444', critical: '#DC2626' };
    return m[s] ?? '#94A3B8';
  }

  function formatDate(d: string | null): string {
    return d ? new Date(d).toLocaleString() : '—';
  }
</script>

<h2 class="title">Incident Reports</h2>
<p class="subtitle">All incident reports across Earth, Galactic, and External sources.</p>

<div class="tabs">
  <button class="tab" class:active={activeTab === 'earth'} onclick={() => (activeTab = 'earth')}>
    Earth Reports
  </button>
  <button class="tab" class:active={activeTab === 'galactic'} onclick={() => (activeTab = 'galactic')}>
    Galactic Reports
  </button>
  <button class="tab" class:active={activeTab === 'external'} onclick={() => (activeTab = 'external')}>
    External Reports
  </button>
</div>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if filtered.length === 0}
  <p class="muted">No {activeTab} incident reports found.</p>
{:else}
  <div class="table-wrap">
    <table class="table">
      <thead>
        <tr>
          <th>Type</th>
          <th>Location</th>
          <th>Severity</th>
          <th>Reporter</th>
          <th>Sector / Base</th>
          <th>Occurred</th>
          <th>Description</th>
        </tr>
      </thead>
      <tbody>
        {#each filtered as r}
          <tr>
            <td class="type-cell">{r.incident_type.replace(/_/g, ' ')}</td>
            <td>{r.location}</td>
            <td>
              <span class="severity-badge" style="color:{severityColor(r.severity)}">
                {r.severity}
              </span>
            </td>
            <td>{r.reporter_name}</td>
            <td>{r.sector_or_base ?? '—'}</td>
            <td class="mono">{formatDate(r.occurred_at)}</td>
            <td class="desc-cell">{r.description}</td>
          </tr>
          {#if r.recommended_action}
            <tr class="action-row">
              <td colspan="7" class="action-cell">
                Recommended action: {r.recommended_action}
              </td>
            </tr>
          {/if}
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.25rem; }
  .subtitle { color:#64748B;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .error { color:#EF4444;font-size:0.82rem; }
  .tabs { display:flex;gap:0.4rem;margin-bottom:1rem; }
  .tab { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.15);color:#94A3B8;border-radius:6px;padding:0.35rem 0.75rem;font-size:0.78rem;cursor:pointer; }
  .tab:hover { border-color:#3ABEFF;color:#E6EDF3; }
  .tab.active { border-color:#3ABEFF;color:#3ABEFF;background:rgba(58,190,255,0.1); }
  .table-wrap { overflow-x:auto; }
  .table { width:100%;border-collapse:collapse;font-size:0.78rem; }
  .table th { text-align:left;color:#64748B;font-weight:600;padding:0.5rem 0.6rem;border-bottom:1px solid rgba(58,190,255,0.1); }
  .table td { padding:0.45rem 0.6rem;color:#E6EDF3;border-bottom:1px solid rgba(255,255,255,0.04);vertical-align:top; }
  .table tr:hover td { background:rgba(58,190,255,0.03); }
  .type-cell { text-transform:capitalize;white-space:nowrap; }
  .severity-badge { font-weight:700;text-transform:capitalize; }
  .mono { font-size:0.72rem;color:#64748B; }
  .desc-cell { max-width:300px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:#94A3B8; }
  .action-row td { background:rgba(245,158,11,0.05); }
  .action-cell { color:#F59E0B;font-size:0.74rem;padding:0.3rem 0.6rem; }
</style>
