<!--
  Nomad: Sanitary Department Overview — view staff, shifts, inventory, inspection
  reports, disposal docs, wastewater docs, and budget/expenditure reports.
  Access: TheNomad
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import {
    sanGetStaffRoster, sanGetAllShifts, sanGetInventory,
    sanGetInspectionReports, sanGetDisposalDocs, sanGetWastewaterDocs,
    sanGetBudgetRequests, sanGetExpenditureReports, sanGetDivisions,
    type StaffRosterEntry, type SanitaryShift, type SanitaryInventoryItem,
    type InspectionReport, type DisposalDoc, type WastewaterDoc,
    type BudgetRequestSummary, type ExpenditureReportSummary, type DivisionRow,
  } from '$lib/stores/sanitary';

  type Tab = 'staff' | 'shifts' | 'inventory' | 'inspection' | 'disposal' | 'wastewater' | 'budget' | 'divisions';

  let activeTab = $state<Tab>('staff');
  let loading = $state(false);
  let error = $state('');

  let staff: StaffRosterEntry[] = $state([]);
  let shifts: SanitaryShift[] = $state([]);
  let inventory: SanitaryInventoryItem[] = $state([]);
  let inspections: InspectionReport[] = $state([]);
  let disposal: DisposalDoc[] = $state([]);
  let wastewater: WastewaterDoc[] = $state([]);
  let budgetReqs: BudgetRequestSummary[] = $state([]);
  let expenditures: ExpenditureReportSummary[] = $state([]);
  let divisions: DivisionRow[] = $state([]);

  let expandedId = $state<string | null>(null);

  onMount(async () => {
    loading = true;
    error = '';
    try {
      [staff, shifts, inventory, inspections, disposal, wastewater, budgetReqs, expenditures, divisions] =
        await Promise.all([
          sanGetStaffRoster(),
          sanGetAllShifts(),
          sanGetInventory(),
          sanGetInspectionReports(),
          sanGetDisposalDocs(),
          sanGetWastewaterDocs(),
          sanGetBudgetRequests(),
          sanGetExpenditureReports(),
          sanGetDivisions(),
        ]);
    } catch (err) {
      error = String(err);
    } finally {
      loading = false;
    }
  });

  const tabs: { id: Tab; label: string }[] = [
    { id: 'staff', label: 'Staff' },
    { id: 'shifts', label: 'Shifts' },
    { id: 'divisions', label: 'Divisions & Quota' },
    { id: 'inventory', label: 'Inventory' },
    { id: 'inspection', label: 'Inspection Reports' },
    { id: 'disposal', label: 'Disposal Docs' },
    { id: 'wastewater', label: 'Wastewater Docs' },
    { id: 'budget', label: 'Budget' },
  ];

  // Selected full-detail doc
  let selectedDisposal: DisposalDoc | null = $state(null);
  let selectedWastewater: WastewaterDoc | null = $state(null);

  function fmtDate(dt: string) {
    return new Date(dt).toLocaleDateString();
  }
  function fmtDateTime(dt: string) {
    return new Date(dt).toLocaleString();
  }
  function toggleExpand(id: string) {
    expandedId = expandedId === id ? null : id;
  }
</script>

<div class="page">
  <h2>Sanitary Department Overview</h2>
  <p class="subtitle">Read-only view of sanitary division data.</p>

  {#if loading}
    <div class="loading">Loading…</div>
  {:else if error}
    <div class="banner error">{error}</div>
  {:else}
    <div class="tabs">
      {#each tabs as t}
        <button class="tab" class:active={activeTab === t.id}
          onclick={() => { activeTab = t.id; expandedId = null; selectedDisposal = null; selectedWastewater = null; }}>
          {t.label}
        </button>
      {/each}
    </div>

    {#if activeTab === 'staff'}
      <table class="tbl">
        <thead><tr><th>Name</th><th>Role</th><th>Division</th><th>Quarter</th></tr></thead>
        <tbody>
          {#each staff as s}
            <tr><td>{s.full_name}</td><td>{s.role_name}</td><td>{s.division_name ?? '—'}</td><td>{s.quarter ?? '—'}</td></tr>
          {/each}
          {#if staff.length === 0}<tr><td colspan="4" class="empty">No staff records.</td></tr>{/if}
        </tbody>
      </table>

    {:else if activeTab === 'shifts'}
      <table class="tbl">
        <thead><tr><th>Staff</th><th>Start</th><th>End</th><th>Quarter</th></tr></thead>
        <tbody>
          {#each shifts as sh}
            <tr><td>{sh.staff_name}</td><td>{fmtDateTime(sh.shift_start)}</td><td>{fmtDateTime(sh.shift_end)}</td><td>{sh.quarter ?? '—'}</td></tr>
          {/each}
          {#if shifts.length === 0}<tr><td colspan="4" class="empty">No shifts found.</td></tr>{/if}
        </tbody>
      </table>

    {:else if activeTab === 'divisions'}
      <table class="tbl">
        <thead><tr><th>Division</th><th>Quota</th><th>Staff Count</th><th>Utilization</th></tr></thead>
        <tbody>
          {#each divisions as d}
            <tr>
              <td>{d.name}</td>
              <td>{d.quota}</td>
              <td>{d.staff_count}</td>
              <td>
                <span class:over-quota={d.staff_count >= d.quota}>
                  {d.staff_count}/{d.quota}
                  {#if d.staff_count >= d.quota} (Full){/if}
                </span>
              </td>
            </tr>
          {/each}
          {#if divisions.length === 0}<tr><td colspan="4" class="empty">No divisions found.</td></tr>{/if}
        </tbody>
      </table>

    {:else if activeTab === 'inventory'}
      <table class="tbl">
        <thead><tr><th>Item</th><th>Category</th><th>Qty</th><th>Unit</th><th>Updated</th></tr></thead>
        <tbody>
          {#each inventory as i}
            <tr><td>{i.item_name}</td><td>{i.category}</td><td>{i.quantity}</td><td>{i.unit}</td><td>{fmtDate(i.updated_at)}</td></tr>
          {/each}
          {#if inventory.length === 0}<tr><td colspan="5" class="empty">No inventory items.</td></tr>{/if}
        </tbody>
      </table>

    {:else if activeTab === 'inspection'}
      <table class="tbl">
        <thead><tr><th>Reporter</th><th>Location</th><th>Area / Machine</th><th>Severity</th><th>Date</th></tr></thead>
        <tbody>
          {#each inspections as r}
            <tr class="clickable" onclick={() => toggleExpand(r.id)}>
              <td>{r.reporter_name}</td>
              <td>{r.location}</td>
              <td>{r.area_or_machine}</td>
              <td><span class="badge sev-{r.severity}">{r.severity}</span></td>
              <td>{fmtDate(r.created_at)}</td>
            </tr>
            {#if expandedId === r.id}
              <tr class="detail-row">
                <td colspan="5">
                  <div class="detail-box">
                    <div class="detail-field"><span class="dk">Findings</span><span>{r.findings}</span></div>
                    {#if r.recommendations}<div class="detail-field"><span class="dk">Recommendations</span><span>{r.recommendations}</span></div>{/if}
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
          {#if inspections.length === 0}<tr><td colspan="5" class="empty">No inspection reports.</td></tr>{/if}
        </tbody>
      </table>

    {:else if activeTab === 'disposal'}
      <!-- Full document view — list + detail panel -->
      {#if selectedDisposal}
        <button class="back-btn" onclick={() => (selectedDisposal = null)}>← Back to list</button>
        <div class="doc-view">
          <h3>{selectedDisposal.waste_category}</h3>
          <div class="doc-meta">
            <span>By: {selectedDisposal.author_name}</span>
            <span>Created: {fmtDate(selectedDisposal.created_at)}</span>
          </div>
          <section class="doc-section">
            <h4>Procedure</h4>
            <p>{selectedDisposal.procedure}</p>
          </section>
          {#if selectedDisposal.safety_requirements}
            <section class="doc-section">
              <h4>Safety Requirements</h4>
              <p>{selectedDisposal.safety_requirements}</p>
            </section>
          {/if}
          {#if selectedDisposal.compliance_notes}
            <section class="doc-section">
              <h4>Compliance Notes</h4>
              <p>{selectedDisposal.compliance_notes}</p>
            </section>
          {/if}
          {#if selectedDisposal.revision_history.length > 0}
            <section class="doc-section">
              <h4>Revision History ({selectedDisposal.revision_history.length})</h4>
              {#each selectedDisposal.revision_history as rev}
                <div class="rev-entry">
                  {#if (rev as Record<string,unknown>).change_summary}<p class="rev-summary">{(rev as Record<string,unknown>).change_summary as string}</p>{/if}
                  {#if (rev as Record<string,unknown>).revised_at}<p class="rev-meta">Revised at: {fmtDateTime((rev as Record<string,unknown>).revised_at as string)}</p>{/if}
                </div>
              {/each}
            </section>
          {/if}
        </div>
      {:else}
        <table class="tbl">
          <thead><tr><th>Waste Category</th><th>Author</th><th>Revisions</th><th>Created</th><th></th></tr></thead>
          <tbody>
            {#each disposal as d}
              <tr>
                <td>{d.waste_category}</td><td>{d.author_name}</td>
                <td>{d.revision_history.length}</td>
                <td>{fmtDate(d.created_at)}</td>
                <td><button class="btn-view" onclick={() => (selectedDisposal = d)}>View Doc</button></td>
              </tr>
            {/each}
            {#if disposal.length === 0}<tr><td colspan="5" class="empty">No disposal docs.</td></tr>{/if}
          </tbody>
        </table>
      {/if}

    {:else if activeTab === 'wastewater'}
      {#if selectedWastewater}
        <button class="back-btn" onclick={() => (selectedWastewater = null)}>← Back to list</button>
        <div class="doc-view">
          <h3>{selectedWastewater.treatment_type}</h3>
          <div class="doc-meta">
            <span>By: {selectedWastewater.author_name}</span>
            <span>Created: {fmtDate(selectedWastewater.created_at)}</span>
          </div>
          <section class="doc-section">
            <h4>Treatment Steps</h4>
            <ol class="steps-list">
              {#each selectedWastewater.steps as step}
                <li>{step}</li>
              {/each}
            </ol>
          </section>
          {#if selectedWastewater.safety_requirements}
            <section class="doc-section">
              <h4>Safety Requirements</h4>
              <p>{selectedWastewater.safety_requirements}</p>
            </section>
          {/if}
          {#if selectedWastewater.compliance_notes}
            <section class="doc-section">
              <h4>Compliance Notes</h4>
              <p>{selectedWastewater.compliance_notes}</p>
            </section>
          {/if}
          {#if selectedWastewater.revision_history.length > 0}
            <section class="doc-section">
              <h4>Revision History ({selectedWastewater.revision_history.length})</h4>
              {#each selectedWastewater.revision_history as rev}
                <div class="rev-entry">
                  {#if (rev as Record<string,unknown>).change_summary}<p class="rev-summary">{(rev as Record<string,unknown>).change_summary as string}</p>{/if}
                  {#if (rev as Record<string,unknown>).revised_at}<p class="rev-meta">Revised at: {fmtDateTime((rev as Record<string,unknown>).revised_at as string)}</p>{/if}
                </div>
              {/each}
            </section>
          {/if}
        </div>
      {:else}
        <table class="tbl">
          <thead><tr><th>Treatment Type</th><th>Author</th><th>Steps</th><th>Revisions</th><th>Created</th><th></th></tr></thead>
          <tbody>
            {#each wastewater as w}
              <tr>
                <td>{w.treatment_type}</td><td>{w.author_name}</td>
                <td>{w.steps.length}</td>
                <td>{w.revision_history.length}</td>
                <td>{fmtDate(w.created_at)}</td>
                <td><button class="btn-view" onclick={() => (selectedWastewater = w)}>View Doc</button></td>
              </tr>
            {/each}
            {#if wastewater.length === 0}<tr><td colspan="6" class="empty">No wastewater docs.</td></tr>{/if}
          </tbody>
        </table>
      {/if}

    {:else if activeTab === 'budget'}
      <h3>Budget Requests</h3>
      <table class="tbl">
        <thead><tr><th>Amount</th><th>Justification</th><th>Status</th><th>Date</th></tr></thead>
        <tbody>
          {#each budgetReqs as b}
            <tr><td>${b.total_amount?.toFixed(2) ?? '—'}</td><td>{b.justification ?? '—'}</td><td>{b.status}</td><td>{fmtDate(b.created_at)}</td></tr>
          {/each}
          {#if budgetReqs.length === 0}<tr><td colspan="4" class="empty">No budget requests.</td></tr>{/if}
        </tbody>
      </table>

      <h3 style="margin-top:1.5rem">Expenditure Reports</h3>
      <table class="tbl">
        <thead><tr><th>Amount</th><th>Foul Play</th><th>Date</th></tr></thead>
        <tbody>
          {#each expenditures as e}
            <tr>
              <td>${e.total_amount?.toFixed(2) ?? '—'}</td>
              <td>{e.foul_play_flag ? `Yes — ${e.foul_play_note ?? ''}` : 'No'}</td>
              <td>{fmtDate(e.created_at)}</td>
            </tr>
          {/each}
          {#if expenditures.length === 0}<tr><td colspan="3" class="empty">No expenditure reports.</td></tr>{/if}
        </tbody>
      </table>
    {/if}
  {/if}
</div>

<style>
  .page { max-width:900px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.25rem; }
  h3 { font-size:0.9rem;color:#94A3B8;margin:0 0 0.5rem; }
  .subtitle { color:#64748B;font-size:0.8rem;margin:0 0 1.25rem; }
  .loading { color:#94A3B8;font-size:0.85rem; }
  .banner { padding:0.6rem 1rem;border-radius:6px;font-size:0.8rem;margin-bottom:1rem; }
  .banner.error { background:rgba(239,68,68,0.15);color:#EF4444;border:1px solid rgba(239,68,68,0.3); }
  .tabs { display:flex;flex-wrap:wrap;gap:0.4rem;margin-bottom:1rem; }
  .tab { padding:0.35rem 0.75rem;border:1px solid #374151;background:#1F2937;color:#94A3B8;border-radius:4px;cursor:pointer;font-size:0.78rem; }
  .tab.active { border-color:#3ABEFF;color:#3ABEFF;background:rgba(58,190,255,0.1); }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { color:#64748B;font-weight:500;text-align:left;padding:0.4rem 0.6rem;border-bottom:1px solid #1F2937; }
  .tbl td { color:#CBD5E1;padding:0.45rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .empty { color:#4B5563;font-style:italic;text-align:center; }
  .over-quota { color:#F59E0B;font-weight:600; }
  .badge { padding:0.15rem 0.4rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .sev-low { background:rgba(16,185,129,0.15);color:#10B981; }
  .sev-medium { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .sev-high { background:rgba(239,68,68,0.15);color:#EF4444; }
  .sev-critical { background:rgba(239,68,68,0.3);color:#EF4444; }
  .clickable { cursor:pointer; }
  .clickable:hover td { background:rgba(58,190,255,0.04); }
  .detail-row td { padding:0;background:rgba(58,190,255,0.03); }
  .detail-box { padding:0.6rem 1rem;display:flex;flex-direction:column;gap:0.4rem; }
  .detail-field { display:flex;gap:0.5rem;font-size:0.78rem; }
  .dk { color:#64748B;min-width:130px;flex-shrink:0; }
  /* Disposal / Wastewater doc view */
  .back-btn { background:none;border:1px solid rgba(58,190,255,0.2);color:#3ABEFF;padding:0.3rem 0.7rem;border-radius:4px;cursor:pointer;font-size:0.78rem;margin-bottom:0.75rem; }
  .back-btn:hover { background:rgba(58,190,255,0.08); }
  .btn-view { background:rgba(58,190,255,0.08);border:1px solid rgba(58,190,255,0.2);color:#3ABEFF;padding:0.2rem 0.5rem;border-radius:4px;cursor:pointer;font-size:0.72rem; }
  .btn-view:hover { background:rgba(58,190,255,0.18); }
  .doc-view { max-width:680px; }
  .doc-view h3 { font-family:'Orbitron',sans-serif;font-size:1rem;color:#3ABEFF;margin:0 0 0.4rem; }
  .doc-meta { display:flex;gap:1.5rem;font-size:0.75rem;color:#64748B;margin-bottom:1rem; }
  .doc-section { margin-bottom:1rem; }
  .doc-section h4 { font-size:0.8rem;color:#94A3B8;margin:0 0 0.35rem; }
  .doc-section p { font-size:0.82rem;color:#CBD5E1;line-height:1.55;margin:0;white-space:pre-wrap; }
  .steps-list { padding-left:1.2rem;margin:0; }
  .steps-list li { font-size:0.82rem;color:#CBD5E1;line-height:1.7; }
  .rev-entry { background:rgba(58,190,255,0.03);border:1px solid rgba(58,190,255,0.08);border-radius:4px;padding:0.4rem 0.6rem;margin-bottom:0.3rem; }
  .rev-summary { font-size:0.8rem;color:#CBD5E1;margin:0 0 0.15rem; }
  .rev-meta { font-size:0.72rem;color:#64748B;margin:0; }
</style>
