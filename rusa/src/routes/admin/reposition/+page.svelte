<!--
  /admin/reposition — Administrator Staff Reposition (UC-ADM-07)
  Change any personnel member's role, name, email, or base location.
  Administrator can reposition Directors and any other staff.
  Uses update_personnel_account (directors.rs) which allows Administrator to change any role.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getPersonnelList, updatePersonnelAccount, type PersonnelListItem } from '$lib/stores/directors';
  import { getRoles, getBaseLocations, type RoleEntry, type BaseLocationEntry } from '$lib/stores/administrator';

  let personnel: PersonnelListItem[] = $state([]);
  let roles: RoleEntry[] = $state([]);
  let locations: BaseLocationEntry[] = $state([]);
  let loading = $state(true);

  let targetUserId = $state('');
  let newRole = $state('');
  let newName = $state('');
  let newEmail = $state('');
  let newLocationId = $state('');

  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  onMount(async () => {
    try {
      [personnel, roles, locations] = await Promise.all([
        getPersonnelList(),
        getRoles(),
        getBaseLocations(),
      ]);
    } catch (e: unknown) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  function prefillFromSelected() {
    const p = personnel.find((x) => x.id === targetUserId);
    if (p) {
      newRole = p.role_name;
      newName = p.full_name;
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!targetUserId) { error = 'Select a personnel member.'; return; }
    submitting = true;
    error = '';
    success = '';
    try {
      await updatePersonnelAccount(targetUserId, {
        role: newRole || undefined,
        full_name: newName || undefined,
        email: newEmail || undefined,
        base_location_id: newLocationId || undefined,
      });
      const name = personnel.find((p) => p.id === targetUserId)?.full_name ?? 'Personnel';
      success = `${name} successfully repositioned${newRole ? ` to ${newRole}` : ''}.`;
      // Refresh list
      personnel = await getPersonnelList();
      targetUserId = '';
      newRole = '';
      newName = '';
      newEmail = '';
      newLocationId = '';
    } catch (e: unknown) {
      error = String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<h1 class="title">Staff Reposition</h1>
<p class="subtitle">Change any personnel member's role, name, email, or base location. Administrator can reposition Directors and any other staff.</p>

{#if loading}
  <p class="hint">Loading…</p>
{:else}
  <form class="form-card" onsubmit={handleSubmit}>
    <label class="field">
      <span class="label">Personnel <span class="req">*</span></span>
      <select class="input" bind:value={targetUserId} onchange={prefillFromSelected}>
        <option value="">— Select personnel —</option>
        {#each personnel as p}
          <option value={p.id}>{p.full_name} — {p.role_name}</option>
        {/each}
      </select>
    </label>

    <label class="field">
      <span class="label">New Role (leave blank to keep current)</span>
      <select class="input" bind:value={newRole}>
        <option value="">— Keep current role —</option>
        {#each roles as r}
          <option value={r.name}>{r.name}</option>
        {/each}
      </select>
    </label>

    <label class="field">
      <span class="label">New Full Name (leave blank to keep current)</span>
      <input type="text" class="input" bind:value={newName} placeholder="Full name" />
    </label>

    <label class="field">
      <span class="label">New Email (leave blank to keep current)</span>
      <input type="email" class="input" bind:value={newEmail} placeholder="email@rusa.org" />
    </label>

    <label class="field">
      <span class="label">New Base Location (leave blank to keep current)</span>
      <select class="input" bind:value={newLocationId}>
        <option value="">— Keep current location —</option>
        {#each locations as loc}
          <option value={loc.id}>{loc.name}{loc.has_data_regulation ? ' [Data Regulation]' : ''}</option>
        {/each}
      </select>
    </label>

    {#if error}<p class="error">{error}</p>{/if}
    {#if success}<p class="success">{success}</p>{/if}

    <button type="submit" class="btn-primary" disabled={submitting || !targetUserId}>
      {submitting ? 'Repositioning…' : 'Apply Reposition'}
    </button>
  </form>

  <div class="info-box">
    <p><strong>Note:</strong> Only fields you fill in will be changed. The Administrator can reposition any personnel including Directors.</p>
    <p>Audit log entry will be written for every change.</p>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#EF4444;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1.25rem; }
  .hint { color:#94A3B8;font-size:0.85rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(239,68,68,0.1);border-radius:8px;padding:1.5rem;max-width:540px;display:flex;flex-direction:column;gap:0.75rem;margin-bottom:1rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.72rem;color:#94A3B8; }
  .req { color:#EF4444; }
  .input { width:100%;background:#0E1428;border:1px solid rgba(239,68,68,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem 0.55rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#EF4444; }
  .btn-primary { padding:0.5rem 1.25rem;background:rgba(239,68,68,0.15);border:1px solid #EF4444;color:#EF4444;border-radius:6px;font-weight:600;cursor:pointer;font-size:0.85rem;align-self:flex-start;transition:background 0.15s; }
  .btn-primary:hover:not(:disabled) { background:rgba(239,68,68,0.3); }
  .btn-primary:disabled { opacity:0.4;cursor:default; }
  .error { color:#EF4444;font-size:0.75rem;margin:0; }
  .success { color:#22C55E;font-size:0.75rem;margin:0; }
  .info-box { background:rgba(58,190,255,0.05);border:1px solid rgba(58,190,255,0.1);border-radius:6px;padding:0.75rem 1rem;max-width:540px;font-size:0.75rem;color:#64748B;display:flex;flex-direction:column;gap:0.3rem; }
  .info-box p { margin:0; }
  .info-box strong { color:#94A3B8; }
</style>
