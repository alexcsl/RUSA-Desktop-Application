<!--
  /directors/wanderer/missions — Active Missions Overview
  TheWanderer views all active missions and their status.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  interface MissionSummary {
    id: string;
    name: string;
    mission_type: string;
    status: string;
    lead_astronaut_name: string;
    started_at: string | null;
    estimated_end: string | null;
  }

  let missions: MissionSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    loading = true; error = '';
    try { missions = await invoke<MissionSummary[]>('ast_get_all_missions'); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  });

  function formatDate(d: string | null) { return d ? new Date(d).toLocaleDateString() : '—'; }
  function typeColor(t: string) {
    return t === 'interstellar' ? '#8B5CF6' : t === 'terrain' ? '#22C55E' : '#3ABEFF';
  }
  function statusColor(s: string) {
    const m: Record<string, string> = { active:'#3ABEFF', pending:'#F59E0B', completed:'#22C55E', aborted:'#EF4444' };
    return m[s] ?? '#94A3B8';
  }
</script>

<h1 class="title">Active Missions</h1>
<p class="subtitle">All ongoing interstellar and terrain missions.</p>

{#if loading}<p class="muted">Loading…</p>
{:else if error}<p class="error">{error}</p>
{:else if missions.length === 0}<p class="muted">No active missions.</p>
{:else}
  <div class="grid">
    {#each missions as m}
      <div class="card">
        <div class="card-top">
          <span class="mission-name">{m.name}</span>
          <span class="type-badge" style="color:{typeColor(m.mission_type)}">{m.mission_type}</span>
        </div>
        <span class="status-badge" style="color:{statusColor(m.status)}">{m.status}</span>
        <p class="meta">Lead: <strong>{m.lead_astronaut_name}</strong></p>
        <p class="meta">Started: {formatDate(m.started_at)} · Est. End: {formatDate(m.estimated_end)}</p>
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .grid { display:grid;grid-template-columns:repeat(auto-fill,minmax(280px,1fr));gap:0.75rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.9rem;display:flex;flex-direction:column;gap:0.3rem; }
  .card-top { display:flex;justify-content:space-between;align-items:flex-start;gap:0.5rem; }
  .mission-name { font-size:0.88rem;font-weight:600;color:#E6EDF3; }
  .type-badge { font-size:0.68rem;font-weight:700;text-transform:uppercase;white-space:nowrap; }
  .status-badge { font-size:0.7rem;font-weight:600;text-transform:capitalize; }
  .meta { font-size:0.72rem;color:#64748B;margin:0; }
  .error { color:#EF4444;font-size:0.75rem; }
</style>
