<!--
  /psychiatry/patient-index — Shared Patient Index (UC-PSY-05)
  Cross-department coordination view — no clinical data exposed.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { psyGetPatientIndex, type PatientIndexEntry } from '$lib/stores/psychiatry';

  let entries: PatientIndexEntry[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let search = $state('');

  onMount(async () => {
    try { entries = await psyGetPatientIndex(); }
    catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    loading = false;
  });

  function filtered() {
    if (!search) return entries;
    const q = search.toLowerCase();
    return entries.filter((e) =>
      e.patient_name.toLowerCase().includes(q) || e.department.toLowerCase().includes(q) || e.care_status.toLowerCase().includes(q)
    );
  }

  function deptBadge(d: string) {
    if (d === 'psychiatry') return 'badge-psy';
    if (d === 'medical') return 'badge-med';
    if (d === 'both') return 'badge-both';
    return '';
  }
</script>

<div class="page">
  <h2>Patient Index</h2>
  <p class="muted sub">Cross-department coordination — no clinical data shown.</p>

  <input class="input search" type="text" placeholder="Search by name, department, or status…" bind:value={search} />

  {#if loading}
    <p class="muted">Loading…</p>
  {:else if error}
    <p class="error">{error}</p>
  {:else if filtered().length === 0}
    <p class="muted">No entries found.</p>
  {:else}
    <table class="tbl">
      <thead>
        <tr><th>Name</th><th>Department</th><th>Status</th><th>Last Updated</th></tr>
      </thead>
      <tbody>
        {#each filtered() as e}
          <tr>
            <td>{e.patient_name}</td>
            <td><span class="badge {deptBadge(e.department)}">{e.department}</span></td>
            <td>{e.care_status}</td>
            <td class="muted">{e.last_updated ? new Date(e.last_updated).toLocaleDateString() : '—'}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .page { max-width:900px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#3ABEFF;margin-bottom:0.2rem; }
  .sub { margin-bottom:1rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .error { color:#EF4444;font-size:0.8rem; }
  .search { max-width:400px;margin-bottom:1rem; }
  .input { width:100%;padding:0.45rem 0.6rem;background:#1F2937;color:#E6EDF3;border:1px solid #374151;border-radius:4px;font-size:0.8rem;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .tbl { width:100%;border-collapse:collapse;font-size:0.8rem; }
  .tbl th { text-align:left;padding:0.5rem 0.6rem;color:#94A3B8;border-bottom:1px solid rgba(58,190,255,0.1); }
  .tbl td { padding:0.5rem 0.6rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .badge { padding:0.15rem 0.5rem;border-radius:4px;font-size:0.7rem;font-weight:600; }
  .badge-psy { background:rgba(139,92,246,0.15);color:#C084FC; }
  .badge-med { background:rgba(16,185,129,0.15);color:#34D399; }
  .badge-both { background:rgba(58,190,255,0.15);color:#3ABEFF; }
</style>
