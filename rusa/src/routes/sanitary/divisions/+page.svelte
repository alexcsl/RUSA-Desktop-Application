<!-- /sanitary/divisions — View Internal Sanitary Department Divisions (all sanitary staff) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { sanGetDivisions, type DivisionRow } from '$lib/stores/sanitary';

  const DIVISION_INFO: Record<string, { description: string; icon: string }> = {
    'Inspector Crew':   { icon: '🔍', description: 'Conducts scheduled and ad-hoc facility inspections; files inspection reports.' },
    'Disposal Crew':    { icon: '♻️', description: 'Handles waste collection, classification, and safe disposal; maintains disposal docs.' },
    'Wastewater Crew':  { icon: '💧', description: 'Operates wastewater treatment processes and documents procedures.' },
    'Cleanup Crew':     { icon: '🧹', description: 'General facility cleaning, decontamination, and routine sanitation tasks.' },
    'Transport Crew':   { icon: '🚛', description: 'Transports materials and waste streams between facility zones.' },
  };

  function info(name: string) {
    return DIVISION_INFO[name] ?? { icon: '🏢', description: 'Internal sanitary division.' };
  }

  let divisions: DivisionRow[] = $state([]);
  let loading = $state(false);
  let error = $state('');

  onMount(async () => {
    loading = true;
    error = '';
    try {
      divisions = await sanGetDivisions();
    } catch (e: unknown) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  function fillPct(div: DivisionRow): number {
    if (div.quota <= 0) return 0;
    return Math.min(100, Math.round((div.staff_count / div.quota) * 100));
  }

  function fillClass(pct: number): string {
    if (pct >= 100) return 'fill-full';
    if (pct >= 60)  return 'fill-ok';
    if (pct >= 30)  return 'fill-low';
    return 'fill-empty';
  }
</script>

<h1 class="title">Sanitary Department Divisions</h1>
<p class="subtitle">Internal divisions, their roles, and current staffing vs. quota.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if loading}<p class="loading">Loading divisions…</p>{/if}

{#if !loading && divisions.length > 0}
  <div class="divisions-grid">
    {#each divisions as div}
      {@const pct = fillPct(div)}
      {@const fc = fillClass(pct)}
      {@const meta = info(div.name)}
      <div class="div-card">
        <div class="div-header">
          <span class="div-icon">{meta.icon}</span>
          <h2 class="div-name">{div.name}</h2>
        </div>
        <p class="div-desc">{meta.description}</p>

        <div class="staffing">
          <div class="staffing-row">
            <span class="staffing-label">Staffing</span>
            <span class="staffing-val">
              <strong class:overfull={div.staff_count > div.quota}>{div.staff_count}</strong>
              <span class="quota-sep">/</span>
              <span class="quota-num">{div.quota}</span>
            </span>
          </div>
          <div class="fill-bar-bg">
            <div class="fill-bar {fc}" style="width:{pct}%"></div>
          </div>
          <span class="fill-pct {fc}">{pct}% filled</span>
        </div>
      </div>
    {/each}
  </div>
{:else if !loading}
  <p class="empty">No divisions found.</p>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#F59E0B;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1.25rem; }
  .loading,.error { color:#94A3B8;font-size:0.8rem; }
  .error { color:#EF4444; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:2rem; }

  .divisions-grid { display:grid;grid-template-columns:repeat(auto-fill,minmax(240px,1fr));gap:0.85rem; }
  .div-card { background:rgba(14,20,40,0.6);border:1px solid rgba(245,158,11,0.15);border-radius:10px;padding:1rem 1rem 0.85rem;display:flex;flex-direction:column;gap:0.55rem; }
  .div-header { display:flex;align-items:center;gap:0.5rem; }
  .div-icon { font-size:1.2rem;line-height:1; }
  .div-name { font-family:'Orbitron',sans-serif;font-size:0.82rem;color:#F59E0B;margin:0; }
  .div-desc { color:#64748B;font-size:0.74rem;line-height:1.45;margin:0; }

  .staffing { display:flex;flex-direction:column;gap:0.25rem;margin-top:0.2rem; }
  .staffing-row { display:flex;justify-content:space-between;align-items:baseline; }
  .staffing-label { font-size:0.68rem;color:#64748B;text-transform:uppercase;letter-spacing:0.03em; }
  .staffing-val { font-size:0.82rem; }
  .staffing-val strong { color:#E6EDF3;font-size:1rem; }
  .staffing-val strong.overfull { color:#F87171; }
  .quota-sep { color:#475569;margin:0 0.15rem; }
  .quota-num { color:#64748B; }

  .fill-bar-bg { height:5px;background:rgba(255,255,255,0.06);border-radius:3px;overflow:hidden; }
  .fill-bar { height:100%;border-radius:3px;transition:width 0.4s ease; }
  .fill-bar.fill-full  { background:#F59E0B; }
  .fill-bar.fill-ok    { background:#34D399; }
  .fill-bar.fill-low   { background:#FBBF24; }
  .fill-bar.fill-empty { background:#EF4444; }

  .fill-pct { font-size:0.68rem;text-align:right; }
  .fill-pct.fill-full  { color:#F59E0B; }
  .fill-pct.fill-ok    { color:#34D399; }
  .fill-pct.fill-low   { color:#FBBF24; }
  .fill-pct.fill-empty { color:#EF4444; }
</style>
