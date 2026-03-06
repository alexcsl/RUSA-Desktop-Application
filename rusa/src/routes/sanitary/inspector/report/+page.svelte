<!-- /sanitary/inspector/report — UC-IC-01: Submit Inspection Report -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    sanSubmitInspectionReport, sanGetInspectionReports,
    type InspectionReport,
  } from '$lib/stores/sanitary';

  let reports: InspectionReport[] = $state([]);
  let selected: InspectionReport | null = $state(null);
  let error = $state('');
  let success = $state('');

  // Form
  let fDate = $state(new Date().toISOString().slice(0, 10));
  let fLocation = $state('');
  let fAreaOrMachine = $state('');
  let fFindings = $state('');
  let fSeverity: string = $state('low');
  let fRecommendations = $state('');

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = '';
    try { reports = await sanGetInspectionReports(); } catch (e: unknown) { error = String(e); }
  }

  async function handleSubmit() {
    if (!fLocation.trim() || !fFindings.trim() || !fAreaOrMachine.trim()) { error = 'Location, area/machine, and findings are required.'; return; }
    error = ''; success = '';
    try {
      await sanSubmitInspectionReport({
        report_date: fDate,
        location: fLocation.trim(),
        area_or_machine: fAreaOrMachine.trim(),
        findings: fFindings.trim(),
        severity: fSeverity,
        recommendations: fRecommendations.trim() || undefined,
      });
      success = 'Report submitted and Head notified.';
      fLocation = ''; fAreaOrMachine = ''; fFindings = ''; fSeverity = 'low'; fRecommendations = '';
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }

  function severityBadge(s: string): string {
    const m: Record<string, string> = { low: 'badge-info', medium: 'badge-warn', high: 'badge-err', critical: 'badge-crit' };
    return m[s] ?? 'badge-info';
  }
</script>

<h1 class="title">Inspection Reports</h1>
<p class="subtitle">UC-IC-01 — Submit sanitary inspection findings with severity classification.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="grid">
  <div class="form-card">
    <h2>New Report</h2>
    <div class="row">
      <div class="form-group">
        <label for="date">Date</label>
        <input id="date" class="input" type="date" bind:value={fDate} />
      </div>
      <div class="form-group">
        <label for="sev">Severity</label>
        <select id="sev" class="input" bind:value={fSeverity}>
          <option value="low">Low</option>
          <option value="medium">Medium</option>
          <option value="high">High</option>
          <option value="critical">Critical</option>
        </select>
      </div>
    </div>
    <div class="form-group" style="margin-top:0.4rem;">
      <label for="loc">Location</label>
      <input id="loc" class="input" bind:value={fLocation} placeholder="e.g. Module B — Corridor 3" />
    </div>
    <div class="form-group" style="margin-top:0.4rem;">
      <label for="area">Area / Machine</label>
      <input id="area" class="input" bind:value={fAreaOrMachine} placeholder="e.g. HVAC Unit 12" />
    </div>
    <div class="form-group" style="margin-top:0.4rem;">
      <label for="find">Findings</label>
      <textarea id="find" class="input" rows="4" bind:value={fFindings} placeholder="Describe inspection findings…"></textarea>
    </div>
    <div class="form-group" style="margin-top:0.4rem;">
      <label for="rec">Recommendations (optional)</label>
      <textarea id="rec" class="input" rows="2" bind:value={fRecommendations} placeholder="Suggested actions…"></textarea>
    </div>
    <button class="btn-primary" style="margin-top:0.5rem;" onclick={handleSubmit}>Submit Report</button>
  </div>

  <div class="list-panel">
    <h2 class="panel-title">Previous Reports ({reports.length})</h2>
    {#each reports as r}
      <button class="card" class:card-selected={selected?.id === r.id} onclick={() => { selected = r; }}>
        <div class="card-row">
          <span class="card-date">{r.report_date}</span>
          <span class="badge {severityBadge(r.severity)}">{r.severity}</span>
        </div>
        <span class="card-loc">{r.location}</span>
      </button>
    {:else}
      <p class="empty">No reports submitted.</p>
    {/each}

    {#if selected}
      <div class="detail-box">
        <h3>{selected.location}</h3>
        <div class="detail-row"><span class="label">Date</span><span>{selected.report_date}</span></div>
        <div class="detail-row"><span class="label">Area / Machine</span><span>{selected.area_or_machine}</span></div>
        <div class="detail-row"><span class="label">Severity</span><span class="badge {severityBadge(selected.severity)}">{selected.severity}</span></div>
        <div class="detail-row"><span class="label">Inspector</span><span>{selected.reporter_name ?? selected.reported_by.slice(0, 8)}</span></div>
        <p class="findings">{selected.findings}</p>
        {#if selected.recommendations}<p class="findings" style="color:#94A3B8;"><strong>Recommendations:</strong> {selected.recommendations}</p>{/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .grid { display:grid;grid-template-columns:1fr 1fr;gap:0.75rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem; }
  .form-card h2 { font-size:0.9rem;color:#E6EDF3;margin:0 0 0.5rem; }
  .row { display:flex;gap:0.5rem;align-items:flex-end;flex-wrap:wrap; }
  .form-group { display:flex;flex-direction:column;gap:0.15rem;flex:1;min-width:120px; }
  .form-group label { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  select.input { cursor:pointer; }
  textarea.input { resize:vertical; }
  .list-panel { display:flex;flex-direction:column;gap:0.35rem;max-height:70vh;overflow-y:auto; }
  .panel-title { font-size:0.85rem;color:#94A3B8;margin:0 0 0.3rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.55rem;cursor:pointer;text-align:left;width:100%;color:#E6EDF3;transition:border-color .15s; }
  .card:hover { border-color:rgba(58,190,255,0.3); }
  .card-selected { border-color:#3ABEFF;background:rgba(58,190,255,0.06); }
  .card-row { display:flex;justify-content:space-between;align-items:center; }
  .card-date { font-size:0.78rem;font-weight:500; }
  .card-loc { font-size:0.72rem;color:#94A3B8; }
  .detail-box { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:0.75rem;margin-top:0.5rem; }
  .detail-box h3 { font-size:0.88rem;color:#E6EDF3;margin:0 0 0.5rem; }
  .detail-row { display:flex;justify-content:space-between;padding:0.2rem 0;border-bottom:1px solid rgba(58,190,255,0.04);font-size:0.78rem; }
  .label { color:#94A3B8; }
  .findings { font-size:0.78rem;color:#CBD5E1;margin:0.5rem 0 0;white-space:pre-wrap; }
  .badge { padding:0.12rem 0.35rem;border-radius:4px;font-size:0.6rem;text-transform:capitalize;display:inline-block; }
  .badge-info { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .badge-warn { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .badge-err { background:rgba(239,68,68,0.15);color:#EF4444; }
  .badge-crit { background:rgba(239,68,68,0.3);color:#FCA5A5;font-weight:700; }
  .btn-primary { padding:0.45rem 0.75rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
</style>
