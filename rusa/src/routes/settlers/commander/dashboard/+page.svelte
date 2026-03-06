<!--
  UC-SC-06: Commander Dashboard — Settler roster, stats, open anomalies, pending requests
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { stlGetDashboard, type DashboardData } from '$lib/stores/settlers';

  let data: DashboardData | null = $state(null);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try { data = await stlGetDashboard(); }
    catch (e: any) { error = e?.message ?? String(e); }
    finally { loading = false; }
  });
</script>

<h2>Commander Dashboard</h2>

{#if loading}
  <p class="dim">Loading dashboard…</p>
{:else if error}
  <p class="err">{error}</p>
{:else if data}
  <!-- Stats cards -->
  <div class="stats">
    <div class="card">
      <span class="num">{data.settlers.length}</span>
      <span class="lbl">Settlers</span>
    </div>
    <div class="card">
      <span class="num warn">{data.open_anomalies}</span>
      <span class="lbl">Open Anomalies</span>
    </div>
    <div class="card">
      <span class="num accent">{data.pending_requests}</span>
      <span class="lbl">Pending Requests</span>
    </div>
    <div class="card">
      <span class="num danger">{data.house_arrests}</span>
      <span class="lbl">House Arrests</span>
    </div>
  </div>

  <!-- Settler roster -->
  <h3>Settler Roster</h3>
  {#if data.settlers.length === 0}
    <p class="dim">No settlers assigned to your settlement.</p>
  {:else}
    <table class="tbl">
      <thead>
        <tr><th>Name</th><th>Role</th><th>Status</th><th>Assigned</th></tr>
      </thead>
      <tbody>
        {#each data.settlers as s}
          <tr>
            <td>{s.full_name}</td>
            <td><span class="badge role">{s.role_name}</span></td>
            <td>
              {#if s.house_arrest}
                <span class="badge arrested">🔒 House Arrest</span>
              {:else}
                <span class="badge active">Active</span>
              {/if}
            </td>
            <td class="dim">{new Date(s.assigned_at).toLocaleDateString()}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
  h3 { font-family:'Orbitron',sans-serif;color:#E6EDF3;font-size:0.9rem;margin:1.2rem 0 0.6rem; }
  .dim { color:#64748B;font-size:0.8rem; }
  .err { color:#EF4444;font-size:0.8rem; }

  .stats { display:grid;grid-template-columns:repeat(auto-fit,minmax(140px,1fr));gap:0.75rem;margin-bottom:0.5rem; }
  .card { background:#111827;border:1px solid rgba(58,190,255,0.15);border-radius:6px;padding:0.8rem 1rem;display:flex;flex-direction:column;align-items:center; }
  .num { font-family:'Orbitron',sans-serif;font-size:1.5rem;color:#E6EDF3; }
  .num.warn { color:#FBBF24; }
  .num.accent { color:#3ABEFF; }
  .num.danger { color:#EF4444; }
  .lbl { font-size:0.7rem;color:#94A3B8;margin-top:0.15rem; }

  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.4rem 0.6rem;border-bottom:1px solid #374151;color:#94A3B8;font-weight:500; }
  .tbl td { padding:0.4rem 0.6rem;border-bottom:1px solid rgba(55,65,81,0.4); }
  .badge { display:inline-block;padding:0.15rem 0.55rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .role { background:rgba(139,92,246,0.15);color:#A78BFA; }
  .active { background:rgba(74,222,128,0.12);color:#4ADE80; }
  .arrested { background:rgba(239,68,68,0.12);color:#EF4444; }
</style>
