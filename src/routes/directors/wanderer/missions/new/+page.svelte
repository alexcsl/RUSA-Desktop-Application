<!-- /directors/wanderer/missions/new — TheWanderer Territory & Missions (UC-DIR-14 + UC-WAN-01) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    renameTerritory, getTerritories, getPersonnelList,
    type TerritorySummary, type PersonnelListItem,
  } from '$lib/stores/directors';
  import {
    assignMission, getAllMissions,
    type MissionSummary,
  } from '$lib/stores/astronauts';

  let territories: TerritorySummary[] = $state([]);
  let missions: MissionSummary[] = $state([]);
  let personnel: PersonnelListItem[] = $state([]);

  // Rename form
  let selectedTerritory = $state('');
  let newName = $state('');
  let renameError = $state('');
  let renameSuccess = $state('');

  // Mission form
  let missionTitle = $state('');
  let missionType = $state('interstellar');
  let dangerLevel = $state('medium');
  let location = $state('');
  let missionObjective = $state('');
  let procedures = $state('');
  let knownDangers = $state('');
  let selectedAstronauts: string[] = $state([]);
  let missionError = $state('');
  let missionSuccess = $state('');

  const validRoles = ['Astronaut'];

  onMount(async () => {
    const [t, m, p] = await Promise.all([getTerritories(), getAllMissions(), getPersonnelList()]);
    territories = t;
    missions = m;
    personnel = p.filter(x => validRoles.includes(x.role_name));
  });

  async function handleRename() {
    renameError = ''; renameSuccess = '';
    if (!selectedTerritory || !newName.trim()) { renameError = 'Select territory and enter name.'; return; }
    try {
      await renameTerritory(selectedTerritory, newName);
      renameSuccess = 'Territory renamed.';
      newName = '';
      territories = await getTerritories();
    } catch (e: unknown) { renameError = e instanceof Error ? e.message : String(e); }
  }

  function toggleAstronaut(id: string) {
    if (selectedAstronauts.includes(id)) {
      selectedAstronauts = selectedAstronauts.filter(a => a !== id);
    } else {
      selectedAstronauts = [...selectedAstronauts, id];
    }
  }

  async function handleAssign() {
    missionError = ''; missionSuccess = '';
    if (!missionTitle.trim()) { missionError = 'Mission title is required.'; return; }
    if (!location.trim()) { missionError = 'Location is required.'; return; }
    if (selectedAstronauts.length === 0) { missionError = 'Select at least one astronaut.'; return; }
    try {
      await assignMission({
        title: missionTitle,
        type: missionType,
        danger_level: dangerLevel || undefined,
        location,
        mission_objective: missionObjective || undefined,
        procedures: procedures || undefined,
        known_dangers: knownDangers || undefined,
        astronaut_ids: selectedAstronauts,
      });
      missionSuccess = 'Mission assigned successfully.';
      missionTitle = ''; location = ''; missionObjective = ''; procedures = '';
      knownDangers = ''; selectedAstronauts = [];
      missions = await getAllMissions();
    } catch (e: unknown) { missionError = e instanceof Error ? e.message : String(e); }
  }

  function statusColor(s: string): string {
    switch (s) {
      case 'active': return '#3ABEFF';
      case 'completed': return '#10B981';
      case 'completion_requested': return '#F59E0B';
      case 'rejected': return '#EF4444';
      default: return '#94A3B8';
    }
  }
</script>

<h1 class="title">Wanderer — Territory & Missions</h1>
<p class="subtitle">Rename territories and assign exploration missions to Astronauts.</p>

<div class="layout">
  <!-- Territory naming -->
  <div class="form-card">
    <h2>Territories</h2>
    {#each territories as t}
      <div class="terr-item">
        <span class="terr-name">{t.name}</span>
        <span class="terr-type">{t.territory_type}</span>
        {#if t.previous_name}<span class="terr-prev">was: {t.previous_name}</span>{/if}
      </div>
    {:else}
      <p class="empty">No territories recorded.</p>
    {/each}

    <h3>Rename Territory</h3>
    <select class="input" bind:value={selectedTerritory}>
      <option value="">— Select —</option>
      {#each territories as t}<option value={t.id}>{t.name}</option>{/each}
    </select>
    <input type="text" class="input" bind:value={newName} placeholder="New name…" />
    {#if renameError}<p class="error">{renameError}</p>{/if}
    {#if renameSuccess}<p class="success">{renameSuccess}</p>{/if}
    <button class="btn-primary" onclick={handleRename}>Rename</button>
  </div>

  <!-- Mission assignment -->
  <div class="form-card">
    <h2>Assign Mission</h2>
    <label class="field"><span class="label">Mission Title *</span>
      <input type="text" class="input" bind:value={missionTitle} placeholder="e.g. Survey Sector 9" />
    </label>
    <div class="row">
      <label class="field" style="flex:1"><span class="label">Type *</span>
        <select class="input" bind:value={missionType}>
          <option value="interstellar">Interstellar</option>
          <option value="terrain">Terrain</option>
        </select>
      </label>
      <label class="field" style="flex:1"><span class="label">Danger Level</span>
        <select class="input" bind:value={dangerLevel}>
          <option value="low">Low</option>
          <option value="medium">Medium</option>
          <option value="high">High</option>
          <option value="critical">Critical</option>
        </select>
      </label>
    </div>
    <label class="field"><span class="label">Location *</span>
      <input type="text" class="input" bind:value={location} placeholder="e.g. Kepler-442b Quadrant North" />
    </label>
    <label class="field"><span class="label">Mission Objective</span>
      <textarea class="textarea" bind:value={missionObjective} rows="2" placeholder="Describe the mission goals…"></textarea>
    </label>
    <label class="field"><span class="label">Procedures</span>
      <textarea class="textarea" bind:value={procedures} rows="2" placeholder="Step-by-step instructions…"></textarea>
    </label>
    <label class="field"><span class="label">Known Dangers</span>
      <textarea class="textarea" bind:value={knownDangers} rows="2" placeholder="Hazards the crew should expect…"></textarea>
    </label>

    <div class="field">
      <span class="label">Assign Astronauts * ({selectedAstronauts.length} selected)</span>
      <div class="astronaut-grid">
        {#each personnel as p}
          <button
            class="astro-chip"
            class:selected={selectedAstronauts.includes(p.id)}
            onclick={() => toggleAstronaut(p.id)}
          >
            {p.full_name}
          </button>
        {:else}
          <p class="empty">No astronauts found.</p>
        {/each}
      </div>
    </div>

    {#if missionError}<p class="error">{missionError}</p>{/if}
    {#if missionSuccess}<p class="success">{missionSuccess}</p>{/if}
    <button class="btn-primary" onclick={handleAssign}>Assign Mission</button>

    <h3>All Missions</h3>
    {#each missions as m}
      <div class="task-item">
        <div class="task-info">
          <span class="task-name">{m.title}</span>
          <span class="task-meta">{m.mission_type} · {m.location}</span>
        </div>
        <span class="badge" style="color:{statusColor(m.status)};background:rgba({m.status === 'active' ? '58,190,255' : m.status === 'completed' ? '16,185,129' : m.status === 'completion_requested' ? '245,158,11' : '148,163,184'},0.15)">{m.status.replace('_', ' ')}</span>
      </div>
    {:else}
      <p class="empty">No missions yet.</p>
    {/each}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .layout { display:flex;gap:1.5rem;flex-wrap:wrap; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;flex:1;min-width:320px;display:flex;flex-direction:column;gap:0.5rem; }
  .form-card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0; }
  h3 { font-size:0.8rem;color:#E6EDF3;margin:0.75rem 0 0.3rem; }
  .row { display:flex;gap:1rem; }
  .terr-item { display:flex;gap:0.75rem;align-items:center;font-size:0.8rem;padding:0.3rem 0;border-bottom:1px solid rgba(255,255,255,0.03); }
  .terr-name { font-weight:500; }
  .terr-type { color:#8B5CF6;font-size:0.7rem; }
  .terr-prev { color:#94A3B8;font-size:0.7rem;font-style:italic; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .astronaut-grid { display:flex;flex-wrap:wrap;gap:0.35rem;margin-top:0.2rem; }
  .astro-chip { padding:0.25rem 0.6rem;border-radius:12px;font-size:0.72rem;background:rgba(58,190,255,0.08);border:1px solid rgba(58,190,255,0.2);color:#94A3B8;cursor:pointer;transition:all 0.15s; }
  .astro-chip:hover { border-color:#3ABEFF;color:#E6EDF3; }
  .astro-chip.selected { background:rgba(58,190,255,0.2);border-color:#3ABEFF;color:#3ABEFF;font-weight:600; }
  .task-item { display:flex;justify-content:space-between;align-items:center;font-size:0.8rem;padding:0.35rem 0;border-bottom:1px solid rgba(255,255,255,0.03); }
  .task-info { display:flex;flex-direction:column;gap:0.1rem; }
  .task-name { font-weight:500; }
  .task-meta { font-size:0.68rem;color:#64748B; }
  .badge { padding:0.1rem 0.35rem;border-radius:4px;font-size:0.65rem;text-transform:capitalize; }
  .btn-primary { padding:0.45rem 1rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .empty { color:#475569;font-size:0.8rem; }
</style>
