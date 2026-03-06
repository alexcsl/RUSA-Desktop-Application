<!-- /astronauts/missions — UC-AS-01: Mission Inbox -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getMyMissions, type MissionSummary } from '$lib/stores/astronauts';

  let missions: MissionSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try {
      missions = await getMyMissions();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  function dangerClass(level: string | null): string {
    switch (level) {
      case 'critical': return 'danger-critical';
      case 'high': return 'danger-high';
      case 'medium': return 'danger-medium';
      default: return 'danger-low';
    }
  }

  function statusClass(status: string): string {
    switch (status) {
      case 'active': return 'status-active';
      case 'completion_requested': return 'status-pending';
      case 'completed': return 'status-completed';
      case 'rejected': return 'status-rejected';
      default: return '';
    }
  }

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleDateString('en-GB', { day:'2-digit', month:'short', year:'numeric' });
  }
</script>

<h1 class="title">Mission Inbox</h1>
<p class="subtitle">Your currently assigned and completed missions.</p>

{#if loading}
  <p class="loading">Loading missions…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if missions.length === 0}
  <div class="empty-state">
    <p>No missions assigned yet.</p>
  </div>
{:else}
  <div class="table-wrap">
    <table>
      <thead>
        <tr>
          <th>Title</th>
          <th>Type</th>
          <th>Danger</th>
          <th>Location</th>
          <th>Status</th>
          <th>Assigned By</th>
          <th>Date</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each missions as m}
          <tr>
            <td class="cell-title">{m.title}</td>
            <td><span class="badge type-badge">{m.mission_type}</span></td>
            <td><span class="badge {dangerClass(m.danger_level)}">{m.danger_level ?? 'n/a'}</span></td>
            <td>{m.location}</td>
            <td><span class="badge {statusClass(m.status)}">{m.status.replace(/_/g, ' ')}</span></td>
            <td class="cell-muted">{m.assigned_by_name}</td>
            <td class="cell-muted">{formatDate(m.created_at)}</td>
            <td>
              <a class="btn-link" href="/astronauts/missions/{m.id}">View</a>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .loading { color:#94A3B8;font-size:0.85rem; }
  .error { color:#EF4444;font-size:0.85rem; }
  .empty-state { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:2rem;text-align:center;color:#475569;font-size:0.85rem; }

  .table-wrap { overflow-x:auto; }
  table { width:100%;border-collapse:collapse;font-size:0.8rem; }
  thead th { text-align:left;color:#94A3B8;font-weight:600;font-size:0.7rem;text-transform:uppercase;padding:0.5rem 0.6rem;border-bottom:1px solid rgba(58,190,255,0.1); }
  tbody tr { border-bottom:1px solid rgba(255,255,255,0.03); }
  tbody tr:hover { background:rgba(58,190,255,0.03); }
  td { padding:0.5rem 0.6rem;vertical-align:middle; }
  .cell-title { font-weight:500; }
  .cell-muted { color:#94A3B8;font-size:0.75rem; }

  .badge { display:inline-block;padding:0.12rem 0.4rem;border-radius:4px;font-size:0.65rem;font-weight:600;text-transform:capitalize; }
  .type-badge { background:rgba(139,92,246,0.15);color:#C084FC; }
  .danger-low { background:rgba(16,185,129,0.15);color:#10B981; }
  .danger-medium { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .danger-high { background:rgba(249,115,22,0.15);color:#F97316; }
  .danger-critical { background:rgba(239,68,68,0.2);color:#EF4444; }
  .status-active { background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .status-pending { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .status-completed { background:rgba(16,185,129,0.15);color:#10B981; }
  .status-rejected { background:rgba(239,68,68,0.15);color:#EF4444; }

  .btn-link { color:#3ABEFF;text-decoration:none;font-size:0.75rem;font-weight:600; }
  .btn-link:hover { text-decoration:underline; }
</style>
