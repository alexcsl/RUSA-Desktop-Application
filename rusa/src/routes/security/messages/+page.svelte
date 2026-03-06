<!-- /security/messages — Security Messaging (UC-SH-04 / UC-SS-02) -->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    secSendSecurityMessage,
    secGetSecurityMessages,
    getSecurityPersonnel,
    type SecMessageSummary,
    type SecurityPersonnelItem,
  } from '$lib/stores/security';

  /* ── tab state ─────────────────────────── */
  let tab: 'inbox' | 'compose' = $state('inbox');

  /* ── inbox ─────────────────────────────── */
  let messages: SecMessageSummary[] = $state([]);
  let loadingInbox = $state(false);

  /* ── compose ───────────────────────────── */
  let personnel: SecurityPersonnelItem[] = $state([]);
  let toIds: string[] = $state([]);
  let subject = $state('');
  let body = $state('');
  let sending = $state(false);
  let error = $state('');
  let success = $state('');

  /* ── search filter ─────────────────────── */
  let search = $state('');
  let filtered = $derived(
    messages.filter((m) => {
      const q = search.toLowerCase();
      return (
        m.subject.toLowerCase().includes(q) ||
        m.body.toLowerCase().includes(q) ||
        m.from_name.toLowerCase().includes(q)
      );
    })
  );

  /* ── init ──────────────────────────────── */
  onMount(async () => {
    loadingInbox = true;
    try {
      [messages, personnel] = await Promise.all([
        secGetSecurityMessages(),
        getSecurityPersonnel(),
      ]);
    } catch {}
    loadingInbox = false;
  });

  /* ── toggle recipient ──────────────────── */
  function toggleRecipient(id: string) {
    if (toIds.includes(id)) {
      toIds = toIds.filter((x) => x !== id);
    } else {
      toIds = [...toIds, id];
    }
  }

  /* ── send ──────────────────────────────── */
  async function handleSend() {
    error = ''; success = '';
    if (!subject.trim()) { error = 'Subject is required.'; return; }
    if (!body.trim()) { error = 'Body is required.'; return; }
    if (toIds.length === 0) { error = 'Select at least one recipient.'; return; }

    sending = true;
    try {
      await secSendSecurityMessage({ recipients_to: toIds, subject, body });
      success = 'Message sent.';
      subject = ''; body = ''; toIds = [];
      messages = await secGetSecurityMessages();
      setTimeout(() => { tab = 'inbox'; success = ''; }, 900);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      sending = false;
    }
  }
</script>

<h1 class="title">Security Messages</h1>

<div class="tab-bar">
  <button class="tab" class:active={tab === 'inbox'} onclick={() => (tab = 'inbox')}>Inbox</button>
  <button class="tab" class:active={tab === 'compose'} onclick={() => (tab = 'compose')}>Compose</button>
</div>

{#if tab === 'inbox'}
  <div class="toolbar">
    <input class="search" type="text" placeholder="Search messages…" bind:value={search} />
  </div>

  {#if loadingInbox}
    <p class="muted">Loading…</p>
  {:else if filtered.length === 0}
    <p class="muted">No security messages found.</p>
  {:else}
    <div class="msg-list">
      {#each filtered as msg}
        <div class="msg-card">
          <div class="msg-header">
            <span class="msg-from">{msg.from_name}</span>
            <span class="msg-date">{new Date(msg.created_at).toLocaleString()}</span>
          </div>
          <p class="msg-subject">{msg.subject}</p>
          <p class="msg-body">{msg.body}</p>
        </div>
      {/each}
    </div>
  {/if}
{:else}
  <!-- Compose -->
  <div class="form-card">
    <h2>Compose Message</h2>

    <div class="field">
      <span class="label">Recipients *</span>
      <div class="chip-picker">
        {#each personnel as p}
          <button
            class="chip"
            class:selected={toIds.includes(p.id)}
            onclick={() => toggleRecipient(p.id)}
          >
            {p.full_name}
            <span class="chip-role">{p.role_name}</span>
          </button>
        {/each}
      </div>
      {#if toIds.length > 0}
        <span class="muted" style="font-size:0.65rem">{toIds.length} selected</span>
      {/if}
    </div>

    <label class="field"><span class="label">Subject *</span>
      <input class="input" type="text" bind:value={subject} placeholder="Message subject…" />
    </label>

    <label class="field"><span class="label">Body *</span>
      <textarea class="textarea" bind:value={body} rows="5" placeholder="Message body…"></textarea>
    </label>

    {#if error}<p class="error">{error}</p>{/if}
    {#if success}<p class="success">{success}</p>{/if}

    <button class="btn-primary" onclick={handleSend} disabled={sending}>
      {sending ? 'Sending…' : 'Send Message'}
    </button>
  </div>
{/if}

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.6rem; }

  /* tabs */
  .tab-bar { display:flex;gap:0.25rem;margin-bottom:0.75rem; }
  .tab { padding:0.35rem 0.85rem;border-radius:6px;font-size:0.75rem;font-weight:600;background:rgba(255,255,255,0.04);border:1px solid rgba(58,190,255,0.1);color:#94A3B8;cursor:pointer;transition:all .15s; }
  .tab.active { background:rgba(58,190,255,0.12);color:#3ABEFF;border-color:rgba(58,190,255,0.35); }

  /* toolbar */
  .toolbar { margin-bottom:0.6rem; }
  .search { width:100%;max-width:400px;background:#0E1428;border:1px solid rgba(58,190,255,0.15);border-radius:6px;color:#E6EDF3;padding:0.4rem 0.6rem;font-size:0.8rem; }
  .search:focus { outline:none;border-color:#3ABEFF; }

  /* message list */
  .msg-list { display:flex;flex-direction:column;gap:0.35rem;max-width:700px; }
  .msg-card { background:rgba(14,20,40,0.5);border:1px solid rgba(58,190,255,0.08);border-radius:6px;padding:0.6rem; }
  .msg-header { display:flex;justify-content:space-between;align-items:center; }
  .msg-from { font-size:0.8rem;font-weight:600;color:#8B5CF6; }
  .msg-date { font-size:0.65rem;color:#6B7280; }
  .msg-subject { font-size:0.82rem;font-weight:500;color:#E6EDF3;margin:0.25rem 0 0.15rem; }
  .msg-body { font-size:0.75rem;color:#94A3B8;margin:0; }

  /* compose form */
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:600px;display:flex;flex-direction:column;gap:0.6rem; }
  .form-card h2 { font-family:'Orbitron',sans-serif;font-size:0.9rem;color:#8B5CF6;margin:0; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }

  /* chip picker */
  .chip-picker { display:flex;flex-wrap:wrap;gap:0.3rem; }
  .chip { padding:0.25rem 0.5rem;border-radius:999px;font-size:0.7rem;background:rgba(255,255,255,0.04);border:1px solid rgba(58,190,255,0.15);color:#94A3B8;cursor:pointer;display:flex;align-items:center;gap:0.25rem;transition:all .15s; }
  .chip.selected { background:rgba(58,190,255,0.15);border-color:#3ABEFF;color:#3ABEFF; }
  .chip-role { font-size:0.55rem;color:#6B7280; }

  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
  .muted { color:#94A3B8;font-size:0.8rem; }
</style>
