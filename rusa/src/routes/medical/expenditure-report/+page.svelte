<!-- /medical/expenditure-report — Submit Expenditure Report (UC-HOM-03)
     HeadOfMedicine only -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    medSubmitExpenditureReport, medGetExpenditureReports,
    type ExpenditureReportSummary, type SubmitExpenditurePayload,
  } from '$lib/stores/medical';

  let reports: ExpenditureReportSummary[] = $state([]);
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  // Form
  let lineItems: { item_name: string; quantity: number; unit_cost: number; total: number }[] = $state([{ item_name: '', quantity: 1, unit_cost: 0, total: 0 }]);

  onMount(async () => { await refresh(); });

  async function refresh() {
    try { reports = await medGetExpenditureReports(); }
    catch (e: unknown) { error = String(e); }
  }

  function addLineItem() { lineItems = [...lineItems, { item_name: '', quantity: 1, unit_cost: 0, total: 0 }]; }
  function removeLineItem(i: number) { lineItems = lineItems.filter((_, idx) => idx !== i); }
  function recalcLineTotal(i: number) {
    lineItems[i].total = lineItems[i].quantity * lineItems[i].unit_cost;
    lineItems = [...lineItems];
  }
  function totalAmount(): number { return lineItems.reduce((sum, li) => sum + (li.total || 0), 0); }

  async function handleSubmit() {
    if (lineItems.some((li) => !li.item_name.trim() || li.quantity <= 0 || li.unit_cost <= 0)) {
      error = 'All line items require a name, positive quantity, and positive unit cost.';
      return;
    }
    error = ''; success = '';
    submitting = true;
    try {
      const payload: SubmitExpenditurePayload = {
        line_items: lineItems.map((li) => ({ item_name: li.item_name, quantity: li.quantity, unit_cost: li.unit_cost, total: li.total })),
        total_amount: totalAmount(),
      };
      await medSubmitExpenditureReport(payload);
      success = 'Expenditure report submitted to The Accountant.';
      lineItems = [{ item_name: '', quantity: 1, unit_cost: 0, total: 0 }];
      await refresh();
    } catch (e: unknown) { error = String(e); }
    finally { submitting = false; }
  }
</script>

<h1 class="title">Budget Expenditure Report</h1>
<p class="subtitle">Submit expenditure reports to The Accountant for review.</p>

<div class="form-card">
  <h3>Line Items</h3>
  {#each lineItems as li, i}
    <div class="row">
      <div class="form-group" style="flex:3">
        <label class="inline-label">Item Name</label>
        <input class="input" bind:value={li.item_name} placeholder="e.g. Surgical Kits" />
      </div>
      <div class="form-group" style="flex:0.7">
        <label class="inline-label">Qty</label>
        <input type="number" class="input" min="1" bind:value={li.quantity} oninput={() => recalcLineTotal(i)} />
      </div>
      <div class="form-group" style="flex:1">
        <label class="inline-label">Unit Cost</label>
        <input type="number" class="input" min="0" step="0.01" bind:value={li.unit_cost} oninput={() => recalcLineTotal(i)} />
      </div>
      <div class="form-group" style="flex:0.8">
        <label class="inline-label">Line Total</label>
        <span class="line-total">${li.total.toFixed(2)}</span>
      </div>
      {#if lineItems.length > 1}
        <button class="btn-xs-danger" onclick={() => removeLineItem(i)}>×</button>
      {/if}
    </div>
  {/each}
  <button class="btn-secondary" onclick={addLineItem}>+ Add Line Item</button>

  <div class="total-row">
    <span>Total:</span>
    <span class="total-amount">${totalAmount().toFixed(2)}</span>
  </div>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}

  <button class="btn-primary" onclick={handleSubmit} disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Expenditure Report'}
  </button>
</div>

<!-- Previous reports -->
{#if reports.length > 0}
  <h3 class="section-title">Submitted Reports</h3>
  <div class="rep-list">
    {#each reports as r}
      <div class="rep-card" class:flagged={r.foul_play_flag}>
        <div class="rep-header">
          {#if r.foul_play_flag}
            <span class="badge badge-flagged">FLAGGED</span>
          {:else}
            <span class="badge badge-ok">OK</span>
          {/if}
          <span class="rep-date">{new Date(r.created_at).toLocaleDateString()}</span>
          <span class="rep-amount">${(r.total_amount ?? 0).toFixed(2)}</span>
        </div>
        {#if r.foul_play_flag && r.foul_play_note}
          <p class="flag-note">⚠ {r.foul_play_note}</p>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;display:flex;flex-direction:column;gap:0.6rem;max-width:700px; }
  h3 { font-size:0.85rem;color:#E6EDF3;margin:0 0 0.3rem; }
  .section-title { font-size:0.85rem;color:#E6EDF3;margin:1.5rem 0 0.5rem; }
  .row { display:flex;gap:0.5rem;align-items:center; }
  .form-group { display:flex;flex-direction:column;gap:0.2rem; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .total-row { display:flex;justify-content:space-between;padding:0.5rem 0;border-top:1px solid rgba(58,190,255,0.08);font-size:0.85rem; }
  .total-amount { color:#3ABEFF;font-weight:600; }
  .btn-primary { padding:0.5rem 1rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .btn-secondary { padding:0.35rem 0.6rem;background:rgba(139,92,246,0.1);border:1px solid rgba(139,92,246,0.3);color:#C084FC;border-radius:6px;cursor:pointer;font-size:0.75rem;align-self:flex-start; }
  .inline-label { font-size:0.65rem;color:#64748B;margin-bottom:0.1rem; }
  .line-total { font-size:0.8rem;color:#3ABEFF;padding:0.45rem 0; }
  .btn-xs-danger { background:none;border:none;color:#EF4444;cursor:pointer;font-size:1rem;padding:0 0.3rem; }
  .rep-list { display:flex;flex-direction:column;gap:0.4rem;max-width:700px; }
  .rep-card { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.65rem; }
  .rep-card.flagged { border-color:rgba(239,68,68,0.3);background:rgba(239,68,68,0.05); }
  .rep-header { display:flex;gap:0.75rem;align-items:center;font-size:0.8rem; }
  .rep-date { color:#94A3B8;font-size:0.75rem; }
  .rep-amount { margin-left:auto;color:#3ABEFF;font-weight:600; }
  .flag-note { font-size:0.75rem;color:#EF4444;margin:0.3rem 0 0; }
  .badge { padding:0.12rem 0.4rem;border-radius:4px;font-size:0.65rem; }
  .badge-flagged { background:rgba(239,68,68,0.15);color:#EF4444; }
  .badge-ok { background:rgba(16,185,129,0.15);color:#10B981; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
</style>
