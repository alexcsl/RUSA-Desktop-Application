<!--
  /me/profile — Own profile data for any authenticated RUSA user.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { getMyProfile, type MyProfileData } from '$lib/stores/auth';

  let profile: MyProfileData | null = $state(null);
  let loading = $state(true);
  let error = $state('');

  onMount(async () => {
    try { profile = await getMyProfile(); }
    catch (e: unknown) { error = String(e); }
    loading = false;
  });
</script>

<div class="page">
  <h2>My Profile</h2>

  {#if loading}
    <p class="muted">Loading…</p>
  {:else if error}
    <div class="banner error">{error}</div>
  {:else if profile}
    <div class="card">
      <div class="row">
        <span class="label">Full Name</span>
        <span class="value">{profile.full_name}</span>
      </div>
      <div class="row">
        <span class="label">Username</span>
        <span class="value mono">{profile.username}</span>
      </div>
      <div class="row">
        <span class="label">Email</span>
        <span class="value">{profile.email ?? '—'}</span>
      </div>
      <div class="row">
        <span class="label">Role</span>
        <span class="value role-badge">{profile.role_name}</span>
      </div>
      <div class="row">
        <span class="label">Base Location</span>
        <span class="value">{profile.base_location ?? '—'}</span>
      </div>
      <div class="row">
        <span class="label">Account Created</span>
        <span class="value">{new Date(profile.created_at).toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' })}</span>
      </div>
      <div class="row">
        <span class="label">Status</span>
        <span class="value {profile.is_active ? 'active' : 'inactive'}">{profile.is_active ? 'Active' : 'Inactive'}</span>
      </div>
    </div>

    <p class="hint">Contact your Director or Administrator to update your profile information.</p>
  {/if}
</div>

<style>
  .page { max-width:600px; }
  h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#3ABEFF;margin:0 0 1.25rem; }
  .muted { color:#94A3B8;font-size:0.85rem; }
  .banner { padding:0.6rem 1rem;border-radius:6px;font-size:0.8rem;margin-bottom:1rem; }
  .banner.error { background:rgba(239,68,68,0.12);color:#EF4444;border:1px solid rgba(239,68,68,0.3); }
  .card { background:#111827;border:1px solid rgba(58,190,255,0.1);border-radius:10px;overflow:hidden;margin-bottom:1rem; }
  .row { display:flex;align-items:baseline;gap:1rem;padding:0.75rem 1.1rem;border-bottom:1px solid rgba(255,255,255,0.04); }
  .row:last-child { border-bottom:none; }
  .label { font-size:0.72rem;color:#64748B;min-width:130px;text-transform:uppercase;letter-spacing:0.04em; }
  .value { font-size:0.85rem;color:#E6EDF3; }
  .mono { font-family:'Fira Mono','Courier New',monospace;font-size:0.8rem;color:#94A3B8; }
  .role-badge { background:rgba(139,92,246,0.15);color:#C084FC;padding:0.15rem 0.55rem;border-radius:4px;font-size:0.75rem;font-weight:600; }
  .active { color:#34D399;font-weight:600; }
  .inactive { color:#EF4444;font-weight:600; }
  .hint { font-size:0.75rem;color:#4B5563;font-style:italic; }
</style>
