<!--
  UC-CE-02: Request Construction Materials (Civil Engineer)
-->
<script lang="ts">
  import { stlRequestMaterials, type SupplyItem } from '$lib/stores/settlers';

  let materials: SupplyItem[] = $state([{ item: '', specification: '', quantity: 1, unit: 'unit' }]);
  let justification = $state('');
  let error = $state('');
  let success = $state('');
  let submitting = $state(false);

  function addRow() { materials = [...materials, { item: '', specification: '', quantity: 1, unit: 'unit' } as SupplyItem]; }
  function removeRow(i: number) { materials = materials.filter((_, idx) => idx !== i); }

  async function handleSubmit() {
    error = ''; success = '';
    const valid = materials.filter(r => r.item.trim());
    if (valid.length === 0) { error = 'Add at least one material.'; return; }
    if (!justification.trim()) { error = 'Justification is required.'; return; }
    submitting = true;
    try {
      const id = await stlRequestMaterials({ items: valid, justification });
      success = `Material request submitted (ID: ${id.slice(0,8)}…). Commander notified.`;
      materials = [{ item: '', specification: '', quantity: 1, unit: 'unit' } as SupplyItem];
      justification = '';
    } catch (e: any) { error = e?.message ?? String(e); }
    finally { submitting = false; }
  }
</script>

<h2>Request Construction Materials</h2>

<form class="form" onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
  <table class="tbl">
    <thead>
      <tr><th>Material</th><th>Spec</th><th>Qty</th><th>Unit</th><th></th></tr>
    </thead>
    <tbody>
      {#each materials as row, i}
        <tr>
          <td><input type="text" bind:value={row.item} placeholder="Material name" /></td>
          <td><input type="text" bind:value={row.specification} placeholder="Grade / spec" /></td>
          <td><input type="number" bind:value={row.quantity} min="1" style="width:60px" /></td>
          <td><input type="text" bind:value={row.unit} placeholder="kg, m³…" style="width:60px" /></td>
          <td>
            {#if materials.length > 1}
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
    <textarea bind:value={justification} rows="3" required placeholder="Why are these materials needed?"></textarea>
  </label>

  {#if error}<p class="err">{error}</p>{/if}
  {#if success}<p class="ok">{success}</p>{/if}

  <button type="submit" class="btn-primary" disabled={submitting}>
    {submitting ? 'Submitting…' : 'Submit Material Request'}
  </button>
</form>

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
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
