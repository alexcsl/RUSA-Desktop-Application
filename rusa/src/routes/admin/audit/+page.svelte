<!--
  /admin/audit — Audit Log Viewer
  Paginated audit trail with filters (table, operation, performer, date range).
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getAuditLog,
    type AuditLogEntry,
    type AuditLogFilter,
  } from '$lib/stores/administrator';

  let entries: AuditLogEntry[] = $state([]);
  let totalCount = $state(0);
  let loading = $state(true);
  let error = $state('');

  /* Filters */
  let tableName = $state('');
  let operation = $state('');
  let performerId = $state('');
  let dateFrom = $state('');
  let dateTo = $state('');

  /* Pagination */
  let page = $state(1);
  const perPage = 40;

  /* Detail expand */
  let expandedId: string | null = $state(null);

  function buildFilter(): AuditLogFilter {
    return {
      table_name: tableName || undefined,
      operation: operation || undefined,
      performer_id: performerId || undefined,
      date_from: dateFrom || undefined,
      date_to: dateTo || undefined,
      page,
      page_size: perPage,
    };
  }

  async function loadPage() {
    loading = true;
    error = '';
    try {
      const res = await getAuditLog(buildFilter());
      entries = res.entries;
      totalCount = res.total_count;
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function applyFilters() { page = 1; loadPage(); }
  function resetFilters() { tableName = ''; operation = ''; performerId = ''; dateFrom = ''; dateTo = ''; page = 1; loadPage(); }
  function prev() { if (page > 1) { page--; loadPage(); } }
  function next() { if (page * perPage < totalCount) { page++; loadPage(); } }
  function toggle(id: string) { expandedId = expandedId === id ? null : id; }

  function fmtDate(iso: string): string {
    return new Date(iso).toLocaleString('en-GB', { day:'2-digit',month:'short',year:'numeric',hour:'2-digit',minute:'2-digit',second:'2-digit' });
  }

  function prettyJson(val: unknown): string {
    if (!val) return '—';
    try { return JSON.stringify(val, null, 2); } catch { return String(val); }
  }

  onMount(loadPage);
</script>

<div class="audit-page">
  <h1 class="title">Audit Log</h1>

  <!-- Filter Bar -->
  <div class="filter-bar">
    <input class="input" placeholder="Table name" bind:value={tableName} />
    <select class="input" bind:value={operation}>
      <option value="">All operations</option>
      <option value="CREATE">CREATE</option>
      <option value="UPDATE">UPDATE</option>
      <option value="DELETE">DELETE</option>
    </select>
    <input class="input" placeholder="Performer ID" bind:value={performerId} />
    <input class="input" type="date" bind:value={dateFrom} />
    <input class="input" type="date" bind:value={dateTo} />
    <button class="btn" onclick={applyFilters}>Filter</button>
    <button class="btn btn-ghost" onclick={resetFilters}>Reset</button>
  </div>

  {#if loading}
    <p class="hint">Loading…</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else}
    <p class="meta">{totalCount.toLocaleString()} record{totalCount === 1 ? '' : 's'} — page {page} of {Math.max(1,Math.ceil(totalCount / perPage))}</p>

    <table class="table">
      <thead>
        <tr>
          <th>Time</th><th>Table</th><th>Op</th><th>Record</th><th>Performer</th><th></th>
        </tr>
      </thead>
      <tbody>
        {#each entries as e}
          <tr class="row" class:expanded={expandedId === e.id}>
            <td class="cell time">{fmtDate(e.performed_at)}</td>
            <td class="cell tbl">{e.table_name}</td>
            <td class="cell op" style="color:{e.operation === 'CREATE' ? '#10B981' : e.operation === 'DELETE' ? '#EF4444' : '#3ABEFF'}">{e.operation}</td>
            <td class="cell rec">{e.record_id}</td>
            <td class="cell perf">{e.performer_name ?? e.performed_by ?? '—'}</td>
            <td class="cell act"><button class="expand-btn" onclick={() => toggle(e.id)}>{expandedId === e.id ? '▲' : '▼'}</button></td>
          </tr>
          {#if expandedId === e.id}
            <tr class="detail-row">
              <td colspan="6">
                <div class="detail-grid">
                  <div class="detail-col">
                    <span class="detail-label">Before</span>
                    <pre class="detail-pre">{prettyJson(e.before_data)}</pre>
                  </div>
                  <div class="detail-col">
                    <span class="detail-label">After</span>
                    <pre class="detail-pre">{prettyJson(e.after_data)}</pre>
                  </div>
                </div>
              </td>
            </tr>
          {/if}
        {:else}
          <tr><td colspan="6" class="empty">No audit entries match your filters.</td></tr>
        {/each}
      </tbody>
    </table>

    <!-- Pagination -->
    <div class="pager">
      <button class="btn btn-sm" onclick={prev} disabled={page <= 1}>← Prev</button>
      <span class="pager-info">Page {page}</span>
      <button class="btn btn-sm" onclick={next} disabled={page * perPage >= totalCount}>Next →</button>
    </div>
  {/if}
</div>

<style>
  .audit-page { max-width:1200px; }
  .title { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#EF4444;margin:0 0 1rem; }

  .filter-bar { display:flex;flex-wrap:wrap;gap:0.5rem;margin-bottom:0.75rem; }
  .input { background:#0E1428;border:1px solid rgba(239,68,68,0.18);border-radius:6px;color:#E6EDF3;font-size:0.78rem;padding:0.4rem 0.6rem;outline:none; }
  .input:focus { border-color:#EF4444; }
  select.input { min-width:130px; }

  .btn { background:rgba(239,68,68,0.15);color:#EF4444;border:1px solid rgba(239,68,68,0.25);border-radius:6px;padding:0.4rem 0.9rem;cursor:pointer;font-size:0.78rem;font-weight:600;transition:background 0.2s; }
  .btn:hover { background:rgba(239,68,68,0.25); }
  .btn-ghost { background:transparent;color:#94A3B8;border-color:rgba(148,163,184,0.2); }
  .btn-ghost:hover { background:rgba(148,163,184,0.08); }
  .btn-sm { padding:0.3rem 0.7rem;font-size:0.72rem; }
  .btn:disabled { opacity:0.4;cursor:default; }

  .meta { color:#94A3B8;font-size:0.75rem;margin-bottom:0.5rem; }
  .hint { color:#94A3B8;font-size:0.82rem; }
  .error { color:#EF4444;font-size:0.82rem; }

  .table { width:100%;border-collapse:collapse; }
  th { text-align:left;font-size:0.68rem;color:#475569;text-transform:uppercase;letter-spacing:0.03em;padding:0.5rem 0.4rem;border-bottom:1px solid rgba(255,255,255,0.06); }
  .row { transition:background 0.15s; }
  .row:hover { background:rgba(239,68,68,0.04); }
  .cell { padding:0.4rem;font-size:0.78rem;border-bottom:1px solid rgba(255,255,255,0.03); }
  .time { color:#94A3B8;white-space:nowrap;font-size:0.72rem; }
  .tbl { color:#E6EDF3; }
  .op { font-weight:700;font-size:0.72rem;text-transform:uppercase; }
  .rec { color:#C084FC;font-family:monospace;font-size:0.7rem;max-width:140px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .perf { color:#94A3B8; }
  .act { text-align:center; }
  .expand-btn { background:none;border:none;color:#475569;cursor:pointer;font-size:0.7rem; }
  .expand-btn:hover { color:#EF4444; }

  .detail-row td { padding:0;border:none; }
  .detail-grid { display:grid;grid-template-columns:1fr 1fr;gap:0.5rem;padding:0.5rem 0.4rem 0.75rem;background:rgba(14,20,40,0.5);border-bottom:1px solid rgba(239,68,68,0.08); }
  .detail-col { display:flex;flex-direction:column;gap:0.2rem; }
  .detail-label { font-size:0.68rem;color:#475569;text-transform:uppercase; }
  .detail-pre { background:#0B0F1A;border:1px solid rgba(255,255,255,0.04);border-radius:4px;padding:0.4rem;font-size:0.68rem;color:#94A3B8;overflow:auto;max-height:200px;white-space:pre-wrap;word-break:break-all; }

  .pager { display:flex;align-items:center;gap:0.75rem;margin-top:0.75rem; }
  .pager-info { color:#94A3B8;font-size:0.75rem; }
  .empty { text-align:center;color:#475569;font-size:0.8rem;padding:2rem 0; }
</style>
