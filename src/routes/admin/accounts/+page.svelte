<!--
  /admin/accounts — Administrator Account Management (UC-ADM-03/04/05)
  Create any account (including Directors), terminate any personnel.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { createAccount, terminateAccount, type CreateAccountPayload } from '$lib/stores/auth';
  import { getPersonnelList, updatePersonnelAccount, type PersonnelListItem } from '$lib/stores/directors';

  let personnel: PersonnelListItem[] = $state([]);
  let activeTab: 'create' | 'manage' = $state('create');

  // Create form
  let fullName = $state('');
  let username = $state('');
  let email = $state('');
  let password = $state('');
  let role = $state('');
  let createError = $state('');
  let createSuccess = $state('');

  // Terminate form
  let terminateUserId = $state('');
  let termError = $state('');
  let termSuccess = $state('');

  const ROLES = [
    'GeneralDirector','TheDirector','TheAccountant','TheLibrarian','TheNomad',
    'TheArtificer','TheObserver','TheWanderer','TheTaskmaster','TheGuardian',
    'TheStatistician','TheCoordinator','TheOverseer','TheAnchorman',
    'AgriculturalEngineer','BiologicalEngineer','DataAnalyst',
    'GalacticSecurityHead','GalacticSecurityStaff','Mathematician','Physicist',
    'Chemist','Biologist','Astronaut','SettlerCommander','CivilEngineer',
    'Farmer','TemporarySetter','SpaceStationSettler','Psychiatrist',
    'PsychiatristAssistant','MedicalStaff','HeadOfMedicine','HeadOfSanitary',
    'InspectorCrew','DisposalCrew','WastewaterCrew','CleanupCrew','TransportCrew',
  ];

  onMount(async () => {
    personnel = await getPersonnelList();
  });

  async function handleCreate() {
    createError = ''; createSuccess = '';
    if (!fullName.trim() || !username.trim() || !password || !role) {
      createError = 'Name, username, password, and role are required.'; return;
    }
    try {
      const result = await createAccount({
        full_name: fullName, username, email: email || null,
        initial_password: password, role, base_location_id: null,
      });
      createSuccess = `Account created: ${result.full_name} (${result.role})`;
      fullName = ''; username = ''; email = ''; password = ''; role = '';
      personnel = await getPersonnelList();
    } catch (e: unknown) { createError = e instanceof Error ? e.message : String(e); }
  }

  async function handleTerminate() {
    termError = ''; termSuccess = '';
    if (!terminateUserId) { termError = 'Select a user.'; return; }
    const name = personnel.find(p => p.id === terminateUserId)?.full_name ?? 'User';
    if (!confirm(`Terminate ${name}? This will soft-delete their account.`)) return;
    try {
      await terminateAccount(terminateUserId);
      termSuccess = `${name} terminated.`;
      terminateUserId = '';
      personnel = await getPersonnelList();
    } catch (e: unknown) { termError = e instanceof Error ? e.message : String(e); }
  }
</script>

<h1 class="title">Account Management</h1>
<p class="subtitle">Create any personnel account (including Directors) or terminate staff.</p>

<div class="tabs">
  <button class="tab" class:active={activeTab === 'create'} onclick={() => (activeTab = 'create')}>Create Account</button>
  <button class="tab" class:active={activeTab === 'manage'} onclick={() => (activeTab = 'manage')}>Terminate</button>
</div>

{#if activeTab === 'create'}
<div class="form-card">
  <h2>New Account</h2>
  <label class="field"><span class="label">Full Name</span>
    <input type="text" class="input" bind:value={fullName} placeholder="John Doe" />
  </label>
  <label class="field"><span class="label">Username</span>
    <input type="text" class="input" bind:value={username} placeholder="john.doe" />
  </label>
  <label class="field"><span class="label">Email (optional)</span>
    <input type="email" class="input" bind:value={email} placeholder="john@rusa.org" />
  </label>
  <label class="field"><span class="label">Initial Password</span>
    <input type="password" class="input" bind:value={password} placeholder="Min 8 characters" />
  </label>
  <label class="field"><span class="label">Role</span>
    <select class="input" bind:value={role}>
      <option value="">— Select Role —</option>
      {#each ROLES as r}<option value={r}>{r}</option>{/each}
    </select>
  </label>
  {#if createError}<p class="error">{createError}</p>{/if}
  {#if createSuccess}<p class="success">{createSuccess}</p>{/if}
  <button class="btn-primary" onclick={handleCreate}>Create Account</button>
</div>
{:else}
<div class="form-card">
  <h2>Terminate Personnel</h2>
  <p class="hint">Administrator can terminate any personnel, including Directors.</p>
  <label class="field"><span class="label">Personnel</span>
    <select class="input" bind:value={terminateUserId}>
      <option value="">— Select —</option>
      {#each personnel as p}<option value={p.id}>{p.full_name} ({p.role_name})</option>{/each}
    </select>
  </label>
  {#if termError}<p class="error">{termError}</p>{/if}
  {#if termSuccess}<p class="success">{termSuccess}</p>{/if}
  <button class="btn-danger" onclick={handleTerminate}>Terminate</button>
</div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#EF4444;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .tabs { display:flex;gap:0.5rem;margin-bottom:1rem; }
  .tab { background:transparent;border:1px solid rgba(239,68,68,0.15);color:#94A3B8;padding:0.4rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .tab.active { background:rgba(239,68,68,0.1);border-color:#EF4444;color:#EF4444; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(239,68,68,0.1);border-radius:8px;padding:1.5rem;max-width:520px;display:flex;flex-direction:column;gap:0.65rem; }
  .form-card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#EF4444;margin:0 0 0.3rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input { width:100%;background:#0E1428;border:1px solid rgba(239,68,68,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#EF4444; }
  .hint { color:#475569;font-size:0.75rem;font-style:italic;margin:0; }
  .btn-primary { padding:0.5rem 1rem;background:rgba(239,68,68,0.15);border:1px solid #EF4444;color:#EF4444;border-radius:6px;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:rgba(239,68,68,0.3); }
  .btn-danger { padding:0.5rem 1rem;background:rgba(239,68,68,0.2);border:1px solid #EF4444;color:#EF4444;border-radius:6px;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-danger:hover { background:rgba(239,68,68,0.4); }
  .error { color:#EF4444;font-size:0.75rem;margin:0; }
  .success { color:#22C55E;font-size:0.75rem;margin:0; }
</style>
