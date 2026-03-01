<!-- /sanitary/head/quotas — UC-HS-07: Set Division Quotas -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    sanGetDivisions, sanSetDivisionQuota,
    type DivisionRow,
  } from '$lib/stores/sanitary';

  let divisions: DivisionRow[] = $state([]);
  let error = $state('');
  let success = $state('');
  let editingId = $state('');
  let newMax: number = $state(0);

  onMount(async () => { await refresh(); });

  async function refresh() {
    error = ''; success = '';
    try { divisions = await sanGetDivisions(); } catch (e: unknown) { error = String(e); }
  }

  function startEdit(d: DivisionRow) {
    editingId = d.id;
    newMax = d.quota;
  }

  async function handleSave(id: string) {
    error = ''; success = '';
    try {
      await sanSetDivisionQuota({ division_id: id, quota: newMax });
      success = 'Quota updated.';
      editingId = '';
      await refresh();
    } catch (e: unknown) { error = String(e); }
  }
</script>

<h1 class="title">Division Quotas</h1>
<p class="subtitle">UC-HS-07 — Configure maximum staff quota for each sanitary division.</p>

{#if error}<p class="error">{error}</p>{/if}
{#if success}<p class="success">{success}</p>{/if}

<div class="table-wrap">
  <table class="data-table">
    <thead>
      <tr><th>Division</th><th>Quota</th><th></th></tr>
    </thead>
    <tbody>
      {#each divisions as d}
        <tr>
          <td class="div-name">{d.name}</td>
          <td>
            {#if editingId === d.id}
              <input class="input quota-input" type="number" min="0" bind:value={newMax} />
            {:else}
              <span class="quota-val">{d.quota}</span>
            {/if}
          </td>
          <td>
            {#if editingId === d.id}
              <button class="btn-primary btn-sm" onclick={() => handleSave(d.id)}>Save</button>
              <button class="btn-secondary btn-sm" onclick={() => { editingId = ''; }}>Cancel</button>
            {:else}
              <button class="btn-secondary btn-sm" onclick={() => startEdit(d)}>Edit</button>
            {/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 0.75rem; }
  .table-wrap { overflow-x:auto; }
  .data-table { width:100%;border-collapse:collapse;font-size:0.8rem;background:rgba(14,20,40,0.4);border-radius:8px;overflow:hidden; }
  .data-table th { text-align:left;color:#94A3B8;padding:0.5rem 0.6rem;border-bottom:1px solid rgba(58,190,255,0.1);font-size:0.75rem; }
  .data-table td { padding:0.45rem 0.6rem;border-bottom:1px solid rgba(58,190,255,0.04); }
  .div-name { font-weight:500;color:#E6EDF3; }
  .quota-val { color:#F59E0B;font-weight:600; }

  .quota-input { width:75px; }
  .input { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.3rem 0.4rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .btn-sm { padding:0.22rem 0.45rem;font-size:0.7rem; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;cursor:pointer; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .btn-secondary { background:transparent;border:1px solid #475569;color:#94A3B8;border-radius:6px;cursor:pointer;margin-left:0.2rem; }
  .btn-secondary:hover { color:#E6EDF3;border-color:#3ABEFF; }
  .error { color:#EF4444;font-size:0.8rem; }
  .success { color:#10B981;font-size:0.8rem; }
</style>
