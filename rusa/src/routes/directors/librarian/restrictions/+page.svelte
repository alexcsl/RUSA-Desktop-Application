<!--
  /directors/librarian/restrictions — Manage Document Access Restrictions (UC-LIB-01)
  TheLibrarian creates, views, and removes document access restrictions.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    libGetRestrictionList,
    libRestrictDocumentAccess,
    libRemoveRestriction,
    type DocRestriction,
  } from '$lib/stores/directors';

  const ALL_ROLES = [
    'Engineer','HeadEngineer','Scientist','HeadScientist','Astronaut','HeadAstronaut',
    'Settler','SettlerCommander','SanitaryWorker','HeadSanitaryWorker',
    'SecurityStaff','SecurityHead','MedicalStaff','HeadMedicalStaff',
    'Psychiatrist','DataAnalyst','TheDirector','TheAccountant','TheLibrarian',
    'TheNomad','TheArtificer','TheObserver','TheWanderer','TheTaskmaster',
    'TheGuardian','TheStatistician','TheCoordinator','TheOverseer','TheAnchorman',
    'Administrator',
  ];

  let restrictions: DocRestriction[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  // New restriction form
  let docId = $state('');
  let restrictionType = $state('full_access_control');
  let selectedRoles: string[] = $state([]);
  let notes = $state('');
  let saving = $state(false);
  let formMsg = $state('');

  onMount(async () => { await loadRestrictions(); });

  async function loadRestrictions() {
    loading = true; error = '';
    try { restrictions = await libGetRestrictionList(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    finally { loading = false; }
  }

  function toggleRole(r: string) {
    selectedRoles = selectedRoles.includes(r)
      ? selectedRoles.filter(x => x !== r)
      : [...selectedRoles, r];
  }

  async function handleCreate() {
    formMsg = '';
    if (!docId.trim()) { formMsg = 'Document ID is required.'; return; }
    if (selectedRoles.length === 0) { formMsg = 'Select at least one allowed role.'; return; }
    saving = true;
    try {
      await libRestrictDocumentAccess(docId.trim(), restrictionType, selectedRoles, notes.trim() || undefined);
      formMsg = 'Restriction created.';
      docId = ''; restrictionType = 'full_access_control'; selectedRoles = []; notes = '';
      await loadRestrictions();
    } catch (e: unknown) { formMsg = e instanceof Error ? e.message : String(e); }
    finally { saving = false; }
  }

  async function handleRemove(id: string) {
    try {
      await libRemoveRestriction(id);
      await loadRestrictions();
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }

  function formatDate(d: string) { return new Date(d).toLocaleDateString(); }
  function typeLabel(t: string) { return t === 'full_access_control' ? 'Full ACL' : 'Field Redaction'; }
</script>

<h1 class="title">Document Restrictions</h1>
<p class="subtitle">Control access to archived documents. Restrictions limit which roles can view or retrieve a document.</p>

<div class="layout">
  <!-- Create form -->
  <div class="form-card">
    <h2>New Restriction</h2>
    <label class="field">
      <span class="label">Document ID (UUID) *</span>
      <input class="input" type="text" bind:value={docId} placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" />
    </label>
    <label class="field">
      <span class="label">Restriction Type *</span>
      <select class="input" bind:value={restrictionType}>
        <option value="full_access_control">Full Access Control</option>
        <option value="field_redaction">Field Redaction</option>
      </select>
    </label>
    <div class="field">
      <span class="label">Allowed Roles * ({selectedRoles.length} selected)</span>
      <div class="role-grid">
        {#each ALL_ROLES as r}
          <button
            class="role-chip"
            class:selected={selectedRoles.includes(r)}
            onclick={() => toggleRole(r)}
          >{r}</button>
        {/each}
      </div>
    </div>
    <label class="field">
      <span class="label">Notes</span>
      <textarea class="textarea" bind:value={notes} rows="2" placeholder="Optional context…"></textarea>
    </label>
    {#if formMsg}<p class="form-msg">{formMsg}</p>{/if}
    <button class="btn-primary" onclick={handleCreate} disabled={saving}>
      {saving ? 'Creating…' : 'Create Restriction'}
    </button>
  </div>

  <!-- Active restrictions list -->
  <div class="list-panel">
    <h2>Active Restrictions</h2>
    {#if loading}<p class="muted">Loading…</p>
    {:else if error}<p class="error">{error}</p>
    {:else if restrictions.length === 0}<p class="muted">No restrictions configured.</p>
    {:else}
      {#each restrictions as r}
        <div class="card">
          <div class="card-header">
            <span class="type-badge">{typeLabel(r.restriction_type)}</span>
            <button class="btn-remove" onclick={() => handleRemove(r.id)}>Remove</button>
          </div>
          <p class="doc-id">Doc: <code>{r.document_id.slice(0,12)}…</code></p>
          <p class="meta">By: <strong>{r.restricted_by_name}</strong> · {formatDate(r.created_at)}</p>
          <div class="roles-row">
            {#each r.allowed_roles as role}
              <span class="role-tag">{role}</span>
            {/each}
          </div>
          {#if r.notes}<p class="notes">{r.notes}</p>{/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#8B5CF6;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .layout { display:flex;gap:1.5rem;flex-wrap:wrap;align-items:flex-start; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(139,92,246,0.15);border-radius:8px;padding:1.25rem;width:420px;display:flex;flex-direction:column;gap:0.6rem;flex-shrink:0; }
  .form-card h2,.list-panel h2 { font-family:'Orbitron',sans-serif;font-size:0.85rem;color:#8B5CF6;margin:0 0 0.4rem; }
  .list-panel { flex:1;min-width:300px; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.42rem 0.55rem;font-size:0.8rem;font-family:'Inter',sans-serif;width:100%;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#8B5CF6; }
  .textarea { resize:vertical; }
  .role-grid { display:flex;flex-wrap:wrap;gap:0.25rem;max-height:120px;overflow-y:auto;padding:0.3rem 0; }
  .role-chip { background:rgba(139,92,246,0.06);border:1px solid rgba(139,92,246,0.15);color:#94A3B8;border-radius:4px;padding:0.1rem 0.4rem;cursor:pointer;font-size:0.65rem;font-family:'Inter',sans-serif; }
  .role-chip.selected { background:rgba(139,92,246,0.2);border-color:#8B5CF6;color:#C084FC; }
  .btn-primary { background:rgba(139,92,246,0.15);border:1px solid #8B5CF6;color:#C084FC;border-radius:6px;padding:0.5rem;cursor:pointer;font-size:0.82rem;font-weight:600; }
  .btn-primary:hover:not(:disabled) { background:rgba(139,92,246,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:default; }
  .form-msg { font-size:0.73rem;color:#22C55E; }
  .muted { color:#475569;font-size:0.8rem;font-style:italic; }
  .card { background:rgba(14,20,40,0.5);border:1px solid rgba(139,92,246,0.1);border-radius:7px;padding:0.75rem;margin-bottom:0.5rem;display:flex;flex-direction:column;gap:0.3rem; }
  .card-header { display:flex;justify-content:space-between;align-items:center; }
  .type-badge { font-size:0.7rem;font-weight:700;color:#C084FC;background:rgba(139,92,246,0.12);padding:0.1rem 0.4rem;border-radius:3px; }
  .btn-remove { background:rgba(239,68,68,0.08);border:1px solid rgba(239,68,68,0.2);color:#F87171;border-radius:4px;padding:0.2rem 0.5rem;cursor:pointer;font-size:0.7rem; }
  .btn-remove:hover { background:rgba(239,68,68,0.16); }
  .doc-id { font-size:0.72rem;color:#64748B;margin:0; }
  code { font-size:0.72rem;color:#3ABEFF; }
  .meta { font-size:0.7rem;color:#475569;margin:0; }
  .roles-row { display:flex;flex-wrap:wrap;gap:0.2rem; }
  .role-tag { font-size:0.62rem;background:rgba(139,92,246,0.1);color:#C4B5FD;border-radius:3px;padding:0.08rem 0.3rem; }
  .notes { font-size:0.75rem;color:#64748B;margin:0;font-style:italic; }
  .error { color:#EF4444;font-size:0.75rem; }
</style>
