<!--
  Observer: Science Archive view — species and matter entries.
  Access: TheObserver
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { dirGetScienceArchive, type DirScienceArchiveRow } from '$lib/stores/directors';

  let entries: DirScienceArchiveRow[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    loading = true; error = '';
    try { entries = await dirGetScienceArchive(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  });

  function typeLabel(t: string): string {
    const m: Record<string, string> = { physical_object: 'Physical Object', species: 'Species', matter: 'Matter' };
    return m[t] ?? t;
  }

  function typeColor(t: string): string {
    const m: Record<string, string> = { physical_object: '#8B5CF6', species: '#22C55E', matter: '#3ABEFF' };
    return m[t] ?? '#94A3B8';
  }
</script>

<h2 class="title">Species and Matter Archive</h2>
<p class="subtitle">Approved species and matter entries from your domain's personnel.</p>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if entries.length === 0}
  <p class="muted">No archive entries found.</p>
{:else}
  <div class="table-wrap">
    <table class="table">
      <thead>
        <tr>
          <th>Name</th>
          <th>Type</th>
          <th>Classification</th>
          <th>Submitted By</th>
          <th>Approved By</th>
          <th>Date</th>
        </tr>
      </thead>
      <tbody>
        {#each entries as e}
          <tr>
            <td class="name-cell">{e.name}</td>
            <td>
              <span class="type-badge" style="color:{typeColor(e.archive_type)}">{typeLabel(e.archive_type)}</span>
            </td>
            <td>{e.classification ?? '—'}</td>
            <td>{e.submitter_name}</td>
            <td>{e.approver_name ?? '—'}</td>
            <td class="mono">{new Date(e.created_at).toLocaleDateString()}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.25rem; }
  .subtitle { color:#64748B;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .error { color:#EF4444;font-size:0.82rem; }
  .table-wrap { overflow-x:auto; }
  .table { width:100%;border-collapse:collapse;font-size:0.78rem; }
  .table th { text-align:left;color:#64748B;font-weight:600;padding:0.5rem 0.6rem;border-bottom:1px solid rgba(58,190,255,0.1); }
  .table td { padding:0.45rem 0.6rem;color:#E6EDF3;border-bottom:1px solid rgba(255,255,255,0.04); }
  .table tr:hover td { background:rgba(58,190,255,0.03); }
  .name-cell { font-weight:600; }
  .type-badge { font-weight:700;font-size:0.72rem; }
  .mono { font-size:0.72rem;color:#64748B; }
</style>
