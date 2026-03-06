<!--
  /engineers/tests — UC-AGE-01 / UC-BE-01: Approved Tests + My Proposals
  Tabbed view: Approved tests (role-scoped) and test proposals history.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getApprovedTests, getMyTestProposals, type ApprovedTest, type TestProposal } from '$lib/stores/engineers';
  import { currentUser } from '$lib/stores/auth';
  import type { SessionUser } from '$lib/stores/auth';

  let user: SessionUser | null = $state(null);
  currentUser.subscribe((v) => (user = v));

  let tests = $state<ApprovedTest[]>([]);
  let proposals = $state<TestProposal[]>([]);
  let loading = $state(true);
  let error = $state('');
  let tab: 'approved' | 'proposals' = $state('approved');

  function roleToScope(role: string | undefined): string {
    if (role === 'AgriculturalEngineer') return 'plants';
    if (role === 'BiologicalEngineer') return 'all_species';
    return 'plants';
  }

  onMount(async () => {
    try {
      const scope = roleToScope(user?.role);
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
    <p class="muted">No approved tests for your scope.</p>
  {:else}
    <div class="table-wrap">
      <table>
        <thead>
          <tr><th>Name</th><th>Category</th><th>Applicable Scope</th><th>Approved</th></tr>
        </thead>
        <tbody>
          {#each tests as t}
            <tr>
              <td>{t.name}</td>
              <td><span class="badge">{t.category}</span></td>
              <td>{t.applicable_scope ?? '—'}</td>
              <td>{t.accepted_at ? new Date(t.accepted_at).toLocaleDateString() : '—'}</td>
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
    <div class="proposals-list">
      {#each proposals as p}
        {@const d = (p.proposal_data ?? {}) as Record<string, unknown>}
        <div class="proposal-card">
          <div class="proposal-header">
            <span class="proposal-name">{d.name ?? 'Unnamed Proposal'}</span>
            <span class="badge" class:pending={p.status==='pending'} class:approved={p.status==='approved'} class:rejected={p.status==='rejected'}>{p.status}</span>
          </div>
          <p class="proposal-goal">{d.goal ?? ''}</p>
          {#if d.procedure}
            <div class="proposal-field"><span class="field-label">Procedure:</span> {d.procedure}</div>
          {/if}
          {#if d.species_scope}
            <div class="proposal-field"><span class="field-label">Scope:</span> {d.species_scope}</div>
          {/if}
          {#if Array.isArray(d.category) && d.category.length}
            <div class="proposal-field"><span class="field-label">Category:</span> {(d.category as string[]).join(', ')}</div>
          {/if}
          {#if d.apparatuses}
            <div class="proposal-field"><span class="field-label">Apparatuses:</span> {d.apparatuses}</div>
          {/if}
          {#if d.required_data}
            <div class="proposal-field"><span class="field-label">Required Data:</span> {d.required_data}</div>
          {/if}
          {#if d.justification}
            <div class="proposal-field"><span class="field-label">Justification:</span> {d.justification}</div>
          {/if}
          {#if p.reviewer_note}
            <div class="reviewer-note"><span class="field-label">Reviewer Note:</span> {p.reviewer_note}</div>
          {/if}
          <div class="proposal-date">Submitted {new Date(p.created_at).toLocaleDateString()}</div>
        </div>
      {/each}
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
  .proposals-list { display:flex;flex-direction:column;gap:0.75rem; }
  .proposal-card { background:#111827;border:1px solid #1E293B;border-radius:8px;padding:1rem;transition:border-color 0.2s; }
  .proposal-card:hover { border-color:rgba(58,190,255,0.3); }
  .proposal-header { display:flex;justify-content:space-between;align-items:center;margin-bottom:0.4rem; }
  .proposal-name { font-size:0.9rem;font-weight:600;color:#E6EDF3; }
  .proposal-goal { font-size:0.8rem;color:#94A3B8;margin-bottom:0.5rem; }
  .proposal-field { font-size:0.75rem;color:#CBD5E1;margin-bottom:0.25rem;line-height:1.4; }
  .field-label { color:#64748B;font-weight:500; }
  .reviewer-note { font-size:0.75rem;color:#F59E0B;margin-top:0.5rem;padding:0.5rem;background:rgba(245,158,11,0.08);border-radius:6px;border:1px solid rgba(245,158,11,0.15); }
  .proposal-date { font-size:0.7rem;color:#475569;margin-top:0.5rem; }
</style>
