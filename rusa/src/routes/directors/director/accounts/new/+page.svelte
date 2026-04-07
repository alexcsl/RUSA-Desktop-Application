<!--
  /directors/director/accounts/new — Create Personnel Account (UC-DIR-01)
  TheDirector can create non-director accounts; Administrator can create any.
-->
<script lang="ts">
  import { createPersonnelAccount } from '$lib/stores/directors';

  const ALL_ROLES = [
    'Engineer','HeadEngineer','Scientist','HeadScientist',
    'Astronaut','HeadAstronaut','Settler','SettlerCommander',
    'SanitaryWorker','HeadSanitaryWorker','SecurityStaff','SecurityHead',
    'MedicalStaff','HeadMedicalStaff','Psychiatrist','PsychiatristAssistant',
    'DataAnalyst','SpaceStationStaff','TheDirector','TheAccountant',
    'TheLibrarian','TheNomad','TheArtificer','TheObserver','TheWanderer',
    'TheTaskmaster','TheGuardian','TheStatistician','TheCoordinator',
    'TheOverseer','TheAnchorman','Administrator',
  ];

  let fullName = $state('');
  let username = $state('');
  let email = $state('');
  let role = $state('Engineer');
  let baseLocation = $state('');
  let loading = $state(false);
  let error = $state('');
  let success = $state('');

  async function handleSubmit() {
    error = ''; success = '';
    if (!fullName.trim() || !username.trim()) {
      error = 'Full name and username are required.';
      return;
    }
    loading = true;
    try {
      const created = await createPersonnelAccount({
        full_name: fullName.trim(),
        username: username.trim(),
        email: email.trim() || undefined,
        initial_password: 'password123',
        role,
        base_location_id: baseLocation.trim() || undefined,
      });
      success = `Account created: ${created.full_name} (${created.username})`;
      fullName = ''; username = ''; email = ''; role = 'Engineer'; baseLocation = '';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }
</script>

<h1 class="title">Create Personnel Account</h1>
<p class="subtitle">Provision a new user account. Default password: <code>password123</code></p>

<div class="form-card">
  <label class="field">
    <span class="label">Full Name *</span>
    <input class="input" type="text" bind:value={fullName} placeholder="e.g. Jane Doe" />
  </label>
  <label class="field">
    <span class="label">Username *</span>
    <input class="input" type="text" bind:value={username} placeholder="e.g. jane.doe" />
  </label>
  <label class="field">
    <span class="label">Email</span>
    <input class="input" type="email" bind:value={email} placeholder="jane@rusa.space" />
  </label>
  <label class="field">
    <span class="label">Role *</span>
    <select class="input" bind:value={role}>
      {#each ALL_ROLES as r}
        <option value={r}>{r}</option>
      {/each}
    </select>
  </label>
  <label class="field">
    <span class="label">Base Location ID (UUID)</span>
    <input class="input" type="text" bind:value={baseLocation} placeholder="Optional UUID" />
  </label>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}
  <button class="btn-primary" onclick={handleSubmit} disabled={loading}>
    {loading ? 'Creating…' : 'Create Account'}
  </button>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1.2rem; }
  code { background:rgba(58,190,255,0.1);color:#3ABEFF;padding:0.1rem 0.3rem;border-radius:3px; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.5rem;max-width:480px;display:flex;flex-direction:column;gap:0.75rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem 0.6rem;font-size:0.82rem;font-family:'Inter',sans-serif;width:100%;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;padding:0.55rem;cursor:pointer;font-size:0.85rem;font-weight:600;margin-top:0.25rem; }
  .btn-primary:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:default; }
  .error { color:#EF4444;font-size:0.75rem; }
  .success { color:#22C55E;font-size:0.75rem; }
</style>
