<!--
  /directors/librarian/redact — Redact Document Field (UC-LIB-02)
  TheLibrarian redacts specific fields from archived documents.
-->
<script lang="ts">
  import { libRedactDocumentField } from '$lib/stores/directors';

  let documentId = $state('');
  let fieldName = $state('');
  let reason = $state('');
  let loading = $state(false);
  let error = $state('');
  let success = $state('');

  const COMMON_FIELDS = [
    'author', 'content', 'summary', 'classification', 'attachments',
    'metadata', 'recipients', 'notes', 'findings',
  ];

  async function handleSubmit() {
    error = ''; success = '';
    if (!documentId.trim() || !fieldName.trim()) {
      error = 'Document ID and field name are required.';
      return;
    }
    loading = true;
    try {
      await libRedactDocumentField(documentId.trim(), fieldName.trim(), reason.trim() || undefined);
      success = `Field "${fieldName}" redacted on document ${documentId.slice(0,8)}…`;
      documentId = ''; fieldName = ''; reason = '';
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }
</script>

<h1 class="title">Redact Document Field</h1>
<p class="subtitle">Mark a specific field on an archived document as redacted. The redaction is enforced on next document render.</p>

<div class="form-card">
  <label class="field">
    <span class="label">Document ID (UUID) *</span>
    <input class="input" type="text" bind:value={documentId} placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" />
  </label>
  <div class="field">
    <span class="label">Field Name *</span>
    <div class="field-row">
      <input class="input" type="text" bind:value={fieldName} placeholder="e.g. content, author, notes" />
    </div>
    <div class="common-fields">
      <span class="hint">Quick select:</span>
      {#each COMMON_FIELDS as f}
        <button class="chip" onclick={() => (fieldName = f)}>{f}</button>
      {/each}
    </div>
  </div>
  <label class="field">
    <span class="label">Reason</span>
    <textarea class="textarea" bind:value={reason} rows="3" placeholder="Legal basis or reason for redaction…"></textarea>
  </label>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}
  <button class="btn-primary" onclick={handleSubmit} disabled={loading}>
    {loading ? 'Applying…' : 'Apply Redaction'}
  </button>
</div>

<div class="info-box">
  <strong>Note:</strong> Redactions are recorded in the audit log and are visible to the Administrator. Only TheLibrarian or Administrator can redact fields.
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#8B5CF6;margin:0 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1.2rem; }
  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(139,92,246,0.15);border-radius:8px;padding:1.5rem;max-width:500px;display:flex;flex-direction:column;gap:0.8rem; }
  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .field-row { display:flex;gap:0.4rem; }
  .input,.textarea { background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem 0.6rem;font-size:0.82rem;font-family:'Inter',sans-serif;width:100%;box-sizing:border-box; }
  .input:focus,.textarea:focus { outline:none;border-color:#8B5CF6; }
  .textarea { resize:vertical; }
  .common-fields { display:flex;flex-wrap:wrap;gap:0.3rem;margin-top:0.3rem; }
  .hint { font-size:0.65rem;color:#475569;line-height:1.6; }
  .chip { background:rgba(139,92,246,0.1);border:1px solid rgba(139,92,246,0.25);color:#C084FC;border-radius:4px;padding:0.12rem 0.4rem;cursor:pointer;font-size:0.68rem;font-family:'Inter',sans-serif; }
  .chip:hover { background:rgba(139,92,246,0.2); }
  .btn-primary { background:rgba(139,92,246,0.15);border:1px solid #8B5CF6;color:#C084FC;border-radius:6px;padding:0.55rem;cursor:pointer;font-size:0.85rem;font-weight:600;margin-top:0.25rem; }
  .btn-primary:hover:not(:disabled) { background:rgba(139,92,246,0.25); }
  .btn-primary:disabled { opacity:0.5;cursor:default; }
  .info-box { background:rgba(139,92,246,0.05);border:1px solid rgba(139,92,246,0.12);border-radius:6px;padding:0.6rem 0.8rem;color:#94A3B8;font-size:0.75rem;max-width:500px;margin-top:1rem; }
  .error { color:#EF4444;font-size:0.75rem; }
  .success { color:#22C55E;font-size:0.75rem; }
</style>
