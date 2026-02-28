<!--
  /directors/meetings — Meeting Management (UC-DIR-03)
  All 13 Directors can create meetings and view scheduled meetings.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    createMeeting, getMeetings, getPersonnelList,
    type MeetingSummary, type PersonnelListItem,
  } from '$lib/stores/directors';

  let meetings: MeetingSummary[] = $state([]);
  let personnel: PersonnelListItem[] = $state([]);

  let title = $state('');
  let agenda = $state('');
  let scheduledAt = $state('');
  let selectedInvitees: string[] = $state([]);
  let inviteeSearch = $state('');

  let error = $state('');
  let success = $state('');

  onMount(async () => {
    const [m, p] = await Promise.all([getMeetings(), getPersonnelList()]);
    meetings = m;
    personnel = p;
  });

  function filteredPersonnel() {
    if (!inviteeSearch.trim()) return personnel.slice(0, 20);
    const q = inviteeSearch.toLowerCase();
    return personnel.filter(
      (p) => p.full_name.toLowerCase().includes(q) || p.role_name.toLowerCase().includes(q)
    );
  }

  function toggleInvitee(id: string) {
    if (selectedInvitees.includes(id)) {
      selectedInvitees = selectedInvitees.filter((i) => i !== id);
    } else {
      selectedInvitees = [...selectedInvitees, id];
    }
  }

  function inviteeName(id: string): string {
    return personnel.find((p) => p.id === id)?.full_name ?? id.slice(0, 8);
  }

  async function handleCreate() {
    error = ''; success = '';
    if (!title.trim()) { error = 'Title is required.'; return; }
    if (!scheduledAt) { error = 'Date/time is required.'; return; }
    try {
      const result = await createMeeting({
        title,
        agenda: agenda || undefined,
        scheduled_at: new Date(scheduledAt).toISOString(),
        invitee_ids: selectedInvitees.length > 0 ? selectedInvitees : [],
      });
      success = `Meeting created (${result.id.slice(0, 8)}…)`;
      title = ''; agenda = ''; scheduledAt = ''; selectedInvitees = [];
      meetings = await getMeetings();
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }

  function formatDate(d: string): string {
    return new Date(d).toLocaleString();
  }
</script>

<h1 class="title">Meetings</h1>
<p class="subtitle">Schedule meetings and invite personnel across the organization.</p>

<div class="layout">
  <!-- Create form -->
  <div class="form-card">
    <h2>New Meeting</h2>
    <label class="field"><span class="label">Title</span>
      <input type="text" class="input" bind:value={title} placeholder="Meeting title…" />
    </label>
    <label class="field"><span class="label">Date & Time</span>
      <input type="datetime-local" class="input" bind:value={scheduledAt} />
    </label>
    <label class="field"><span class="label">Agenda</span>
      <textarea class="textarea" bind:value={agenda} rows="3" placeholder="Topics to discuss…"></textarea>
    </label>

    <div class="invitee-section">
      <span class="label">Invitees ({selectedInvitees.length})</span>
      <div class="chips">
        {#each selectedInvitees as inv}
          <span class="chip">
            {inviteeName(inv)}
            <button class="chip-x" onclick={() => toggleInvitee(inv)}>×</button>
          </span>
        {/each}
      </div>
      <input type="text" class="input-sm" bind:value={inviteeSearch} placeholder="Search personnel…" />
      <div class="invitee-list">
        {#each filteredPersonnel() as p}
          <button
            class="inv-row" class:inv-selected={selectedInvitees.includes(p.id)}
            onclick={() => toggleInvitee(p.id)}
          >
            <span>{p.full_name}</span>
            <span class="role-hint">{p.role_name}</span>
          </button>
        {/each}
      </div>
    </div>

    {#if error}<p class="error">{error}</p>{/if}
    {#if success}<p class="success">{success}</p>{/if}
    <button class="btn-primary" onclick={handleCreate}>Create Meeting</button>
  </div>

  <!-- Meetings list -->
  <div class="meetings-panel">
    <h2>Scheduled Meetings</h2>
    {#each meetings as m}
      <div class="card">
        <div class="card-header">
          <span class="card-title">{m.title}</span>
          <span class="card-date">{formatDate(m.scheduled_at)}</span>
        </div>
        {#if m.agenda}
          <p class="card-body">{m.agenda}</p>
        {/if}
      </div>
    {:else}
      <p class="empty">No meetings scheduled.</p>
    {/each}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .layout { display:flex;gap:1.5rem;flex-wrap:wrap; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;width:420px;display:flex;flex-direction:column;gap:0.6rem; }
  .form-card h2,.meetings-panel h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0 0 0.5rem; }
  .meetings-panel { flex:1;min-width:320px; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .input-sm { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:4px;padding:0.3rem;font-size:0.75rem;font-family:'Inter',sans-serif;margin-bottom:0.3rem;box-sizing:border-box; }
  .invitee-section { display:flex;flex-direction:column;gap:0.3rem; }
  .chips { display:flex;flex-wrap:wrap;gap:0.25rem; }
  .chip { display:inline-flex;align-items:center;gap:0.25rem;background:rgba(139,92,246,0.15);border:1px solid rgba(139,92,246,0.3);border-radius:4px;padding:0.12rem 0.4rem;font-size:0.7rem;color:#C084FC; }
  .chip-x { background:none;border:none;color:#EF4444;cursor:pointer;font-size:0.8rem;padding:0; }
  .invitee-list { max-height:120px;overflow-y:auto;border:1px solid rgba(58,190,255,0.08);border-radius:4px; }
  .inv-row { display:flex;justify-content:space-between;width:100%;background:transparent;border:none;border-bottom:1px solid rgba(255,255,255,0.03);color:#E6EDF3;padding:0.3rem 0.5rem;cursor:pointer;font-size:0.75rem;font-family:'Inter',sans-serif;text-align:left; }
  .inv-row:hover { background:rgba(58,190,255,0.05); }
  .inv-row.inv-selected { background:rgba(139,92,246,0.1);color:#C084FC; }
  .role-hint { color:#475569;font-size:0.65rem; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;padding:0.5rem;cursor:pointer;font-size:0.85rem;font-weight:600; }
  .btn-primary:hover { background:rgba(58,190,255,0.25); }
  .card { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.7rem;margin-bottom:0.4rem; }
  .card-header { display:flex;justify-content:space-between;align-items:center; }
  .card-title { font-size:0.85rem;font-weight:500; }
  .card-date { font-size:0.7rem;color:#94A3B8; }
  .card-body { font-size:0.8rem;color:#94A3B8;margin:0.3rem 0 0;white-space:pre-wrap; }
  .empty { color:#475569;font-size:0.8rem;font-style:italic; }
  .error { color:#EF4444;font-size:0.75rem; }
  .success { color:#22C55E;font-size:0.75rem; }
</style>
