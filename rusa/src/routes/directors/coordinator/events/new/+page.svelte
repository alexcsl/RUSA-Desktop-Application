<!-- /directors/coordinator/events/new — TheCoordinator Event Management (UC-DIR-18) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    createEvent, getEvents, uploadEventDocument, getEventDocuments,
    getPersonnelList,
    type EventSummary, type EventDocSummary, type PersonnelListItem, type EventAttendeeInput,
  } from '$lib/stores/directors';

  let events: EventSummary[] = $state([]);
  let personnel: PersonnelListItem[] = $state([]);
  let selectedEvent: EventSummary | null = $state(null);
  let eventDocs: EventDocSummary[] = $state([]);

  // Create form
  let title = $state('');
  let eventDate = $state('');
  let location = $state('');
  let agenda = $state('');
  let attendees: EventAttendeeInput[] = $state([]);
  let newAttendeeName = $state('');
  let newAttendeeIsExternal = $state(false);
  let newAttendeeUserId = $state('');
  let error = $state('');
  let success = $state('');

  // Upload form
  let uploadDocType = $state('other');
  let uploadError = $state('');
  let uploadSuccess = $state('');

  onMount(async () => {
    [events, personnel] = await Promise.all([getEvents(), getPersonnelList()]);
  });

  function addAttendee() {
    if (!newAttendeeName.trim()) return;
    attendees = [...attendees, {
      name: newAttendeeName,
      is_external: newAttendeeIsExternal,
      user_id: newAttendeeIsExternal ? undefined : newAttendeeUserId || undefined,
    }];
    newAttendeeName = ''; newAttendeeUserId = ''; newAttendeeIsExternal = false;
  }

  function removeAttendee(i: number) {
    attendees = attendees.filter((_, idx) => idx !== i);
  }

  async function handleCreate() {
    error = ''; success = '';
    if (!title.trim()) { error = 'Title required.'; return; }
    try {
      await createEvent({
        title,
        event_date: eventDate ? new Date(eventDate + 'T00:00:00Z').toISOString() : undefined,
        location: location || undefined, agenda: agenda || undefined, attendees,
      });
      success = 'Event created.';
      title = ''; eventDate = ''; location = ''; agenda = ''; attendees = [];
      events = await getEvents();
    } catch (e: unknown) { error = e instanceof Error ? e.message : String(e); }
  }

  async function selectEvent(ev: EventSummary) {
    selectedEvent = ev;
    eventDocs = await getEventDocuments(ev.id);
  }

  async function handleUpload(e: Event) {
    const target = e.target as HTMLInputElement;
    if (!target.files?.length || !selectedEvent) return;
    uploadError = ''; uploadSuccess = '';
    const file = target.files[0];
    try {
      const buffer = await file.arrayBuffer();
      const bytes = Array.from(new Uint8Array(buffer));
      await uploadEventDocument(selectedEvent.id, bytes, file.name, file.type || 'application/octet-stream', uploadDocType);
      uploadSuccess = `Uploaded ${file.name}`;
      eventDocs = await getEventDocuments(selectedEvent.id);
    } catch (err: unknown) { uploadError = err instanceof Error ? err.message : String(err); }
    target.value = '';
  }
</script>

<h1 class="title">Coordinator — Event Management</h1>
<p class="subtitle">Create events, manage attendees, and upload event documents.</p>

<div class="layout">
  <!-- Create form -->
  <div class="form-card">
    <h2>New Event</h2>
    <label class="field"><span class="label">Title</span>
      <input type="text" class="input" bind:value={title} placeholder="Event title…" />
    </label>
    <label class="field"><span class="label">Date</span>
      <input type="date" class="input" bind:value={eventDate} />
    </label>
    <label class="field"><span class="label">Location</span>
      <input type="text" class="input" bind:value={location} placeholder="e.g. Conference Hall A" />
    </label>
    <label class="field"><span class="label">Agenda</span>
      <textarea class="textarea" bind:value={agenda} rows="3" placeholder="Event agenda…"></textarea>
    </label>

    <div class="attendee-section">
      <span class="label">Attendees ({attendees.length})</span>
      {#each attendees as att, i}
        <div class="att-chip">
          <span>{att.name} {att.is_external ? '(ext)' : ''}</span>
          <button class="att-remove" onclick={() => removeAttendee(i)}>×</button>
        </div>
      {/each}
      <div class="att-add">
        <input type="text" class="input-sm" bind:value={newAttendeeName} placeholder="Name…" />
        <label class="check-label">
          <input type="checkbox" bind:checked={newAttendeeIsExternal} /> External
        </label>
        {#if !newAttendeeIsExternal}
          <select class="input-sm" bind:value={newAttendeeUserId}>
            <option value="">Link user (opt)</option>
            {#each personnel as p}<option value={p.id}>{p.full_name}</option>{/each}
          </select>
        {/if}
        <button class="btn-sm-add" onclick={addAttendee}>+</button>
      </div>
    </div>

    {#if error}<p class="error">{error}</p>{/if}
    {#if success}<p class="success">{success}</p>{/if}
    <button class="btn-primary" onclick={handleCreate}>Create Event</button>
  </div>

  <!-- Event list + docs -->
  <div class="events-panel">
    <h2>Events</h2>
    {#each events as ev}
      <button class="card" class:selected={selectedEvent?.id === ev.id} onclick={() => selectEvent(ev)}>
        <div class="card-title">{ev.title}</div>
        <div class="card-meta">
          {#if ev.event_date}<span>{ev.event_date}</span>{/if}
          {#if ev.location}<span>{ev.location}</span>{/if}
        </div>
      </button>
    {:else}
      <p class="empty">No events yet.</p>
    {/each}

    {#if selectedEvent}
      <div class="docs-section">
        <h3>Documents for: {selectedEvent.title}</h3>
        {#each eventDocs as doc}
          <div class="doc-item">{doc.original_filename} <span class="doc-type">({doc.document_type})</span></div>
        {:else}
          <p class="empty">No documents uploaded.</p>
        {/each}

        <div class="upload-row">
          <select class="input-sm" bind:value={uploadDocType}>
            <option value="ticket">Ticket</option>
            <option value="invoice">Invoice</option>
            <option value="contract">Contract</option>
            <option value="other">Other</option>
          </select>
          <input
            type="file"
            class="file-input"
            accept=".pdf,.doc,.docx,.xls,.xlsx,.csv,.txt,.png,.jpg,.jpeg,.gif,.webp,.svg"
            onchange={handleUpload}
          />
          <span class="file-hint">PDF, DOC, XLS, CSV, TXT, PNG, JPG, GIF, WEBP, SVG</span>
        </div>
        {#if uploadError}<p class="error">{uploadError}</p>{/if}
        {#if uploadSuccess}<p class="success">{uploadSuccess}</p>{/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .layout { display:flex;gap:1.5rem;flex-wrap:wrap; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;width:420px;display:flex;flex-direction:column;gap:0.6rem; }
  .form-card h2,.events-panel h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0 0 0.5rem; }
  .events-panel { flex:1;min-width:320px; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .input-sm { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:4px;padding:0.3rem;font-size:0.75rem;font-family:'Inter',sans-serif; }
  .attendee-section { display:flex;flex-direction:column;gap:0.3rem; }
  .att-chip { display:inline-flex;align-items:center;gap:0.3rem;background:rgba(139,92,246,0.15);border:1px solid rgba(139,92,246,0.3);border-radius:4px;padding:0.15rem 0.5rem;font-size:0.75rem;color:#C084FC; }
  .att-remove { background:none;border:none;color:#EF4444;cursor:pointer;font-size:0.85rem; }
  .att-add { display:flex;gap:0.3rem;align-items:center;flex-wrap:wrap; }
  .check-label { font-size:0.7rem;color:#94A3B8;display:flex;align-items:center;gap:0.2rem; }
  .btn-sm-add { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:4px;padding:0.2rem 0.5rem;cursor:pointer;font-size:0.8rem; }
  .card { width:100%;text-align:left;background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.6rem;margin-bottom:0.35rem;cursor:pointer;color:#E6EDF3; }
  .card:hover { border-color:rgba(58,190,255,0.3); }
  .card.selected { border-color:#3ABEFF;background:rgba(58,190,255,0.08); }
  .card-title { font-size:0.8rem;font-weight:500;margin-bottom:0.15rem; }
  .card-meta { font-size:0.7rem;color:#94A3B8;display:flex;gap:0.75rem; }
  .docs-section { margin-top:1rem;background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:8px;padding:1rem; }
  h3 { font-size:0.85rem;color:#E6EDF3;margin:0 0 0.5rem; }
  .doc-item { font-size:0.8rem;padding:0.25rem 0;border-bottom:1px solid rgba(255,255,255,0.03); }
  .doc-type { color:#94A3B8;font-size:0.7rem; }
  .upload-row { display:flex;gap:0.5rem;align-items:center;margin-top:0.5rem;flex-wrap:wrap; }
  .file-input { font-size:0.75rem;color:#94A3B8; }
  .file-hint { font-size:0.65rem;color:#64748B;font-style:italic; }
  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .empty { color:#475569;font-size:0.8rem; }
</style>
