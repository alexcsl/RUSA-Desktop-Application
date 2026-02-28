<!--
  /messaging/inbox — Channel Inbox View
  Displays messages received by the current user on a specific channel.
  Source of truth: 00_MASTER_GUIDE.md §6
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { currentUser } from '$lib/stores/auth';
  import {
    getInbox, getMessageDetail, msgMarkRead,
    CHANNEL_LABELS, refreshUnreadCounts,
    type InboxMessage, type MessageDetail, type Channel,
  } from '$lib/stores/messaging';

  let channel: Channel = $state('general');
  let messages: InboxMessage[] = $state([]);
  let selectedDetail: MessageDetail | null = $state(null);
  let loading = $state(true);
  let error = $state('');

  // Reactively watch URL channel param
  $effect(() => {
    const ch = $page.url.searchParams.get('channel');
    if (ch && ['general', 'security', 'medical_heads', 'broadcast'].includes(ch)) {
      channel = ch as Channel;
      loadInbox();
    }
  });

  onMount(async () => {
    if (!$currentUser) { goto('/auth'); return; }
    const ch = $page.url.searchParams.get('channel') || 'general';
    channel = ch as Channel;
    await loadInbox();
  });

  async function loadInbox() {
    loading = true; error = '';
    try {
      messages = await getInbox(channel);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally { loading = false; }
  }

  async function openMessage(msg: InboxMessage) {
    try {
      selectedDetail = await getMessageDetail(msg.id);
      if (!msg.read_at) {
        await msgMarkRead(msg.id);
        msg.read_at = new Date().toISOString();
        refreshUnreadCounts();
      }
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function closeDetail() {
    selectedDetail = null;
  }

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleString(undefined, {
      month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
    });
  }
</script>

<h1 class="title">{CHANNEL_LABELS[channel]} Inbox</h1>

{#if error}
  <div class="error-banner">{error}</div>
{/if}

{#if loading}
  <p class="loading">Loading messages…</p>
{:else if selectedDetail}
  <!-- Message Detail Panel -->
  <div class="detail-panel">
    <button class="back-btn" onclick={closeDetail}>← Back to inbox</button>
    <div class="detail-header">
      <h2 class="detail-subject">
        {#if selectedDetail.recalled}
          <span class="recalled-tag">RECALLED</span>
        {/if}
        {selectedDetail.subject}
      </h2>
      <span class="detail-meta">
        From: <strong>{selectedDetail.sender_name}</strong>
        &nbsp;·&nbsp;
        {formatDate(selectedDetail.sent_at)}
      </span>
    </div>

    <div class="recipients-list">
      {#each [...selectedDetail.recipients_to, ...selectedDetail.recipients_cc, ...selectedDetail.recipients_bcc] as r}
        <span class="recipient-chip" class:unread={!r.read_at}>
          <span class="recipient-type">{r.recipient_type.toUpperCase()}</span>
          {r.full_name}
        </span>
      {/each}
    </div>

    {#if selectedDetail.recalled}
      <p class="recalled-notice">This message was recalled by the sender.</p>
    {:else}
      <div class="detail-body">{selectedDetail.body}</div>
    {/if}

    {#if selectedDetail.attachments.length > 0}
      <div class="attachments-section">
        <h3>Attachments</h3>
        {#each selectedDetail.attachments as att}
          <div class="attachment-item">
            <span class="att-name">{att.file_name}</span>
            <span class="att-size">
              {att.file_size_bytes ? `${Math.round(att.file_size_bytes / 1024)} KB` : ''}
            </span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{:else if messages.length === 0}
  <p class="empty-state">No messages in this channel.</p>
{:else}
  <!-- Message List -->
  <div class="message-list">
    {#each messages as msg}
      <button
        class="message-row"
        class:unread={!msg.read_at}
        class:recalled={msg.recalled}
        onclick={() => openMessage(msg)}
      >
        <div class="msg-from">{msg.sender_name}</div>
        <div class="msg-subject">
          {#if msg.recalled}
            <span class="recalled-tag">RECALLED</span>
          {/if}
          {msg.subject}
        </div>
        <div class="msg-date">{formatDate(msg.sent_at)}</div>
      </button>
    {/each}
  </div>
{/if}

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

  .loading {
    color: #94a3b8;
    font-style: italic;
  }

  .empty-state {
    color: #475569;
    font-style: italic;
    text-align: center;
    margin-top: 3rem;
  }

  /* Message list */
  .message-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .message-row {
    display: grid;
    grid-template-columns: 180px 1fr 120px;
    gap: 1rem;
    padding: 0.65rem 1rem;
    border: none;
    border-radius: 4px;
    background: rgba(17, 24, 39, 0.6);
    color: #94a3b8;
    text-align: left;
    cursor: pointer;
    transition: background 0.15s;
    font-size: 0.85rem;
    width: 100%;
  }

  .message-row:hover {
    background: rgba(58, 190, 255, 0.08);
  }

  .message-row.unread {
    color: #e6edf3;
    font-weight: 600;
    border-left: 3px solid #3abeff;
  }

  .message-row.recalled {
    opacity: 0.55;
  }

  .msg-from {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .msg-subject {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .msg-date {
    text-align: right;
    font-size: 0.8rem;
    color: #475569;
  }

  .recalled-tag {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
    font-size: 0.65rem;
    font-weight: 700;
    padding: 0.1rem 0.35rem;
    border-radius: 3px;
    margin-right: 0.35rem;
    vertical-align: middle;
  }

  /* Detail panel */
  .detail-panel {
    max-width: 700px;
  }

  .back-btn {
    background: none;
    border: none;
    color: #3abeff;
    cursor: pointer;
    font-size: 0.85rem;
    padding: 0;
    margin-bottom: 1rem;
  }

  .back-btn:hover {
    text-decoration: underline;
  }

  .detail-header {
    margin-bottom: 1rem;
  }

  .detail-subject {
    font-family: 'Orbitron', sans-serif;
    font-size: 1.1rem;
    color: #e6edf3;
    margin: 0 0 0.25rem;
  }

  .detail-meta {
    font-size: 0.8rem;
    color: #94a3b8;
  }

  .recipients-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    margin-bottom: 1rem;
  }

  .recipient-chip {
    background: rgba(17, 24, 39, 0.8);
    border: 1px solid rgba(58, 190, 255, 0.15);
    color: #94a3b8;
    font-size: 0.75rem;
    padding: 0.15rem 0.5rem;
    border-radius: 12px;
  }

  .recipient-type {
    color: #475569;
    font-weight: 600;
    margin-right: 0.25rem;
  }

  .recalled-notice {
    color: #ef4444;
    font-style: italic;
    padding: 1rem;
    background: rgba(239, 68, 68, 0.08);
    border-radius: 6px;
  }

  .detail-body {
    background: rgba(17, 24, 39, 0.6);
    border: 1px solid rgba(58, 190, 255, 0.08);
    border-radius: 6px;
    padding: 1rem 1.25rem;
    color: #e6edf3;
    font-size: 0.9rem;
    line-height: 1.6;
    white-space: pre-wrap;
  }

  .attachments-section {
    margin-top: 1rem;
  }

  .attachments-section h3 {
    font-size: 0.85rem;
    color: #94a3b8;
    margin: 0 0 0.5rem;
  }

  .attachment-item {
    display: flex;
    justify-content: space-between;
    padding: 0.35rem 0.75rem;
    background: rgba(17, 24, 39, 0.6);
    border-radius: 4px;
    font-size: 0.8rem;
    color: #e6edf3;
    margin-bottom: 0.25rem;
  }

  .att-size {
    color: #475569;
  }
</style>
