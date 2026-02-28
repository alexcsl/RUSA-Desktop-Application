<!--
  UC-SC-07: Commander Supply Request (direct to Directors)
-->
<script lang="ts">
  import { stlSubmitCommanderSupply, type SupplyItem } from '$lib/stores/settlers';

  let items: SupplyItem[] = $state([{ item: '', specification: '', quantity: 1, reason: '' }]);
  let justification = $state('');
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  function addRow() { items = [...items, { item: '', specification: '', quantity: 1, reason: '' }]; }
  function removeRow(i: number) { items = items.filter((_, idx) => idx !== i); }

  async function handleSubmit() {
    error = ''; success = '';
    const valid = items.filter(r => r.item.trim());
    if (valid.length === 0) { error = 'Add at least one item.'; return; }
    if (!justification.trim()) { error = 'Justification is required.'; return; }
    submitting = true;
    try {
      const id = await stlSubmitCommanderSupply({ items: valid, justification });
      success = `Commander supply request submitted (ID: ${id.slice(0,8)}…). Directors notified.`;
      items = [{ item: '', specification: '', quantity: 1, reason: '' } as SupplyItem];
      justification = '';
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }
</script>

<h2>Commander Supply Request</h2>
<p class="hint">This request goes directly to the Directors board.</p>

<form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  <table class="tbl">
    <thead>
      <tr><th>Item</th><th>Spec / Details</th><th>Qty</th><th>Reason</th><th></th></tr>
    </thead>
    <tbody>
      {#each items as row, i}
        <tr>
          <td><input type="text" bind:value={row.item} placeholder="Item name" /></td>
          <td><input type="text" bind:value={row.specification} placeholder="Spec" /></td>
          <td><input type="number" bind:value={row.quantity} min="1" style="width:60px" /></td>
          <td><input type="text" bind:value={row.reason} placeholder="Reason" /></td>
          <td>
            {#if items.length > 1}
              <button type="button" class="btn-del" onclick={() => removeRow(i)}>✕</button>
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>

  <button type="button" class="btn-ghost" onclick={addRow}>+ Add Row</button>

  <label>
    Justification *
    <textarea bind:value={justification} rows="3" required placeholder="Why are these supplies needed?"></textarea>
  </label>

  {#if error}<p class="err">{error}</p>{/if}
  {#if success}<p class="ok">{success}</p>{/if}

  <button type="submit" class="btn-primary" disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit to Directors'}
  </button>
</form>

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 0.3rem; }
  .hint { color:#64748B;font-size:0.75rem;margin:0 0 1rem; }
  .form { display:flex;flex-direction:column;gap:0.75rem;max-width:720px; }
  .form label { display:flex;flex-direction:column;gap:0.2rem;font-size:0.8rem;color:#94A3B8; }
  .form textarea { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.4rem 0.5rem;font-size:0.8rem;resize:vertical; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.3rem 0.4rem;border-bottom:1px solid #374151;color:#94A3B8;font-weight:500; }
  .tbl td { padding:0.3rem 0.4rem; }
  .tbl input { background:#111827;border:1px solid #374151;border-radius:4px;color:#E6EDF3;padding:0.3rem 0.4rem;font-size:0.8rem;width:100%; }
  .btn-del { background:none;border:none;color:#EF4444;cursor:pointer;font-size:0.85rem; }
  .btn-ghost { background:none;border:1px dashed #374151;color:#3ABEFF;padding:0.35rem 0.9rem;border-radius:4px;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .err { color:#EF4444;font-size:0.8rem; }
  .ok { color:#4ADE80;font-size:0.8rem; }
  .btn-primary { background:#3ABEFF;color:#0B0F1A;border:none;border-radius:4px;padding:0.5rem 1.2rem;cursor:pointer;font-weight:600;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:hover { background:#60CFFF; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
