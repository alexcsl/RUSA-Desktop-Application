<!--
  /psychiatry/patients/new — Create new patient record (UC-PSY-01)
  Psychiatrist picks a user from the directory and fills patient_profile JSONB.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import {
    psyGetUserDirectory,
    psyCreatePatientRecord,
    type UserPickerEntry,
  } from '$lib/stores/psychiatry';

  let users: UserPickerEntry[] = $state([]);
  let search = $state('');
  let selectedUserId = $state('');
  let diagnosis = $state('');
  let notes = $state('');
  let profileFields = $state('');
  let loading = $state(true);
  let saving = $state(false);
  let error = $state('');
  let success = $state('');

  onMount(async () => {
    try { users = await psyGetUserDirectory(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    loading = false;
  });

  function filteredUsers() {
    if (!search) return users;
    const q = search.toLowerCase();
    return users.filter((u) =>
      u.full_name.toLowerCase().includes(q) || u.username.toLowerCase().includes(q) || u.role_name.toLowerCase().includes(q)
    );
  }

  async function handleSubmit() {
    if (!selectedUserId) { error = 'Please select a user.'; return; }
    saving = true;
    error = '';
    success = '';

    let profileObj: Record<string, unknown> = {};
    if (diagnosis) profileObj.diagnosis = diagnosis;
    if (profileFields.trim()) {
      try { profileObj = { ...profileObj, ...JSON.parse(profileFields) }; }
      catch { error = 'Profile fields must be valid JSON.'; saving = false; return; }
    }

    try {
      const result = await psyCreatePatientRecord({
        user_id: selectedUserId,
        patient_profile: profileObj,
        initial_notes: notes || undefined,
      });
      success = `Patient record created (ID: ${result.id}).`;
      setTimeout(() => goto(`/psychiatry/patients/${result.id}`), 1200);
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    saving = false;
  }
</script>

<div class="page">
  <h2>New Patient Record</h2>

  {#if success}
    <div class="toast-success">{success}</div>
  {/if}
  {#if error}
    <div class="toast-error">{error}</div>
  {/if}

  <section class="card">
    <h3>Select User</h3>
    <input class="input" type="text" placeholder="Search by name, username, or role…" bind:value={search} />

    {#if loading}
      <p class="muted">Loading users…</p>
    {:else}
      <div class="user-list">
        {#each filteredUsers() as u}
          <label class="user-row" class:selected={selectedUserId === u.id}>
            <input type="radio" name="user" value={u.id} bind:group={selectedUserId} />
            <span class="uname">{u.full_name}</span>
            <span class="urole">{u.role_name}</span>
            <span class="uid muted">@{u.username}</span>
          </label>
        {/each}
        {#if filteredUsers().length === 0}
          <p class="muted">No users match.</p>
        {/if}
      </div>
    {/if}
  </section>

  <section class="card">
    <h3>Patient Profile</h3>
    <label class="field-label">Diagnosis
      <input class="input" type="text" bind:value={diagnosis} placeholder="Primary diagnosis" />
    </label>
    <label class="field-label">Initial Notes
      <textarea class="input ta" bind:value={notes} rows="3" placeholder="Observations, referral notes…"></textarea>
    </label>
    <label class="field-label">Additional Profile (JSON)
      <textarea class="input ta mono" bind:value={profileFields} rows="3" placeholder="allergies: [], medications: []"></textarea>
    </label>
  </section>

  <button class="btn-primary" onclick={handleSubmit} disabled={saving}>
    {saving ? 'Saving…' : 'Create Patient Record'}
  </button>
</div>

<style>
  .page { max-width:700px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#3ABEFF;margin-bottom:1rem; }
  h3 { font-family:'Orbitron',sans-serif;font-size:0.85rem;color:#C084FC;margin-bottom:0.6rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-bottom:1rem; }
  .input { width:100%;padding:0.45rem 0.6rem;background:#1F2937;color:#E6EDF3;border:1px solid #374151;border-radius:4px;font-size:0.8rem;margin-bottom:0.5rem;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .ta { resize:vertical; }
  .mono { font-family:'Cascadia Code','Fira Mono',monospace;font-size:0.75rem; }
  .field-label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.6rem; }

  .user-list { max-height:250px;overflow-y:auto;border:1px solid rgba(255,255,255,0.05);border-radius:4px; }
  .user-row { display:flex;align-items:center;gap:0.5rem;padding:0.4rem 0.6rem;cursor:pointer;font-size:0.8rem;border-bottom:1px solid rgba(255,255,255,0.03); }
  .user-row:hover { background:rgba(58,190,255,0.04); }
  .user-row.selected { background:rgba(58,190,255,0.1);border-left:3px solid #3ABEFF; }
  .uname { flex:1;color:#E6EDF3; }
  .urole { color:#8B5CF6;font-size:0.7rem;min-width:110px; }
  .uid { font-size:0.7rem; }

  .btn-primary { padding:0.5rem 1.2rem;background:rgba(58,190,255,0.15);color:#3ABEFF;border:1px solid rgba(58,190,255,0.4);border-radius:6px;cursor:pointer;font-size:0.85rem;font-weight:600; }
  .btn-primary:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .toast-success { background:rgba(16,185,129,0.1);border:1px solid rgba(16,185,129,0.3);color:#34D399;padding:0.5rem 0.8rem;border-radius:6px;font-size:0.8rem;margin-bottom:0.8rem; }
  .toast-error { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);color:#EF4444;padding:0.5rem 0.8rem;border-radius:6px;font-size:0.8rem;margin-bottom:0.8rem; }
</style>
