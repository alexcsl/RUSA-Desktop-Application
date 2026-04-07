<!--
  /directors/director/accounts — Personnel Accounts List (UC-DIR-01)
  TheDirector views all active personnel accounts.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getPersonnelList, type PersonnelListItem } from '$lib/stores/directors';
  import { goto } from '$app/navigation';

  let personnel: PersonnelListItem[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let search = $state('');

  const filtered = $derived(
    search.trim()
      ? personnel.filter(
          (p) =>
            p.full_name.toLowerCase().includes(search.toLowerCase()) ||
            p.username.toLowerCase().includes(search.toLowerCase()) ||
            p.role_name.toLowerCase().includes(search.toLowerCase())
        )
      : personnel
  );

  onMount(async () => {
    try {
      personnel = await getPersonnelList();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });
</script>

<div class="page">
  <div class="header">
    <div>
      <h2>Personnel Accounts</h2>
      <p class="subtitle">{personnel.length} active accounts</p>
    </div>
    <button class="btn-primary" onclick={() => goto('/directors/director/accounts/new')}>
      + New Account
    </button>
  </div>

  {#if error}
    <div class="banner error">{error}</div>
  {/if}

  <input class="search" type="text" bind:value={search} placeholder="Search by name, username, or role…" />

  {#if loading}
    <p class="muted">Loading personnel…</p>
  {:else if filtered.length === 0}
    <p class="muted">{search ? 'No accounts match the search.' : 'No active accounts found.'}</p>
  {:else}
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Full Name</th>
            <th>Username</th>
            <th>Role</th>
          </tr>
        </thead>
        <tbody>
          {#each filtered as p}
            <tr>
              <td class="name">{p.full_name}</td>
              <td class="mono">{p.username}</td>
              <td><span class="role-badge">{p.role_name}</span></td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .page { max-width:700px; }
  .header { display:flex;justify-content:space-between;align-items:flex-start;margin-bottom:1rem; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.25rem; }
  .subtitle { color:#64748B;font-size:0.8rem;margin:0; }
  .banner.error { padding:0.6rem 1rem;border-radius:6px;font-size:0.8rem;margin-bottom:1rem;background:rgba(239,68,68,0.15);color:#EF4444;border:1px solid rgba(239,68,68,0.3); }
  .search { background:#1F2937;border:1px solid #374151;color:#E6EDF3;border-radius:6px;padding:0.45rem 0.7rem;font-size:0.82rem;font-family:inherit;width:100%;box-sizing:border-box;margin-bottom:1rem; }
  .search:focus { outline:none;border-color:#3ABEFF; }
  .muted { color:#475569;font-size:0.8rem;font-style:italic; }
  .table-wrap { overflow-x:auto; }
  table { width:100%;border-collapse:collapse;font-size:0.8rem; }
  thead tr { border-bottom:1px solid rgba(58,190,255,0.15); }
  th { color:#64748B;font-size:0.7rem;font-weight:600;text-transform:uppercase;letter-spacing:0.05em;padding:0.4rem 0.6rem;text-align:left; }
  td { padding:0.5rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04);color:#CBD5E1; }
  .name { color:#E6EDF3;font-weight:500; }
  .mono { font-family:'Courier New',monospace;font-size:0.75rem;color:#94A3B8; }
  .role-badge { background:rgba(58,190,255,0.1);color:#3ABEFF;border-radius:4px;padding:0.1rem 0.4rem;font-size:0.68rem;white-space:nowrap; }
  .btn-primary { padding:0.45rem 1rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.82rem;font-weight:600;white-space:nowrap; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
</style>
