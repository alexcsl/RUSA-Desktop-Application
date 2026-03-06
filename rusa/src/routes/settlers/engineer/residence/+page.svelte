<!--
  UC-CE-04: View Residence Information (Civil Engineer)
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { stlGetResidence, type ResidenceInfo } from '$lib/stores/settlers';

  let info: ResidenceInfo | null = $state(null);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try { info = await stlGetResidence(); }
    catch (e: any) { error = e?.message ?? String(e); }
    finally { loading = false; }
  });
</script>

<h2>Residence Information</h2>

{#if loading}
  <p class="dim">Loading…</p>
{:else if error}
  <p class="err">{error}</p>
{:else if info}
  <div class="card">
    <div class="row">
      <div class="field">
        <span class="lbl">Settlement</span>
        <span class="val">{info.settlement_name}</span>
      </div>
      <div class="field">
        <span class="lbl">Planet</span>
        <span class="val">{info.planet}</span>
      </div>
    </div>
    <div class="row">
      <div class="field">
        <span class="lbl">Residence Unit</span>
        <span class="val">{info.residence_unit ?? '—'}</span>
      </div>
      <div class="field">
        <span class="lbl">Arrived</span>
        <span class="val">{info.arrived_at ?? '—'}</span>
      </div>
    </div>
  </div>
{:else}
  <p class="dim">No residence data available.</p>
{/if}

<style>
  h2 { font-family:'Orbitron',sans-serif;color:#3ABEFF;font-size:1.1rem;margin:0 0 1rem; }
  .dim { color:#64748B;font-size:0.8rem; }
  .err { color:#EF4444;font-size:0.8rem; }
  .card { background:#111827;border:1px solid rgba(58,190,255,0.15);border-radius:6px;padding:1rem;max-width:560px; }
  .row { display:flex;gap:1.5rem;margin-bottom:0.7rem; }
  .field { display:flex;flex-direction:column;flex:1; }
  .lbl { font-size:0.7rem;color:#64748B;margin-bottom:0.1rem;text-transform:uppercase;letter-spacing:0.04em; }
  .val { font-size:0.85rem;color:#E6EDF3; }
</style>
