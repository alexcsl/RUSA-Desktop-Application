<!--
  UC-SC-04/05: Request Settler Repatriation + Set House Arrest (Commander)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    stlRequestRepatriation,
    stlSetHouseArrest,
    stlGetDashboard,
    stlListSettlementComplaints,
    type DashboardSettler,
    type ComplaintSummary,
  } from '$lib/stores/settlers';

  let settlers: DashboardSettler[] = $state([]);
  let complaints: ComplaintSummary[] = $state([]);
  let error = $state('');
  let success = $state('');

  /* --- Repatriation form --- */
  let repSettlerId = $state('');
  let repComplaintId = $state('');
  let repReason = $state('');
  let repSubmitting = $state(false);

  /* --- House arrest --- */
  let haSettlerId = $state('');
  let haValue = $state(true);
  let haSubmitting = $state(false);

  onMount(async () => {
    try {
      const [dash, comps] = await Promise.all([
        stlGetDashboard(),
        stlListSettlementComplaints(),
      ]);
      settlers = dash.settlers;
      complaints = comps;
    } catch (e: any) { error = e?.message ?? String(e); }
  });

  function selectedComplaint(): ComplaintSummary | undefined {
    return complaints.find((c) => c.id === repComplaintId);
  }

  async function submitRepatriation() {
    error = ''; success = '';
    if (!repSettlerId) { error = 'Select a settler.'; return; }
    if (!repComplaintId) { error = 'Select a complaint.'; return; }
    if (!repReason.trim()) { error = 'Reason is required.'; return; }
    repSubmitting = true;
    try {
      const id = await stlRequestRepatriation({
        settler_id: repSettlerId,
        complaint_id: repComplaintId,
        reason: repReason,
      });
      success = `Repatriation request submitted (ID: ${id.slice(0, 8)}…). Directors will vote.`;
      repSettlerId = ''; repComplaintId = ''; repReason = '';
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { repSubmitting = false; }
  }

  async function toggleHouseArrest() {
    error = ''; success = '';
    if (!haSettlerId) { error = 'Select a settler to toggle house arrest.'; return; }
    haSubmitting = true;
    try {
      await stlSetHouseArrest({ settler_id: haSettlerId, arrest: haValue });
      success = `House arrest ${haValue ? 'enabled' : 'disabled'} for settler.`;
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { haSubmitting = false; }
  }
</script>

<h2>Repatriation & House Arrest</h2>

{#if error}<p class="err">{error}</p>{/if}
{#if success}<p class="ok">{success}</p>{/if}

<!-- REPATRIATION -->
<section class="card">
  <h3>Request Settler Repatriation</h3>
  <p class="hint">Select a settler and a supporting complaint. Directors must vote to approve.</p>

  <form class="form" onsubmit={(e) => { e.preventDefault(); submitRepatriation(); }}>
    <label>
      Settler *
      <select bind:value={repSettlerId} required>
        <option value="">— Select settler —</option>
        {#each settlers as s}
          <option value={s.user_id}>{s.full_name} ({s.role_name})</option>
        {/each}
      </select>
    </label>

    <label>
      Related Complaint *
      <select bind:value={repComplaintId} required>
        <option value="">— Select a complaint —</option>
        {#each complaints as c}
          <option value={c.id}>
            {c.subject_name} — {c.incident_description.slice(0, 70)}{c.incident_description.length > 70 ? '…' : ''} ({c.status})
          </option>
        {/each}
      </select>
      {#if complaints.length === 0}
        <span class="field-hint">No complaints found.</span>
      {/if}
    </label>

    {#if selectedComplaint()}
      {@const c = selectedComplaint()!}
      <div class="preview-card">
        <div class="preview-row"><span class="lbl">Subject</span><span>{c.subject_name}</span></div>
        <div class="preview-row"><span class="lbl">Status</span><span>{c.status}</span></div>
        <div class="preview-row full"><span class="lbl">Description</span><span>{c.incident_description}</span></div>
      </div>
    {/if}

    <label>
      Reason *
      <textarea bind:value={repReason} rows="3" required placeholder="Justify the repatriation…"></textarea>
    </label>

    <button type="submit" class="btn-primary" disabled={repSubmitting || !repSettlerId || !repComplaintId}>
      {repSubmitting ? 'Submitting…' : 'Submit Repatriation Request'}
    </button>
  </form>
</section>

<!-- HOUSE ARREST -->
<section class="card">
  <h3>Toggle House Arrest</h3>

  <form class="form" onsubmit={(e) => { e.preventDefault(); toggleHouseArrest(); }}>
    <label>
      Settler *
      <select bind:value={haSettlerId} required>
        <option value="">— Select settler —</option>
        {#each settlers as s}
          <option value={s.user_id}>
            {s.full_name} ({s.role_name}){s.house_arrest ? ' 🔒 Arrested' : ''}
          </option>
        {/each}
      </select>
    </label>

    <label class="chk-row">
      <input type="checkbox" bind:checked={haValue} />
      Enable House Arrest
    </label>

    <button type="submit" class="btn-primary" disabled={haSubmitting}>
      {haSubmitting ? 'Saving…' : 'Apply'}
    </button>
  </form>
</section>

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
  h3 { font-family:'Orbitron',sans-serif;color:#E6EDF3;font-size:0.9rem;margin:0 0 0.3rem; }
  .hint { color:#64748B;font-size:0.75rem;margin:0 0 0.7rem; }
  .card { background:#111827;border:1px solid rgba(58,190,255,0.15);border-radius:6px;padding:1rem;margin-bottom:1rem; }
  .form { display:flex;flex-direction:column;gap:0.65rem;max-width:540px; }
  .form label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8; }
  .form select, .form textarea {
    background:#0B0F1A;border:1px solid #374151;border-radius:4px;
    color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem;
  }
  .form select { cursor:pointer; }
  .form textarea { resize:vertical; }
  .chk-row { flex-direction:row!important;align-items:center;gap:0.35rem; }
  .field-hint { color:#F59E0B;font-size:0.72rem;margin-top:0.2rem; }
  .preview-card {
    background:#0B0F1A;border:1px solid rgba(58,190,255,0.1);border-radius:5px;
    padding:0.6rem 0.8rem;display:grid;grid-template-columns:1fr 1fr;gap:0.35rem 0.75rem;
  }
  .preview-row { display:flex;flex-direction:column;gap:0.1rem;font-size:0.78rem; }
  .preview-row.full { grid-column:1/-1; }
  .lbl { color:#64748B;font-size:0.7rem;text-transform:uppercase;letter-spacing:0.04em; }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem; }
  .btn-primary { background:#3ABEFF;color:#0B0F1A;border:none;border-radius:4px;padding:0.5rem 1.2rem;cursor:pointer;font-weight:600;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:#60CFFF; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
