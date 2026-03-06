<!--
  /engineers/experiments/[id]/log — UC-AGE-03 / UC-BE-04: Daily Experiment Log
  Supports attendees, tests, and species linking.
-->
<script lang="ts">
  import { page } from '$app/stores';
  import { logDailyExperiment } from '$lib/stores/engineers';
  import { goto } from '$app/navigation';

  let experimentId = $state('');
  page.subscribe((p) => (experimentId = p.params.id ?? ''));

  let logDate = $state(new Date().toISOString().split('T')[0]);
  let ragStatus = $state('green');
  let completedActions = $state('');
  let pendingActions = $state('');
  let collectedDataRaw = $state('');
  let attendeeIdsRaw = $state('');
  let testIdsRaw = $state('');
  let speciesIdsRaw = $state('');
  let submitting = $state(false);
  let message = $state('');
  let isError = $state(false);

  function splitIds(raw: string): string[] {
    return raw.split(',').map(s => s.trim()).filter(Boolean);
  }

  async function handleSubmit() {
    if (!logDate) return;
    submitting = true;
    message = '';
    try {
      let collected: Record<string, unknown> | undefined;
      if (collectedDataRaw.trim()) {
        try { collected = JSON.parse(collectedDataRaw); } catch {
          message = 'Invalid JSON in collected data field.';
          isError = true;
          submitting = false;
          return;
        }
      }
      await logDailyExperiment({
        experiment_id: experimentId,
        log_date: logDate,
        rag_status: ragStatus,
        completed_actions: completedActions || undefined,
        pending_actions: pendingActions || undefined,
        collected_data: collected,
        attendee_ids: splitIds(attendeeIdsRaw).length ? splitIds(attendeeIdsRaw) : undefined,
        test_ids: splitIds(testIdsRaw).length ? splitIds(testIdsRaw) : undefined,
        species_ids: splitIds(speciesIdsRaw).length ? splitIds(speciesIdsRaw) : undefined,
      });
      message = 'Daily log recorded successfully.';
      isError = false;
      completedActions = ''; pendingActions = ''; collectedDataRaw = '';
      attendeeIdsRaw = ''; testIdsRaw = ''; speciesIdsRaw = '';
    } catch (e: any) {
      message = e?.toString() ?? 'Failed to log session.';
      isError = true;
    } finally {
      submitting = false;
    }
  }
</script>

<h2>Daily Experiment Log</h2>
<p class="hint">Log a session for experiment <code>{experimentId.slice(0, 8)}…</code></p>

<div class="form-card">
  <div class="row">
    <div class="form-group" style="flex:1">
      <label for="date">Log Date *</label>
      <input id="date" type="date" bind:value={logDate} />
    </div>
    <div class="form-group" style="flex:1">
      <label for="rag">RAG Status</label>
      <select id="rag" bind:value={ragStatus}>
        <option value="green">🟢 Green</option>
        <option value="amber">🟡 Amber</option>
        <option value="red">🔴 Red</option>
      </select>
    </div>
  </div>
  <div class="form-group">
    <label for="done">Completed Actions</label>
    <textarea id="done" bind:value={completedActions} rows="3" placeholder="What was accomplished…"></textarea>
  </div>
  <div class="form-group">
    <label for="pending">Pending Actions</label>
    <textarea id="pending" bind:value={pendingActions} rows="3" placeholder="What remains to do…"></textarea>
  </div>
  <div class="form-group">
    <label for="data">Collected Data (JSON)</label>
    <textarea id="data" bind:value={collectedDataRaw} rows="4" placeholder={'{"soil_ph": 6.4, "moisture": "38%"}'}></textarea>
  </div>

  <h3 class="sub-heading">Linked Records (comma-separated UUIDs)</h3>
  <div class="form-group">
    <label for="attendees">Attendee IDs</label>
    <input id="attendees" bind:value={attendeeIdsRaw} placeholder="e.g. uuid1, uuid2" />
  </div>
  <div class="form-group">
    <label for="tests">Test IDs</label>
    <input id="tests" bind:value={testIdsRaw} placeholder="UUIDs of approved tests used" />
  </div>
  <div class="form-group">
    <label for="species">Species IDs (from archive)</label>
    <input id="species" bind:value={speciesIdsRaw} placeholder="UUIDs of species studied" />
  </div>

  <div class="btn-row">
    <button class="btn-primary" onclick={handleSubmit} disabled={submitting || !logDate}>
      {submitting ? 'Saving…' : 'Save Log Entry'}
    </button>
    <button class="btn-secondary" onclick={() => goto(`/engineers/experiments/${experimentId}`)}>Back to Experiment</button>
  </div>
  {#if message}<p class:msg-ok={!isError} class:msg-err={isError}>{message}</p>{/if}
</div>

<style>
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin-bottom:0.5rem; }
  h3 { font-size:0.85rem;color:#E6EDF3; }
  .sub-heading { margin-top:1rem;margin-bottom:0.5rem;font-family:'Orbitron',sans-serif;font-size:0.8rem;color:#8B5CF6; }
  .hint { color:#64748B;font-size:0.8rem;margin-bottom:1rem; }
  .hint code { color:#A5B4FC; }
  .form-card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:650px; }
  .row { display:flex;gap:1rem; }
  .form-group { margin-bottom:0.75rem; }
  .form-group label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.25rem; }
  .form-group input, .form-group textarea, .form-group select { width:100%;background:#0B0F1A;border:1px solid #334155;border-radius:6px;color:#E6EDF3;padding:0.5rem;font-size:0.8rem; }
  .form-group textarea { resize:vertical;font-family:'Fira Code',monospace; }
  .btn-row { display:flex;gap:0.75rem; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-secondary { background:transparent;border:1px solid #475569;color:#94A3B8;padding:0.45rem 1rem;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-secondary:hover { color:#E6EDF3;border-color:#3ABEFF; }
  .msg-ok { color:#22C55E;font-size:0.8rem;margin-top:0.5rem; }
  .msg-err { color:#EF4444;font-size:0.8rem;margin-top:0.5rem; }
</style>
