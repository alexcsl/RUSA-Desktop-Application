<!-- /sanitary/head/budget/report — UC-HS-02: Submit Expenditure Report -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    sanSubmitExpenditureReport, sanGetExpenditureReports,
    type SubmitExpenditurePayload, type ExpenditureReportSummary,
  } from '$lib/stores/sanitary';

  let reports: ExpenditureReportSummary[] = $state([]);
  let error = $state('');
  let success = $state('');

  let lineItems: { item_name: string; quantity: number; unit_cost: number }[] = $state([
    { item_name: '', quantity: 1, unit_cost: 0 },
  ]);

  function grandTotal(): number {
    return Math.round(lineItems.reduce((s, li) => s + li.quantity * li.unit_cost, 0) * 100) / 100;
  }
  function addRow() { lineItems = [...lineItems, { item_name: '', quantity: 1, unit_cost: 0 }]; }
  function removeRow(i: number) { lineItems = lineItems.filter((_, idx) => idx !== i); }

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = '';
    try { reports = await sanGetExpenditureReports(); } catch (e: unknown) { error = String(e); }
  }

  async function handleSubmit() {
    if (lineItems.some((li) => !li.item_name.trim())) { error = 'All items need a name.'; return; }
    if (grandTotal() <= 0) { error = 'Total must be positive.'; return; }
    error = ''; success = '';
    try {
      const payload: SubmitExpenditurePayload = {
        line_items: lineItems.map((li) => ({ item_name: li.item_name, quantity: li.quantity, unit_cost: li.unit_cost, total: li.quantity * li.unit_cost })),
        total_amount: grandTotal(),
      };
      await sanSubmitExpenditureReport(payload);
      success = 'Expenditure report submitted to The Accountant.';
      lineItems = [{ item_name: '', quantity: 1, unit_cost: 0 }];
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }
</script>

<h1 class="title">Expenditure Report</h1>
<p class="subtitle">UC-HS-02 — Submit spending reports to The Accountant for review.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="form-card">
  <h2>Line Items</h2>
  <table class="line-table">
    <thead><tr><th>Item</th><th>Qty</th><th>Unit Cost</th><th>Total</th><th></th></tr></thead>
    <tbody>
      {#each lineItems as li, i}
        <tr>
          <td><input class="input" bind:value={li.item_name} placeholder="Item name" /></td>
          <td><input class="input num" type="number" min="1" bind:value={li.quantity} /></td>
          <td><input class="input num" type="number" min="0" step="0.01" bind:value={li.unit_cost} /></td>
          <td class="total-cell">{(li.quantity * li.unit_cost).toFixed(2)}</td>
          <td><button class="btn-icon" onclick={() => removeRow(i)}>✕</button></td>
        </tr>
      {/each}
    </tbody>
  </table>
  <div class="row" style="justify-content:space-between;align-items:center;margin-top:0.4rem;">
    <button class="btn-secondary" onclick={addRow}>+ Add Row</button>
    <span class="grand-total">Total: <strong>{grandTotal().toFixed(2)}</strong></span>
  </div>
  <button class="btn-primary" style="margin-top:0.5rem;" onclick={handleSubmit}>Submit Report</button>
</div>

<h2 class="section-title">Submitted Reports</h2>
{#each reports as r}
  <div class="card">
    <div class="row" style="justify-content:space-between;">
      <span class="card-title">{new Date(r.created_at).toLocaleDateString()}</span>
      <span class="amount">{r.total_amount?.toFixed(2) ?? 'N/A'}</span>
    </div>
    {#if r.foul_play_flag}<span class="badge-err">FLAGGED</span>{/if}
    {#if r.foul_play_note}<p class="card-meta">{r.foul_play_note}</p>{/if}
  </div>
{:else}
  <p class="empty">No expenditure reports submitted yet.</p>
{/each}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .section-title { font-family:'Orbitron',sans-serif;font-size:0.95rem;color:#3ABEFF;margin:1.25rem 0 0.5rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-bottom:1rem; }
  .row { display:flex;gap:0.6rem;align-items:flex-end;flex-wrap:wrap; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.7rem;margin-bottom:0.4rem; }
  .card-title { font-size:0.85rem;font-weight:500; }
  .card-meta { font-size:0.7rem;color:#94A3B8;margin:0.15rem 0 0; }
  .amount { color:#F59E0B;font-weight:600;font-size:0.85rem; }
  .btn-primary { padding:0.45rem 0.75rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
</style>
