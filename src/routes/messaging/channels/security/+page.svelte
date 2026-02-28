<!--
  /messaging/channels/security — UC-SH-04 / UC-SS-02 / UC-GUA-04
  Security Inter-Team Communication Line (universal messaging version).
  Access: GalacticSecurityHead, GalacticSecurityStaff, TheGuardian (read+write),
          TheOverseer (read-only), Administrator.
  Replaces legacy /directors/guardian/secure-messaging for new universal flow.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { currentUser } from '$lib/stores/auth';
  import {
    getInbox, getSentMessages, sendMessage, getMessageDetail, msgMarkRead,
    getEligibleRecipients, refreshUnreadCounts,
    type InboxMessage, type MessageDetail, type EligibleRecipient,
    type MessageSummary,
  } from '$lib/stores/messaging';

  const CHANNEL = 'security' as const;
  const ALLOWED_ROLES = [
    'GalacticSecurityHead', 'GalacticSecurityStaff',
    'TheGuardian', 'TheOverseer', 'Administrator',
  ];
  const SEND_ROLES = [
    'GalacticSecurityHead', 'GalacticSecurityStaff',
    'TheGuardian', 'Administrator',
  ];

  let canSend = $state(false);
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
    canSend = SEND_ROLES.includes($currentUser.role);
    await loadInbox();
    if (canSend) {
      try { recipients = await getEligibleRecipients(CHANNEL); } catch {}
      loadSent();
    }
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
          (p.full_name.toLowerCase().includes(q) ||
           p.username.toLowerCase().includes(q) ||
           p.role_name.toLowerCase().includes(q)),
      )
      .slice(0, 12);
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

      success = 'Secure message sent.';
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
    <h1 class="title">Security Communication Line</h1>
    <p class="subtitle">
      {#if !canSend}
        Read-only access — Overseer monitoring mode
      {:else}
        Encrypted inter-team security communications
      {/if}
    </p>
  </div>
  {#if canSend}
    <button class="compose-btn" onclick={() => { showCompose = !showCompose; }}>
      {showCompose ? 'Cancel' : '+ Compose'}
    </button>
  {/if}
</div>

{#if error}
  <div class="error-banner">{error}</div>
{/if}
{#if success}
  <div class="success-banner">{success}</div>
{/if}

<!-- Inline compose (hidden for read-only roles) -->
{#if showCompose && canSend}
  <form class="compose-box" onsubmit={(e) => { e.preventDefault(); handleSend(); }}>
    <div class="field">
      <label for="sec-recipient-search">To</label>
      <input
        id="sec-recipient-search"
        type="text"
        placeholder="Search security personnel…"
        bind:value={personSearch}
        class="inp"
      />
      {#if personSearch.length > 0}
        <div class="dropdown">
          {#each filteredRecipients() as p}
            <button type="button" class="dropdown-item" onclick={() => addTo(p.id)}>
              {p.full_name} <span class="meta">{p.role_name}</span>
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
    <textarea bind:value={composeBody} rows="4" placeholder="Secure message…" class="inp"></textarea>
    <button type="submit" class="send-btn" disabled={sending}>
      {sending ? 'Sending…' : 'Send Secure Message'}
    </button>
  </form>
{/if}

<!-- View tabs -->
<div class="view-tabs">
  <button class="tab-btn" class:active={viewTab === 'inbox'} onclick={() => { viewTab = 'inbox'; selected = null; }}>
    Inbox {messages.length > 0 ? `(${messages.length})` : ''}
  </button>
  {#if canSend}
    <button class="tab-btn" class:active={viewTab === 'sent'} onclick={() => { viewTab = 'sent'; selected = null; }}>
      Sent {sentMessages.length > 0 ? `(${sentMessages.length})` : ''}
    </button>
  {/if}
</div>

<!-- Inbox / Sent split view -->
<div class="split">
  <div class="list">
    {#if loading}
      <div class="empty">Loading…</div>
    {:else if viewTab === 'inbox'}
      {#if messages.length === 0}
        <div class="empty">No security messages.</div>
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
        <div class="empty">No sent security messages.</div>
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
        {#if selected.recipients_cc.length > 0}
          <div class="detail-meta">
            <span class="meta-lbl">CC:</span>
            {selected.recipients_cc.map((r) => r.full_name).join(', ')}
          </div>
        {/if}

        <div class="detail-body">{selected.body}</div>

        {#if selected.attachments && selected.attachments.length > 0}
          <div class="detail-meta">
            <span class="meta-lbl">Attachments:</span>
            {#each selected.attachments as att}
              <a href={att.storage_path} target="_blank" class="att-link">{att.file_name}</a>
            {/each}
          </div>
        {/if}
      </div>
    {:else}
      <div class="empty">Select a message to read.</div>
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
    border: 1px solid rgba(245, 158, 11, 0.15);
    color: #94a3b8;
    padding: 0.4rem 1rem;
    font-size: 0.8rem;
    cursor: pointer;
    border-radius: 6px 6px 0 0;
    transition: background 0.15s, color 0.15s;
  }

  .tab-btn:hover {
    color: #e6edf3;
    background: rgba(245, 158, 11, 0.08);
  }

  .tab-btn.active {
    background: rgba(245, 158, 11, 0.12);
    color: #f59e0b;
    border-bottom-color: transparent;
    font-weight: 600;
  }

  .title {
    font-family: 'Orbitron', sans-serif;
    font-size: 1.15rem;
    color: #f59e0b;
    margin: 0;
  }

  .subtitle {
    font-size: 0.75rem;
    color: #64748b;
    margin: 0.15rem 0 0;
  }

  .compose-btn {
    background: linear-gradient(135deg, #f59e0b 0%, #ef4444 100%);
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
    border: 1px solid rgba(245, 158, 11, 0.2);
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
    border-color: #f59e0b;
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
    border: 1px solid rgba(245, 158, 11, 0.2);
    border-radius: 6px;
    max-height: 140px;
    overflow-y: auto;
    z-index: 10;
  }

  .dropdown-item {
    display: flex;
    justify-content: space-between;
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
    background: rgba(245, 158, 11, 0.1);
  }

  .meta {
    color: #475569;
    font-size: 0.7rem;
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
    background: rgba(245, 158, 11, 0.12);
    border: 1px solid rgba(245, 158, 11, 0.25);
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
    background: linear-gradient(135deg, #f59e0b 0%, #ef4444 100%);
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
    border-color: rgba(245, 158, 11, 0.3);
  }

  .msg-row.active {
    border-color: #f59e0b;
    background: rgba(245, 158, 11, 0.06);
  }

  .msg-row.unread {
    border-left: 3px solid #f59e0b;
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
    border: 1px solid rgba(245, 158, 11, 0.12);
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
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
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

  .att-link {
    color: #f59e0b;
    text-decoration: underline;
    font-size: 0.8rem;
  }

  .empty {
    padding: 2rem;
    text-align: center;
    color: #475569;
    font-size: 0.85rem;
  }
</style>
