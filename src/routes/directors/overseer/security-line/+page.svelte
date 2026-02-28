<!-- /directors/overseer/security-line — TheOverseer Read-Only Security View (UC-DIR-19) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getBroadcastRequestQueue, getSubordinateTasks, type BroadcastRequestSummary, type TaskSummary } from '$lib/stores/directors';

  let broadcasts: BroadcastRequestSummary[] = $state([]);
  let tasks: TaskSummary[] = $state([]);

  onMount(async () => {
    [broadcasts, tasks] = await Promise.all([getBroadcastRequestQueue(), getSubordinateTasks()]);
  });
</script>

<h1 class="title">Overseer — Security Overview</h1>
<p class="subtitle">Read-only view of security broadcasts and subordinate task status.</p>

<h2 class="section">Recent Security Broadcasts</h2>
<div class="list">
  {#each broadcasts.filter(b => b.type_ === 'security') as b}
    <div class="card">
      <div class="row"><span class="card-title">{b.subject}</span><span class="badge badge-{b.status}">{b.status}</span></div>
      <p class="card-body">{b.content}</p>
      <span class="meta">{new Date(b.created_at).toLocaleString()} — {b.requester_name}</span>
    </div>
  {:else}
    <p class="empty">No security broadcasts.</p>
  {/each}
</div>

<h2 class="section">Subordinate Tasks</h2>
<div class="list">
  {#each tasks as t}
    <div class="card">
      <div class="row"><span class="card-title">{t.title}</span><span class="badge">{t.status}</span></div>
      <span class="meta">{t.assignee_name}{t.due_date ? ` — Due: ${t.due_date}` : ''}</span>
    </div>
  {:else}
    <p class="empty">No tasks.</p>
  {/each}
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1.25rem; }
  .section { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:1rem 0 0.5rem; }
  .list { display:flex;flex-direction:column;gap:0.4rem;max-width:700px; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.7rem; }
  .row { display:flex;justify-content:space-between;align-items:center; }
  .card-title { font-size:0.85rem;font-weight:500; }
  .card-body { font-size:0.8rem;color:#94A3B8;margin:0.3rem 0 0; }
  .meta { font-size:0.7rem;color:#475569; }
  .badge { padding:0.1rem 0.35rem;border-radius:4px;font-size:0.65rem;background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .badge-pending { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .badge-approved { background:rgba(16,185,129,0.15);color:#10B981; }
  .badge-sent { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .empty { color:#475569;font-size:0.8rem; }
</style>
