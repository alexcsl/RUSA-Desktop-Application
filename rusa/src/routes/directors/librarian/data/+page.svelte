<!--
  /directors/librarian/data — TheLibrarian's data access panel.
  Allows browsing any table (full access including medical, bypasses ALLOWED_BROWSE_TABLES),
  and soft-deleting records from archiveable tables.
  Theme: purple #8B5CF6 (librarian).
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { libSoftDeleteRecord, type LibSoftDeletePayload } from '$lib/stores/directors';
  import { browseData, type BrowseDataPayload } from '$lib/stores/data_analysts';

  const TABLES = [
    'document_restrictions',
    'document_redactions',
    'event_documents',
    'personnel_relocations',
    'broadcast_requests',
    'tasks',
    'meetings',
    'users',
    'experiments',
    'incident_reports',
    'termination_records',
    'notifications',
  ];

  const DELETABLE = [
    'document_restrictions',
    'document_redactions',
    'event_documents',
    'personnel_relocations',
    'broadcast_requests',
    'tasks',
    'meetings',
  ];

  let selectedTable = $state('document_restrictions');
  let filterColumn = $state('');
  let filterValue = $state('');
  let browseRows: Record<string, unknown>[] | null = $state(null);
  let browseLoading = $state(false);
  let browseError: string | null = $state(null);

  let deleteId = $state('');
  let deleteTable = $state('document_restrictions');
  let deleteReason = $state('');
  let deleteLoading = $state(false);
  let deleteResult: string | null = $state(null);
  let deleteError: string | null = $state(null);

  async function handleBrowse(e: Event) {
    e.preventDefault();
    browseLoading = true;
    browseError = null;
    browseRows = null;
    try {
      const payload: BrowseDataPayload = {
        table_name: selectedTable,
        filter_column: filterColumn.trim() || undefined,
        filter_value: filterValue.trim() || undefined,
        limit: 100,
      };
      browseRows = await browseData(payload);
    } catch (err: unknown) {
      browseError = String(err);
    } finally {
      browseLoading = false;
    }
  }

  async function handleDelete(e: Event) {
    e.preventDefault();
    if (!deleteId.trim()) return;
    deleteLoading = true;
    deleteResult = null;
    deleteError = null;
    try {
      const payload: LibSoftDeletePayload = {
        table_name: deleteTable,
        record_id: deleteId.trim(),
        reason: deleteReason.trim() || undefined,
      };
      await libSoftDeleteRecord(payload);
      deleteResult = `Record ${deleteId} soft-deleted from ${deleteTable}.`;
      deleteId = '';
      deleteReason = '';
    } catch (err: unknown) {
      deleteError = String(err);
    } finally {
      deleteLoading = false;
    }
  }

  function colKeys(rows: Record<string, unknown>[]): string[] {
    if (!rows.length) return [];
    return Object.keys(rows[0]);
  }
  function cellVal(v: unknown): string {
    if (v === null || v === undefined) return '—';
    if (typeof v === 'object') return JSON.stringify(v);
    return String(v);
  }
</script>

<div class="page">
  <h2>Data Access</h2>
  <p class="subtitle">Browse all system tables and manage record lifecycle.</p>

  <!-- Browse panel -->
  <section class="panel">
    <h3>Browse Table</h3>
    <form class="browse-form" onsubmit={handleBrowse}>
      <label>
        Table
        <select bind:value={selectedTable}>
          {#each TABLES as t}
            <option value={t}>{t}</option>
          {/each}
        </select>
      </label>
      <label>
        Filter Column (optional)
        <input type="text" bind:value={filterColumn} placeholder="e.g. status" />
      </label>
      <label>
        Filter Value (optional)
        <input type="text" bind:value={filterValue} placeholder="e.g. pending" />
      </label>
      <button type="submit" class="btn-primary" disabled={browseLoading}>
        {browseLoading ? 'Loading…' : 'Browse'}
      </button>
    </form>

    {#if browseError}
      <div class="error">{browseError}</div>
    {:else if browseRows !== null}
      {#if browseRows.length === 0}
        <div class="empty">No rows found.</div>
      {:else}
        <div class="count">{browseRows.length} row{browseRows.length !== 1 ? 's' : ''} returned.</div>
        <div class="table-wrap">
          <table>
            <thead>
              <tr>
                {#each colKeys(browseRows) as col}
                  <th>{col}</th>
                {/each}
              </tr>
            </thead>
            <tbody>
              {#each browseRows as row}
                <tr>
                  {#each colKeys(browseRows) as col}
                    <td>{cellVal(row[col])}</td>
                  {/each}
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    {/if}
  </section>

  <!-- Delete panel -->
  <section class="panel">
    <h3>Soft-Delete Record</h3>
    <p class="hint">Marks a record as deleted without removing it from the database.</p>
    <form class="delete-form" onsubmit={handleDelete}>
      <label>
        Table
        <select bind:value={deleteTable}>
          {#each DELETABLE as t}
            <option value={t}>{t}</option>
          {/each}
        </select>
      </label>
      <label>
        Record ID (UUID) <span class="req">*</span>
        <input type="text" bind:value={deleteId} placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" required />
      </label>
      <label>
        Reason (optional)
        <input type="text" bind:value={deleteReason} placeholder="Reason for deletion" />
      </label>
      <button type="submit" class="btn-danger" disabled={deleteLoading || !deleteId.trim()}>
        {deleteLoading ? 'Deleting…' : 'Soft-Delete'}
      </button>
    </form>
    {#if deleteResult}
      <div class="success">{deleteResult}</div>
    {/if}
    {#if deleteError}
      <div class="error">{deleteError}</div>
    {/if}
  </section>

  <div class="quick-links">
    <a href="/directors/librarian/redact" class="quick-link">→ Redact Document Field</a>
    <a href="/directors/librarian/restrictions" class="quick-link">→ Manage Access Restrictions</a>
  </div>
</div>

<style>
  .page { padding:2rem;max-width:1100px;margin:0 auto; }
  h2 { color:#8B5CF6;font-family:'Orbitron',sans-serif;margin-bottom:0.25rem; }
  .subtitle { color:#94A3B8;font-size:0.85rem;margin-bottom:1.5rem; }
  .panel { background:#111827;border:1px solid rgba(139,92,246,0.2);border-radius:8px;padding:1.25rem;margin-bottom:1.5rem; }
  h3 { color:#C084FC;font-size:1rem;margin:0 0 0.75rem; }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:0.75rem; }
  .browse-form,.delete-form { display:flex;flex-wrap:wrap;gap:0.75rem;align-items:flex-end;margin-bottom:1rem; }
  label { display:flex;flex-direction:column;gap:0.25rem;font-size:0.8rem;color:#94A3B8;flex:1;min-width:160px; }
  input,select { background:#1F2937;border:1px solid #374151;color:#E6EDF3;border-radius:6px;padding:0.45rem 0.65rem;font-size:0.85rem; }
  input:focus,select:focus { outline:none;border-color:#8B5CF6; }
  .btn-primary { background:#8B5CF6;color:#fff;border:none;border-radius:6px;padding:0.5rem 1.2rem;font-weight:600;cursor:pointer;font-size:0.85rem;white-space:nowrap;align-self:flex-end; }
  .btn-primary:hover:not(:disabled) { background:#7C3AED; }
  .btn-danger { background:#DC2626;color:#fff;border:none;border-radius:6px;padding:0.5rem 1.2rem;font-weight:600;cursor:pointer;font-size:0.85rem;white-space:nowrap;align-self:flex-end; }
  .btn-danger:hover:not(:disabled) { background:#B91C1C; }
  button:disabled { opacity:0.5;cursor:not-allowed; }
  .success { background:rgba(16,185,129,0.1);border:1px solid #10B981;color:#10B981;padding:0.6rem 0.75rem;border-radius:6px;font-size:0.82rem;margin-top:0.5rem; }
  .error { background:rgba(239,68,68,0.1);border:1px solid #EF4444;color:#EF4444;padding:0.6rem 0.75rem;border-radius:6px;font-size:0.82rem;margin-top:0.5rem; }
  .empty { color:#64748B;font-size:0.85rem;padding:0.5rem 0; }
  .count { color:#64748B;font-size:0.75rem;margin-bottom:0.5rem; }
  .table-wrap { overflow-x:auto;max-height:340px;overflow-y:auto; }
  table { width:100%;border-collapse:collapse;font-size:0.75rem; }
  th { text-align:left;padding:0.4rem 0.6rem;color:#64748B;border-bottom:1px solid #1F2937;font-weight:500;background:#0B0F1A;position:sticky;top:0; }
  td { padding:0.4rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04);color:#CBD5E1;max-width:200px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis; }
  .req { color:#EF4444; }
  .quick-links { display:flex;gap:1rem;flex-wrap:wrap; }
  .quick-link { color:#8B5CF6;font-size:0.85rem;text-decoration:none; }
  .quick-link:hover { color:#C084FC; }
</style>
