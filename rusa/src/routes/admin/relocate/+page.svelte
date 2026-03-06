<!--
  /admin/relocate — Administrator Relocation (UC-ADM-06)
  Can relocate ANY personnel, including Directors (unlike TheNomad).
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { relocatePersonnel, getPersonnelList, type PersonnelListItem } from '$lib/stores/directors';

  let personnel: PersonnelListItem[] = $state([]);
  let targetUserId = $state('');
  let origin = $state('');
  let destination = $state('');
  let relocationType: 'temporary' | 'permanent' = $state('temporary');
  let effectiveDate = $state('');
  let error = $state('');
  let success = $state('');

  onMount(async () => {
    personnel = await getPersonnelList();
    effectiveDate = new Date().toISOString().split('T')[0];
  });

  async function handleSubmit() {
    error = ''; success = '';
    if (!targetUserId || !origin.trim() || !destination.trim() || !effectiveDate) {
      error = 'All fields are required.'; return;
    }
    try {
      const id = await relocatePersonnel({
        target_user_id: targetUserId, origin_location: origin,
        destination, relocation_type: relocationType, effective_date: effectiveDate,
      });
      const name = personnel.find(p => p.id === targetUserId)?.full_name ?? 'User';
      success = `${name} relocation recorded (ID: ${id.slice(0,8)}…)`;
      targetUserId = ''; origin = ''; destination = '';
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }
</script>

<h1 class="title">Relocate Personnel</h1>
<p class="subtitle">Administrator can relocate any personnel, including Directors.</p>

<div class="form-card">
  <label class="field"><span class="label">Personnel</span>
    <select class="input" bind:value={targetUserId}>
      <option value="">— Select —</option>
      {#each personnel as p}<option value={p.id}>{p.full_name} ({p.role_name})</option>{/each}
    </select>
  </label>
  <label class="field"><span class="label">Origin Location</span>
    <input type="text" class="input" bind:value={origin} placeholder="Base Alpha, Lab 3…" />
  </label>
  <label class="field"><span class="label">Destination</span>
    <input type="text" class="input" bind:value={destination} placeholder="Base Beta, Sector 7…" />
  </label>
  <label class="field"><span class="label">Type</span>
    <select class="input" bind:value={relocationType}>
      <option value="temporary">Temporary</option>
      <option value="permanent">Permanent</option>
    </select>
  </label>
  <label class="field"><span class="label">Effective Date</span>
    <input type="date" class="input" bind:value={effectiveDate} />
  </label>
  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}
  <button class="btn-primary" onclick={handleSubmit}>Submit Relocation</button>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#EF4444;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(239,68,68,0.1);border-radius:8px;padding:1.5rem;max-width:520px;display:flex;flex-direction:column;gap:0.65rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input { width:100%;background:#0E1428;border:1px solid rgba(239,68,68,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#EF4444; }
  .btn-primary { padding:0.5rem 1rem;background:rgba(239,68,68,0.15);border:1px solid #EF4444;color:#EF4444;border-radius:6px;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:rgba(239,68,68,0.3); }
  .error { color:#EF4444;font-size:0.75rem;margin:0; }
  .success { color:#22C55E;font-size:0.75rem;margin:0; }
</style>
