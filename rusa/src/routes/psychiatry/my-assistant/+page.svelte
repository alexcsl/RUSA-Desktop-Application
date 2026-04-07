<!--
  /psychiatry/my-assistant — Psychiatrist assigns / removes their one assistant (FR-7-09)
  "each psychiatrist can only have one assistant" — Interview 7_PsychiatryDivision
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    psyGetMyAssistant,
    psyAssignAssistant,
    psyRemoveAssistant,
    psyGetAssistantCandidates,
    type AssistantAssignment,
    type UserPickerEntry,
  } from '$lib/stores/psychiatry';

  let current: AssistantAssignment | null = $state(null);
  let candidates: UserPickerEntry[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let success = $state('');
  let saving = $state(false);
  let removing = $state(false);

  let search = $state('');
  let selectedId = $state('');

  onMount(async () => {
    try {
      [current, candidates] = await Promise.all([
        psyGetMyAssistant(),
        psyGetAssistantCandidates(),
      ]);
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    loading = false;
  });

  function filtered(): UserPickerEntry[] {
    if (!search.trim()) return candidates;
    const q = search.toLowerCase();
    return candidates.filter(
      (c) => c.full_name.toLowerCase().includes(q) || c.username.toLowerCase().includes(q)
    );
  }

  async function assign() {
    if (!selectedId) { error = 'Please select an assistant first.'; return; }
    saving = true; error = ''; success = '';
    try {
      await psyAssignAssistant(selectedId);
      current = await psyGetMyAssistant();
      selectedId = '';
      success = 'Assistant assigned successfully.';
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    saving = false;
  }

  async function remove() {
    removing = true; error = ''; success = '';
    try {
      await psyRemoveAssistant();
      current = null;
      success = 'Assistant removed.';
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
    removing = false;
  }
</script>

<div class="page">
  <h2>My Assistant</h2>
  <p class="sub">Each psychiatrist may have one assistant. The assistant handles scheduling and has read-only access to recovery logs.</p>

  {#if success}<div class="banner success">{success}</div>{/if}
  {#if error}<div class="banner error">{error}</div>{/if}

  {#if loading}
    <p class="muted">Loading…</p>
  {:else}
    <!-- Current assignment -->
    <section class="card">
      <h3>Current Assignment</h3>
      {#if current}
        <div class="assignment-row">
          <div class="avatar">{current.assistant_name[0]}</div>
          <div class="info">
            <span class="name">{current.assistant_name}</span>
            <span class="meta">Assigned {new Date(current.assigned_at).toLocaleDateString()}</span>
          </div>
          <button class="btn-remove" onclick={remove} disabled={removing}>
            {removing ? 'Removing…' : 'Remove'}
          </button>
        </div>
      {:else}
        <p class="muted">No assistant currently assigned.</p>
      {/if}
    </section>

    <!-- Assign new -->
    <section class="card">
      <h3>{current ? 'Replace Assistant' : 'Assign Assistant'}</h3>
      {#if current}
        <p class="warn-note">Assigning a new assistant will remove the current one.</p>
      {/if}
      <input class="input" type="text" placeholder="Search by name or username…" bind:value={search} />
      {#if candidates.length === 0}
        <p class="muted">No Psychiatrist Assistants registered in the system.</p>
      {:else}
        <div class="pick-list">
          {#each filtered() as c}
            <label class="pick-row" class:selected={selectedId === c.id}>
              <input type="radio" name="assistant" value={c.id} bind:group={selectedId} />
              <span class="name">{c.full_name}</span>
              <span class="uname muted">@{c.username}</span>
            </label>
          {/each}
          {#if filtered().length === 0}
            <p class="muted">No matches.</p>
          {/if}
        </div>
        <button class="btn-primary" onclick={assign} disabled={saving || !selectedId}>
          {saving ? 'Assigning…' : 'Assign'}
        </button>
      {/if}
    </section>
  {/if}
</div>

<style>
  .page { max-width:680px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.15rem;color:#3ABEFF;margin-bottom:0.2rem; }
  h3 { font-family:'Orbitron',sans-serif;font-size:0.85rem;color:#C084FC;margin-bottom:0.7rem; }
  .sub { font-size:0.8rem;color:#94A3B8;margin-bottom:1.2rem; }
  .muted { color:#94A3B8;font-size:0.8rem; }
  .card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1rem;margin-bottom:1rem; }
  .banner { padding:0.6rem 0.9rem;border-radius:6px;font-size:0.8rem;margin-bottom:0.8rem; }
  .banner.success { background:rgba(16,185,129,0.1);border:1px solid rgba(16,185,129,0.3);color:#34D399; }
  .banner.error { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);color:#F87171; }
  .warn-note { font-size:0.77rem;color:#FBBF24;margin-bottom:0.6rem; }

  .assignment-row { display:flex;align-items:center;gap:0.9rem; }
  .avatar { width:38px;height:38px;border-radius:50%;background:rgba(139,92,246,0.2);color:#C084FC;display:flex;align-items:center;justify-content:center;font-size:1rem;font-weight:700;flex-shrink:0; }
  .info { flex:1;display:flex;flex-direction:column;gap:0.15rem; }
  .name { font-size:0.88rem;color:#E6EDF3; }
  .meta { font-size:0.72rem;color:#94A3B8; }
  .uname { font-size:0.72rem; }
  .btn-remove { padding:0.25rem 0.7rem;background:rgba(239,68,68,0.1);color:#F87171;border:1px solid rgba(239,68,68,0.3);border-radius:4px;cursor:pointer;font-size:0.75rem; }
  .btn-remove:hover:not(:disabled) { background:rgba(239,68,68,0.2); }
  .btn-remove:disabled { opacity:0.5;cursor:not-allowed; }

  .input { width:100%;padding:0.45rem 0.6rem;background:#1F2937;color:#E6EDF3;border:1px solid #374151;border-radius:4px;font-size:0.8rem;margin-bottom:0.5rem;box-sizing:border-box; }
  .input:focus { outline:none;border-color:#3ABEFF; }
  .pick-list { max-height:220px;overflow-y:auto;border:1px solid rgba(255,255,255,0.05);border-radius:4px;margin-bottom:0.7rem; }
  .pick-row { display:flex;align-items:center;gap:0.6rem;padding:0.4rem 0.7rem;cursor:pointer;font-size:0.82rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .pick-row:hover { background:rgba(58,190,255,0.04); }
  .pick-row.selected { background:rgba(139,92,246,0.1);border-left:3px solid #8B5CF6; }
  .btn-primary { padding:0.5rem 1.2rem;background:rgba(139,92,246,0.15);color:#C084FC;border:1px solid rgba(139,92,246,0.4);border-radius:6px;cursor:pointer;font-size:0.82rem;font-weight:600; }
  .btn-primary:hover:not(:disabled) { background:rgba(139,92,246,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
</style>
