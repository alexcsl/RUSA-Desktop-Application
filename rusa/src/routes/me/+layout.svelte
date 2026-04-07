<!--
  /me — Universal "My Account" layout for every authenticated RUSA user.
  Accessible regardless of role. Provides Profile, Appointments, and
  Schedule Access settings.
-->
<script lang="ts">
  import { currentUser, logout } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { onDestroy } from 'svelte';
  import type { SessionUser } from '$lib/stores/auth';
  import { getDefaultRoute } from '$lib/routing';

  let { children } = $props();
  let user: SessionUser | null = $state(null);
  const unsubUser = currentUser.subscribe((v) => (user = v));

  let pathVal = $state('');
  const unsubPage = page.subscribe((p) => (pathVal = p.url.pathname));

  onDestroy(() => { unsubUser(); unsubPage(); });

  async function handleLogout() { await logout(); goto('/auth'); }

  function backRoute(): string {
    if (!user) return '/auth';
    return getDefaultRoute(user.role);
  }

  const navLinks = [
    { label: 'My Profile',       href: '/me/profile' },
    { label: 'My Appointments',  href: '/me/appointments' },
    { label: 'Schedule Access',  href: '/me/schedule-access' },
  ];
</script>

<div class="shell">
  <header class="top-bar">
    <div class="brand">
      <img src="/Logo.png" alt="RUSA" class="logo" />
      <h1>My Account</h1>
    </div>
    <div class="user-section">
      {#if user}
        <span class="user-name">{user.full_name}</span>
        <span class="role-tag">{user.role}</span>
        <button class="btn-sm" onclick={handleLogout}>Log Out</button>
      {/if}
    </div>
  </header>

  <div class="body">
    <nav class="side-nav">
      <a href={backRoute()} class="back-link">← Back</a>
      {#each navLinks as link}
        <a href={link.href} class:active={pathVal.startsWith(link.href)}>
          {link.label}
        </a>
      {/each}
    </nav>
    <main class="main-content">
      {@render children()}
    </main>
  </div>
</div>

<style>
  .shell { display:flex;flex-direction:column;height:100vh;background:#0B0F1A;color:#E6EDF3;font-family:'Inter',sans-serif; }
  .top-bar { display:flex;justify-content:space-between;align-items:center;padding:0.6rem 1.25rem;background:#111827;border-bottom:1px solid rgba(58,190,255,0.15); }
  .brand { display:flex;align-items:center;gap:0.6rem; }
  .brand h1 { font-family:'Orbitron',sans-serif;font-size:1rem;color:#3ABEFF;margin:0; }
  .logo { width:28px;height:28px; }
  .user-section { display:flex;align-items:center;gap:0.6rem; }
  .user-name { font-size:0.8rem; }
  .role-tag { background:rgba(139,92,246,0.2);color:#C084FC;padding:0.1rem 0.4rem;border-radius:4px;font-size:0.7rem; }
  .btn-sm { background:transparent;color:#94A3B8;border:1px solid #475569;border-radius:4px;padding:0.2rem 0.45rem;cursor:pointer;font-size:0.7rem; }
  .btn-sm:hover { color:#E6EDF3;border-color:#3ABEFF; }
  .body { display:flex;flex:1;overflow:hidden; }
  .side-nav { width:180px;min-width:150px;background:#111827;border-right:1px solid rgba(58,190,255,0.1);overflow-y:auto;padding:0.5rem; }
  .side-nav a { display:block;padding:0.55rem 0.75rem;margin-bottom:0.15rem;border-radius:6px;color:#94A3B8;text-decoration:none;font-size:0.8rem; }
  .side-nav a:hover { color:#E6EDF3;background:rgba(58,190,255,0.05); }
  .side-nav a.active { color:#3ABEFF;background:rgba(58,190,255,0.1); }
  .back-link { color:#64748B !important;border:1px solid #374151;margin-bottom:0.5rem !important; }
  .back-link:hover { color:#94A3B8 !important;border-color:#475569; }
  .main-content { flex:1;overflow-y:auto;padding:1.25rem; }
</style>
