<!-- /astronauts/missions/[id] — UC-AS-02 + UC-AS-06: Mission Detail & Team -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import {
    getMissionDetail, getEvidenceUrls,
    type MissionDetailBundle, type EvidenceFileUrl,
  } from '$lib/stores/astronauts';

  let missionId = $state('');
  const unsub = page.subscribe((p) => (missionId = p.params.id));
  import { onDestroy } from 'svelte';
  onDestroy(unsub);

  let bundle: MissionDetailBundle | null = $state(null);
  let evidenceUrls: EvidenceFileUrl[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let activeTab: 'overview' | 'team' | 'reports' | 'completion' = $state('overview');

  onMount(async () => {
    try {
      bundle = await getMissionDetail(missionId);
      if (bundle.completion_request?.evidence_storage_paths?.length) {
        evidenceUrls = await getEvidenceUrls(bundle.completion_request.id);
      }
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleDateString('en-GB', { day: '2-digit', month: 'short', year: 'numeric' });
  }

  function ragColor(rag: string | null): string {
    switch (rag) {
      case 'red': return '#EF4444';
      case 'amber': return '#F59E0B';
      case 'green': return '#10B981';
      default: return '#94A3B8';
    }
  }

  function statusLabel(s: string): string {
    return s.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function dangerClass(level: string | null): string {
    switch (level) {
      case 'critical': return 'danger-critical';
      case 'high': return 'danger-high';
      case 'medium': return 'danger-medium';
      default: return 'danger-low';
    }
  }
</script>

{#if loading}
  <p class="loading">Loading mission…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if bundle}
  {@const m = bundle.mission}
  <div class="header-row">
    <div>
      <h1 class="title">{m.title}</h1>
      <div class="meta">
        <span class="badge type-badge">{m.mission_type}</span>
        <span class="badge {dangerClass(m.danger_level)}">{m.danger_level ?? 'n/a'}</span>
        <span class="badge status-badge">{statusLabel(m.status)}</span>
        <span class="meta-text">📍 {m.location}</span>
        <span class="meta-text">Assigned by {m.assigned_by_name}</span>
        <span class="meta-text">{formatDate(m.created_at)}</span>
      </div>
    </div>
    <div class="action-btns">
      {#if m.status === 'active' || m.status === 'completion_requested'}
        <a class="btn-primary" href="/astronauts/missions/{m.id}/status-report">Submit Status Report</a>
      {/if}
      {#if m.status === 'active'}
        <a class="btn-secondary" href="/astronauts/missions/{m.id}/completion">Request Completion</a>
      {/if}
      <a class="btn-back" href="/astronauts/missions">← Back</a>
    </div>
  </div>

  <!-- Tabs -->
  <div class="tabs">
    <button class:tab-active={activeTab === 'overview'} onclick={() => (activeTab = 'overview')}>Overview</button>
    <button class:tab-active={activeTab === 'team'} onclick={() => (activeTab = 'team')}>Team ({bundle.team.length})</button>
    <button class:tab-active={activeTab === 'reports'} onclick={() => (activeTab = 'reports')}>Reports ({bundle.status_reports.length})</button>
    <button class:tab-active={activeTab === 'completion'} onclick={() => (activeTab = 'completion')}>Completion</button>
  </div>

  <!-- Tab content -->
  {#if activeTab === 'overview'}
    <div class="card">
      <h2>Mission Objective</h2>
      <p class="body-text">{m.mission_objective ?? 'No objective documented.'}</p>
    </div>
    <div class="card-row">
      <div class="card half">
        <h2>Procedures</h2>
        <p class="body-text">{m.procedures ?? 'None specified.'}</p>
      </div>
      <div class="card half">
        <h2>Known Dangers</h2>
        <p class="body-text">{m.known_dangers ?? 'None documented.'}</p>
      </div>
    </div>
  {:else if activeTab === 'team'}
    <div class="card">
      <h2>Mission Team</h2>
      <p class="subtitle">Fellow astronauts assigned to this mission and their completed mission counters.</p>
      {#if bundle.team.length === 0}
        <p class="empty">No team data available.</p>
      {:else}
        <table>
          <thead>
            <tr>
              <th>Astronaut</th>
              <th>Interstellar</th>
              <th>Terrain</th>
              <th>Total</th>
            </tr>
          </thead>
          <tbody>
            {#each bundle.team as t}
              <tr>
                <td>{t.full_name}</td>
                <td class="num">{t.interstellar_count}</td>
                <td class="num">{t.terrain_count}</td>
                <td class="num total">{t.interstellar_count + t.terrain_count}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  {:else if activeTab === 'reports'}
    <div class="card">
      <h2>Status Reports</h2>
      {#if bundle.status_reports.length === 0}
        <p class="empty">No reports submitted yet.</p>
      {:else}
        {#each bundle.status_reports as r}
          <div class="report-item">
            <div class="report-header">
              <span class="rag-dot" style="background:{ragColor(r.rag_status)}"></span>
              <span class="report-date">{formatDate(r.report_date)}</span>
              <span class="report-author">by {r.submitter_name}</span>
              {#if r.month_tracker}<span class="report-month">{r.month_tracker}</span>{/if}
            </div>
            <p class="report-body">{r.current_status}</p>
            {#if r.issues_blockers}
              <p class="report-issues"><strong>Issues:</strong> {r.issues_blockers}</p>
            {/if}
            {#if r.collected_samples_last_month}
              <p class="report-detail"><strong>Samples:</strong> {r.collected_samples_last_month}</p>
            {/if}
            {#if r.progress_last_month}
              <p class="report-detail"><strong>Progress:</strong> {r.progress_last_month}</p>
            {/if}
            {#if r.plans_next_month}
              <p class="report-detail"><strong>Plans:</strong> {r.plans_next_month}</p>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
  {:else if activeTab === 'completion'}
    <div class="card">
      <h2>Completion Request</h2>
      {#if !bundle.completion_request}
        <p class="empty">No completion request submitted.</p>
        {#if m.status === 'active'}
          <a class="btn-primary" href="/astronauts/missions/{m.id}/completion">Submit Completion Request</a>
        {/if}
      {:else}
        {@const cr = bundle.completion_request}
        <div class="completion-status">
          <span class="badge status-{cr.status.replace('pending_', 'pending-')}">{statusLabel(cr.status)}</span>
          <span class="meta-text">Submitted {formatDate(cr.created_at)}</span>
        </div>
        <div class="completion-body">
          <h3>Findings Summary</h3>
          <p class="body-text">{cr.findings_summary}</p>
        </div>
        {#if cr.wanderer_note}
          <div class="note-box wanderer-note">
            <strong>Wanderer Note:</strong> {cr.wanderer_note}
          </div>
        {/if}
        {#if cr.taskmaster_note}
          <div class="note-box taskmaster-note">
            <strong>Taskmaster Note:</strong> {cr.taskmaster_note}
          </div>
        {/if}
        {#if evidenceUrls.length > 0}
          <div class="evidence-section">
            <h3>Evidence Files</h3>
            {#each evidenceUrls as ev}
              <a class="evidence-link" href={ev.signed_url} target="_blank" rel="noopener">
                📎 {ev.filename}
              </a>
            {/each}
          </div>
        {/if}
      {/if}
    </div>
  {/if}
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0; }
  .loading { color:#94A3B8; }
  .error { color:#EF4444; }
  .empty { color:#475569;font-size:0.85rem; }

  .header-row { display:flex;justify-content:space-between;align-items:flex-start;gap:1rem;margin-bottom:1rem;flex-wrap:wrap; }
  .meta { display:flex;flex-wrap:wrap;gap:0.4rem;align-items:center;margin-top:0.4rem; }
  .meta-text { color:#94A3B8;font-size:0.75rem; }
  .action-btns { display:flex;gap:0.5rem;flex-wrap:wrap;align-items:flex-start; }

  .badge { display:inline-block;padding:0.12rem 0.4rem;border-radius:4px;font-size:0.65rem;font-weight:600;text-transform:capitalize; }
  .type-badge { background:rgba(139,92,246,0.15);color:#C084FC; }
  .status-badge { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .danger-low { background:rgba(16,185,129,0.15);color:#10B981; }
  .danger-medium { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .danger-high { background:rgba(249,115,22,0.15);color:#F97316; }
  .danger-critical { background:rgba(239,68,68,0.2);color:#EF4444; }

  .btn-primary { display:inline-block;padding:0.45rem 1rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;text-decoration:none; }
  .btn-secondary { display:inline-block;padding:0.45rem 1rem;background:rgba(139,92,246,0.15);border:1px solid #8B5CF6;border-radius:6px;color:#C084FC;font-weight:600;cursor:pointer;font-size:0.8rem;text-decoration:none; }
  .btn-secondary:hover { background:rgba(139,92,246,0.3); }
  .btn-back { color:#94A3B8;font-size:0.75rem;text-decoration:none; }
  .btn-back:hover { color:#E6EDF3; }

  /* Tabs */
  .tabs { display:flex;gap:0;border-bottom:1px solid rgba(58,190,255,0.1);margin-bottom:1rem; }
  .tabs button { background:none;border:none;border-bottom:2px solid transparent;color:#94A3B8;padding:0.5rem 1rem;cursor:pointer;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .tabs button:hover { color:#E6EDF3; }
  .tabs .tab-active { color:#3ABEFF;border-bottom-color:#3ABEFF; }

  /* Cards */
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;margin-bottom:1rem; }
  .card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0 0 0.5rem; }
  .card h3 { font-size:0.8rem;color:#E6EDF3;margin:0.75rem 0 0.3rem; }
  .card-row { display:flex;gap:1rem;flex-wrap:wrap; }
  .half { flex:1;min-width:280px; }
  .body-text { color:#CBD5E1;font-size:0.85rem;line-height:1.5;margin:0;white-space:pre-wrap; }

  /* Team table */
  table { width:100%;border-collapse:collapse;font-size:0.8rem;margin-top:0.5rem; }
  th { text-align:left;color:#94A3B8;font-weight:600;font-size:0.7rem;padding:0.4rem 0.6rem;border-bottom:1px solid rgba(58,190,255,0.1); }
  td { padding:0.4rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.03); }
  .num { text-align:center;color:#3ABEFF; }
  .total { font-weight:700;color:#C084FC; }

  /* Reports */
  .report-item { padding:0.6rem;border-bottom:1px solid rgba(255,255,255,0.04);margin-bottom:0.3rem; }
  .report-header { display:flex;gap:0.5rem;align-items:center;margin-bottom:0.3rem; }
  .rag-dot { width:10px;height:10px;border-radius:50%;flex-shrink:0; }
  .report-date { color:#E6EDF3;font-weight:600;font-size:0.8rem; }
  .report-author { color:#94A3B8;font-size:0.75rem; }
  .report-month { color:#8B5CF6;font-size:0.7rem; }
  .report-body { color:#CBD5E1;font-size:0.8rem;margin:0.15rem 0; }
  .report-issues { color:#F59E0B;font-size:0.78rem;margin:0.15rem 0; }
  .report-detail { color:#94A3B8;font-size:0.78rem;margin:0.1rem 0; }

  /* Completion */
  .completion-status { display:flex;gap:0.5rem;align-items:center;margin-bottom:0.75rem; }
  .completion-body { margin-bottom:0.75rem; }
  .note-box { padding:0.6rem;border-radius:6px;font-size:0.8rem;margin-bottom:0.5rem; }
  .wanderer-note { background:rgba(139,92,246,0.1);border-left:3px solid #8B5CF6;color:#CBD5E1; }
  .taskmaster-note { background:rgba(58,190,255,0.1);border-left:3px solid #3ABEFF;color:#CBD5E1; }
  .evidence-section { margin-top:0.75rem; }
  .evidence-link { display:block;color:#3ABEFF;font-size:0.8rem;text-decoration:none;padding:0.2rem 0; }
  .evidence-link:hover { text-decoration:underline; }

  /* Status badges by completion status */
  :global(.status-pending-wanderer) { background:rgba(245,158,11,0.15);color:#F59E0B; }
  :global(.status-pending-taskmaster) { background:rgba(139,92,246,0.15);color:#C084FC; }
  :global(.status-approved) { background:rgba(16,185,129,0.15);color:#10B981; }
  :global(.status-rejected) { background:rgba(239,68,68,0.15);color:#EF4444; }
</style>
