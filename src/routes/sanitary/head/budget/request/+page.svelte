<!-- /sanitary/head/budget/request — UC-HS-01: Submit Budget Funding Request -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    sanSubmitBudgetRequest, sanGetBudgetRequests,
    type SubmitBudgetRequestPayload, type BudgetRequestSummary,
  } from '$lib/stores/sanitary';

  let requests: BudgetRequestSummary[] = $state([]);
  let error = $state('');
  let success = $state('');

  // Form state
  let lineItems: { item_name: string; quantity: number; unit_cost: number; total: number }[] = $state([
    { item_name: '', quantity: 1, unit_cost: 0, total: 0 },
  ]);
  let justification = $state('');
  let invoiceFile: File | null = $state(null);

  $effect(() => {
    lineItems.forEach((li) => {
      li.total = Math.round(li.quantity * li.unit_cost * 100) / 100;
    });
  });

  function grandTotal(): number {
    return Math.round(lineItems.reduce((s, li) => s + li.quantity * li.unit_cost, 0) * 100) / 100;
  }

  function addRow() {
    lineItems = [...lineItems, { item_name: '', quantity: 1, unit_cost: 0, total: 0 }];
  }
  function removeRow(i: number) {
    lineItems = lineItems.filter((_, idx) => idx !== i);
  }

  function handleFileInput(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files?.[0]) invoiceFile = input.files[0];
  }

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = '';
    try { requests = await sanGetBudgetRequests(); } catch (e: unknown) { error = String(e); }
  }

  async function handleSubmit() {
    if (!invoiceFile) { error = 'Invoice file is required.'; return; }
    if (lineItems.some((li) => !li.item_name.trim())) { error = 'All line items must have a name.'; return; }
    error = ''; success = '';
    try {
      const bytes = Array.from(new Uint8Array(await invoiceFile.arrayBuffer()));
      const payload: SubmitBudgetRequestPayload = {
        line_items: lineItems.map((li) => ({
          item_name: li.item_name,
          quantity: li.quantity,
          unit_cost: li.unit_cost,
          total: li.quantity * li.unit_cost,
        })),
        total_amount: grandTotal(),
        justification: justification.trim() || undefined,
        invoice_bytes: bytes,
        invoice_filename: invoiceFile.name,
        invoice_content_type: invoiceFile.type || 'application/octet-stream',
      };
      await sanSubmitBudgetRequest(payload);
      success = 'Budget request submitted and sent to Directors for voting.';
      lineItems = [{ item_name: '', quantity: 1, unit_cost: 0, total: 0 }];
      justification = '';
      invoiceFile = null;
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }

  function statusBadge(s: string): string {
    const m: Record<string, string> = {
      pending_vote: 'badge-warn', approved: 'badge-ok', rejected: 'badge-err',
    };
    return m[s] ?? 'badge-default';
  }
</script>

<h1 class="title">Budget Funding Request</h1>
<p class="subtitle">UC-HS-01 — Submit an itemized budget request with invoice. Enters Directors voting queue.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="form-card">
  <h2>Line Items</h2>
  <table class="line-table">
    <thead>
      <tr><th>Item</th><th>Qty</th><th>Unit Cost</th><th>Total</th><th></th></tr>
    </thead>
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
  <div class="row" style="justify-content:space-between;align-items:center;margin-top:0.5rem;">
    <button class="btn-secondary" onclick={addRow}>+ Add Row</button>
    <span class="grand-total">Grand Total: <strong>{grandTotal().toFixed(2)}</strong></span>
  </div>

  <div class="form-group" style="margin-top:0.75rem;">
    <label for="justification">Justification</label>
    <textarea id="justification" class="input" rows="3" bind:value={justification} placeholder="Explain why funds are needed…"></textarea>
  </div>

  <div class="form-group">
    <label for="invoice">Invoice File (required)</label>
    <input id="invoice" type="file" accept=".pdf,.png,.jpg,.jpeg" onchange={handleFileInput} />
  </div>

  <button class="btn-primary" onclick={handleSubmit}>Submit Budget Request</button>
</div>

<h2 class="section-title">Previous Requests</h2>
{#each requests as r}
  <div class="card">
    <div class="row" style="justify-content:space-between;">
      <span class="card-title">{new Date(r.created_at).toLocaleDateString()}</span>
      <span class="badge {statusBadge(r.status)}">{r.status.replace(/_/g, ' ')}</span>
    </div>
    <div class="card-meta">
      <span>Total: {r.total_amount?.toFixed(2) ?? 'N/A'}</span>
      {#if r.justification}<span>— {r.justification}</span>{/if}
    </div>
  </div>
{:else}
  <p class="empty">No budget requests yet.</p>
{/each}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .section-title { font-family:'Orbitron',sans-serif;font-size:0.95rem;color:#3ABEFF;margin:1.25rem 0 0.5rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-bottom:1rem; }
  .form-card h2 { font-size:0.9rem;color:#E6EDF3;margin:0 0 0.5rem; }
  .line-table { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .line-table th { text-align:left;color:#94A3B8;padding:0.3rem 0.4rem;border-bottom:1px solid rgba(58,190,255,0.1); }
  .line-table td { padding:0.25rem 0.3rem; }
  .total-cell { color:#F59E0B;font-weight:600;text-align:right;min-width:70px; }
  .num { width:80px;text-align:right; }
  .grand-total { color:#F59E0B;font-size:0.85rem; }
  .row { display:flex;gap:0.6rem;align-items:flex-end; }
  .form-group { display:flex;flex-direction:column;gap:0.2rem; }
  .form-group label { font-size:0.7rem;color:#94A3B8; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem 0.5rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  textarea.input { resize:vertical; }
  .card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.7rem;margin-bottom:0.4rem; }
  .card-title { font-size:0.85rem;font-weight:500; }
  .card-meta { font-size:0.7rem;color:#94A3B8;margin-top:0.2rem; }
  .badge { padding:0.12rem 0.4rem;border-radius:4px;font-size:0.65rem;text-transform:capitalize; }
  .badge-warn { background:rgba(245,158,11,0.15);color:#F59E0B; }
  .badge-ok { background:rgba(16,185,129,0.15);color:#10B981; }
  .badge-err { background:rgba(239,68,68,0.15);color:#EF4444; }
  .badge-default { background:rgba(148,163,184,0.15);color:#94A3B8; }
  .btn-primary { padding:0.45rem 0.75rem;background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer;font-size:0.8rem; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-secondary { padding:0.35rem 0.6rem;background:transparent;border:1px solid #475569;color:#94A3B8;border-radius:6px;cursor:pointer;font-size:0.75rem; }
  .btn-secondary:hover { color:#E6EDF3;border-color:#3ABEFF; }
  .btn-icon { background:none;border:none;color:#EF4444;cursor:pointer;font-size:0.9rem; }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
  .empty { color:#475569;font-size:0.8rem;text-align:center;padding:1rem; }
</style>
