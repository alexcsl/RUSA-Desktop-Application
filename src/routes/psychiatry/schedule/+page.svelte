<!--
  /psychiatry/schedule — Manage Psychiatrist Schedule (UC-PSY-04)
  View, add, and delete availability slots.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    psyGetSchedule,
    psyManageSchedule,
    psyDeleteScheduleSlot,
    type ScheduleSlot,
  } from '$lib/stores/psychiatry';

  let slots: ScheduleSlot[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let success = $state('');

  // New slot form
  let showForm = $state(false);
  let slotStart = $state('');
  let slotEnd = $state('');
  let isAvailable = $state(true);
  let blockedReason = $state('');
  let saving = $state(false);

  onMount(async () => { await loadSlots(); });

  async function loadSlots() {
    loading = true;
    try { slots = await psyGetSchedule(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    loading = false;
  }

  async function addSlot() {
    if (!slotStart || !slotEnd) { error = 'Start and end are required.'; return; }
    saving = true;
    error = '';
    success = '';
    try {
      await psyManageSchedule({
        slots: [{
          slot_start: new Date(slotStart).toISOString(),
          slot_end: new Date(slotEnd).toISOString(),
          is_available: isAvailable,
          blocked_reason: blockedReason || undefined,
        }],
      });
      showForm = false;
      slotStart = '';
      slotEnd = '';
      isAvailable = true;
      blockedReason = '';
      success = 'Slot added.';
      await loadSlots();
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    saving = false;
  }

  async function deleteSlot(id: string) {
    try {
      await psyDeleteScheduleSlot(id);
      slots = slots.filter((s) => s.id !== id);
      success = 'Slot deleted.';
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }
</script>

<div class="page">
  <div class="head-row">
    <h2>My Schedule</h2>
    <button class="btn-accent-sm" onclick={() => (showForm = !showForm)}>
      {showForm ? 'Cancel' : '+ Add Slot'}
    </button>
  </div>

  {#if success}<div class="toast-success">{success}</div>{/if}
  {#if error}<div class="toast-error">{error}</div>{/if}

  {#if showForm}
    <div class="form-box">
      <label class="field-label">Slot Start
        <input class="input" type="datetime-local" bind:value={slotStart} />
      </label>
      <label class="field-label">Slot End
        <input class="input" type="datetime-local" bind:value={slotEnd} />
      </label>
      <label class="field-label chk">
        <input type="checkbox" bind:checked={isAvailable} /> Available
      </label>
      {#if !isAvailable}
        <label class="field-label">Blocked Reason
          <input class="input" type="text" bind:value={blockedReason} placeholder="e.g. Admin time" />
        </label>
      {/if}
      <button class="btn-primary" onclick={addSlot} disabled={saving}>
        {saving ? 'Saving…' : 'Save Slot'}
      </button>
    </div>
  {/if}

  {#if loading}
    <p class="muted">Loading…</p>
  {:else if slots.length === 0}
    <p class="muted">No schedule slots. Add one above.</p>
  {:else}
    <table class="tbl">
      <thead>
        <tr><th>Start</th><th>End</th><th>Available</th><th>Reason</th><th></th></tr>
      </thead>
      <tbody>
        {#each slots as s}
          <tr>
            <td>{new Date(s.slot_start).toLocaleString()}</td>
            <td>{new Date(s.slot_end).toLocaleString()}</td>
            <td>
              {#if s.is_available}
                <span class="badge badge-avail">Open</span>
              {:else}
                <span class="badge badge-blocked">Blocked</span>
              {/if}
            </td>
            <td class="muted">{s.blocked_reason ?? '—'}</td>
            <td>
              <button class="btn-danger-sm" onclick={() => deleteSlot(s.id)}>Delete</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .page { max-width:900px; }
  .head-row { display:flex;justify-content:space-between;align-items:center;margin-bottom:1rem; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#3ABEFF;margin:0; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .form-box { background:#111827;border:1px solid rgba(139,92,246,0.15);border-radius:8px;padding:1rem;margin-bottom:1rem; }
  .input { width:100%;padding:0.45rem 0.6rem;background:#1F2937;color:#E6EDF3;border:1px solid #374151;border-radius:4px;font-size:0.8rem;margin-top:0.2rem;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .field-label { display:block;font-size:0.75rem;color:#94A3B8;margin-bottom:0.6rem; }
  .chk { display:flex;align-items:center;gap:0.4rem;font-size:0.8rem;color:#E6EDF3; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.5rem 0.6rem;color:#94A3B8;border-bottom:1px solid rgba(58,190,255,0.1); }
  .tbl td { padding:0.5rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .badge { padding:0.15rem 0.5rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .badge-avail { background:rgba(16,185,129,0.15);color:#34D399; }
  .badge-blocked { background:rgba(245,158,11,0.15);color:#FBBF24; }
  .btn-accent-sm { background:rgba(58,190,255,0.1);color:#3ABEFF;border:1px solid rgba(58,190,255,0.3);border-radius:4px;padding:0.25rem 0.6rem;cursor:pointer;font-size:0.7rem; }
  .btn-accent-sm:hover { background:rgba(58,190,255,0.2); }
  .btn-danger-sm { background:rgba(239,68,68,0.1);color:#EF4444;border:1px solid rgba(239,68,68,0.3);border-radius:4px;padding:0.25rem 0.5rem;cursor:pointer;font-size:0.7rem; }
  .btn-danger-sm:hover { background:rgba(239,68,68,0.2); }
  .btn-primary { padding:0.5rem 1.2rem;background:rgba(58,190,255,0.15);color:#3ABEFF;border:1px solid rgba(58,190,255,0.4);border-radius:6px;cursor:pointer;font-size:0.8rem;font-weight:600;margin-top:0.5rem; }
  .btn-primary:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .toast-success { background:rgba(16,185,129,0.1);border:1px solid rgba(16,185,129,0.3);color:#34D399;padding:0.5rem 0.8rem;border-radius:6px;font-size:0.8rem;margin-bottom:0.8rem; }
  .toast-error { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);color:#EF4444;padding:0.5rem 0.8rem;border-radius:6px;font-size:0.8rem;margin-bottom:0.8rem; }
</style>
