<!--
  /admin — Administrator Dashboard (landing page)
  System-wide overview: user counts, vote stats, recent activity feed.
  Red accent theme matching admin layout.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getSystemStats,
    getRecentActivity,
    type SystemStats,
    type RecentActivity,
  } from '$lib/stores/administrator';

  let stats: SystemStats | null = $state(null);
  let activity: RecentActivity[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try {
      const [s, a] = await Promise.all([getSystemStats(), getRecentActivity(30)]);
      stats = s;
      activity = a;
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  function timeAgo(iso: string): string {
    const diff = Date.now() - new Date(iso).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return 'just now';
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    const days = Math.floor(hrs / 24);
    return `${days}d ago`;
  }

  function opColor(op: string): string {
    if (op === 'CREATE') return '#10B981';
    if (op === 'UPDATE') return '#3ABEFF';
    if (op === 'DELETE') return '#EF4444';
    return '#94A3B8';
  }
</script>

<div class="dashboard">
  <div class="header-row">
    <h1 class="title">System Dashboard</h1>
    <span class="subtitle">Administrator overview — all subsystems</span>
  </div>

  {#if loading}
    <p class="loading">Loading system stats...</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if stats}
    <!-- Stat Cards -->
    <div class="stat-grid">
      <div class="stat-card">
        <span class="stat-value">{stats.active_users}</span>
        <span class="stat-label">Active Users</span>
        <span class="stat-detail">{stats.deactivated_users} deactivated · {stats.terminated_users} terminated</span>
      </div>
      <div class="stat-card">
        <span class="stat-value vote-val">{stats.open_vote_sessions}</span>
        <span class="stat-label">Open Votes</span>
        <span class="stat-detail">{stats.decided_vote_sessions} decided · {stats.overridden_vote_sessions} overridden · {stats.terminated_vote_sessions} terminated</span>
      </div>
      <div class="stat-card">
        <span class="stat-value request-val">{stats.pending_requests}</span>
        <span class="stat-label">Pending Requests</span>
        <span class="stat-detail">{stats.active_meetings} upcoming meetings</span>
      </div>
      <div class="stat-card">
        <span class="stat-value audit-val">{stats.recent_audit_entries_24h}</span>
        <span class="stat-label">Audit Events (24h)</span>
        <span class="stat-detail">{stats.total_audit_entries.toLocaleString()} total entries</span>
      </div>
      <div class="stat-card">
        <span class="stat-value notif-val">{stats.total_notifications_unread}</span>
        <span class="stat-label">Unread Notifications</span>
        <span class="stat-detail">System-wide across all users</span>
      </div>
      <div class="stat-card">
        <span class="stat-value loc-val">{stats.total_base_locations}</span>
        <span class="stat-label">Base Locations</span>
        <span class="stat-detail">{stats.total_roles} roles configured</span>
      </div>
    </div>

    <!-- Recent Activity Feed -->
    <div class="activity-section">
      <h2 class="section-title">Recent Activity</h2>
      <div class="activity-list">
        {#each activity as item}
          <div class="activity-item">
            <span class="activity-op" style="color:{opColor(item.operation)}">{item.operation}</span>
            <span class="activity-summary">{item.summary ?? `${item.operation} on ${item.table_name}`}</span>
            <span class="activity-actor">{item.performer_name ?? 'System'}</span>
            <span class="activity-time">{timeAgo(item.performed_at)}</span>
          </div>
        {:else}
          <p class="empty">No recent activity.</p>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .dashboard { max-width: 1100px; }
  .header-row { display:flex;align-items:baseline;gap:0.75rem;margin-bottom:1.25rem; }
  .title { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#EF4444;margin:0; }
  .subtitle { color:#94A3B8;font-size:0.8rem; }
  .loading { color:#94A3B8;font-size:0.85rem; }
  .error { color:#EF4444;font-size:0.85rem; }

  .stat-grid { display:grid;grid-template-columns:repeat(3,1fr);gap:0.75rem;margin-bottom:1.5rem; }
  .stat-card { background:rgba(14,20,40,0.6);border:1px solid rgba(239,68,68,0.1);border-radius:8px;padding:1rem 1.2rem;display:flex;flex-direction:column;gap:0.15rem; }
  .stat-value { font-family:'Orbitron',sans-serif;font-size:1.8rem;font-weight:700;color:#EF4444; }
  .stat-value.vote-val { color:#F59E0B; }
  .stat-value.request-val { color:#3ABEFF; }
  .stat-value.audit-val { color:#8B5CF6; }
  .stat-value.notif-val { color:#10B981; }
  .stat-value.loc-val { color:#C084FC; }
  .stat-label { font-size:0.8rem;color:#E6EDF3;font-weight:600; }
  .stat-detail { font-size:0.7rem;color:#475569; }

  .activity-section { background:rgba(14,20,40,0.4);border:1px solid rgba(239,68,68,0.08);border-radius:8px;padding:1rem 1.2rem; }
  .section-title { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#EF4444;margin:0 0 0.75rem; }
  .activity-list { display:flex;flex-direction:column;gap:0.25rem; }
  .activity-item { display:grid;grid-template-columns:70px 1fr 150px 70px;align-items:center;padding:0.4rem 0.3rem;border-bottom:1px solid rgba(255,255,255,0.03);font-size:0.78rem; }
  .activity-op { font-weight:700;font-size:0.7rem;text-transform:uppercase; }
  .activity-summary { color:#E6EDF3;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .activity-actor { color:#94A3B8;text-align:right;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .activity-time { color:#475569;text-align:right;font-size:0.7rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
</style>
