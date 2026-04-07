<!--
  /data/analyst/write — UC-DA-04: Write (insert) a row into any non-medical table.
  Data Analyst selects a target table, adds key→value field pairs, then submits.
-->
<script lang="ts">
  import { daWriteData } from '$lib/stores/data_analysts';

  // Same whitelist as the Rust backend — medical tables excluded.
  const ALLOWED_TABLES = [
    'users', 'roles', 'base_locations', 'experiments', 'experiment_daily_logs',
    'science_archive', 'species_archive', 'approved_tests', 'test_proposals',
    'missions', 'territory_names', 'events', 'event_documents', 'incident_reports',
    'daily_security_reports', 'data_requests', 'data_responses', 'progress_reports',
    'audit_log', 'tasks', 'vote_sessions', 'vote_records', 'settlements',
    'sanitary_tasks', 'sanitary_shifts', 'sanitary_divisions',
  ];

  interface Field { key: string; value: string; }

  let tableName = $state('');
  let fields: Field[] = $state([{ key: '', value: '' }]);
  let submitting = $state(false);
  let success = $state('');
  let error = $state('');

  function addField() {
    fields = [...fields, { key: '', value: '' }];
  }

  function removeField(i: number) {
    fields = fields.filter((_, idx) => idx !== i);
  }

  function updateKey(i: number, val: string) {
    fields = fields.map((f, idx) => idx === i ? { ...f, key: val } : f);
  }

  function updateValue(i: number, val: string) {
    fields = fields.map((f, idx) => idx === i ? { ...f, value: val } : f);
  }

  async function handleSubmit() {
    error = '';
    success = '';

    if (!tableName) { error = 'Select a table.'; return; }

    const validFields = fields.filter((f) => f.key.trim());
    if (validFields.length === 0) { error = 'Add at least one field.'; return; }

    // Build row_data object — attempt numeric coercion for bare numbers
    const row_data: Record<string, unknown> = {};
    for (const f of validFields) {
      const k = f.key.trim();
      const v = f.value.trim();
      const asNum = Number(v);
      row_data[k] = v !== '' && !isNaN(asNum) && v !== '' ? asNum : v;
    }

    submitting = true;
    try {
      await daWriteData({ table_name: tableName, row_data });
      success = `Row inserted into "${tableName}" successfully.`;
      fields = [{ key: '', value: '' }];
      tableName = '';
    } catch (e: unknown) {
      error = String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<div class="page">
  <h2>Write Data</h2>
  <p class="hint">Insert a new row into any allowed non-medical table. All writes are audit-logged.</p>

  {#if success}<div class="banner success">{success}</div>{/if}
  {#if error}<div class="banner error">{error}</div>{/if}

  <form class="form-card" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    <label class="field">
      <span class="label">Target Table <span class="req">*</span></span>
      <select class="input" bind:value={tableName}>
        <option value="">— Select table —</option>
        {#each ALLOWED_TABLES as t}
          <option value={t}>{t}</option>
        {/each}
      </select>
    </label>

    <div class="fields-section">
      <div class="fields-header">
        <span class="label">Fields <span class="req">*</span></span>
        <button type="button" class="btn-add" onclick={addField}>+ Add Field</button>
      </div>

      {#each fields as f, i}
        <div class="field-row">
          <input
            class="input col-input"
            placeholder="column_name"
            value={f.key}
            oninput={(e) => updateKey(i, (e.target as HTMLInputElement).value)}
          />
          <span class="eq">=</span>
          <input
            class="input val-input"
            placeholder="value"
            value={f.value}
            oninput={(e) => updateValue(i, (e.target as HTMLInputElement).value)}
          />
          {#if fields.length > 1}
            <button type="button" class="btn-remove" onclick={() => removeField(i)}>×</button>
          {/if}
        </div>
      {/each}
    </div>

    <button
      type="submit"
      class="btn-primary"
      disabled={submitting || !tableName || fields.every((f) => !f.key.trim())}
    >
      {submitting ? 'Inserting…' : 'Insert Row'}
    </button>
  </form>
</div>

<style>
  .page { max-width:640px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .hint { color:#64748B;font-size:0.8rem;margin:0 0 1.25rem; }
  .banner { padding:0.6rem 0.9rem;border-radius:6px;font-size:0.8rem;margin-bottom:1rem; }
  .banner.success { background:rgba(16,185,129,0.1);border:1px solid rgba(16,185,129,0.3);color:#34D399; }
  .banner.error { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);color:#F87171; }
  .form-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;display:flex;flex-direction:column;gap:0.85rem; }
  .field { display:flex;flex-direction:column;gap:0.25rem; }
  .label { font-size:0.72rem;color:#94A3B8; }
  .req { color:#EF4444; }
  .input { background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem 0.6rem;font-size:0.8rem; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  select.input { width:100%;box-sizing:border-box; }
  .fields-section { display:flex;flex-direction:column;gap:0.4rem; }
  .fields-header { display:flex;justify-content:space-between;align-items:center;margin-bottom:0.1rem; }
  .field-row { display:flex;align-items:center;gap:0.4rem; }
  .col-input { flex:1; }
  .val-input { flex:1.5; }
  .eq { color:#64748B;font-size:0.85rem; }
  .btn-add { background:rgba(58,190,255,0.08);border:1px solid rgba(58,190,255,0.25);color:#3ABEFF;border-radius:4px;padding:0.2rem 0.5rem;cursor:pointer;font-size:0.72rem; }
  .btn-add:hover { background:rgba(58,190,255,0.15); }
  .btn-remove { background:none;border:none;color:#EF4444;cursor:pointer;font-size:1rem;line-height:1;padding:0 0.2rem; }
  .btn-primary { padding:0.5rem 1.25rem;background:rgba(58,190,255,0.15);border:1px solid rgba(58,190,255,0.4);color:#3ABEFF;border-radius:6px;font-weight:600;cursor:pointer;font-size:0.85rem;align-self:flex-start; }
  .btn-primary:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.4;cursor:default; }
</style>
