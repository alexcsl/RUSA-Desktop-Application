<!--
  /data/analyst/browse — UC-DA-03: Read all non-medical data (Data Analyst only)
  Allows browsing any whitelisted table with optional column filter.
  Results render as a dynamic table. Auth guard enforced by backend (DataAnalyst only).
-->
<script lang="ts">
  import { browseData, type BrowseDataPayload } from '$lib/stores/data_analysts';

  const ALLOWED_TABLES = [
    'users', 'roles', 'experiments', 'experiment_logs', 'species_archive',
    'missions', 'territories', 'events', 'event_documents',
    'incident_reports', 'daily_security_reports',
    'data_requests', 'data_responses',
    'test_proposals', 'approved_tests',
    'help_requests', 'progress_reports', 'audit_logs',
    'announcements', 'sanitary_tasks', 'sanitary_schedules', 'sanitary_divisions',
  ];

  let tableName = $state(ALLOWED_TABLES[0]);
  let filterColumn = $state('');
  let filterValue = $state('');
  let limit = $state(50);
  let offset = $state(0);

  let rows: Record<string, unknown>[] = $state([]);
  let columns: string[] = $state([]);
  let loading = $state(false);
  let error = $state('');
  let loaded = $state(false);

  async function loadData() {
    loading = true; error = ''; rows = []; columns = [];
    try {
      const payload: BrowseDataPayload = {
        table_name: tableName,
        limit,
        offset,
        filter_column: filterColumn.trim() || undefined,
        filter_value: filterValue.trim() || undefined,
      };
      rows = await browseData(payload);
      if (rows.length > 0) {
        columns = Object.keys(rows[0]);
      }
      loaded = true;
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function cellValue(val: unknown): string {
    if (val === null || val === undefined) return '—';
    if (typeof val === 'object') return JSON.stringify(val).slice(0, 80);
    return String(val);
  }

  function nextPage() { offset += limit; loadData(); }
  function prevPage() { offset = Math.max(0, offset - limit); loadData(); }
</script>

<div class="page">
  <h1 class="title">Data Browser</h1>
  <p class="subtitle">Read access to all non-medical data tables. Medical data (patient records, treatment logs) is excluded.</p>

  <div class="controls">
    <div class="ctrl-row">
      <label class="field">
        <span class="lbl">Table</span>
        <select class="input" bind:value={tableName}>
          {#each ALLOWED_TABLES as t}
            <option value={t}>{t}</option>
          {/each}
        </select>
      </label>
      <label class="field">
        <span class="lbl">Limit</span>
        <select class="input sm" bind:value={limit}>
          <option value={25}>25</option>
          <option value={50}>50</option>
          <option value={100}>100</option>
          <option value={200}>200</option>
        </select>
      </label>
    </div>
    <div class="ctrl-row">
      <label class="field flex-1">
        <span class="lbl">Filter Column</span>
        <input class="input" type="text" placeholder="column_name" bind:value={filterColumn} />
      </label>
      <label class="field flex-1">
        <span class="lbl">Filter Value</span>
        <input class="input" type="text" placeholder="search text (ILIKE)" bind:value={filterValue} />
      </label>
    </div>
    <button class="btn-primary" onclick={loadData} disabled={loading}>
      {loading ? 'Loading…' : 'Load Data'}
    </button>
  </div>

  {#if error}
    <p class="error">{error}</p>
  {:else if loading}
    <p class="muted">Querying {tableName}…</p>
  {:else if loaded && rows.length === 0}
    <div class="empty">No rows returned. Try adjusting filters or selecting a different table.</div>
  {:else if rows.length > 0}
    <div class="result-header">
      <span class="result-info">{rows.length} rows · {tableName} · offset {offset}</span>
      <div class="pagination">
        <button class="btn-sm" onclick={prevPage} disabled={offset === 0}>← Prev</button>
        <button class="btn-sm" onclick={nextPage} disabled={rows.length < limit}>Next →</button>
      </div>
    </div>
    <div class="table-wrap">
      <table class="data-table">
        <thead>
          <tr>
            {#each columns as col}
              <th>{col}</th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each rows as row}
            <tr>
              {#each columns as col}
                <td title={cellValue(row[col])}>{cellValue(row[col])}</td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
    <div class="pagination" style="margin-top:0.5rem">
      <button class="btn-sm" onclick={prevPage} disabled={offset === 0}>← Prev</button>
      <span class="muted">Page {Math.floor(offset / limit) + 1}</span>
      <button class="btn-sm" onclick={nextPage} disabled={rows.length < limit}>Next →</button>
    </div>
  {/if}
</div>

<style>
  .page { max-width:100%;overflow:hidden; }
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.2rem; }
  .subtitle { color:#94A3B8;font-size:0.78rem;margin:0 0 1rem;line-height:1.5; }
  .controls { background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:1rem;margin-bottom:1rem;display:flex;flex-direction:column;gap:0.6rem; }
  .ctrl-row { display:flex;gap:0.6rem;flex-wrap:wrap; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .flex-1 { flex:1;min-width:160px; }
  .lbl { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem 0.6rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input.sm { width:80px; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .btn-primary { padding:0.45rem 1.2rem;background:linear-gradient(135deg,#3ABEFF,#8B5CF6);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.82rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-sm { padding:0.25rem 0.65rem;background:#1E293B;border:1px solid rgba(58,190,255,0.2);border-radius:5px;color:#E6EDF3;font-size:0.75rem;cursor:pointer; }
  .btn-sm:disabled { opacity:0.4;cursor:not-allowed; }
  .btn-sm:hover:not(:disabled) { border-color:#3ABEFF; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .error { color:#EF4444;font-size:0.8rem;margin:0.5rem 0; }
  .empty { color:#94A3B8;font-size:0.85rem;background:#111827;border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:1rem;text-align:center; }
  .result-header { display:flex;justify-content:space-between;align-items:center;margin-bottom:0.4rem; }
  .result-info { font-size:0.75rem;color:#94A3B8; }
  .pagination { display:flex;align-items:center;gap:0.4rem; }
  .table-wrap { overflow-x:auto;border:1px solid rgba(58,190,255,0.1);border-radius:6px;max-height:420px;overflow-y:auto; }
  .data-table { width:100%;border-collapse:collapse;font-size:0.72rem; }
  .data-table th { background:#0E1428;color:#3ABEFF;padding:0.4rem 0.6rem;text-align:left;border-bottom:1px solid rgba(58,190,255,0.2);white-space:nowrap;position:sticky;top:0;z-index:1; }
  .data-table td { padding:0.3rem 0.6rem;color:#E6EDF3;border-bottom:1px solid rgba(255,255,255,0.04);max-width:200px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .data-table tr:hover td { background:rgba(58,190,255,0.04); }
</style>
