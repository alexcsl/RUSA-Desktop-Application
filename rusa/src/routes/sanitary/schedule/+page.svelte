<!-- /sanitary/schedule — UC-STAS-02: View My Schedule -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { sanGetMySchedule, type SanitaryShift } from '$lib/stores/sanitary';

  let shifts: SanitaryShift[] = $state([]);
  let error = $state('');

  onMount(async () => {
    try { shifts = await sanGetMySchedule(); } catch (e: unknown) { error = String(e); }
  });

  function dayLabel(dt: string): string {
    return new Date(dt).toLocaleDateString(undefined, { weekday: 'short', month: 'short', day: 'numeric' });
  }

  function timeLabel(dt: string): string {
    return new Date(dt).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
  }

  function dateKey(dt: string): string {
    return new Date(dt).toISOString().slice(0, 10);
  }

  function isToday(dt: string): boolean {
    return dateKey(dt) === new Date().toISOString().slice(0, 10);
  }
  function isPast(dt: string): boolean {
    return new Date(dt) < new Date();
  }
</script>

<h1 class="title">My Schedule</h1>
<p class="subtitle">UC-STAS-02 — View your assigned shifts across sanitary divisions.</p>

{#if error}<p class="error">{error}</p>{/if}

{#if shifts.length === 0}
  <p class="empty">No shifts scheduled for you.</p>
{:else}
  <div class="timeline">
    {#each shifts as s}
      <div class="shift" class:today={isToday(s.shift_start)} class:past={isPast(s.shift_end)}>
        <div class="shift-date">{dayLabel(s.shift_start)}</div>
        <div class="shift-body">
          <span class="shift-time">{timeLabel(s.shift_start)} – {timeLabel(s.shift_end)}</span>
          {#if s.quarter}<span class="shift-div">{s.quarter}</span>{/if}
        </div>
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .timeline { display:flex;flex-direction:column;gap:0.4rem; }
  .shift { display:flex;gap:0.8rem;align-items:center;background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.06);border-radius:6px;padding:0.55rem 0.7rem;transition:border-color .15s; }
  .shift.today { border-color:#F59E0B;background:rgba(245,158,11,0.06); }
  .shift.past { opacity:0.5; }
  .shift-date { min-width:110px;font-size:0.78rem;font-weight:500;color:#E6EDF3; }
  .shift-body { display:flex;gap:0.6rem;align-items:center; }
  .shift-time { font-size:0.78rem;color:#3ABEFF;font-weight:500;min-width:110px; }
  .shift-div { font-size:0.75rem;color:#94A3B8; }
  .error { color:#EF4444;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.85rem;text-align:center;padding:2rem; }
</style>
