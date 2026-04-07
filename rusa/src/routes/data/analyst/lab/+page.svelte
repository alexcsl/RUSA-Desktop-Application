<!--
  /data/analyst/lab — UC-DA-05: Data Lab — Operations, Filters & Graphs
  Analysts load data from a table OR paste JSON, then apply statistical operations.
  Graphs render as inline SVG bar charts — no external library.
  Auth guard enforced at backend (DataAnalyst only).
-->
<script lang="ts">
  import { browseData, computeOperation, type ComputeResult } from '$lib/stores/data_analysts';

  const ALLOWED_TABLES = [
    'users', 'roles', 'base_locations',
    'experiments', 'experiment_daily_logs', 'science_archive', 'species_archive',
    'approved_tests', 'test_proposals',
    'missions', 'territory_names', 'events', 'event_documents',
    'incident_reports', 'daily_security_reports',
    'data_requests', 'data_responses',
    'progress_reports', 'audit_log',
    'tasks', 'vote_sessions', 'vote_records',
    'settlements', 'sanitary_tasks', 'sanitary_shifts', 'sanitary_divisions',
  ];

  const OPERATIONS = [
    { value: 'avg', label: 'Average (mean)' },
    { value: 'mode', label: 'Mode (most frequent)' },
    { value: 'median', label: 'Median' },
    { value: 'variance', label: 'Variance & Std Dev' },
    { value: 'max', label: 'Maximum' },
    { value: 'min', label: 'Minimum' },
    { value: 'filter', label: 'Filter (row match)' },
    { value: 'pivot', label: 'Pivot (group by column)' },
    { value: 'graph', label: 'Graph (bar chart)' },
  ] as const;

  // ── Data loading state ───────────────────────────────────────────
  let loadTable = $state(ALLOWED_TABLES[0]);
  let loadLimit = $state(50);
  let loadFilter = $state('');
  let loadFilterVal = $state('');
  let loadLoading = $state(false);
  let loadError = $state('');

  let rawJson = $state('');
  let jsonError = $state('');
  let dataset: Record<string, unknown>[] = $state([]);
  let datasetColumns: string[] = $state([]);
  let datasetLoaded = $state(false);

  // ── Operation state ─────────────────────────────────────────────
  let operation: typeof OPERATIONS[number]['value'] = $state('avg');
  let column = $state('');
  let filterColumn = $state('');
  let filterValue = $state('');
  let aggColumn = $state('');

  let computing = $state(false);
  let opError = $state('');
  let result: ComputeResult | null = $state(null);

  // ── Derived chart data ──────────────────────────────────────────
  type ChartData = { labels: string[]; values: number[]; agg_column: string };
  let chartData: ChartData | null = $derived.by(() => {
    if (!result) return null;
    const r = result.result as Record<string, unknown>;
    if (!r?.is_chart) return null;
    return {
      labels: r.labels as string[],
      values: r.values as number[],
      agg_column: (r.agg_column as string) ?? 'count',
    };
  });

  let isTable = $derived.by(() => {
    if (!result) return false;
    return !!(result.result as Record<string, unknown>)?.is_table;
  });

  let tableResult = $derived.by(() => {
    if (!isTable) return [] as Record<string, unknown>[];
    return (result!.result as Record<string, unknown>).rows as Record<string, unknown>[];
  });

  let tableColumns = $derived.by(() => {
    if (!isTable || tableResult.length === 0) return [] as string[];
    return Object.keys(tableResult[0]);
  });

  // ── Functions ────────────────────────────────────────────────────
  async function loadFromTable() {
    loadLoading = true; loadError = ''; jsonError = '';
    try {
      const rows = await browseData({
        table_name: loadTable,
        limit: loadLimit,
        filter_column: loadFilter.trim() || undefined,
        filter_value: loadFilterVal.trim() || undefined,
      });
      dataset = rows;
      datasetColumns = rows.length > 0 ? Object.keys(rows[0]) : [];
      rawJson = JSON.stringify(rows, null, 2);
      if (datasetColumns.length > 0) column = datasetColumns[0];
      datasetLoaded = true;
    } catch (e: unknown) {
      loadError = e instanceof Error ? e.message : String(e);
    } finally {
      loadLoading = false;
    }
  }

  function parseRawJson() {
    jsonError = '';
    try {
      const parsed = JSON.parse(rawJson);
      if (!Array.isArray(parsed)) {
        jsonError = 'JSON must be an array of objects.';
        return;
      }
      dataset = parsed;
      datasetColumns = parsed.length > 0 ? Object.keys(parsed[0]) : [];
      if (datasetColumns.length > 0 && !column) column = datasetColumns[0];
      datasetLoaded = true;
    } catch {
      jsonError = 'Invalid JSON. Please check the format.';
    }
  }

  async function compute() {
    if (!column) { opError = 'Select a target column.'; return; }
    if (dataset.length === 0) { opError = 'Load data first.'; return; }
    computing = true; opError = ''; result = null;
    try {
      result = await computeOperation({
        operation,
        data: dataset,
        column,
        filter_column: filterColumn.trim() || undefined,
        filter_value: filterValue.trim() || undefined,
        agg_column: aggColumn.trim() || undefined,
      });
    } catch (e: unknown) {
      opError = e instanceof Error ? e.message : String(e);
    } finally {
      computing = false;
    }
  }

  function scalarResult(): string {
    if (!result) return '';
    const r = result.result as Record<string, unknown>;
    if (r.value !== undefined) return String(r.value);
    if (r.variance !== undefined) return `Variance: ${Number(r.variance).toFixed(4)}  Std Dev: ${Number(r.std_dev).toFixed(4)}`;
    return JSON.stringify(r, null, 2);
  }

  function cellValue(val: unknown): string {
    if (val === null || val === undefined) return '—';
    if (typeof val === 'object') return JSON.stringify(val).slice(0, 60);
    return String(val);
  }

  // SVG bar chart helpers
  const CHART_W = 600;
  const CHART_H = 180;
  const PAD = { top: 24, right: 16, bottom: 36, left: 8 };

  function chartBars(cd: ChartData) {
    if (!cd || cd.labels.length === 0) return [];
    const maxVal = Math.max(...cd.values, 1);
    const barW = Math.floor((CHART_W - PAD.left - PAD.right) / cd.labels.length);
    const innerH = CHART_H - PAD.top - PAD.bottom;
    return cd.labels.map((label, i) => ({
      x: PAD.left + i * barW + Math.floor(barW * 0.1),
      y: PAD.top + innerH - Math.round((cd.values[i] / maxVal) * innerH),
      width: Math.floor(barW * 0.8),
      height: Math.round((cd.values[i] / maxVal) * innerH),
      label: label.length > 12 ? label.slice(0, 12) + '…' : label,
      value: cd.values[i],
      labelX: PAD.left + i * barW + Math.floor(barW / 2),
    }));
  }
</script>

<div class="page">
  <h1 class="title">Data Lab</h1>
  <p class="subtitle">
    Apply statistical operations and visualize data. Load from a table or paste raw JSON.
  </p>

  <div class="two-col">
    <!-- LEFT: Data Loading -->
    <section class="panel">
      <h2 class="panel-title">1 · Load Data</h2>

      <div class="load-row">
        <label class="field flex-1">
          <span class="lbl">Table</span>
          <select class="input" bind:value={loadTable}>
            {#each ALLOWED_TABLES as t}<option value={t}>{t}</option>{/each}
          </select>
        </label>
        <label class="field" style="width:70px">
          <span class="lbl">Limit</span>
          <select class="input" bind:value={loadLimit}>
            <option value={25}>25</option>
            <option value={50}>50</option>
            <option value={100}>100</option>
          </select>
        </label>
      </div>
      <div class="load-row">
        <input class="input flex-1" type="text" placeholder="Filter column" bind:value={loadFilter} />
        <input class="input flex-1" type="text" placeholder="Filter value" bind:value={loadFilterVal} />
        <button class="btn-secondary" onclick={loadFromTable} disabled={loadLoading}>
          {loadLoading ? '…' : 'Load'}
        </button>
      </div>
      {#if loadError}<p class="error">{loadError}</p>{/if}

      <div class="field" style="margin-top:0.6rem">
        <span class="lbl">Or paste JSON array</span>
        <textarea class="textarea mono" rows="6" placeholder={'[{"col": "val"}, ...]'} bind:value={rawJson}></textarea>
        {#if jsonError}<p class="error">{jsonError}</p>{/if}
        <button class="btn-sm" onclick={parseRawJson} style="margin-top:0.25rem">Parse JSON</button>
      </div>

      {#if datasetLoaded}
        <div class="dataset-info">
          <span class="pill">{dataset.length} rows</span>
          <span class="pill">{datasetColumns.length} columns</span>
          {#if datasetColumns.length > 0}
            <span class="pill-cols">{datasetColumns.slice(0, 5).join(', ')}{datasetColumns.length > 5 ? '…' : ''}</span>
          {/if}
        </div>
      {/if}
    </section>

    <!-- RIGHT: Operations -->
    <section class="panel">
      <h2 class="panel-title">2 · Operations & Filters</h2>

      <label class="field">
        <span class="lbl">Operation</span>
        <select class="input" bind:value={operation}>
          {#each OPERATIONS as op}
            <option value={op.value}>{op.label}</option>
          {/each}
        </select>
      </label>

      <label class="field">
        <span class="lbl">Target Column</span>
        {#if datasetColumns.length > 0}
          <select class="input" bind:value={column}>
            {#each datasetColumns as col}<option value={col}>{col}</option>{/each}
          </select>
        {:else}
          <input class="input" type="text" placeholder="column_name" bind:value={column} />
        {/if}
      </label>

      {#if operation === 'filter'}
        <label class="field">
          <span class="lbl">Match Column</span>
          {#if datasetColumns.length > 0}
            <select class="input" bind:value={filterColumn}>
              <option value="">— same as target —</option>
              {#each datasetColumns as col}<option value={col}>{col}</option>{/each}
            </select>
          {:else}
            <input class="input" type="text" placeholder="column to match" bind:value={filterColumn} />
          {/if}
        </label>
        <label class="field">
          <span class="lbl">Match Value (substring)</span>
          <input class="input" type="text" placeholder="search text…" bind:value={filterValue} />
        </label>
      {/if}

      {#if operation === 'pivot' || operation === 'graph'}
        <label class="field">
          <span class="lbl">Aggregate Column (optional, default = count)</span>
          {#if datasetColumns.length > 0}
            <select class="input" bind:value={aggColumn}>
              <option value="">— count rows —</option>
              {#each datasetColumns as col}<option value={col}>{col}</option>{/each}
            </select>
          {:else}
            <input class="input" type="text" placeholder="numeric column to sum" bind:value={aggColumn} />
          {/if}
        </label>
      {/if}

      {#if opError}<p class="error">{opError}</p>{/if}

      <button class="btn-primary" onclick={compute} disabled={computing || !datasetLoaded}>
        {computing ? 'Computing…' : 'Run Operation'}
      </button>
    </section>
  </div>

  <!-- Result Panel -->
  {#if result}
    <section class="result-panel">
      <h2 class="panel-title">3 · Result <span class="op-badge">{result.operation}</span> on <code>{result.column}</code> · {result.row_count} input rows</h2>

      {#if chartData}
        <!-- Graph: SVG Bar Chart -->
        <p class="chart-label">{chartData.agg_column} per {result.column}</p>
        <div class="chart-wrap">
          <svg
            width="100%"
            viewBox="0 0 {CHART_W} {CHART_H}"
            xmlns="http://www.w3.org/2000/svg"
            class="chart-svg"
          >
            <!-- Y-axis baseline -->
            <line
              x1={PAD.left} y1={CHART_H - PAD.bottom}
              x2={CHART_W - PAD.right} y2={CHART_H - PAD.bottom}
              stroke="#334155" stroke-width="1"
            />
            {#each chartBars(chartData) as bar}
              <!-- Bar -->
              <rect
                x={bar.x} y={bar.y}
                width={bar.width} height={bar.height}
                fill="url(#bar-grad)" rx="2"
              />
              <!-- Value label above bar -->
              <text
                x={bar.labelX}
                y={bar.y - 4}
                text-anchor="middle"
                fill="#E6EDF3"
                font-size="10"
                font-family="Inter,sans-serif"
              >{typeof bar.value === 'number' ? bar.value.toFixed(1) : bar.value}</text>
              <!-- X-axis label -->
              <text
                x={bar.labelX}
                y={CHART_H - PAD.bottom + 14}
                text-anchor="middle"
                fill="#94A3B8"
                font-size="9"
                font-family="Inter,sans-serif"
              >{bar.label}</text>
            {/each}
            <!-- Gradient def -->
            <defs>
              <linearGradient id="bar-grad" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#3ABEFF"/>
                <stop offset="100%" stop-color="#8B5CF6"/>
              </linearGradient>
            </defs>
          </svg>
        </div>
        <details class="data-details">
          <summary class="muted">Raw pivot data</summary>
          <pre class="payload">{JSON.stringify({ labels: chartData.labels, values: chartData.values }, null, 2)}</pre>
        </details>

      {:else if isTable}
        <!-- Filtered rows table -->
        <p class="result-meta">{(result.result as Record<string, unknown>).matched as number} rows matched.</p>
        <div class="table-wrap">
          <table class="data-table">
            <thead>
              <tr>{#each tableColumns as col}<th>{col}</th>{/each}</tr>
            </thead>
            <tbody>
              {#each tableResult.slice(0, 100) as row}
                <tr>{#each tableColumns as col}<td title={cellValue(row[col])}>{cellValue(row[col])}</td>{/each}</tr>
              {/each}
            </tbody>
          </table>
        </div>

      {:else}
        <!-- Scalar result -->
        <div class="scalar-result">
          <span class="scalar-value">{scalarResult()}</span>
          <span class="scalar-label">({result.operation} of {result.column})</span>
        </div>
      {/if}
    </section>
  {/if}
</div>

<style>
  .page { max-width:100%; }
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.2rem; }
  .subtitle { color:#94A3B8;font-size:0.78rem;margin:0 0 1rem; }
  .two-col { display:grid;grid-template-columns:1fr 1fr;gap:0.75rem;margin-bottom:0.75rem; }
  @media (max-width:900px) { .two-col { grid-template-columns:1fr; } }

  .panel { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:0.9rem;display:flex;flex-direction:column;gap:0.5rem; }
  .panel-title { font-family:'Orbitron',sans-serif;font-size:0.8rem;color:#E6EDF3;margin:0 0 0.25rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .flex-1 { flex:1; }
  .lbl { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.4rem 0.6rem;font-size:0.8rem;font-family:'Inter',sans-serif;width:100%;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .textarea { background:#0B0F1A;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.5rem;font-size:0.72rem;font-family:'Inter',sans-serif;resize:vertical;width:100%;box-sizing:border-box; }
  .textarea.mono { font-family:'Fira Code','Consolas',monospace; }
  .textarea:focus { outline:none;border-color:#3ABEFF; }
  .load-row { display:flex;gap:0.4rem;align-items:flex-end; }
  .btn-primary { padding:0.45rem 1.2rem;background:linear-gradient(135deg,#3ABEFF,#8B5CF6);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.82rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-secondary { padding:0.4rem 0.8rem;background:#1E293B;border:1px solid rgba(58,190,255,0.2);border-radius:6px;color:#E6EDF3;font-size:0.8rem;cursor:pointer;white-space:nowrap; }
  .btn-secondary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-sm { padding:0.25rem 0.65rem;background:#1E293B;border:1px solid rgba(58,190,255,0.2);border-radius:5px;color:#E6EDF3;font-size:0.75rem;cursor:pointer; }
  .error { color:#EF4444;font-size:0.75rem;margin:0; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .dataset-info { display:flex;flex-wrap:wrap;gap:0.3rem;padding-top:0.25rem; }
  .pill { background:rgba(58,190,255,0.1);color:#3ABEFF;border-radius:4px;padding:0.1rem 0.4rem;font-size:0.68rem; }
  .pill-cols { color:#94A3B8;font-size:0.68rem;align-self:center; }
  .op-badge { background:rgba(139,92,246,0.2);color:#C084FC;padding:0.1rem 0.4rem;border-radius:4px;font-size:0.7rem;font-family:'Inter',sans-serif; }
  code { background:#0E1428;padding:0.1rem 0.3rem;border-radius:3px;font-size:0.75rem;color:#93C5FD; }

  /* Result panel */
  .result-panel { background:#0E1428;border:1px solid rgba(139,92,246,0.2);border-radius:8px;padding:0.9rem; }
  .chart-label { font-size:0.78rem;color:#94A3B8;margin:0 0 0.5rem; }
  .chart-wrap { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:6px;padding:0.5rem;overflow-x:auto; }
  .chart-svg { min-width:400px; }
  .data-details summary { cursor:pointer;font-size:0.75rem;color:#94A3B8;margin-top:0.5rem; }
  .payload { background:#0B0F1A;border:1px solid #1E293B;border-radius:4px;padding:0.5rem;font-size:0.7rem;color:#93C5FD;overflow-x:auto;max-height:150px; }
  .result-meta { font-size:0.78rem;color:#94A3B8;margin:0 0 0.5rem; }
  .scalar-result { display:flex;flex-direction:column;align-items:flex-start;gap:0.2rem;padding:0.75rem; }
  .scalar-value { font-size:2rem;font-family:'Orbitron',sans-serif;color:#3ABEFF; }
  .scalar-label { font-size:0.75rem;color:#94A3B8; }
  .table-wrap { overflow-x:auto;border:1px solid rgba(58,190,255,0.1);border-radius:6px;max-height:300px;overflow-y:auto; }
  .data-table { width:100%;border-collapse:collapse;font-size:0.72rem; }
  .data-table th { background:#0E1428;color:#3ABEFF;padding:0.35rem 0.5rem;text-align:left;border-bottom:1px solid rgba(58,190,255,0.2);white-space:nowrap;position:sticky;top:0;z-index:1; }
  .data-table td { padding:0.25rem 0.5rem;color:#E6EDF3;border-bottom:1px solid rgba(255,255,255,0.04);max-width:180px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .data-table tr:hover td { background:rgba(58,190,255,0.04); }
</style>
