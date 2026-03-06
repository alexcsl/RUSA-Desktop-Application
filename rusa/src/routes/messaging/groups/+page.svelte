<!--
  /messaging/groups — Messaging Groups Management
  Directors & Administrator can create and manage messaging groups.
  Other users have read-only visibility to groups they belong to.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { currentUser } from '$lib/stores/auth';
  import {
    getMessagingGroups, createMessagingGroup, getGroupMembers,
    addGroupMember, removeGroupMember, deleteMessagingGroup,
    getEligibleRecipients,
    type GroupSummary, type GroupMember, type EligibleRecipient,
  } from '$lib/stores/messaging';

  const managerRoles = [
    'TheAnchorman', 'TheGuardian', 'TheOverseer',
    'GalacticSecurityHead', 'HeadOfMedicine', 'GeneralDirector', 'Administrator',
  ];

  let groups: GroupSummary[] = $state([]);
  let selected: GroupSummary | null = $state(null);
  let members: GroupMember[] = $state([]);
  let allRecipients: EligibleRecipient[] = $state([]);

  // Create form
  let showCreate = $state(false);
  let newName = $state('');
  let newDescription = $state('');

  // Add member
  let memberSearch = $state('');

  let loading = $state(false);
  let error = $state('');
  let success = $state('');

  let isManager = $state(false);

  // Delete confirmation modal
  let deleteConfirmGroup: GroupSummary | null = $state(null);
  let deleting = $state(false);

  onMount(async () => {
    if (!$currentUser) { goto('/auth'); return; }
    isManager = managerRoles.includes($currentUser.role);
    await loadGroups();
    if (isManager) {
      try {
        allRecipients = await getEligibleRecipients('general');
      } catch { /* best-effort */ }
    }
  });

  async function loadGroups() {
    loading = true; error = '';
    try {
      groups = await getMessagingGroups();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    } finally { loading = false; }
  }

  async function selectGroup(g: GroupSummary) {
    selected = g;
    error = '';
    try {
      members = await getGroupMembers(g.id);
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleCreate() {
    error = ''; success = '';
    if (!newName.trim()) { error = 'Group name is required.'; return; }
    try {
      await createMessagingGroup(newName.trim(), newDescription.trim() || undefined);
      success = `Group "${newName.trim()}" created.`;
      newName = ''; newDescription = ''; showCreate = false;
      await loadGroups();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleAddMember(userId: string) {
    if (!selected) return;
    error = ''; success = '';
    try {
      await addGroupMember(selected.id, userId);
      members = await getGroupMembers(selected.id);
      memberSearch = '';
      success = 'Member added.';
      await loadGroups(); // refresh counts
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleRemoveMember(userId: string) {
    if (!selected) return;
    error = ''; success = '';
    try {
      await removeGroupMember(selected.id, userId);
      members = await getGroupMembers(selected.id);
      success = 'Member removed.';
      await loadGroups();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function promptDeleteGroup() {
    if (!selected) return;
    deleteConfirmGroup = selected;
  }

  function cancelDelete() {
    deleteConfirmGroup = null;
  }

  async function confirmDeleteGroup() {
    if (!deleteConfirmGroup) return;
    error = ''; success = ''; deleting = true;
    try {
      await deleteMessagingGroup(deleteConfirmGroup.id);
      success = `Group "${deleteConfirmGroup.name}" deleted.`;
      deleteConfirmGroup = null;
      selected = null;
      members = [];
      await loadGroups();
    } catch (e: unknown) {
      error = e instanceof Error ? e.message : String(e);
      deleteConfirmGroup = null;
    } finally { deleting = false; }
  }

  function filteredCandidates(): EligibleRecipient[] {
    const q = memberSearch.toLowerCase();
    const memberIds = new Set(members.map((m) => m.user_id));
    return allRecipients
      .filter(
        (p) =>
          !memberIds.has(p.id) &&
          (p.full_name.toLowerCase().includes(q) ||
           p.username.toLowerCase().includes(q)),
      )
      .slice(0, 10);
  }
</script>

<h1 class="title">Messaging Groups</h1>

{#if error}
  <div class="error-banner">{error}</div>
{/if}
{#if success}
  <div class="success-banner">{success}</div>
{/if}

<div class="split">
  <!-- Left: group list -->
  <div class="list-panel">
    {#if isManager}
      {#if showCreate}
        <div class="create-box">
          <input type="text" bind:value={newName} placeholder="Group name" class="inp" />
          <input type="text" bind:value={newDescription} placeholder="Description (optional)" class="inp" />
          <div class="create-actions">
            <button class="btn-primary" onclick={handleCreate}>Create</button>
            <button class="btn-ghost" onclick={() => { showCreate = false; }}>Cancel</button>
          </div>
        </div>
      {:else}
        <button class="btn-primary create-trigger" onclick={() => { showCreate = true; }}>
          + New Group
        </button>
      {/if}
    {/if}

    {#if loading}
      <div class="empty">Loading…</div>
    {:else if groups.length === 0}
      <div class="empty">No groups yet.</div>
    {:else}
      {#each groups as g}
        <button
          class="group-row"
          class:active={selected?.id === g.id}
          onclick={() => selectGroup(g)}
        >
          <span class="group-name">{g.name}</span>
          <span class="group-count">{g.member_count} members</span>
        </button>
      {/each}
    {/if}
  </div>

  <!-- Right: group detail -->
  <div class="detail-panel">
    {#if selected}
      <div class="detail-card">
        <div class="detail-header">
          <h2 class="detail-name">{selected.name}</h2>
          {#if isManager}
            <button class="delete-btn" onclick={promptDeleteGroup}>Delete</button>
          {/if}
        </div>
        {#if selected.description}
          <p class="detail-desc">{selected.description}</p>
        {/if}

        <h3 class="section-title">Members ({members.length})</h3>

        {#if isManager}
          <div class="add-member">
            <input
              type="text"
              placeholder="Search to add member…"
              bind:value={memberSearch}
              class="inp"
            />
            {#if memberSearch.length > 0}
              <div class="dropdown">
                {#each filteredCandidates() as c}
                  <button class="dropdown-item" onclick={() => handleAddMember(c.id)}>
                    {c.full_name} <span class="meta">{c.role_name}</span>
                  </button>
                {/each}
                {#if filteredCandidates().length === 0}
                  <div class="dropdown-empty">No matches</div>
                {/if}
              </div>
            {/if}
          </div>
        {/if}

        <div class="members-list">
          {#each members as m}
            <div class="member-row">
              <span class="member-name">{m.full_name}</span>
              <span class="member-role">{m.role_name}</span>
              {#if isManager}
                <button class="remove-btn" onclick={() => handleRemoveMember(m.user_id)}>
                  Remove
                </button>
              {/if}
            </div>
          {:else}
            <div class="empty">No members.</div>
          {/each}
        </div>
      </div>
    {:else}
      <div class="empty">Select a group to manage.</div>
    {/if}
  </div>
</div>

<!-- Delete confirmation modal -->
{#if deleteConfirmGroup}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div class="delete-overlay" role="dialog" aria-modal="true" tabindex="-1" onclick={cancelDelete} onkeydown={(e) => { if (e.key === 'Escape') cancelDelete(); }}>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="delete-modal" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
      <div class="delete-modal-header">
        <span class="delete-modal-icon">⚠</span>
        <h2>Delete Group</h2>
      </div>
      <p class="delete-modal-body">
        Are you sure you want to delete <strong>"{deleteConfirmGroup.name}"</strong>?
        This action cannot be undone.
      </p>
      <div class="delete-modal-actions">
        <button class="delete-modal-cancel" onclick={cancelDelete} disabled={deleting}>Cancel</button>
        <button class="delete-modal-confirm" onclick={confirmDeleteGroup} disabled={deleting}>
          {deleting ? 'Deleting…' : 'Yes, Delete'}
        </button>
      </div>
    </div>
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

  .list-panel {
    width: 260px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow-y: auto;
  }

  .create-trigger {
    width: 100%;
  }

  .create-box {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    margin-bottom: 0.5rem;
  }

  .create-actions {
    display: flex;
    gap: 0.4rem;
  }

  .inp {
    background: #0e1428;
    border: 1px solid rgba(58, 190, 255, 0.2);
    border-radius: 6px;
    padding: 0.45rem 0.65rem;
    color: #e6edf3;
    font-size: 0.85rem;
    width: 100%;
    box-sizing: border-box;
  }

  .inp:focus {
    outline: none;
    border-color: #3abeff;
  }

  .btn-primary {
    background: linear-gradient(135deg, #3abeff 0%, #8b5cf6 100%);
    color: #0b0f1a;
    font-weight: 700;
    font-size: 0.8rem;
    border: none;
    border-radius: 6px;
    padding: 0.4rem 1rem;
    cursor: pointer;
  }

  .btn-ghost {
    background: none;
    border: 1px solid rgba(58, 190, 255, 0.2);
    color: #94a3b8;
    font-size: 0.8rem;
    border-radius: 6px;
    padding: 0.4rem 1rem;
    cursor: pointer;
  }

  .group-row {
    display: flex;
    justify-content: space-between;
    width: 100%;
    background: rgba(15, 23, 42, 0.5);
    border: 1px solid transparent;
    border-radius: 6px;
    padding: 0.55rem 0.75rem;
    color: #e6edf3;
    cursor: pointer;
    text-align: left;
    transition: border-color 0.15s;
  }

  .group-row:hover {
    border-color: rgba(58, 190, 255, 0.3);
  }

  .group-row.active {
    border-color: #3abeff;
    background: rgba(58, 190, 255, 0.06);
  }

  .group-name {
    font-weight: 600;
    font-size: 0.85rem;
  }

  .group-count {
    font-size: 0.7rem;
    color: #64748b;
  }

  .detail-panel {
    flex: 1;
    overflow-y: auto;
  }

  .detail-card {
    background: rgba(15, 23, 42, 0.5);
    border: 1px solid rgba(58, 190, 255, 0.12);
    border-radius: 8px;
    padding: 1.25rem;
  }

  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .detail-name {
    font-family: 'Orbitron', sans-serif;
    font-size: 1rem;
    color: #e6edf3;
    margin: 0;
  }

  .delete-btn {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #ef4444;
    padding: 0.3rem 0.8rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .detail-desc {
    color: #94a3b8;
    font-size: 0.85rem;
    margin: 0 0 1rem;
  }

  .section-title {
    font-size: 0.85rem;
    color: #3abeff;
    margin: 1rem 0 0.5rem;
  }

  .add-member {
    position: relative;
    margin-bottom: 0.75rem;
  }

  .dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: #111827;
    border: 1px solid rgba(58, 190, 255, 0.2);
    border-radius: 6px;
    max-height: 160px;
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
    padding: 0.4rem 0.75rem;
    cursor: pointer;
    font-size: 0.8rem;
    text-align: left;
  }

  .dropdown-item:hover {
    background: rgba(58, 190, 255, 0.1);
  }

  .dropdown-empty {
    padding: 0.4rem 0.75rem;
    color: #475569;
    font-size: 0.8rem;
    font-style: italic;
  }

  .meta {
    color: #475569;
    font-size: 0.7rem;
  }

  .members-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .member-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.5rem;
    background: rgba(15, 23, 42, 0.3);
    border-radius: 4px;
  }

  .member-name {
    font-size: 0.85rem;
    color: #e6edf3;
    flex: 1;
  }

  .member-role {
    font-size: 0.7rem;
    color: #64748b;
  }

  .remove-btn {
    background: none;
    border: none;
    color: #ef4444;
    font-size: 0.7rem;
    cursor: pointer;
    padding: 0.15rem 0.4rem;
  }

  .remove-btn:hover {
    text-decoration: underline;
  }

  .empty {
    padding: 2rem;
    text-align: center;
    color: #475569;
    font-size: 0.85rem;
  }

  /* ── Delete confirmation modal ── */
  .delete-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 0.2s ease;
  }

  .delete-modal {
    background: #1f2937;
    border: 2px solid rgba(239, 68, 68, 0.6);
    border-radius: 12px;
    padding: 1.5rem 2rem;
    max-width: 420px;
    width: 90%;
    box-shadow: 0 0 30px rgba(239, 68, 68, 0.25);
    animation: slideUp 0.25s ease;
  }

  .delete-modal-header {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    margin-bottom: 0.75rem;
  }

  .delete-modal-header h2 {
    font-family: 'Orbitron', sans-serif;
    font-size: 1rem;
    color: #ef4444;
    margin: 0;
  }

  .delete-modal-icon {
    font-size: 1.3rem;
  }

  .delete-modal-body {
    font-size: 0.85rem;
    color: #cbd5e1;
    line-height: 1.5;
    margin: 0 0 1.25rem;
  }

  .delete-modal-body strong {
    color: #f59e0b;
  }

  .delete-modal-actions {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
  }

  .delete-modal-cancel {
    background: transparent;
    border: 1px solid #475569;
    color: #94a3b8;
    padding: 0.4rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8rem;
  }

  .delete-modal-cancel:hover {
    color: #e6edf3;
    border-color: #94a3b8;
  }

  .delete-modal-confirm {
    background: rgba(239, 68, 68, 0.2);
    border: 1px solid #ef4444;
    color: #ef4444;
    padding: 0.4rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .delete-modal-confirm:hover {
    background: rgba(239, 68, 68, 0.35);
  }

  .delete-modal-confirm:disabled,
  .delete-modal-cancel:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
  @keyframes slideUp { from { transform: translateY(20px); opacity: 0; } to { transform: translateY(0); opacity: 1; } }
</style>
