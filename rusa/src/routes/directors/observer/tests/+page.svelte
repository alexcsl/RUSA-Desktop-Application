<!--
  Observer: Approved Tests view — shows tests approved for Biologist / Chemist / AgEng / BioEng.
  Access: TheObserver
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { dirGetApprovedTests, type DirApprovedTestRow } from '$lib/stores/directors';

  let tests: DirApprovedTestRow[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    loading = true; error = '';
    try { tests = await dirGetApprovedTests(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  });

  function formatDate(d: string): string {
    return new Date(d).toLocaleDateString();
  }
</script>

<h2 class="title">Approved Tests</h2>
<p class="subtitle">Tests approved for biological, chemical, agricultural, and biological engineering personnel.</p>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if tests.length === 0}
  <p class="muted">No approved tests found.</p>
{:else}
  <div class="table-wrap">
    <table class="table">
      <thead>
        <tr>
          <th>Name</th>
          <th>Category</th>
          <th>Scope</th>
          <th>Accepted</th>
          <th>Date</th>
        </tr>
      </thead>
      <tbody>
        {#each tests as t}
          <tr>
            <td class="name-cell">{t.name}</td>
            <td>{t.category}</td>
            <td>{t.applicable_scope.replace(/_/g, ' ')}</td>
            <td class="mono">{t.accepted_at ? formatDate(t.accepted_at) : '—'}</td>
            <td class="mono">{formatDate(t.created_at)}</td>
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
  .goal-cell { max-width:250px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:#94A3B8; }
  .mono { font-size:0.72rem;color:#64748B; }
</style>
