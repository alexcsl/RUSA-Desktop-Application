<!-- /medical/schedule — View My Work Schedule (UC-MED-04) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { medGetMyShifts, type ShiftEntry } from '$lib/stores/medical';

  let shifts: ShiftEntry[] = $state([]);
  let error = $state('');

  onMount(async () => {
    try { shifts = await medGetMyShifts(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  });

  function fmtDateTime(iso: string): string {
    return new Date(iso).toLocaleString('en-GB', {
      year: 'numeric', month: 'short', day: 'numeric',
      hour: '2-digit', minute: '2-digit',
    });
  }

  function shiftDuration(start: string, end: string): string {
    const ms = new Date(end).getTime() - new Date(start).getTime();
    const hrs = Math.floor(ms / 3600000);
    const mins = Math.floor((ms % 3600000) / 60000);
    return `${hrs}h${mins > 0 ? ` ${mins}m` : ''}`;
  }

  function isPast(end: string): boolean { return new Date(end) < new Date(); }
  function isCurrent(start: string, end: string): boolean {
    const now = new Date();
    return new Date(start) <= now && now <= new Date(end);
  }
</script>

<h1 class="title">My Work Schedule</h1>
<p class="subtitle">Upcoming and past shifts allocated to you.</p>

{#if error}<p class="error">{error}</p>{/if}

{#if shifts.length === 0}
  <div class="empty-card">
    <p>No shifts allocated yet.</p>
  </div>
{:else}
  <div class="shift-list">
    {#each shifts as s}
      <div class="shift-card" class:past={isPast(s.shift_end)} class:current={isCurrent(s.shift_start, s.shift_end)}>
        <div class="shift-header">
          <span class="shift-date">{fmtDateTime(s.shift_start)}</span>
          <span class="shift-sep">→</span>
          <span class="shift-date">{fmtDateTime(s.shift_end)}</span>
          <span class="shift-duration">{shiftDuration(s.shift_start, s.shift_end)}</span>
          {#if isCurrent(s.shift_start, s.shift_end)}
            <span class="badge badge-active">ACTIVE</span>
          {:else if isPast(s.shift_end)}
            <span class="badge badge-past">Completed</span>
          {:else}
            <span class="badge badge-upcoming">Upcoming</span>
          {/if}
        </div>
        {#if s.specialty}
          <div class="kv"><span class="k">Specialty:</span><span>{s.specialty}</span></div>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .shift-list { display:flex;flex-direction:column;gap:0.5rem;max-width:700px; }
  .shift-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.75rem; }
  .shift-card.current { border-color:#10B981;background:rgba(16,185,129,0.05); }
  .shift-card.past { opacity:0.5; }
  .shift-header { display:flex;align-items:center;gap:0.5rem;flex-wrap:wrap; }
  .shift-date { font-size:0.85rem;color:#E6EDF3;font-weight:500; }
  .shift-sep { color:#475569;font-size:0.85rem; }
  .shift-duration { color:#94A3B8;font-size:0.75rem;margin-left:auto; }
  .kv { display:flex;gap:0.5rem;font-size:0.8rem;margin-top:0.3rem; }
  .k { color:#94A3B8;min-width:80px; }
  .badge { padding:0.12rem 0.4rem;border-radius:4px;font-size:0.6rem;font-weight:600; }
  .badge-active { background:rgba(16,185,129,0.15);color:#10B981; }
  .badge-past { background:rgba(148,163,184,0.1);color:#94A3B8; }
  .badge-upcoming { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .error { color:#EF4444;font-size:0.8rem; }
  .empty-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:2rem;text-align:center;color:#475569; }
</style>
