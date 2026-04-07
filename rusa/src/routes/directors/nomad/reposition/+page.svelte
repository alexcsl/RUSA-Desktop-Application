<!--
  /directors/nomad/reposition — Reposition Personnel (UC-NOM-02)
  The Nomad can change the role of any non-Director staff member.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getPersonnelList, dirNomadRepositionPersonnel, dirGetRoleList, type PersonnelListItem, type DirRoleEntry } from '$lib/stores/directors';

  let personnel: PersonnelListItem[] = $state([]);
  let roles: DirRoleEntry[] = $state([]);
  let loading = $state(true);

  let targetUserId = $state('');
  let newRole = $state('');
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  onMount(async () => {
    try {
      [personnel, roles] = await Promise.all([getPersonnelList(), dirGetRoleList()]);
    } catch (e: unknown) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  function prefill() {
    const p = personnel.find((x) => x.id === targetUserId);
    if (p) newRole = p.role_name;
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!targetUserId || !newRole) { error = 'Select a personnel member and a new role.'; return; }
    submitting = true;
    error = '';
    success = '';
    try {
      await dirNomadRepositionPersonnel({ target_user_id: targetUserId, new_role: newRole });
      const name = personnel.find((p) => p.id === targetUserId)?.full_name ?? 'Personnel';
      success = `${name} successfully repositioned to ${newRole}.`;
      personnel = await getPersonnelList();
      targetUserId = '';
      newRole = '';
    } catch (e: unknown) {
      error = String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<div class="page">
  <h2>Reposition Personnel</h2>
  <p class="subtitle">Change a staff member's role. Directors and Administrators cannot be repositioned here.</p>

  {#if loading}
    <p class="muted">Loading…</p>
  {:else}
    <form class="form-card" onsubmit={handleSubmit}>
      <label class="field">
        <span class="label">Personnel <span class="req">*</span></span>
        <select class="input" bind:value={targetUserId} onchange={prefill}>
          <option value="">— Select personnel —</option>
          {#each personnel as p}
            <option value={p.id}>{p.full_name} — {p.role_name}</option>
          {/each}
        </select>
      </label>

      <label class="field">
        <span class="label">New Role <span class="req">*</span></span>
        <select class="input" bind:value={newRole}>
          <option value="">— Select new role —</option>
          {#each roles as r}
            <option value={r.name}>{r.name}</option>
          {/each}
        </select>
      </label>

      {#if error}<p class="msg-error">{error}</p>{/if}
      {#if success}<p class="msg-success">{success}</p>{/if}

      <button type="submit" class="btn-primary" disabled={submitting || !targetUserId || !newRole}>
        {submitting ? 'Repositioning…' : 'Apply Reposition'}
      </button>
    </form>

    <div class="info-box">
      <p><strong>Note:</strong> Only the role will be changed. A Director or Administrator account cannot be the target of this action.</p>
      <p>An audit log entry is written for every change.</p>
    </div>
  {/if}
</div>

<style>
  .page { max-width:540px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1.25rem; }
  .muted { color:#94A3B8;font-size:0.85rem; }
  .form-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.5rem;display:flex;flex-direction:column;gap:0.75rem;margin-bottom:1rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.72rem;color:#94A3B8; }
  .req { color:#EF4444; }
  .input { width:100%;background:#1F2937;border:1px solid #374151;color:#E6EDF3;border-radius:6px;padding:0.45rem 0.55rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .btn-primary { padding:0.5rem 1.25rem;background:rgba(58,190,255,0.15);border:1px solid rgba(58,190,255,0.4);color:#3ABEFF;border-radius:6px;font-weight:600;cursor:pointer;font-size:0.85rem;align-self:flex-start; }
  .btn-primary:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.4;cursor:default; }
  .msg-error { color:#EF4444;font-size:0.75rem;margin:0; }
  .msg-success { color:#34D399;font-size:0.75rem;margin:0; }
  .info-box { background:rgba(58,190,255,0.04);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.75rem 1rem;font-size:0.75rem;color:#64748B;display:flex;flex-direction:column;gap:0.3rem; }
  .info-box p { margin:0; }
  .info-box strong { color:#94A3B8; }
</style>
