<!--
  /directors/taskmaster/conclusions — Concluded task closure requests (read-only)
  TheTaskmaster views all concluded/closed tasks.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { dirGetClosureRequestsForDirector, type DirClosureRequestRow } from '$lib/stores/directors';

  let conclusions: DirClosureRequestRow[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try { conclusions = await dirGetClosureRequestsForDirector(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  });

  function formatDate(d: string | null): string {
    return d ? new Date(d).toLocaleDateString() : '—';
  }
</script>

<h1 class="title">Task Conclusions</h1>
<p class="subtitle">Concluded tasks with closure summaries.</p>

{#if loading}<p class="muted">Loading…</p>
{:else if error}<p class="error">{error}</p>
{:else if conclusions.length === 0}<p class="muted">No concluded tasks found.</p>
{:else}
  <div class="list">
    {#each conclusions as item}
      <div class="card">
        <div class="card-header">
          <span class="card-title">{item.title}</span>
          <span class="date-tag">Completed: {formatDate(item.completed_at)}</span>
        </div>
        <p class="meta">Submitted by: <strong>{item.submitter_name}</strong> · Requested: {formatDate(item.created_at)}</p>
        {#if item.conclusion_summary}
          <p class="summary">{item.conclusion_summary}</p>
        {:else}
          <p class="muted-inline">No conclusion summary provided.</p>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .error { color:#EF4444;font-size:0.75rem; }
  .list { display:flex;flex-direction:column;gap:0.5rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.9rem;display:flex;flex-direction:column;gap:0.35rem; }
  .card-header { display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:0.4rem; }
  .card-title { font-size:0.88rem;font-weight:600;color:#E6EDF3; }
  .date-tag { font-size:0.68rem;color:#22C55E; }
  .meta { font-size:0.72rem;color:#64748B;margin:0; }
  .summary { font-size:0.8rem;color:#94A3B8;margin:0;line-height:1.45; }
  .muted-inline { font-size:0.75rem;color:#475569;font-style:italic;margin:0; }
</style>
