<!--
  /messaging/sent — Sent Messages view
  Lists messages the current user has sent, grouped by channel.
  Supports recalling unread messages.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { currentUser } from '$lib/stores/auth';
  import {
    getSentMessages, getMessageDetail, msgRecall,
    CHANNEL_LABELS,
    type MessageSummary, type MessageDetail, type Channel,
  } from '$lib/stores/messaging';

  // Determine which channels this user can send to (mirrors Rust can_send_on_channel)
  function getSendableChannels(): Channel[] {
    const role = $currentUser?.role;
    if (!role) return [];
    const channels: Channel[] = ['general'];

    const securitySendRoles = [
      'GalacticSecurityHead', 'GalacticSecurityStaff', 'TheGuardian', 'Administrator',
    ];
    if (securitySendRoles.includes(role)) channels.push('security');

    if (role === 'HeadOfMedicine' || role === 'Administrator') channels.push('medical_heads');

    const broadcastSendRoles = ['TheAnchorman', 'TheGuardian', 'Administrator'];
    if (broadcastSendRoles.includes(role)) channels.push('broadcast');

    return channels;
  }

  let sendableChannels: Channel[] = $state([]);
  let channel: Channel = $state('general');
  let messages: MessageSummary[] = $state([]);
  let selected: MessageDetail | null = $state(null);
  let loading = $state(false);
  let error = $state('');
  let recallSuccess = $state('');

  // Recall confirmation modal
  let recallConfirmId: string | null = $state(null);
  let recalling = $state(false);

  onMount(() => {
    if (!$currentUser) { goto('/auth'); return; }
    sendableChannels = getSendableChannels();
    const preChannel = $page.url.searchParams.get('channel');
    if (preChannel && sendableChannels.includes(preChannel as Channel)) {
      channel = preChannel as Channel;
    }
  });

  async function load() {
    loading = true; error = ''; selected = null;
    try {
      messages = await getSentMessages(channel);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally { loading = false; }
  }

  $effect(() => { if (channel) load(); });

  async function openDetail(msgId: string) {
    try {
      selected = await getMessageDetail(msgId);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function promptRecall(msgId: string) {
    recallConfirmId = msgId;
  }

  function cancelRecall() {
    recallConfirmId = null;
  }

  async function confirmRecall() {
    if (!recallConfirmId) return;
    recallSuccess = ''; error = ''; recalling = true;
    try {
      await msgRecall(recallConfirmId);
      recallSuccess = 'Message recalled.';
      recallConfirmId = null;
      selected = null;
      await load();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
      recallConfirmId = null;
    } finally { recalling = false; }
  }

  function fmtDate(iso: string): string {
    return new Date(iso).toLocaleString(undefined, {
      month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
    });
  }
</script>

<div class="sent-header">
  <h1 class="title">Sent Messages</h1>
  <select class="channel-select" bind:value={channel}>
    {#each sendableChannels as ch}
      <option value={ch}>{CHANNEL_LABELS[ch]}</option>
    {/each}
  </select>
</div>

{#if error}
  <div class="error-banner">{error}</div>
{/if}
{#if recallSuccess}
  <div class="success-banner">{recallSuccess}</div>
{/if}

<div class="split">
  <!-- Message list -->
  <div class="list">
    {#if loading}
      <div class="empty">Loading…</div>
    {:else if messages.length === 0}
      <div class="empty">No sent messages in this channel.</div>
    {:else}
      {#each messages as msg}
        <button
          class="msg-row"
          class:active={selected?.id === msg.id}
          class:recalled={msg.recalled}
          onclick={() => openDetail(msg.id)}
        >
          <div class="msg-top">
            <span class="msg-subject">{msg.subject}{msg.recalled ? ' [RECALLED]' : ''}</span>
            <span class="msg-date">{fmtDate(msg.sent_at)}</span>
          </div>
          <div class="msg-preview">{msg.body_preview}</div>
        </button>
      {/each}
    {/if}
  </div>

  <!-- Detail panel -->
  <div class="detail">
    {#if selected}
      <div class="detail-card">
        <h2 class="detail-subject">{selected.subject}</h2>
        {#if selected.recalled}
          <div class="recalled-badge">This message has been recalled</div>
        {/if}
        <div class="detail-meta">
          <span class="meta-label">Sent:</span>
          <span>{fmtDate(selected.sent_at)}</span>
        </div>

        <!-- Recipients -->
        {#if selected.recipients_to.length > 0}
          <div class="detail-meta">
            <span class="meta-label">To:</span>
            <span>{selected.recipients_to.map((r) => r.full_name).join(', ')}</span>
          </div>
        {/if}
        {#if selected.recipients_cc.length > 0}
          <div class="detail-meta">
            <span class="meta-label">CC:</span>
            <span>{selected.recipients_cc.map((r) => r.full_name).join(', ')}</span>
          </div>
        {/if}
        {#if selected.recipients_bcc.length > 0}
          <div class="detail-meta">
            <span class="meta-label">BCC:</span>
            <span>{selected.recipients_bcc.map((r) => r.full_name).join(', ')}</span>
          </div>
        {/if}

        <div class="detail-body">{selected.body}</div>

        {#if selected.attachments && selected.attachments.length > 0}
          <div class="detail-meta">
            <span class="meta-label">Attachments:</span>
            {#each selected.attachments as att}
              <a href={att.storage_path} target="_blank" class="att-link">{att.file_name}</a>
            {/each}
          </div>
        {/if}

        {#if !selected.recalled}
          <button class="recall-btn" onclick={() => promptRecall(selected!.id)}>
            Recall Message
          </button>
          <p class="recall-hint">Only works if no recipient has read it yet.</p>
        {/if}
      </div>
    {:else}
      <div class="empty">Select a message to view details.</div>
    {/if}
  </div>
</div>

<!-- Recall confirmation modal -->
{#if recallConfirmId}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="recall-overlay" role="dialog" aria-modal="true" tabindex="-1" onclick={cancelRecall} onkeydown={(e) => { if (e.key === 'Escape') cancelRecall(); }}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="recall-modal" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
      <div class="recall-modal-header">
        <span class="recall-modal-icon">⚠</span>
        <h2>Recall Message</h2>
      </div>
      <p class="recall-modal-body">
        Are you sure you want to recall this message? This action cannot be undone.
        It will only succeed if <strong>no recipient has read it yet</strong>.
      </p>
      <div class="recall-modal-actions">
        <button class="recall-modal-cancel" onclick={cancelRecall} disabled={recalling}>Cancel</button>
        <button class="recall-modal-confirm" onclick={confirmRecall} disabled={recalling}>
          {recalling ? 'Recalling…' : 'Yes, Recall'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .sent-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .title {
    font-family: 'Orbitron', sans-serif;
    font-size: 1.2rem;
    color: #3abeff;
    margin: 0;
  }

  .channel-select {
    background: #0e1428;
    border: 1px solid rgba(58, 190, 255, 0.2);
    border-radius: 6px;
    color: #e6edf3;
    padding: 0.35rem 0.6rem;
    font-size: 0.8rem;
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

  .split {
    display: flex;
    gap: 1rem;
    height: calc(100vh - 200px);
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
    padding: 0.6rem 0.75rem;
    cursor: pointer;
    color: #e6edf3;
    transition: border-color 0.15s;
  }

  .msg-row:hover {
    border-color: rgba(58, 190, 255, 0.3);
  }

  .msg-row.active {
    border-color: #3abeff;
    background: rgba(58, 190, 255, 0.06);
  }

  .msg-row.recalled {
    opacity: 0.5;
  }

  .msg-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
  }

  .msg-subject {
    font-weight: 600;
    font-size: 0.85rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .msg-date {
    font-size: 0.7rem;
    color: #64748b;
    white-space: nowrap;
  }

  .msg-preview {
    font-size: 0.75rem;
    color: #94a3b8;
    margin-top: 0.15rem;
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
    border: 1px solid rgba(58, 190, 255, 0.12);
    border-radius: 8px;
    padding: 1.25rem;
  }

  .detail-subject {
    font-family: 'Orbitron', sans-serif;
    font-size: 1rem;
    color: #e6edf3;
    margin: 0 0 0.75rem;
  }

  .recalled-badge {
    display: inline-block;
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #ef4444;
    font-size: 0.7rem;
    padding: 0.2rem 0.6rem;
    border-radius: 12px;
    margin-bottom: 0.75rem;
  }

  .detail-meta {
    font-size: 0.8rem;
    color: #94a3b8;
    margin-bottom: 0.35rem;
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }

  .meta-label {
    font-weight: 700;
    color: #64748b;
    min-width: 55px;
  }

  .detail-body {
    margin-top: 1rem;
    font-size: 0.875rem;
    color: #e6edf3;
    line-height: 1.6;
    white-space: pre-wrap;
  }

  .att-link {
    color: #3abeff;
    text-decoration: underline;
    font-size: 0.8rem;
  }

  .recall-btn {
    margin-top: 1rem;
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #ef4444;
    padding: 0.4rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .recall-btn:hover {
    background: rgba(239, 68, 68, 0.25);
  }

  .recall-hint {
    font-size: 0.7rem;
    color: #475569;
    margin-top: 0.25rem;
  }

  /* ── Recall confirmation modal ── */
  .recall-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 0.2s ease;
  }

  .recall-modal {
    background: #1f2937;
    border: 2px solid rgba(239, 68, 68, 0.6);
    border-radius: 12px;
    padding: 1.5rem 2rem;
    max-width: 420px;
    width: 90%;
    box-shadow: 0 0 30px rgba(239, 68, 68, 0.25);
    animation: slideUp 0.25s ease;
  }

  .recall-modal-header {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    margin-bottom: 0.75rem;
  }

  .recall-modal-header h2 {
    font-family: 'Orbitron', sans-serif;
    font-size: 1rem;
    color: #ef4444;
    margin: 0;
  }

  .recall-modal-icon {
    font-size: 1.3rem;
  }

  .recall-modal-body {
    font-size: 0.85rem;
    color: #cbd5e1;
    line-height: 1.5;
    margin: 0 0 1.25rem;
  }

  .recall-modal-body strong {
    color: #f59e0b;
  }

  .recall-modal-actions {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
  }

  .recall-modal-cancel {
    background: transparent;
    border: 1px solid #475569;
    color: #94a3b8;
    padding: 0.4rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8rem;
  }

  .recall-modal-cancel:hover {
    color: #e6edf3;
    border-color: #94a3b8;
  }

  .recall-modal-confirm {
    background: rgba(239, 68, 68, 0.2);
    border: 1px solid #ef4444;
    color: #ef4444;
    padding: 0.4rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .recall-modal-confirm:hover {
    background: rgba(239, 68, 68, 0.35);
  }

  .recall-modal-confirm:disabled,
  .recall-modal-cancel:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
  @keyframes slideUp { from { transform: translateY(20px); opacity: 0; } to { transform: translateY(0); opacity: 1; } }

  .empty {
    padding: 2rem;
    text-align: center;
    color: #475569;
    font-size: 0.85rem;
  }
</style>
