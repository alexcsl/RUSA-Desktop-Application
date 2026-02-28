<!--
  /engineers/experiments — UC-AGE-06 / UC-BE-02: Experiment Archive
  Lists engineering experiments filtered by role discipline.
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { getExperimentArchive, type ExperimentSummary } from '$lib/stores/engineers';
  import { currentUser } from '$lib/stores/auth';
  import type { SessionUser } from '$lib/stores/auth';

  let user: SessionUser | null = $state(null);
  const unsubUser = currentUser.subscribe((v) => (user = v));
  onDestroy(unsubUser);

  let experiments: ExperimentSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try {
      experiments = await getExperimentArchive();
    } catch (e: any) {
      error = e?.toString() ?? 'Failed to load experiments.';
    } finally {
      loading = false;
    }
  });

  function statusColor(s: string): string {
    switch (s) {
      case 'closed': return '#22C55E';
      case 'active': return '#3ABEFF';
      case 'approved': return '#A5B4FC';
      case 'proposed': return '#F59E0B';
      case 'rejected': return '#EF4444';
      case 'conclusion_requested': return '#F97316';
      default: return '#94A3B8';
    }
  }

  function friendlyRole(role: string | undefined): string {
    if (role === 'AgriculturalEngineer') return 'Agricultural Engineering';
    if (role === 'BiologicalEngineer') return 'Biological Engineering';
    return 'Engineering';
  }
</script>

<h2>Experiment Archive</h2>
<p class="hint">Showing experiments for <strong>{friendlyRole(user?.role)}</strong></p>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if experiments.length === 0}
  <p class="muted">No experiments found for your discipline.</p>
{:else}
  <table class="tbl">
    <thead>
      <tr>
        <th>Title</th>
        <th>Type</th>
        <th>Status</th>
        <th>Proposed By</th>
        <th>Created</th>
        <th>Actions</th>
      </tr>
    </thead>
    <tbody>
      {#each experiments as e}
        <tr>
          <td><a href="/engineers/experiments/{e.id}">{e.title}</a></td>
          <td class="tag">{e.experiment_type}</td>
          <td><span style="color:{statusColor(e.status)};font-weight:600;font-size:0.75rem">{e.status}</span></td>
          <td>{e.proposer_name}</td>
          <td>{new Date(e.created_at).toLocaleDateString()}</td>
          <td class="actions">
            {#if e.status === 'approved' || e.status === 'active'}
              <a href="/engineers/experiments/{e.id}/log" class="link-sm">Log</a>
            {/if}
            {#if e.status === 'active'}
              <a href="/engineers/experiments/{e.id}/conclusion" class="link-sm">Conclude</a>
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.5rem; }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:1rem; }
  .hint strong { color:#E6EDF3; }
  .muted { color:#64748B;font-size:0.85rem; }
  .error { color:#EF4444;font-size:0.85rem; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.5rem;color:#94A3B8;border-bottom:1px solid rgba(58,190,255,0.15);font-size:0.7rem;text-transform:uppercase; }
  .tbl td { padding:0.5rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .tbl a { color:#3ABEFF;text-decoration:none; }
  .tbl a:hover { text-decoration:underline; }
  .tag { background:rgba(139,92,246,0.15);color:#C084FC;padding:0.1rem 0.4rem;border-radius:4px;font-size:0.7rem; }
  .actions { display:flex;gap:0.5rem; }
  .link-sm { font-size:0.75rem;color:#A5B4FC;text-decoration:none; }
  .link-sm:hover { text-decoration:underline; }
</style>
