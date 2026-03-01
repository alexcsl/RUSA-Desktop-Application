<!--
  Psychiatry Division layout — shared nav bar for all /psychiatry/* routes.
  Role-based navigation: Psychiatrist links, Assistant links, Patient access settings.
  Matches Directors dark space theme (Orbitron / Inter, #0B0F1A / #111827 / #3ABEFF).
-->
<script lang="ts">
  import { currentUser, logout } from '$lib/stores/auth';
  import type { SessionUser } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { onMount, onDestroy } from 'svelte';
  import { psyGetAccessSettings, type AccessSetting } from '$lib/stores/psychiatry';

  let { children } = $props();
  let user: SessionUser | null = $state(null);
  currentUser.subscribe((v) => (user = v));

  let pathVal = $state('');
  page.subscribe((p) => (pathVal = p.url.pathname));

  let pendingRequests = $state(0);
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  /** Poll for pending access requests (patient role) */
  async function pollAccessRequests() {
    try {
      const settings = await psyGetAccessSettings();
      pendingRequests = settings.filter((s) => !s.granted).length;
    } catch {
      // Not a patient or no access settings — fine
    }
  }

  onMount(() => {
    pollAccessRequests();
    pollTimer = setInterval(pollAccessRequests, 15_000);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
  });

  interface NavLink { label: string; href: string; roles: string[]; }

  const navLinks: NavLink[] = [
    // Psychiatrist links
    { label: 'My Patients', href: '/psychiatry/patients', roles: ['Psychiatrist'] },
    { label: 'New Patient', href: '/psychiatry/patients/new', roles: ['Psychiatrist'] },
    { label: 'My Schedule', href: '/psychiatry/schedule', roles: ['Psychiatrist'] },
    { label: 'Patient Index', href: '/psychiatry/patient-index', roles: ['Psychiatrist'] },
    // Assistant links
    { label: 'Patient List', href: '/psychiatry/assistant/patients', roles: ['PsychiatristAssistant'] },
    { label: 'Book Appointment', href: '/psychiatry/assistant/schedule', roles: ['PsychiatristAssistant'] },
    { label: 'Request Access', href: '/psychiatry/assistant/access-request', roles: ['PsychiatristAssistant'] },
    // Patient (any role — access settings)
    { label: 'Access Settings', href: '/psychiatry/patient/access-settings', roles: ['all'] },
    // Messages for all
    { label: 'Messages', href: '/messaging/inbox?channel=general', roles: ['all'] },
  ];

  function visibleLinks(role: string | undefined): NavLink[] {
    if (!role) return [];
    return navLinks.filter((l) => l.roles.includes('all') || l.roles.includes(role));
  }

  async function handleLogout() { await logout(); goto('/auth'); }
</script>

<div class="shell">
  <header class="top-bar">
    <div class="brand">
      <img src="/Logo.png" alt="RUSA" class="logo" />
      <h1>Psychiatry Division</h1>
    </div>
    <div class="user-section">
      {#if pendingRequests > 0}
        <span class="badge-pending" title="Pending access requests">🔔 {pendingRequests}</span>
      {/if}
      {#if user}
        <span class="user-name">{user.full_name}</span>
        <span class="role-tag">{user.role}</span>
        <button class="btn-sm" onclick={handleLogout}>Log Out</button>
      {/if}
    </div>
  </header>

  <div class="body">
    <nav class="side-nav">
      {#each visibleLinks(user?.role) as link}
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
  .badge-pending { color:#F59E0B;font-size:0.75rem;cursor:default; }
  .btn-sm { background:transparent;color:#94A3B8;border:1px solid #475569;border-radius:4px;padding:0.2rem 0.45rem;cursor:pointer;font-size:0.7rem; }
  .btn-sm:hover { color:#E6EDF3;border-color:#3ABEFF; }

  .body { display:flex;flex:1;overflow:hidden; }
  .side-nav { width:200px;min-width:160px;background:#111827;border-right:1px solid rgba(58,190,255,0.1);overflow-y:auto;padding:0.5rem; }
  .side-nav a { display:block;padding:0.55rem 0.75rem;margin-bottom:0.15rem;border-radius:6px;color:#94A3B8;text-decoration:none;font-size:0.8rem; }
  .side-nav a:hover { color:#E6EDF3;background:rgba(58,190,255,0.05); }
  .side-nav a.active { color:#3ABEFF;background:rgba(58,190,255,0.1); }
  .main-content { flex:1;overflow-y:auto;padding:1.25rem; }
</style>
