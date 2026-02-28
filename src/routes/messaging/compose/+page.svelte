<!--
  /messaging/compose — Compose & Send Message
  Supports all channels with role-based channel access.
  Features: To/CC/BCC user picker, group targets, scheduled send.
  Source of truth: 00_MASTER_GUIDE.md §6
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { currentUser } from '$lib/stores/auth';
  import {
    sendMessage, getEligibleRecipients, getMessagingGroups,
    CHANNEL_LABELS,
    type EligibleRecipient, type GroupSummary, type Channel,
  } from '$lib/stores/messaging';

  // Determine which channels this user can send to
  function getSendableChannels(): Channel[] {
    const role = $currentUser?.role;
    if (!role) return [];
    const channels: Channel[] = ['general'];

    const securitySendRoles = [
      'GalacticSecurityHead', 'GalacticSecurityStaff', 'TheGuardian', 'Administrator',
    ];
    if (securitySendRoles.includes(role)) channels.push('security');

    if (role === 'HeadOfMedicine' || role === 'Administrator') {
      channels.push('medical_heads');
    }

    const broadcastSendRoles = ['TheAnchorman', 'TheGuardian', 'Administrator'];
    if (broadcastSendRoles.includes(role)) channels.push('broadcast');

    return channels;
  }

  // Form state
  let channel: Channel = $state('general');
  let subject = $state('');
  let body = $state('');
  let toIds: string[] = $state([]);
  let ccIds: string[] = $state([]);
  let bccIds: string[] = $state([]);
  let selectedGroupIds: string[] = $state([]);
  let scheduledAt = $state('');

  // Data
  let recipients: EligibleRecipient[] = $state([]);
  let groups: GroupSummary[] = $state([]);
  let sendableChannels: Channel[] = $state([]);

  // UI state
  let personSearch = $state('');
  let addTarget: 'to' | 'cc' | 'bcc' = $state('to');
  let error = $state('');
  let success = $state('');
  let sending = $state(false);

  onMount(async () => {
    if (!$currentUser) { goto('/auth'); return; }
    sendableChannels = getSendableChannels();
    // Check if pre-selected channel via URL
    const preChannel = $page.url.searchParams.get('channel');
    if (preChannel && sendableChannels.includes(preChannel as Channel)) {
      channel = preChannel as Channel;
    }
    await loadRecipients();
    try { groups = await getMessagingGroups(); } catch { /* ignore */ }
  });

  async function loadRecipients() {
    try {
      recipients = await getEligibleRecipients(channel);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  // Reload recipients when channel changes
  $effect(() => {
    if (channel) loadRecipients();
  });

  function filteredRecipients(): EligibleRecipient[] {
    const q = personSearch.toLowerCase();
    const allSelected = [...toIds, ...ccIds, ...bccIds];
    return recipients
      .filter(
        (p) =>
          !allSelected.includes(p.id) &&
          (p.full_name.toLowerCase().includes(q) ||
           p.username.toLowerCase().includes(q) ||
           p.role_name.toLowerCase().includes(q)),
      )
      .slice(0, 15);
  }

  function addRecipient(id: string) {
    if (addTarget === 'to') toIds = [...toIds, id];
    else if (addTarget === 'cc') ccIds = [...ccIds, id];
    else bccIds = [...bccIds, id];
    personSearch = '';
  }

  function removeRecipient(id: string, bucket: 'to' | 'cc' | 'bcc') {
    if (bucket === 'to') toIds = toIds.filter((i) => i !== id);
    else if (bucket === 'cc') ccIds = ccIds.filter((i) => i !== id);
    else bccIds = bccIds.filter((i) => i !== id);
  }

  function nameById(id: string): string {
    return recipients.find((p) => p.id === id)?.full_name ?? id.slice(0, 8);
  }

  function toggleGroup(gid: string) {
    if (selectedGroupIds.includes(gid)) {
      selectedGroupIds = selectedGroupIds.filter((g) => g !== gid);
    } else {
      selectedGroupIds = [...selectedGroupIds, gid];
    }
  }

  async function handleSend() {
    error = ''; success = ''; sending = true;
    try {
      if (!subject.trim()) throw new Error('Subject is required.');
      if (!body.trim()) throw new Error('Body is required.');
      if (toIds.length === 0 && selectedGroupIds.length === 0) {
        throw new Error('At least one "To" recipient or group is required.');
      }

      // Convert datetime-local value to ISO 8601 UTC string for Rust DateTime<Utc>
      let parsedScheduledAt: string | undefined;
      if (scheduledAt) {
        // datetime-local gives "2026-02-27T19:15" — append seconds + UTC zone
        parsedScheduledAt = scheduledAt.length === 16
          ? scheduledAt + ':00Z'
          : scheduledAt.endsWith('Z') ? scheduledAt : scheduledAt + 'Z';
      }

      await sendMessage({
        subject,
        body,
        channel,
        recipients_to: toIds,
        recipients_cc: ccIds.length ? ccIds : undefined,
        recipients_bcc: bccIds.length ? bccIds : undefined,
        scheduled_at: parsedScheduledAt,
        group_ids: selectedGroupIds.length ? selectedGroupIds : undefined,
      });

      success = scheduledAt ? 'Message scheduled successfully.' : 'Message sent successfully.';
      // Reset form
      subject = ''; body = ''; toIds = []; ccIds = []; bccIds = [];
      selectedGroupIds = []; scheduledAt = '';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally { sending = false; }
  }
</script>

<h1 class="title">Compose Message</h1>

{#if error}
  <div class="error-banner">{error}</div>
{/if}
{#if success}
  <div class="success-banner">{success}</div>
{/if}

<form class="compose-form" onsubmit={(e) => { e.preventDefault(); handleSend(); }}>
  <!-- Channel selector -->
  <div class="field">
    <label for="channel">Channel</label>
    <select id="channel" bind:value={channel}>
      {#each sendableChannels as ch}
        <option value={ch}>{CHANNEL_LABELS[ch]}</option>
      {/each}
    </select>
  </div>

  <!-- Recipient picker -->
  <div class="field">
    <label for="recipient-search">Recipients</label>
    <div class="recipient-controls">
      <select bind:value={addTarget} class="type-select">
        <option value="to">To</option>
        <option value="cc">CC</option>
        <option value="bcc">BCC</option>
      </select>
      <input
        id="recipient-search"
        type="text"
        placeholder="Search people…"
        bind:value={personSearch}
        class="search-input"
      />
    </div>

    {#if personSearch.length > 0}
      <div class="dropdown">
        {#each filteredRecipients() as p}
          <button type="button" class="dropdown-item" onclick={() => addRecipient(p.id)}>
            <span class="person-name">{p.full_name}</span>
            <span class="person-role">{p.role_name}</span>
          </button>
        {/each}
        {#if filteredRecipients().length === 0}
          <div class="dropdown-empty">No matching personnel</div>
        {/if}
      </div>
    {/if}

    <!-- Selected recipients by bucket -->
    {#each ['to', 'cc', 'bcc'] as bucket}
      {@const ids = bucket === 'to' ? toIds : bucket === 'cc' ? ccIds : bccIds}
      {#if ids.length > 0}
        <div class="chip-row">
          <span class="chip-label">{bucket.toUpperCase()}:</span>
          {#each ids as id}
            <span class="chip">
              {nameById(id)}
              <button type="button" class="chip-remove" onclick={() => removeRecipient(id, bucket as 'to' | 'cc' | 'bcc')}>×</button>
            </span>
          {/each}
        </div>
      {/if}
    {/each}
  </div>

  <!-- Group targets -->
  {#if groups.length > 0}
    <div class="field">
      <label for="group-targets">Group Targets (optional)</label>
      <div class="group-list">
        {#each groups as g}
          <label class="group-item">
            <input
              type="checkbox"
              checked={selectedGroupIds.includes(g.id)}
              onchange={() => toggleGroup(g.id)}
            />
            {g.name}
          </label>
        {/each}
      </div>
    </div>
  {/if}

  <!-- Subject -->
  <div class="field">
    <label for="subject">Subject</label>
    <input id="subject" type="text" bind:value={subject} placeholder="Message subject" />
  </div>

  <!-- Body -->
  <div class="field">
    <label for="body">Body</label>
    <textarea id="body" bind:value={body} rows="8" placeholder="Write your message…"></textarea>
  </div>

  <!-- Scheduled send -->
  <div class="field">
    <label for="scheduled">Schedule Send (optional)</label>
    <input id="scheduled" type="datetime-local" bind:value={scheduledAt} />
  </div>

  <button type="submit" class="send-btn" disabled={sending}>
    {sending ? 'Sending…' : scheduledAt ? 'Schedule' : 'Send'}
  </button>
</form>

<style>
  .title {
    font-family: 'Orbitron', sans-serif;
    font-size: 1.2rem;
    color: #3abeff;
    margin: 0 0 1rem;
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #ef4444;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
    font-size: 0.875rem;
  }

  .success-banner {
    background: rgba(16, 185, 129, 0.15);
    border: 1px solid rgba(16, 185, 129, 0.4);
    color: #10b981;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    margin-bottom: 1rem;
    font-size: 0.875rem;
  }

  .compose-form {
    max-width: 640px;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .field label {
    font-size: 0.8rem;
    color: #94a3b8;
    font-weight: 600;
  }

  .field input, .field textarea, .field select {
    background: #0e1428;
    border: 1px solid rgba(58, 190, 255, 0.2);
    border-radius: 6px;
    padding: 0.5rem 0.75rem;
    color: #e6edf3;
    font-size: 0.875rem;
    font-family: inherit;
  }

  .field input:focus, .field textarea:focus, .field select:focus {
    outline: none;
    border-color: #3abeff;
    box-shadow: 0 0 6px rgba(58, 190, 255, 0.3);
  }

  .field textarea {
    resize: vertical;
  }

  .recipient-controls {
    display: flex;
    gap: 0.5rem;
  }

  .type-select {
    width: 70px;
    background: #0e1428;
    border: 1px solid rgba(58, 190, 255, 0.2);
    border-radius: 6px;
    color: #e6edf3;
    font-size: 0.8rem;
    padding: 0.4rem;
  }

  .search-input {
    flex: 1;
  }

  .dropdown {
    background: #111827;
    border: 1px solid rgba(58, 190, 255, 0.2);
    border-radius: 6px;
    max-height: 180px;
    overflow-y: auto;
  }

  .dropdown-item {
    display: flex;
    justify-content: space-between;
    width: 100%;
    background: none;
    border: none;
    color: #e6edf3;
    padding: 0.4rem 0.75rem;
    cursor: pointer;
    font-size: 0.8rem;
    text-align: left;
  }

  .dropdown-item:hover {
    background: rgba(58, 190, 255, 0.1);
  }

  .person-role {
    color: #475569;
    font-size: 0.75rem;
  }

  .dropdown-empty {
    padding: 0.5rem 0.75rem;
    color: #475569;
    font-size: 0.8rem;
    font-style: italic;
  }

  .chip-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.3rem;
    margin-top: 0.25rem;
  }

  .chip-label {
    font-size: 0.7rem;
    color: #475569;
    font-weight: 700;
    min-width: 30px;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    background: rgba(58, 190, 255, 0.12);
    border: 1px solid rgba(58, 190, 255, 0.25);
    color: #e6edf3;
    font-size: 0.75rem;
    padding: 0.15rem 0.45rem;
    border-radius: 12px;
  }

  .chip-remove {
    background: none;
    border: none;
    color: #ef4444;
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0;
    line-height: 1;
  }

  .group-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .group-item {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.8rem;
    color: #94a3b8;
    cursor: pointer;
  }

  .send-btn {
    align-self: flex-start;
    background: linear-gradient(135deg, #3abeff 0%, #8b5cf6 100%);
    color: #0b0f1a;
    font-weight: 700;
    font-size: 0.9rem;
    border: none;
    border-radius: 6px;
    padding: 0.6rem 2rem;
    cursor: pointer;
    transition: box-shadow 0.15s;
  }

  .send-btn:hover:not(:disabled) {
    box-shadow: 0 0 12px rgba(58, 190, 255, 0.6);
  }

  .send-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
