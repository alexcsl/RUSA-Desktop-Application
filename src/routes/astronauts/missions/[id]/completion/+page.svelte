<!-- /astronauts/missions/[id]/completion — UC-AS-04: Mission Completion Request -->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { submitCompletionRequest } from '$lib/stores/astronauts';

  let missionId = $state('');
  const unsub = page.subscribe((p) => (missionId = p.params.id));
  onDestroy(unsub);

  let findingsSummary = $state('');
  let selectedFiles: File[] = $state([]);
  let submitting = $state(false);
  let error = $state('');
  let success = $state('');

  function handleFileSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files) {
      selectedFiles = [...selectedFiles, ...Array.from(input.files)];
    }
    input.value = '';
  }

  function removeFile(index: number) {
    selectedFiles = selectedFiles.filter((_, i) => i !== index);
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  async function handleSubmit() {
    error = '';
    success = '';
    if (!findingsSummary.trim()) {
      error = 'Findings summary is required.';
      return;
    }
    submitting = true;
    try {
      // Read files into byte arrays for Tauri transport
      const fileNames: string[] = [];
      const contentTypes: string[] = [];
      const fileBytesList: number[][] = [];

      for (const f of selectedFiles) {
        fileNames.push(f.name);
        contentTypes.push(f.type || 'application/octet-stream');
        const buf = await f.arrayBuffer();
        fileBytesList.push(Array.from(new Uint8Array(buf)));
      }

      await submitCompletionRequest(
        {
          mission_id: missionId,
          findings_summary: findingsSummary,
          file_names: fileNames,
          content_types: contentTypes,
        },
        fileBytesList,
      );

      success = 'Completion request submitted! Redirecting…';
      setTimeout(() => goto(`/astronauts/missions/${missionId}`), 1500);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      submitting = false;
    }
  }
</script>

<a class="btn-back" href="/astronauts/missions/{missionId}">← Back to Mission</a>

<h1 class="title">Request Mission Completion</h1>
<p class="subtitle">Submit your findings and evidence for Wanderer review.</p>

<div class="form-card">
  <label class="field">
    <span class="label">Findings Summary <span class="required">*</span></span>
    <textarea
      class="textarea"
      bind:value={findingsSummary}
      rows="6"
      placeholder="Describe your findings, conclusions, and mission outcomes…"
    ></textarea>
  </label>

  <div class="field">
    <span class="label">Evidence Files</span>
    <label class="file-upload-btn">
      <input type="file" multiple onchange={handleFileSelect} class="file-input-hidden" />
      <span>+ Add Files</span>
    </label>

    {#if selectedFiles.length > 0}
      <div class="file-list">
        {#each selectedFiles as file, i}
          <div class="file-item">
            <span class="file-icon">📎</span>
            <span class="file-name">{file.name}</span>
            <span class="file-size">{formatSize(file.size)}</span>
            <button class="file-remove" onclick={() => removeFile(i)}>×</button>
          </div>
        {/each}
        <p class="file-count">{selectedFiles.length} file{selectedFiles.length !== 1 ? 's' : ''} selected</p>
      </div>
    {:else}
      <p class="hint">Upload images, documents, or data files as evidence. Optional.</p>
    {/if}
  </div>

  {#if error}<p class="error">{error}</p>{/if}
  {#if success}<p class="success">{success}</p>{/if}

  <button class="btn-primary" onclick={handleSubmit} disabled={submitting}>
    {submitting ? 'Uploading & Submitting…' : 'Submit Completion Request'}
  </button>
</div>

<style>
  .title { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0.5rem 0 0.3rem; }
  .subtitle { color:#94A3B8;font-size:0.8rem;margin:0 0 1rem; }
  .btn-back { color:#94A3B8;font-size:0.75rem;text-decoration:none; }
  .btn-back:hover { color:#E6EDF3; }

  .form-card { background:rgba(14,20,40,0.6);border:1px solid rgba(58,190,255,0.1);border-radius:8px;padding:1.25rem;max-width:640px;display:flex;flex-direction:column;gap:0.75rem; }

  .field { display:flex;flex-direction:column;gap:0.2rem; }
  .label { font-size:0.7rem;color:#94A3B8; }
  .required { color:#EF4444; }
  .textarea { width:100%;background:#0E1428;border:1px solid rgba(58,190,255,0.15);color:#E6EDF3;border-radius:6px;padding:0.45rem;font-size:0.8rem;font-family:'Inter',sans-serif;resize:vertical;box-sizing:border-box; }
  .textarea:focus { outline:none;border-color:#3ABEFF; }
  .hint { color:#475569;font-size:0.75rem;margin:0.2rem 0 0; }

  .file-upload-btn { display:inline-flex;padding:0.4rem 0.9rem;background:rgba(58,190,255,0.1);border:1px dashed rgba(58,190,255,0.3);border-radius:6px;color:#3ABEFF;cursor:pointer;font-size:0.8rem;font-weight:500;align-self:flex-start; }
  .file-upload-btn:hover { background:rgba(58,190,255,0.2); }
  .file-input-hidden { display:none; }

  .file-list { margin-top:0.4rem;display:flex;flex-direction:column;gap:0.2rem; }
  .file-item { display:flex;align-items:center;gap:0.4rem;padding:0.3rem 0.5rem;background:rgba(255,255,255,0.02);border-radius:4px;font-size:0.78rem; }
  .file-icon { font-size:0.85rem; }
  .file-name { flex:1;color:#E6EDF3;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .file-size { color:#94A3B8;font-size:0.7rem;min-width:60px;text-align:right; }
  .file-remove { background:none;border:none;color:#EF4444;font-size:1rem;cursor:pointer;padding:0 0.2rem; }
  .file-count { color:#94A3B8;font-size:0.7rem;margin:0.2rem 0 0; }

  .btn-primary { padding:0.5rem 1.25rem;background:linear-gradient(135deg,#3ABEFF 0%,#8B5CF6 100%);border:none;border-radius:6px;color:white;font-weight:600;cursor:pointer;font-size:0.8rem;align-self:flex-start; }
  .btn-primary:disabled { opacity:0.5;cursor:not-allowed; }
  .error { color:#EF4444;font-size:0.8rem;margin:0; }
  .success { color:#10B981;font-size:0.8rem;margin:0; }
</style>
