<!--
  /directors/personnel-logs — Combined termination + relocation log for all directors.
  Uses get_personnel_activity_log backend command.
  Theme: blue #3ABEFF (directors default).
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getPersonnelActivityLog, type PersonnelActivityEntry } from '$lib/stores/directors';

  let entries: PersonnelActivityEntry[] = $state([]);
  let loading = $state(true);
  let error: string | null = $state(null);
  let filter = $state('all'); // 'all' | 'termination' | 'relocation'
  let search = $state('');

  onMount(async () => {
    try {
      entries = await getPersonnelActivityLog();
    } catch (e: unknown) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  const filtered = $derived(
    entries.filter((e) => {
      if (filter !== 'all' && e.entry_type !== filter) return false;
      if (search.trim()) {
        const q = search.toLowerCase();
        return (
          e.target_name.toLowerCase().includes(q) ||
          e.actor_name.toLowerCase().includes(q) ||
          e.description.toLowerCase().includes(q)
        );
      }
      return true;
    })
  );

  function typeLabel(t: string) {
    return t === 'termination' ? 'Termination' : 'Relocation';
  }
  function typeClass(t: string) {
    return t === 'termination' ? 'badge-term' : 'badge-relo';
  }
</script>

<div class="page">
  <h2>Personnel Logs</h2>
  <p class="subtitle">Combined record of terminations and relocations across all personnel.</p>

  {#if error}
    <div class="error">{error}</div>
  {:else if loading}
    <div class="loading">Loading…</div>
  {:else}
    <div class="controls">
      <input type="text" bind:value={search} placeholder="Search by name or description…" class="search" />
      <div class="filter-row">
        {#each ['all', 'termination', 'relocation'] as f}
          <button
            class="filter-btn"
            class:active={filter === f}
            onclick={() => (filter = f)}
          >
            {f === 'all' ? 'All' : typeLabel(f)}
          </button>
        {/each}
      </div>
    </div>

    {#if filtered.length === 0}
      <div class="empty">No records found.</div>
    {:else}
      <div class="count">{filtered.length} record{filtered.length !== 1 ? 's' : ''}</div>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Type</th>
              <th>Person</th>
              <th>By</th>
              <th>Details</th>
              <th>Effective Date</th>
              <th>Logged At</th>
            </tr>
          </thead>
          <tbody>
            {#each filtered as entry}
              <tr>
                <td><span class="badge {typeClass(entry.entry_type)}">{typeLabel(entry.entry_type)}</span></td>
                <td class="name-cell">{entry.target_name}</td>
                <td class="name-cell">{entry.actor_name}</td>
                <td class="desc-cell">{entry.description}</td>
                <td>{entry.effective_date}</td>
                <td>{new Date(entry.created_at).toLocaleDateString()}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  {/if}
</div>

<style>
  .page { padding:2rem;max-width:1100px;margin:0 auto; }
  h2 { color:#3ABEFF;font-family:'Orbitron',sans-serif;margin-bottom:0.25rem; }
  .subtitle { color:#94A3B8;font-size:0.85rem;margin-bottom:1.5rem; }
  .error { background:rgba(239,68,68,0.1);border:1px solid #EF4444;color:#EF4444;padding:0.75rem 1rem;border-radius:6px;margin-bottom:1rem;font-size:0.85rem; }
  .loading,.empty { color:#94A3B8;font-size:0.9rem;padding:1rem 0; }
  .controls { display:flex;flex-direction:column;gap:0.6rem;margin-bottom:1rem; }
  .search { background:#1F2937;border:1px solid #374151;color:#E6EDF3;border-radius:6px;padding:0.5rem 0.75rem;font-size:0.9rem;width:100%;max-width:400px; }
  .search:focus { outline:none;border-color:#3ABEFF; }
  .filter-row { display:flex;gap:0.4rem; }
  .filter-btn { background:#1F2937;border:1px solid #374151;color:#94A3B8;border-radius:4px;padding:0.25rem 0.65rem;font-size:0.8rem;cursor:pointer; }
  .filter-btn.active { border-color:#3ABEFF;color:#3ABEFF; }
  .count { color:#64748B;font-size:0.8rem;margin-bottom:0.6rem; }
  .table-wrap { overflow-x:auto; }
  table { width:100%;border-collapse:collapse;font-size:0.85rem; }
  th { text-align:left;padding:0.5rem 0.75rem;color:#64748B;border-bottom:1px solid #1F2937;font-weight:500; }
  td { padding:0.5rem 0.75rem;border-bottom:1px solid rgba(255,255,255,0.04);color:#E6EDF3; }
  tr:hover td { background:rgba(58,190,255,0.04); }
  .name-cell { color:#CBD5E1; }
  .desc-cell { color:#94A3B8;max-width:280px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis; }
  .badge { display:inline-block;padding:0.15rem 0.5rem;border-radius:4px;font-size:0.75rem;font-weight:600; }
  .badge-term { background:rgba(239,68,68,0.15);color:#EF4444; }
  .badge-relo { background:rgba(58,190,255,0.15);color:#3ABEFF; }
</style>
