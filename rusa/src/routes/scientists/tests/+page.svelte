<!--
  /scientists/tests — UC-PH-04 / UC-CH-02 / UC-BIO-05: Approved Test Archive
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getApprovedTests, getMyTestProposals, type ApprovedTest, type TestProposal } from '$lib/stores/scientists';

  let tests = $state<ApprovedTest[]>([]);
  let proposals = $state<TestProposal[]>([]);
  let loading = $state(true);
  let error = $state('');
  let tab: 'approved' | 'proposals' = $state('approved');

  /* scope auto-set from role */
  let scope = $state('chemical');
  try {
    const r = localStorage.getItem('user_role');
    if (r === 'Physicist') scope = 'physical';
    if (r === 'Chemist')   scope = 'chemical';
    if (r === 'Biologist')  scope = 'biological';
  } catch {}

  onMount(async () => {
    try {
      const [t, p] = await Promise.all([getApprovedTests(scope), getMyTestProposals()]);
      tests = t;
      proposals = p;
    } catch (e: any) {
      error = e?.toString() ?? 'Failed to load tests.';
    } finally {
      loading = false;
    }
  });
</script>

<h2>Tests</h2>

<div class="tabs">
  <button class:active={tab === 'approved'} onclick={() => (tab = 'approved')}>Approved Tests</button>
  <button class:active={tab === 'proposals'} onclick={() => (tab = 'proposals')}>My Proposals</button>
</div>

{#if loading}
  <p class="muted">Loading…</p>
{:else if error}
  <p class="error">{error}</p>
{:else if tab === 'approved'}
  {#if tests.length === 0}
    <p class="muted">No approved tests for scope "{scope}".</p>
  {:else}
    <div class="table-wrap">
      <table>
        <thead>
          <tr><th>Name</th><th>Category</th><th>Species Scope</th><th>Approved</th></tr>
        </thead>
        <tbody>
          {#each tests as t}
            <tr>
              <td>{t.name}</td>
              <td><span class="badge">{t.category}</span></td>
              <td>{t.species_scope ?? '—'}</td>
              <td>{t.approved_at ? new Date(t.approved_at).toLocaleDateString() : '—'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
{:else}
  {#if proposals.length === 0}
    <p class="muted">You have not submitted any test proposals yet.</p>
  {:else}
    <div class="table-wrap">
      <table>
        <thead>
          <tr><th>Name</th><th>Status</th><th>Submitted</th></tr>
        </thead>
        <tbody>
          {#each proposals as p}
            <tr>
              <td>{p.name}</td>
              <td><span class="badge" class:pending={p.status==='pending'} class:approved={p.status==='approved'} class:rejected={p.status==='rejected'}>{p.status}</span></td>
              <td>{new Date(p.created_at).toLocaleDateString()}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.75rem; }
  .tabs { display:flex;gap:0.5rem;margin-bottom:1rem; }
  .tabs button { background:transparent;border:1px solid #334155;color:#94A3B8;padding:0.35rem 0.75rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .tabs button.active { border-color:#3ABEFF;color:#3ABEFF;background:rgba(58,190,255,0.1); }
  .table-wrap { overflow-x:auto; }
  table { width:100%;border-collapse:collapse;font-size:0.8rem; }
  th { text-align:left;color:#64748B;font-size:0.7rem;text-transform:uppercase;padding:0.4rem 0.75rem;border-bottom:1px solid #1E293B; }
  td { padding:0.4rem 0.75rem;color:#E6EDF3;border-bottom:1px solid #111827; }
  .badge { padding:0.15rem 0.5rem;border-radius:4px;font-size:0.7rem;background:rgba(58,190,255,0.15);color:#3ABEFF; }
  .badge.pending { background:rgba(250,204,21,0.15);color:#FACC15; }
  .badge.approved { background:rgba(34,197,94,0.15);color:#22C55E; }
  .badge.rejected { background:rgba(239,68,68,0.15);color:#EF4444; }
  .muted { color:#64748B;font-size:0.85rem; }
  .error { color:#EF4444;font-size:0.85rem; }
</style>
