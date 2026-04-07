<!--
  Taskmaster: Mission Status Reports
  All status reports submitted by Astronauts across all missions.
  Access: TheTaskmaster
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getAllStatusReports, type StatusReportItem } from '$lib/stores/astronauts';

  let reports: StatusReportItem[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let expandedId = $state<string | null>(null);

  onMount(async () => {
    try {
      reports = await getAllStatusReports();
    } catch (e: unknown) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  function toggleExpand(id: string) {
    expandedId = expandedId === id ? null : id;
  }

  function fmtDate(dt: string | null) {
    if (!dt) return '—';
    return new Date(dt).toLocaleDateString();
  }

  function ragColor(rag: string | null) {
    if (rag === 'green') return 'rag-green';
    if (rag === 'amber') return 'rag-amber';
    if (rag === 'red') return 'rag-red';
    return '';
  }
</script>

<div class="page">
  <h2>Mission Status Reports</h2>
  <p class="subtitle">All astronaut status reports across active missions. Click a row to expand.</p>

  {#if loading}
    <p class="muted">Loading…</p>
  {:else if error}
    <div class="banner error">{error}</div>
  {:else}
    <table class="tbl">
      <thead>
        <tr>
          <th>Submitted By</th>
          <th>Status</th>
          <th>RAG</th>
          <th>Month</th>
          <th>Report Date</th>
        </tr>
      </thead>
      <tbody>
        {#each reports as r}
          <tr class="clickable" onclick={() => toggleExpand(r.id)}>
            <td>{r.submitter_name}</td>
            <td>{r.current_status}</td>
            <td>
              {#if r.rag_status}
                <span class="badge {ragColor(r.rag_status)}">{r.rag_status}</span>
              {:else}—{/if}
            </td>
            <td>{r.month_tracker ?? '—'}</td>
            <td>{fmtDate(r.report_date)}</td>
          </tr>
          {#if expandedId === r.id}
            <tr class="detail-row">
              <td colspan="5">
                <div class="detail-box">
                  {#if r.progress_last_month}
                    <div class="detail-field"><span class="dk">Progress Last Month</span><span>{r.progress_last_month}</span></div>
                  {/if}
                  {#if r.plans_next_month}
                    <div class="detail-field"><span class="dk">Plans Next Month</span><span>{r.plans_next_month}</span></div>
                  {/if}
                  {#if r.collected_samples_last_month}
                    <div class="detail-field"><span class="dk">Samples Collected</span><span>{r.collected_samples_last_month}</span></div>
                  {/if}
                  {#if r.issues_blockers}
                    <div class="detail-field"><span class="dk">Issues / Blockers</span><span class="highlight-red">{r.issues_blockers}</span></div>
                  {/if}
                  <div class="detail-field"><span class="dk">Submitted</span><span>{new Date(r.created_at).toLocaleString()}</span></div>
                </div>
              </td>
            </tr>
          {/if}
        {/each}
        {#if reports.length === 0}
          <tr><td colspan="5" class="empty">No status reports found.</td></tr>
        {/if}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .page { max-width:900px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.25rem; }
  .subtitle { color:#64748B;font-size:0.8rem;margin:0 0 1.25rem; }
  .muted { color:#94A3B8;font-size:0.85rem; }
  .banner { padding:0.6rem 1rem;border-radius:6px;font-size:0.8rem;margin-bottom:1rem; }
  .banner.error { background:rgba(239,68,68,0.15);color:#EF4444;border:1px solid rgba(239,68,68,0.3); }
  .tbl { width:100%;border-collapse:collapse;font-size:0.78rem; }
  .tbl th { color:#64748B;font-weight:500;text-align:left;padding:0.4rem 0.5rem;border-bottom:1px solid #1F2937; }
  .tbl td { color:#CBD5E1;padding:0.4rem 0.5rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .clickable { cursor:pointer; }
  .clickable:hover td { background:rgba(58,190,255,0.04); }
  .detail-row td { padding:0;background:rgba(58,190,255,0.03); }
  .detail-box { padding:0.6rem 1rem;display:flex;flex-direction:column;gap:0.4rem; }
  .detail-field { display:flex;gap:0.5rem;font-size:0.78rem; }
  .dk { color:#64748B;min-width:150px;flex-shrink:0; }
  .highlight-red { color:#FCA5A5; }
  .empty { color:#4B5563;font-style:italic;text-align:center; }
  .badge { padding:0.12rem 0.35rem;border-radius:4px;font-size:0.68rem;font-weight:600; }
  .rag-green { background:rgba(16,185,129,0.15);color:#10B981; }
  .rag-amber { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .rag-red { background:rgba(239,68,68,0.15);color:#EF4444; }
</style>
