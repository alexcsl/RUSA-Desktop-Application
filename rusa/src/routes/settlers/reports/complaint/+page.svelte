<!--
  UC-PS-04 / UC-TS-02: Submit Troublesome Settler Report (Complaint)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { stlSubmitComplaint } from '$lib/stores/settlers';
  import { stlGetSettlementMembers, type SettlementMember } from '$lib/stores/settlers';
  import { currentUser } from '$lib/stores/auth';
  import type { SessionUser } from '$lib/stores/auth';

  let user: SessionUser | null = $state(null);
  const unsub = currentUser.subscribe((v) => (user = v));
  import { onDestroy } from 'svelte';
  onDestroy(() => unsub());

  let settlers: SettlementMember[] = $state([]);
  let subjectUserId = $state('');
  let incidentDescription = $state('');
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  onMount(async () => {
    try {
      const members = await stlGetSettlementMembers();
      settlers = members.filter((s) => s.user_id !== user?.id);
    } catch (e: any) {
      error = 'Could not load settlers: ' + (e?.message ?? e);
    }
  });

  async function handleSubmit() {
    error = ''; success = '';
    if (!subjectUserId) { error = 'Please select the settler.'; return; }
    if (!incidentDescription.trim()) { error = 'Incident description is required.'; return; }
    submitting = true;
    try {
      const id = await stlSubmitComplaint({
        subject_user_id: subjectUserId,
        incident_description: incidentDescription,
      });
      success = `Complaint submitted (ID: ${id.slice(0,8)}…). Commander notified.`;
      incidentDescription = ''; subjectUserId = '';
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }
</script>

<h2>Report Troublesome Settler</h2>

<form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  {#if settlers.length > 0}
    <label>
      Settler *
      <select bind:value={subjectUserId}>
        <option value="">— Select settler —</option>
        {#each settlers as s}
          <option value={s.user_id}>{s.full_name} ({s.role_name})</option>
        {/each}
      </select>
    </label>
  {:else}
    <label>
      Settler *
      <select disabled>
        <option>Loading settlers…</option>
      </select>
    </label>
  {/if}

  <label>
    Incident Description *
    <textarea bind:value={incidentDescription} rows="5" required
      placeholder="Describe the misconduct or incident…"></textarea>
  </label>

  {#if error}<p class="err">{error}</p>{/if}
  {#if success}<p class="ok">{success}</p>{/if}

  <button type="submit" class="btn-primary" disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Complaint'}
  </button>
</form>

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
  .form { display:flex;flex-direction:column;gap:0.75rem;max-width:560px; }
  .form label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8; }
  .form input, .form select, .form textarea { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem; }
  .form textarea { resize:vertical; }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem; }
  .btn-primary { background:#3ABEFF;color:#0B0F1A;border:none;border-radius:4px;padding:0.5rem 1.2rem;cursor:pointer;font-weight:600;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:#60CFFF; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
