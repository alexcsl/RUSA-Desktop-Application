<!--
  /directors/taskmaster/experiments — All experiments view (read-only)
  TheTaskmaster views all experiments across the organization.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { dirGetExperiments, type DirExperimentRow } from '$lib/stores/directors';

  let experiments: DirExperimentRow[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try { experiments = await dirGetExperiments(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  });

  function formatDate(d: string) { return new Date(d).toLocaleDateString(); }

  function statusColor(s: string): string {
    const m: Record<string, string> = {
      proposed: '#F59E0B', active: '#3ABEFF', completed: '#22C55E',
      denied: '#EF4444', approved: '#8B5CF6',
    };
    return m[s] ?? '#94A3B8';
  }

  function typeColor(t: string): string {
    const m: Record<string, string> = {
      physics: '#3ABEFF', chemistry: '#8B5CF6', biology: '#22C55E',
      matter: '#F59E0B', environmental: '#10B981',
    };
    return m[t.toLowerCase()] ?? '#94A3B8';
  }
</script>

<h1 class="title">Experiments</h1>
<p class="subtitle">All experiments across the organization.</p>

{#if loading}<p class="muted">Loading…</p>
{:else if error}<p class="error">{error}</p>
{:else if experiments.length === 0}<p class="muted">No experiments found.</p>
{:else}
  <div class="list">
    {#each experiments as exp}
      <div class="card">
        <div class="card-header">
          <span class="card-title">{exp.title}</span>
          <div class="badges">
            <span class="badge" style="color:{typeColor(exp.experiment_type)}">{exp.experiment_type}</span>
            <span class="badge" style="color:{statusColor(exp.status)}">{exp.status}</span>
          </div>
        </div>
        <p class="meta">Proposed by: <strong>{exp.proposer_name}</strong> · {formatDate(exp.created_at)}</p>
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
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.9rem;display:flex;flex-direction:column;gap:0.3rem; }
  .card-header { display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:0.4rem; }
  .card-title { font-size:0.88rem;font-weight:600;color:#E6EDF3; }
  .badges { display:flex;gap:0.5rem; }
  .badge { font-size:0.68rem;font-weight:600;text-transform:capitalize; }
  .meta { font-size:0.72rem;color:#64748B;margin:0; }
</style>
