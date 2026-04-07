<!--
  Nomad: Settler & Settlement Logs — full view across all settlements.
  Covers: Progress Reports, Building Logs, Farm Health, Anomaly Reports,
          Settler Reports (complaints), Settlement Inventory, Supply Requests.
  Access: TheNomad, TheOverseer
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    dirNomadGetProgressReports, dirNomadGetBuildingLogs, dirNomadGetFarmLogs,
    dirNomadGetAnomalyReports, dirNomadGetSettlerReports,
    dirNomadGetSettlementInventory, dirNomadGetSupplyRequests,
    type DirProgressReportRow, type DirBuildingLogRow, type DirFarmHealthRow,
    type DirAnomalyReportRow, type DirSettlerComplaintRow,
    type DirSettlementInventoryRow, type DirSupplyRequestRow,
  } from '$lib/stores/directors';

  type Tab = 'progress' | 'building' | 'farm' | 'anomaly' | 'complaints' | 'inventory' | 'supply';
  let activeTab = $state<Tab>('progress');
  let loading = $state(false);
  let error = $state('');

  let progressReports: DirProgressReportRow[] = $state([]);
  let buildingLogs: DirBuildingLogRow[] = $state([]);
  let farmLogs: DirFarmHealthRow[] = $state([]);
  let anomalyReports: DirAnomalyReportRow[] = $state([]);
  let settlerReports: DirSettlerComplaintRow[] = $state([]);
  let inventory: DirSettlementInventoryRow[] = $state([]);
  let supplyRequests: DirSupplyRequestRow[] = $state([]);

  // Detail panel — generic expanded row id
  let expandedId = $state<string | null>(null);

  onMount(async () => {
    loading = true;
    error = '';
    try {
      [progressReports, buildingLogs, farmLogs, anomalyReports, settlerReports, inventory, supplyRequests] =
        await Promise.all([
          dirNomadGetProgressReports(),
          dirNomadGetBuildingLogs(),
          dirNomadGetFarmLogs(),
          dirNomadGetAnomalyReports(),
          dirNomadGetSettlerReports(),
          dirNomadGetSettlementInventory(),
          dirNomadGetSupplyRequests(),
        ]);
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  });

  const tabs: { id: Tab; label: string }[] = [
    { id: 'progress', label: 'Progress Reports' },
    { id: 'building', label: 'Building Health' },
    { id: 'farm', label: 'Farm Health' },
    { id: 'anomaly', label: 'Anomaly Reports' },
    { id: 'complaints', label: 'Settler Reports' },
    { id: 'inventory', label: 'Inventory' },
    { id: 'supply', label: 'Supply Requests' },
  ];

  function toggleExpand(id: string) {
    expandedId = expandedId === id ? null : id;
  }

  function fmtDate(dt: string) { return new Date(dt).toLocaleDateString(); }
  function ragColor(rag: string | null) {
    if (rag === 'green') return 'rag-green';
    if (rag === 'amber') return 'rag-amber';
    if (rag === 'red') return 'rag-red';
    return '';
  }
  function sevColor(sev: string) {
    if (sev === 'low') return 'sev-low';
    if (sev === 'medium') return 'sev-med';
    if (sev === 'high' || sev === 'critical') return 'sev-high';
    return '';
  }
</script>

<div class="page">
  <h2>Settler &amp; Settlement Logs</h2>
  <p class="subtitle">Full view across all settlements. Click any row to expand details.</p>

  {#if loading}
    <div class="loading">Loading…</div>
  {:else if error}
    <div class="banner error">{error}</div>
  {:else}
    <div class="tabs">
      {#each tabs as t}
        <button class="tab" class:active={activeTab === t.id} onclick={() => { activeTab = t.id; expandedId = null; }}>
          {t.label}
        </button>
      {/each}
    </div>

    {#if activeTab === 'progress'}
      <table class="tbl">
        <thead><tr><th>Settlement</th><th>Task</th><th>By</th><th>Week</th><th>RAG</th><th>Date</th></tr></thead>
        <tbody>
          {#each progressReports as r}
            <tr class="clickable" onclick={() => toggleExpand(r.id)}>
              <td>{r.settlement_name ?? '—'}</td><td>{r.task_title}</td><td>{r.submitted_by_name}</td>
              <td>{r.week ?? '—'}</td>
              <td>{#if r.rag_status}<span class="badge {ragColor(r.rag_status)}">{r.rag_status}</span>{:else}—{/if}</td>
              <td>{fmtDate(r.created_at)}</td>
            </tr>
            {#if expandedId === r.id}
              <tr class="detail-row">
                <td colspan="6">
                  <div class="detail-box">
                    <div class="detail-field"><span class="dk">Progress Made</span><span>{r.progress_made}</span></div>
                    {#if r.materials_equipment}<div class="detail-field"><span class="dk">Materials / Equipment</span><span>{r.materials_equipment}</span></div>{/if}
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
          {#if progressReports.length === 0}<tr><td colspan="6" class="empty">No records.</td></tr>{/if}
        </tbody>
      </table>

    {:else if activeTab === 'building'}
      <table class="tbl">
        <thead><tr><th>Settlement</th><th>Building</th><th>Checked By</th><th>Date</th><th>Status</th></tr></thead>
        <tbody>
          {#each buildingLogs as r}
            <tr class="clickable" onclick={() => toggleExpand(r.id)}>
              <td>{r.settlement_name ?? '—'}</td><td>{r.building_name}</td><td>{r.submitted_by_name}</td>
              <td>{fmtDate(r.check_date)}</td><td>{r.status}</td>
            </tr>
            {#if expandedId === r.id}
              <tr class="detail-row">
                <td colspan="5">
                  <div class="detail-box">
                    {#if r.findings}<div class="detail-field"><span class="dk">Findings</span><span>{r.findings}</span></div>{/if}
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
          {#if buildingLogs.length === 0}<tr><td colspan="5" class="empty">No records.</td></tr>{/if}
        </tbody>
      </table>

    {:else if activeTab === 'farm'}
      <table class="tbl">
        <thead><tr><th>Settlement</th><th>Type</th><th>Subject</th><th>By</th><th>Date</th><th>Condition</th></tr></thead>
        <tbody>
          {#each farmLogs as r}
            <tr class="clickable" onclick={() => toggleExpand(r.id)}>
              <td>{r.settlement_name ?? '—'}</td><td>{r.subject_type}</td><td>{r.subject_name}</td>
              <td>{r.submitted_by_name}</td><td>{fmtDate(r.log_date)}</td><td>{r.condition}</td>
            </tr>
            {#if expandedId === r.id}
              <tr class="detail-row">
                <td colspan="6">
                  <div class="detail-box">
                    {#if r.treatment}<div class="detail-field"><span class="dk">Treatment</span><span>{r.treatment}</span></div>{/if}
                    {#if r.notes}<div class="detail-field"><span class="dk">Notes</span><span>{r.notes}</span></div>{/if}
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
          {#if farmLogs.length === 0}<tr><td colspan="6" class="empty">No records.</td></tr>{/if}
        </tbody>
      </table>

    {:else if activeTab === 'anomaly'}
      <table class="tbl">
        <thead><tr><th>Settlement</th><th>By</th><th>Location</th><th>Severity</th><th>Status</th><th>Date</th></tr></thead>
        <tbody>
          {#each anomalyReports as r}
            <tr class="clickable" onclick={() => toggleExpand(r.id)}>
              <td>{r.settlement_name ?? '—'}</td><td>{r.submitted_by_name}</td>
              <td>{r.location_in_settlement ?? '—'}</td>
              <td><span class="badge {sevColor(r.severity)}">{r.severity}</span></td>
              <td>{r.status}</td><td>{fmtDate(r.created_at)}</td>
            </tr>
            {#if expandedId === r.id}
              <tr class="detail-row">
                <td colspan="6">
                  <div class="detail-box">
                    <div class="detail-field"><span class="dk">Description</span><span>{r.description}</span></div>
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
          {#if anomalyReports.length === 0}<tr><td colspan="6" class="empty">No records.</td></tr>{/if}
        </tbody>
      </table>

    {:else if activeTab === 'complaints'}
      <table class="tbl">
        <thead><tr><th>Settlement</th><th>Reported By</th><th>Subject</th><th>Status</th><th>Date</th></tr></thead>
        <tbody>
          {#each settlerReports as r}
            <tr class="clickable" onclick={() => toggleExpand(r.id)}>
              <td>{r.settlement_name ?? '—'}</td><td>{r.reported_by_name}</td>
              <td>{r.subject_name ?? '—'}</td><td>{r.status}</td>
              <td>{fmtDate(r.created_at)}</td>
            </tr>
            {#if expandedId === r.id}
              <tr class="detail-row">
                <td colspan="5">
                  <div class="detail-box">
                    <div class="detail-field"><span class="dk">Incident</span><span>{r.incident_description}</span></div>
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
          {#if settlerReports.length === 0}<tr><td colspan="5" class="empty">No records.</td></tr>{/if}
        </tbody>
      </table>

    {:else if activeTab === 'inventory'}
      <table class="tbl">
        <thead><tr><th>Settlement</th><th>Item</th><th>Category</th><th>Qty</th><th>Unit</th><th>Min</th><th>Updated</th></tr></thead>
        <tbody>
          {#each inventory as r}
            <tr class:low-stock={r.quantity <= r.min_threshold}>
              <td>{r.settlement_name ?? '—'}</td><td>{r.item_name}</td><td>{r.category ?? '—'}</td>
              <td>{r.quantity}</td><td>{r.unit ?? '—'}</td><td>{r.min_threshold}</td>
              <td>{fmtDate(r.updated_at)}</td>
            </tr>
          {/each}
          {#if inventory.length === 0}<tr><td colspan="7" class="empty">No records.</td></tr>{/if}
        </tbody>
      </table>

    {:else if activeTab === 'supply'}
      <table class="tbl">
        <thead><tr><th>Settlement</th><th>Submitted By</th><th>Status</th><th>Date</th></tr></thead>
        <tbody>
          {#each supplyRequests as r}
            <tr class="clickable" onclick={() => toggleExpand(r.id)}>
              <td>{r.settlement_name ?? '—'}</td><td>{r.submitted_by_name}</td>
              <td>{r.status}</td>
              <td>{fmtDate(r.created_at)}</td>
            </tr>
            {#if expandedId === r.id}
              <tr class="detail-row">
                <td colspan="4">
                  <div class="detail-box">
                    {#if r.justification}<div class="detail-field"><span class="dk">Justification</span><span>{r.justification}</span></div>{/if}
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
          {#if supplyRequests.length === 0}<tr><td colspan="4" class="empty">No records.</td></tr>{/if}
        </tbody>
      </table>
    {/if}
  {/if}
</div>

<style>
  .page { max-width:980px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.25rem; }
  .subtitle { color:#64748B;font-size:0.8rem;margin:0 0 1.25rem; }
  .loading { color:#94A3B8;font-size:0.85rem; }
  .banner { padding:0.6rem 1rem;border-radius:6px;font-size:0.8rem;margin-bottom:1rem; }
  .banner.error { background:rgba(239,68,68,0.15);color:#EF4444;border:1px solid rgba(239,68,68,0.3); }
  .tabs { display:flex;flex-wrap:wrap;gap:0.4rem;margin-bottom:1rem; }
  .tab { padding:0.3rem 0.65rem;border:1px solid #374151;background:#1F2937;color:#94A3B8;border-radius:4px;cursor:pointer;font-size:0.76rem; }
  .tab.active { border-color:#3ABEFF;color:#3ABEFF;background:rgba(58,190,255,0.1); }
  .tbl { width:100%;border-collapse:collapse;font-size:0.78rem; }
  .tbl th { color:#64748B;font-weight:500;text-align:left;padding:0.4rem 0.5rem;border-bottom:1px solid #1F2937; }
  .tbl td { color:#CBD5E1;padding:0.4rem 0.5rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .clickable { cursor:pointer; }
  .clickable:hover td { background:rgba(58,190,255,0.04); }
  .detail-row td { padding:0;background:rgba(58,190,255,0.03); }
  .detail-box { padding:0.6rem 1rem;display:flex;flex-direction:column;gap:0.4rem; }
  .detail-field { display:flex;gap:0.5rem;font-size:0.78rem; }
  .dk { color:#64748B;min-width:130px;flex-shrink:0; }
  .empty { color:#4B5563;font-style:italic;text-align:center; }
  .low-stock td { color:#F59E0B; }
  .badge { padding:0.12rem 0.35rem;border-radius:4px;font-size:0.68rem;font-weight:600; }
  .rag-green { background:rgba(16,185,129,0.15);color:#10B981; }
  .rag-amber { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .rag-red { background:rgba(239,68,68,0.15);color:#EF4444; }
  .sev-low { background:rgba(16,185,129,0.15);color:#10B981; }
  .sev-med { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .sev-high { background:rgba(239,68,68,0.15);color:#EF4444; }
</style>
