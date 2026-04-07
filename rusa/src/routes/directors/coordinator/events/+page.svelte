<!--
  /directors/coordinator/events — Event List (UC-CORD-02)
  TheCoordinator views all organization events.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getEvents, type EventSummary } from '$lib/stores/directors';

  let events: EventSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    loading = true; error = '';
    try { events = await getEvents(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  });

  function formatDate(d: string) { return new Date(d).toLocaleString(); }
  function statusColor(s: string) {
    const m: Record<string, string> = { upcoming:'#3ABEFF', active:'#22C55E', completed:'#8B5CF6', cancelled:'#EF4444' };
    return m[s] ?? '#94A3B8';
  }
</script>

<h1 class="title">Events</h1>
<p class="subtitle">All scheduled organization events.</p>

<div class="header-actions">
  <a href="/directors/coordinator/events/new" class="btn-new">+ New Event</a>
</div>

{#if loading}<p class="muted">Loading…</p>
{:else if error}<p class="error">{error}</p>
{:else if events.length === 0}<p class="muted">No events scheduled.</p>
{:else}
  <div class="event-list">
    {#each events as ev}
      <div class="card">
        <div class="card-header">
          <span class="event-title">{ev.title}</span>
          <span class="status-badge" style="color:{statusColor(ev.status)}">{ev.status}</span>
        </div>
        <p class="meta">
          {formatDate(ev.starts_at)}
          {#if ev.ends_at} → {formatDate(ev.ends_at)}{/if}
        </p>
        {#if ev.description}<p class="desc">{ev.description}</p>{/if}
        <a class="docs-link" href="/directors/coordinator/events/{ev.id}/documents">View Documents →</a>
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .header-actions { margin-bottom:1rem; }
  .btn-new { display:inline-block;background:rgba(58,190,255,0.12);border:1px solid rgba(58,190,255,0.35);color:#3ABEFF;border-radius:6px;padding:0.35rem 0.8rem;font-size:0.8rem;text-decoration:none; }
  .btn-new:hover { background:rgba(58,190,255,0.22); }
  .muted { color:#475569;font-size:0.82rem;font-style:italic; }
  .event-list { display:flex;flex-direction:column;gap:0.6rem; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:0.9rem;display:flex;flex-direction:column;gap:0.3rem; }
  .card-header { display:flex;justify-content:space-between;align-items:center; }
  .event-title { font-size:0.9rem;font-weight:600;color:#E6EDF3; }
  .status-badge { font-size:0.7rem;font-weight:600;text-transform:capitalize; }
  .meta { font-size:0.72rem;color:#64748B;margin:0; }
  .desc { font-size:0.8rem;color:#94A3B8;margin:0; }
  .docs-link { font-size:0.73rem;color:#8B5CF6;text-decoration:none;align-self:flex-start;margin-top:0.2rem; }
  .docs-link:hover { text-decoration:underline; }
  .error { color:#EF4444;font-size:0.75rem; }
</style>
