<!--
  /directors/anchorman/broadcast/new — Send Informational Broadcast (UC-ANC-01)
  TheAnchorman sends organization-wide or targeted informational messages.
-->
<script lang="ts">
  import { sendInformationalBroadcast } from '$lib/stores/directors';

  let subject = $state('');
  let content = $state('');
  let targetScope = $state('company_wide');
  let targetIdsRaw = $state('');
  let loading = $state(false);
  let error = $state('');
  let success = $state('');

  const SCOPES = [
    { value: 'company_wide', label: 'Company-Wide' },
    { value: 'sector', label: 'Sector' },
    { value: 'planet', label: 'Planet' },
    { value: 'group', label: 'Group' },
    { value: 'individual', label: 'Individual' },
  ];

  async function handleSubmit() {
    error = ''; success = '';
    if (!subject.trim() || !content.trim()) {
      error = 'Subject and content are required.';
      return;
    }
    const targetIds = targetScope === 'company_wide'
      ? []
      : targetIdsRaw.split(',').map(s => s.trim()).filter(Boolean);
    if (targetScope !== 'company_wide' && targetIds.length === 0) {
      error = 'Provide at least one target ID for non-company-wide scope.';
      return;
    }
    loading = true;
    try {
      await sendInformationalBroadcast({
        subject: subject.trim(),
        content: content.trim(),
        target_scope: targetScope,
        target_ids: targetIds,
      });
      success = 'Broadcast sent successfully.';
      subject = ''; content = ''; targetScope = 'company_wide'; targetIdsRaw = '';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }
</script>

<h1 class="title">New Informational Broadcast</h1>
<p class="subtitle">Send an organization-wide or targeted informational message.</p>

<div class="form-card">
  <label class="field">
    <span class="label">Subject *</span>
    <input class="input" type="text" bind:value={subject} placeholder="Broadcast subject…" />
  </label>
  <label class="field">
    <span class="label">Content *</span>
    <textarea class="textarea" bind:value={content} rows="5" placeholder="Message content…"></textarea>
  </label>
  <label class="field">
    <span class="label">Target Scope *</span>
    <select class="input" bind:value={targetScope}>
      {#each SCOPES as s}
        <option value={s.value}>{s.label}</option>
      {/each}
    </select>
  </label>
  {#if targetScope !== 'company_wide'}
    <label class="field">
      <span class="label">Target IDs (comma-separated UUIDs)</span>
      <input class="input" type="text" bind:value={targetIdsRaw} placeholder="uuid1, uuid2, …" />
    </label>
  {/if}

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}
  <button class="btn-primary" onclick={handleSubmit} disabled={loading}>
    {loading ? 'Sending…' : 'Send Broadcast'}
  </button>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1.2rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.5rem;max-width:520px;display:flex;flex-direction:column;gap:0.75rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .input,.textarea { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem 0.6rem;font-size:0.82rem;font-family:'Inter',sans-serif;width:100%;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#3ABEFF; }
  .textarea { resize:vertical; }
  .btn-primary { background:rgba(58,190,255,0.15);border:1px solid #3ABEFF;color:#3ABEFF;border-radius:6px;padding:0.55rem;cursor:pointer;font-size:0.85rem;font-weight:600;margin-top:0.25rem; }
  .btn-primary:hover:not(:disabled) { background:rgba(58,190,255,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:default; }
  .error { color:#EF4444;font-size:0.75rem; }
  .success { color:#22C55E;font-size:0.75rem; }
</style>
