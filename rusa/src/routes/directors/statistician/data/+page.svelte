<!--
  /directors/statistician/data — Data Browser for TheStatistician
  Browse non-medical tables, run statistical operations, visualize numeric columns,
  and insert new rows into allowed tables.
-->
<script lang="ts">
  import { browseData, computeOperation, daWriteData, type ComputeResult } from '$lib/stores/data_analysts';

  const ALLOWED_TABLES = ['missions', 'territories', 'experiments', 'tasks', 'settlements', 'base_locations', 'events'];
  const OPERATIONS = ['avg', 'median', 'mode', 'variance', 'max', 'min'] as const;

  // ── Read / Browse ─────────────────────────────────────────────────────────────
  let selectedTable = $state('');
  let filterColumn = $state('');
  let filterValue = $state('');
  let limit = $state(50);

  let data: Record<string, unknown>[] = $state([]);
  let columns: string[] = $state([]);
  let browseLoading = $state(false);
  let browseError = $state('');

  // ── Statistical Operation ─────────────────────────────────────────────────────
  let selectedOp = $state<typeof OPERATIONS[number]>('avg');
  let selectedCol = $state('');
  let computeResult: ComputeResult | null = $state(null);
  let computeLoading = $state(false);
  let computeError = $state('');

  let numericColumns = $derived(
    columns.filter((col) => data.length > 0 && typeof data[0][col] === 'number')
  );

  // ── Write / Insert ────────────────────────────────────────────────────────────
  let writeTable = $state('');
  let writeJson = $state('{}');
  let writeLoading = $state(false);
  let writeError = $state('');
  let writeSuccess = $state('');

  async function handleBrowse(e: Event) {
    e.preventDefault();
    if (!selectedTable) { browseError = 'Select a table.'; return; }
    browseLoading = true; browseError = ''; data = []; columns = []; computeResult = null;
    try {
      const payload: { table_name: string; limit: number; filter_column?: string; filter_value?: string } = {
        table_name: selectedTable,
        limit,
      };
      if (filterColumn && filterValue) {
        payload.filter_column = filterColumn;
        payload.filter_value = filterValue;
      }
      data = await browseData(payload);
      columns = data.length > 0 ? Object.keys(data[0]) : [];
      if (numericColumns.length > 0) selectedCol = numericColumns[0];
    } catch (err) {
      browseError = String(err);
    } finally {
      browseLoading = false;
    }
  }

  async function handleCompute(e: Event) {
    e.preventDefault();
    if (!selectedCol) { computeError = 'Select a column.'; return; }
    computeLoading = true; computeError = ''; computeResult = null;
    try {
      computeResult = await computeOperation({ operation: selectedOp, data, column: selectedCol });
    } catch (err) {
      computeError = String(err);
    } finally {
      computeLoading = false;
    }
  }

  async function handleWrite(e: Event) {
    e.preventDefault();
    writeError = ''; writeSuccess = '';
    if (!writeTable) { writeError = 'Select a table.'; return; }
    let rowData: Record<string, unknown>;
    try {
      rowData = JSON.parse(writeJson);
    } catch {
      writeError = 'Invalid JSON. Please check the row data.';
      return;
    }
    if (typeof rowData !== 'object' || Array.isArray(rowData)) {
      writeError = 'Row data must be a JSON object.';
      return;
    }
    writeLoading = true;
    try {
      await daWriteData({ table_name: writeTable, row_data: rowData });
      writeSuccess = `Row inserted into "${writeTable}".`;
      writeJson = '{}';
    } catch (err) {
      writeError = String(err);
    } finally {
      writeLoading = false;
    }
  }

  function barValues(): { label: string; value: number; pct: number }[] {
    if (!selectedCol || data.length === 0) return [];
    const vals = data.map((row, i) => ({
      label: String(row['name'] ?? row['title'] ?? row['id'] ?? i),
      value: Number(row[selectedCol]) || 0,
    }));
    const max = Math.max(...vals.map((v) => v.value), 1);
    return vals.map((v) => ({ ...v, pct: Math.round((v.value / max) * 100) }));
  }

  let bars = $derived(barValues());
</script>

<h1 class="title">Data Operations</h1>
<p class="subtitle">Read, analyze, and write non-medical station data.</p>

<!-- Browse / Read Section -->
<section class="section">
  <h3 class="section-title">Read Data</h3>
  <form onsubmit={handleBrowse} class="browse-form">
    <div class="row3">
      <div class="field">
        <label for="table">Table</label>
        <select id="table" bind:value={selectedTable} required>
          <option value="">— Select table —</option>
          {#each ALLOWED_TABLES as t}
            <option value={t}>{t}</option>
          {/each}
        </select>
      </div>
      <div class="field">
        <label for="filter_col">Filter Column (opt.)</label>
        <input id="filter_col" type="text" bind:value={filterColumn} placeholder="e.g. status" />
      </div>
      <div class="field">
        <label for="filter_val">Filter Value (opt.)</label>
        <input id="filter_val" type="text" bind:value={filterValue} placeholder="e.g. active" />
      </div>
    </div>
    <div class="row-limit">
      <div class="field field-sm">
        <label for="limit">Limit</label>
        <input id="limit" type="number" bind:value={limit} min="1" max="500" />
      </div>
      <button type="submit" class="btn-load" disabled={browseLoading}>
        {browseLoading ? 'Loading…' : 'Load Data'}
      </button>
    </div>
  </form>
  {#if browseError}<p class="err">{browseError}</p>{/if}
</section>

{#if data.length > 0}
  <!-- Data Table -->
  <section class="section">
    <p class="row-count">{data.length} rows · {columns.length} columns</p>
    <div class="table-wrap">
      <table>
        <thead>
          <tr>{#each columns as col}<th>{col}</th>{/each}</tr>
        </thead>
        <tbody>
          {#each data as row}
            <tr>{#each columns as col}<td>{row[col] ?? '—'}</td>{/each}</tr>
          {/each}
        </tbody>
      </table>
    </div>
  </section>

  <!-- Operations Section -->
  <section class="section">
    <h3 class="section-title">Statistical Operation</h3>
    {#if numericColumns.length === 0}
      <p class="muted">No numeric columns available for operations.</p>
    {:else}
      <form onsubmit={handleCompute} class="op-form">
        <div class="row2">
          <div class="field">
            <label for="op">Operation</label>
            <select id="op" bind:value={selectedOp}>
              {#each OPERATIONS as op}<option value={op}>{op}</option>{/each}
            </select>
          </div>
          <div class="field">
            <label for="col">Column</label>
            <select id="col" bind:value={selectedCol}>
              {#each numericColumns as c}<option value={c}>{c}</option>{/each}
            </select>
          </div>
        </div>
        <button type="submit" class="btn-load" disabled={computeLoading}>
          {computeLoading ? 'Computing…' : 'Compute'}
        </button>
      </form>
      {#if computeError}<p class="err">{computeError}</p>{/if}
      {#if computeResult}
        <div class="result-box">
          <span class="result-op">{computeResult.operation.toUpperCase()}</span>
          <span class="result-col">({computeResult.column})</span>
          <span class="result-val">= {computeResult.result}</span>
          <span class="result-rows">{computeResult.row_count} rows</span>
        </div>
      {/if}
    {/if}
  </section>

  <!-- Graph Section -->
  {#if selectedCol && numericColumns.includes(selectedCol) && bars.length > 0}
    <section class="section">
      <h3 class="section-title">Bar Chart — {selectedCol}</h3>
      <div class="chart">
        {#each bars.slice(0, 20) as bar}
          <div class="bar-row">
            <span class="bar-label" title={bar.label}>{bar.label.slice(0, 16)}</span>
            <div class="bar-track">
              <div class="bar-fill" style="width:{bar.pct}%"></div>
            </div>
            <span class="bar-val">{bar.value}</span>
          </div>
        {/each}
        {#if bars.length > 20}
          <p class="muted">Showing first 20 of {bars.length} rows.</p>
        {/if}
      </div>
    </section>
  {/if}
{/if}

<!-- Write / Insert Section -->
<section class="section">
  <h3 class="section-title">Write Data</h3>
  <p class="section-desc">Insert a new row into any allowed non-medical table.</p>
  <form onsubmit={handleWrite} class="browse-form">
    <div class="field">
      <label for="write-table">Table</label>
      <select id="write-table" bind:value={writeTable} required>
        <option value="">— Select table —</option>
        {#each ALLOWED_TABLES as t}
          <option value={t}>{t}</option>
        {/each}
      </select>
    </div>
    <div class="field">
      <label for="write-json">Row Data (JSON object)</label>
      <textarea
        id="write-json"
        bind:value={writeJson}
        rows="4"
        placeholder='e.g. {{"name": "Alpha Sector", "status": "active"}}'
        class="json-input"
      ></textarea>
    </div>
    {#if writeError}<p class="err">{writeError}</p>{/if}
    {#if writeSuccess}<p class="ok">{writeSuccess}</p>{/if}
    <button type="submit" class="btn-write" disabled={writeLoading}>
      {writeLoading ? 'Inserting…' : 'Insert Row'}
    </button>
  </form>
</section>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .section { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:1rem;margin-bottom:1rem; }
  .section-title { font-family:'Orbitron',sans-serif;font-size:0.82rem;color:#8B5CF6;margin:0 0 0.75rem; }
  .section-desc { font-size:0.75rem;color:#64748B;margin:0 0 0.75rem; }
  .browse-form,.op-form { display:flex;flex-direction:column;gap:0.65rem; }
  .row3 { display:grid;grid-template-columns:1fr 1fr 1fr;gap:0.65rem; }
  .row2 { display:grid;grid-template-columns:1fr 1fr;gap:0.65rem; }
  .row-limit { display:flex;align-items:flex-end;gap:0.65rem; }
  .field { display:flex;flex-direction:column;gap:0.25rem; }
  .field-sm { width:90px; }
  label { font-size:0.72rem;color:#64748B;font-weight:500; }
  input,select { background:#1F2937;border:1px solid #374151;color:#E6EDF3;border-radius:6px;padding:0.4rem 0.55rem;font-size:0.8rem;font-family:inherit; }
  input:focus,select:focus { outline:none;border-color:#3ABEFF; }
  .json-input { background:#1F2937;border:1px solid #374151;color:#E6EDF3;border-radius:6px;padding:0.4rem 0.55rem;font-size:0.78rem;font-family:'Fira Mono','Courier New',monospace;resize:vertical; }
  .json-input:focus { outline:none;border-color:#3ABEFF; }
  .btn-load { padding:0.42rem 1rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem;font-weight:600;white-space:nowrap;align-self:flex-start; }
  .btn-load:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-load:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-write { padding:0.42rem 1rem;background:rgba(16,185,129,0.15);border:1px solid #10B981;color:#10B981;border-radius:6px;cursor:pointer;font-size:0.8rem;font-weight:600;align-self:flex-start; }
  .btn-write:hover:not(:disabled) { background:rgba(16,185,129,0.25); }
  .btn-write:disabled { opacity:0.5;cursor:not-allowed; }
  .err { color:#EF4444;font-size:0.75rem;margin:0.3rem 0 0; }
  .ok { color:#34D399;font-size:0.75rem;margin:0.3rem 0 0; }
  .muted { color:#475569;font-size:0.78rem;font-style:italic;margin:0; }
  .row-count { font-size:0.72rem;color:#64748B;margin:0 0 0.6rem; }
  .table-wrap { overflow:auto;max-height:280px; }
  table { width:100%;border-collapse:collapse;font-size:0.75rem; }
  thead tr { border-bottom:1px solid rgba(58,190,255,0.15);position:sticky;top:0;background:#111827; }
  th { color:#64748B;font-weight:600;font-size:0.68rem;padding:0.4rem 0.6rem;text-align:left;text-transform:uppercase;white-space:nowrap; }
  td { padding:0.35rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04);color:#94A3B8;white-space:nowrap;max-width:160px;overflow:hidden;text-overflow:ellipsis; }
  .result-box { display:flex;align-items:center;gap:0.5rem;margin-top:0.75rem;padding:0.6rem 0.9rem;background:rgba(139,92,246,0.1);border:1px solid rgba(139,92,246,0.25);border-radius:6px; }
  .result-op { color:#8B5CF6;font-weight:700;font-size:0.82rem; }
  .result-col { color:#64748B;font-size:0.78rem; }
  .result-val { color:#E6EDF3;font-size:1rem;font-weight:700;margin-left:0.25rem; }
  .result-rows { margin-left:auto;font-size:0.7rem;color:#475569; }
  .chart { display:flex;flex-direction:column;gap:0.3rem; }
  .bar-row { display:flex;align-items:center;gap:0.5rem; }
  .bar-label { width:110px;font-size:0.7rem;color:#64748B;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;text-align:right; }
  .bar-track { flex:1;height:14px;background:rgba(255,255,255,0.05);border-radius:3px;overflow:hidden; }
  .bar-fill { height:100%;background:linear-gradient(90deg,#3ABEFF,#8B5CF6);border-radius:3px;transition:width 0.3s ease; }
  .bar-val { width:50px;font-size:0.7rem;color:#94A3B8;text-align:right; }
</style>
