<!-- /data/request/new — UC-DRQ-01: Submit Data Request (any authenticated user) -->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { submitDataRequest } from '$lib/stores/data_analysts';
  import { currentUser } from '$lib/stores/auth';
  import { getDefaultRoute } from '$lib/routing';

  let role = $state('');
  currentUser.subscribe((u) => (role = u?.role ?? ''));

  let datasetDescription = $state('');
  let scope = $state('');
  let purpose = $state('');
  let urgency: 'low' | 'medium' | 'high' | 'critical' = $state('medium');
  let sensitivityNote = $state('');
  let submitting = $state(false);
  let error = $state('');
  let success = $state('');

  async function handleSubmit() {
    if (!datasetDescription.trim() || !scope.trim() || !purpose.trim()) {
      error = 'Please fill in all required fields.';
      return;
    }
    submitting = true; error = ''; success = '';
    try {
      const requestId = await submitDataRequest({
        dataset_description: datasetDescription.trim(),
        scope: scope.trim(),
        purpose: purpose.trim(),
        urgency,
        sensitivity_note: sensitivityNote.trim() || undefined,
      });
      success = 'Data request submitted successfully! Routing to The Statistician for review.';
      // Reset form after short delay, then navigate to status tracker
      setTimeout(() => goto(`/data/request/${requestId}/status`), 1500);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<div class="page">
  <h1 class="title">Submit Data Request</h1>
  <p class="subtitle">Request cross-department data. Routed to The Statistician for review (bypass authority — no Directors vote).</p>

  <form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
    <div class="field">
      <label class="label" for="dataset">Requested Dataset <span class="req">*</span></label>
      <textarea id="dataset" class="textarea" rows="3" placeholder="Describe the data you need…"
        bind:value={datasetDescription}></textarea>
    </div>

    <div class="row">
      <div class="field flex-1">
        <label class="label" for="scope">Scope <span class="req">*</span></label>
        <input id="scope" class="input" placeholder="e.g., Q1 2025, Zone Alpha, Engineering dept…"
          bind:value={scope} />
      </div>
      <div class="field" style="width:160px">
        <label class="label" for="urgency">Urgency <span class="req">*</span></label>
        <select id="urgency" class="input" bind:value={urgency}>
          <option value="low">Low</option>
          <option value="medium">Medium</option>
          <option value="high">High</option>
          <option value="critical">Critical</option>
        </select>
      </div>
    </div>

    <div class="field">
      <label class="label" for="purpose">Purpose <span class="req">*</span></label>
      <textarea id="purpose" class="textarea" rows="2" placeholder="Why do you need this data?"
        bind:value={purpose}></textarea>
    </div>

    <div class="field">
      <label class="label" for="sensitivity">Data Sensitivity Note</label>
      <input id="sensitivity" class="input" placeholder="Any sensitivity concerns (optional)…"
        bind:value={sensitivityNote} />
    </div>

    {#if error}<p class="error">{error}</p>{/if}
    {#if success}<p class="success">{success}</p>{/if}

    <div class="btn-row">
      <button type="submit" class="btn-primary" disabled={submitting}>
        {submitting ? 'Submitting…' : 'Submit Request'}
      </button>
      <button type="button" class="btn-back" onclick={() => goto(getDefaultRoute(role))}>
        ← Back to Dashboard
      </button>
    </div>
  </form>
</div>

<style>
  .page { max-width:640px; }
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1.2rem; }
  .form { display:flex;flex-direction:column;gap:0.75rem; }
  .field { display:flex;flex-direction:column;gap:0.25rem; }
  .row { display:flex;gap:0.75rem;align-items:flex-start; }
  .flex-1 { flex:1; }
  .label { font-size:0.8rem;color:#E6EDF3;font-weight:500; }
  .req { color:#EF4444; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .btn-row { display:flex;gap:0.5rem;margin-top:0.25rem; }
  .btn-primary { padding:0.5rem 1.2rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.85rem; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-back { padding:0.5rem 1.2rem;background:transparent;border:1px solid #475569;border-radius:6px;color:#94A3B8;cursor:pointer;font-size:0.85rem; }
  .btn-back:hover { color:#E6EDF3;border-color:#3ABEFF; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
</style>
