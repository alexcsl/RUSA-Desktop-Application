<!--
  /messaging/channels/medical-heads — UC-HOM-04
  Dedicated Medical Heads inter-communication channel.
  Access: HeadOfMedicine, Administrator only.
  Combined inbox + quick compose in one view.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { currentUser } from '$lib/stores/auth';
  import {
    getInbox, getSentMessages, sendMessage,
    getMessageDetail, msgMarkRead, getEligibleRecipients,
    refreshUnreadCounts,
    type InboxMessage, type MessageDetail, type EligibleRecipient,
    type MessageSummary,
  } from '$lib/stores/messaging';

  const CHANNEL = 'medical_heads' as const;
  const ALLOWED_ROLES = ['HeadOfMedicine', 'Administrator'];

  let messages: InboxMessage[] = $state([]);
  let sentMessages: MessageSummary[] = $state([]);
  let selected: MessageDetail | null = $state(null);
  let recipients: EligibleRecipient[] = $state([]);
  let loading = $state(false);
  let error = $state('');
  let success = $state('');

  // View tab: inbox or sent
  let viewTab: 'inbox' | 'sent' = $state('inbox');

  // Compose inline
  let showCompose = $state(false);
  let composeSubject = $state('');
  let composeBody = $state('');
  let composeToIds: string[] = $state([]);
  let personSearch = $state('');
  let sending = $state(false);

  onMount(async () => {
    if (!$currentUser || !ALLOWED_ROLES.includes($currentUser.role)) {
      goto('/forbidden');
      return;
    }
    await Promise.all([loadInbox(), loadRecipients(), loadSent()]);
  });

  async function loadInbox() {
    loading = true; error = '';
    try {
      messages = await getInbox(CHANNEL);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally { loading = false; }
  }

  async function loadSent() {
    try {
      sentMessages = await getSentMessages(CHANNEL);
    } catch { /* ignore — user may not have send access */ }
  }

  async function loadRecipients() {
    try {
      recipients = await getEligibleRecipients(CHANNEL);
    } catch { /* ignore */ }
  }

  async function openMessage(msg: InboxMessage) {
    try {
      selected = await getMessageDetail(msg.id);
      if (!msg.read_at) {
        await msgMarkRead(msg.id);
        messages = messages.map((m) =>
          m.id === msg.id ? { ...m, read_at: new Date().toISOString() } : m,
        );
        refreshUnreadCounts();
      }
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function openSentDetail(msgId: string) {
    try {
      selected = await getMessageDetail(msgId);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function addTo(id: string) {
    if (!composeToIds.includes(id)) composeToIds = [...composeToIds, id];
    personSearch = '';
  }

  function removeTo(id: string) {
    composeToIds = composeToIds.filter((i) => i !== id);
  }

  function nameById(id: string): string {
    return recipients.find((r) => r.id === id)?.full_name ?? id.slice(0, 8);
  }

  function filteredRecipients(): EligibleRecipient[] {
    const q = personSearch.toLowerCase();
    return recipients
      .filter(
        (p) =>
          !composeToIds.includes(p.id) &&
          (p.full_name.toLowerCase().includes(q) || p.username.toLowerCase().includes(q)),
      )
      .slice(0, 10);
  }

  async function handleSend() {
    error = ''; success = ''; sending = true;
    try {
      if (!composeSubject.trim()) throw new Error('Subject required.');
      if (!composeBody.trim()) throw new Error('Body required.');
      if (composeToIds.length === 0) throw new Error('At least one recipient required.');

      await sendMessage({
        subject: composeSubject,
        body: composeBody,
        channel: CHANNEL,
        recipients_to: composeToIds,
      });

      success = 'Message sent.';
      composeSubject = ''; composeBody = ''; composeToIds = [];
      showCompose = false;
      await loadInbox();
      loadSent();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally { sending = false; }
  }

  function fmtDate(iso: string): string {
    return new Date(iso).toLocaleString(undefined, {
      month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
    });
  }
</script>

<div class="page-header">
  <div>
    <h1 class="title">Medical Heads Channel</h1>
    <p class="subtitle">Secure inter-Head-of-Medicine communications</p>
  </div>
  <button class="compose-btn" onclick={() => { showCompose = !showCompose; }}>
    {showCompose ? 'Cancel' : '+ Compose'}
  </button>
</div>

{#if error}
  <div class="error-banner">{error}</div>
{/if}
{#if success}
  <div class="success-banner">{success}</div>
{/if}

<!-- Inline compose -->
{#if showCompose}
  <form class="compose-box" onsubmit={(e) => { e.preventDefault(); handleSend(); }}>
    <div class="field">
      <label for="med-recipient-search">To</label>
      <input
        id="med-recipient-search"
        type="text"
        placeholder="Search medical heads…"
        bind:value={personSearch}
        class="inp"
      />
      {#if personSearch.length > 0}
        <div class="dropdown">
          {#each filteredRecipients() as p}
            <button type="button" class="dropdown-item" onclick={() => addTo(p.id)}>
              {p.full_name}
            </button>
          {/each}
        </div>
      {/if}
      {#if composeToIds.length > 0}
        <div class="chip-row">
          {#each composeToIds as id}
            <span class="chip">
              {nameById(id)}
              <button type="button" class="chip-x" onclick={() => removeTo(id)}>×</button>
            </span>
          {/each}
        </div>
      {/if}
    </div>
    <input type="text" bind:value={composeSubject} placeholder="Subject" class="inp" />
    <textarea bind:value={composeBody} rows="4" placeholder="Message…" class="inp"></textarea>
    <button type="submit" class="send-btn" disabled={sending}>
      {sending ? 'Sending…' : 'Send'}
    </button>
  </form>
{/if}

<!-- View tabs -->
<div class="view-tabs">
  <button class="tab-btn" class:active={viewTab === 'inbox'} onclick={() => { viewTab = 'inbox'; selected = null; }}>
    Inbox {messages.length > 0 ? `(${messages.length})` : ''}
  </button>
  <button class="tab-btn" class:active={viewTab === 'sent'} onclick={() => { viewTab = 'sent'; selected = null; }}>
    Sent {sentMessages.length > 0 ? `(${sentMessages.length})` : ''}
  </button>
</div>

<!-- Inbox / Sent split view -->
<div class="split">
  <div class="list">
    {#if loading}
      <div class="empty">Loading…</div>
    {:else if viewTab === 'inbox'}
      {#if messages.length === 0}
        <div class="empty">No messages.</div>
      {:else}
        {#each messages as msg}
          <button
            class="msg-row"
            class:unread={!msg.read_at}
            class:active={selected?.id === msg.id}
            class:recalled={msg.recalled}
            onclick={() => openMessage(msg)}
          >
            <div class="msg-top">
              <span class="msg-sender">{msg.sender_name}</span>
              <span class="msg-date">{fmtDate(msg.sent_at)}</span>
            </div>
            <div class="msg-subject">{msg.subject}</div>
            <div class="msg-preview">{msg.body_preview}</div>
          </button>
        {/each}
      {/if}
    {:else}
      {#if sentMessages.length === 0}
        <div class="empty">No sent messages.</div>
      {:else}
        {#each sentMessages as msg}
          <button
            class="msg-row"
            class:active={selected?.id === msg.id}
            class:recalled={msg.recalled}
            onclick={() => openSentDetail(msg.id)}
          >
            <div class="msg-top">
              <span class="msg-sender">You</span>
              <span class="msg-date">{fmtDate(msg.sent_at)}</span>
            </div>
            <div class="msg-subject">{msg.subject}{msg.recalled ? ' [RECALLED]' : ''}</div>
            <div class="msg-preview">{msg.body_preview}</div>
          </button>
        {/each}
      {/if}
    {/if}
  </div>

  <div class="detail">
    {#if selected}
      <div class="detail-card">
        <h2 class="detail-subject">{selected.subject}</h2>
        {#if selected.recalled}
          <div class="recalled-tag">RECALLED</div>
        {/if}
        <div class="detail-meta">
          <span class="meta-lbl">From:</span> {selected.sender_name}
        </div>
        <div class="detail-meta">
          <span class="meta-lbl">Sent:</span> {fmtDate(selected.sent_at)}
        </div>
        {#if selected.recipients_to.length > 0}
          <div class="detail-meta">
            <span class="meta-lbl">To:</span>
            {selected.recipients_to.map((r) => r.full_name).join(', ')}
          </div>
        {/if}
        <div class="detail-body">{selected.body}</div>
      </div>
    {:else}
      <div class="empty">Select a message.</div>
    {/if}
  </div>
</div>

<style>
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 1rem;
  }

  .view-tabs {
    display: flex;
    gap: 2px;
    margin-bottom: 0.75rem;
  }

  .tab-btn {
    background: rgba(15, 23, 42, 0.5);
    border: 1px solid rgba(16, 185, 129, 0.15);
    color: #94a3b8;
    padding: 0.4rem 1rem;
    font-size: 0.8rem;
    cursor: pointer;
    border-radius: 6px 6px 0 0;
    transition: background 0.15s, color 0.15s;
  }

  .tab-btn:hover {
    color: #e6edf3;
    background: rgba(16, 185, 129, 0.08);
  }

  .tab-btn.active {
    background: rgba(16, 185, 129, 0.12);
    color: #10b981;
    border-bottom-color: transparent;
    font-weight: 600;
  }

  .title {
    font-family: 'Orbitron', sans-serif;
    font-size: 1.15rem;
    color: #10b981;
    margin: 0;
  }

  .subtitle {
    font-size: 0.75rem;
    color: #64748b;
    margin: 0.15rem 0 0;
  }

  .compose-btn {
    background: linear-gradient(135deg, #10b981 0%, #3abeff 100%);
    color: #0b0f1a;
    font-weight: 700;
    font-size: 0.8rem;
    border: none;
    border-radius: 6px;
    padding: 0.45rem 1rem;
    cursor: pointer;
  }

  .error-banner {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #ef4444;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    margin-bottom: 0.75rem;
    font-size: 0.85rem;
  }

  .success-banner {
    background: rgba(16, 185, 129, 0.15);
    border: 1px solid rgba(16, 185, 129, 0.4);
    color: #10b981;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    margin-bottom: 0.75rem;
    font-size: 0.85rem;
  }

  .compose-box {
    background: rgba(15, 23, 42, 0.5);
    border: 1px solid rgba(16, 185, 129, 0.2);
    border-radius: 8px;
    padding: 1rem;
    margin-bottom: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-width: 600px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    position: relative;
  }

  .field label {
    font-size: 0.75rem;
    color: #94a3b8;
    font-weight: 600;
  }

  .inp {
    background: #0e1428;
    border: 1px solid rgba(58, 190, 255, 0.2);
    border-radius: 6px;
    padding: 0.45rem 0.65rem;
    color: #e6edf3;
    font-size: 0.85rem;
    font-family: inherit;
    width: 100%;
    box-sizing: border-box;
  }

  .inp:focus {
    outline: none;
    border-color: #10b981;
  }

  textarea.inp {
    resize: vertical;
  }

  .dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: #111827;
    border: 1px solid rgba(16, 185, 129, 0.2);
    border-radius: 6px;
    max-height: 140px;
    overflow-y: auto;
    z-index: 10;
  }

  .dropdown-item {
    display: block;
    width: 100%;
    background: none;
    border: none;
    color: #e6edf3;
    padding: 0.35rem 0.65rem;
    cursor: pointer;
    font-size: 0.8rem;
    text-align: left;
  }

  .dropdown-item:hover {
    background: rgba(16, 185, 129, 0.1);
  }

  .chip-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin-top: 0.2rem;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    background: rgba(16, 185, 129, 0.12);
    border: 1px solid rgba(16, 185, 129, 0.25);
    color: #e6edf3;
    font-size: 0.75rem;
    padding: 0.12rem 0.4rem;
    border-radius: 12px;
  }

  .chip-x {
    background: none;
    border: none;
    color: #ef4444;
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0;
    line-height: 1;
  }

  .send-btn {
    align-self: flex-start;
    background: linear-gradient(135deg, #10b981 0%, #3abeff 100%);
    color: #0b0f1a;
    font-weight: 700;
    font-size: 0.85rem;
    border: none;
    border-radius: 6px;
    padding: 0.5rem 1.5rem;
    cursor: pointer;
  }

  .send-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .split {
    display: flex;
    gap: 1rem;
    height: calc(100vh - 300px);
  }

  .list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .msg-row {
    display: block;
    width: 100%;
    text-align: left;
    background: rgba(15, 23, 42, 0.5);
    border: 1px solid transparent;
    border-radius: 6px;
    padding: 0.55rem 0.75rem;
    color: #e6edf3;
    cursor: pointer;
    transition: border-color 0.15s;
  }

  .msg-row:hover {
    border-color: rgba(16, 185, 129, 0.3);
  }

  .msg-row.active {
    border-color: #10b981;
    background: rgba(16, 185, 129, 0.06);
  }

  .msg-row.unread {
    border-left: 3px solid #10b981;
  }

  .msg-row.recalled {
    opacity: 0.45;
  }

  .msg-top {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .msg-sender {
    font-weight: 600;
    font-size: 0.8rem;
  }

  .msg-date {
    font-size: 0.7rem;
    color: #64748b;
    white-space: nowrap;
  }

  .msg-subject {
    font-size: 0.82rem;
    margin-top: 0.1rem;
  }

  .msg-preview {
    font-size: 0.72rem;
    color: #94a3b8;
    margin-top: 0.1rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .detail {
    flex: 1;
    overflow-y: auto;
  }

  .detail-card {
    background: rgba(15, 23, 42, 0.5);
    border: 1px solid rgba(16, 185, 129, 0.12);
    border-radius: 8px;
    padding: 1.25rem;
  }

  .detail-subject {
    font-family: 'Orbitron', sans-serif;
    font-size: 1rem;
    color: #e6edf3;
    margin: 0 0 0.75rem;
  }

  .recalled-tag {
    display: inline-block;
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
    font-size: 0.7rem;
    padding: 0.15rem 0.5rem;
    border-radius: 12px;
    margin-bottom: 0.5rem;
  }

  .detail-meta {
    font-size: 0.8rem;
    color: #94a3b8;
    margin-bottom: 0.25rem;
  }

  .meta-lbl {
    font-weight: 700;
    color: #64748b;
  }

  .detail-body {
    margin-top: 1rem;
    font-size: 0.875rem;
    color: #e6edf3;
    line-height: 1.6;
    white-space: pre-wrap;
  }

  .empty {
    padding: 2rem;
    text-align: center;
    color: #475569;
    font-size: 0.85rem;
  }
</style>
