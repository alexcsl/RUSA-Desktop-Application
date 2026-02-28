<!--
  Data area layout — shared nav bar for all /data/* routes.
  Serves Data Analyst role (sidebar with analyst-specific links)
  and cross-role data request pages (submit, status tracker).
-->
<script lang="ts">
  import { currentUser, logout } from '$lib/stores/auth';
  import type { SessionUser } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { getNotifications, markNotificationRead, type NotificationItem } from '$lib/stores/directors';
  import { getDefaultRoute } from '$lib/routing';
  import { onMount, onDestroy } from 'svelte';

  let { children } = $props();
  let user: SessionUser | null = $state(null);
  currentUser.subscribe((v) => (user = v));

  let pathVal = $state('');
  page.subscribe((p) => (pathVal = p.url.pathname));

  let notifications: NotificationItem[] = $state([]);
  let showNotifs = $state(false);
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  async function pollNotifications() {
    try { notifications = await getNotifications(); } catch {}
  }

  onMount(() => {
    pollNotifications();
    pollTimer = setInterval(pollNotifications, 10_000);
  });

  onDestroy(() => { if (pollTimer) clearInterval(pollTimer); });

  async function dismissNotif(id: string) {
    await markNotificationRead(id);
    notifications = notifications.filter((n) => n.id !== id);
  }

  function formatNotifType(type_: string): string {
    const map: Record<string, string> = {
      'data_request:submitted': 'Request Submitted',
      'data_request:approved': 'Request Approved',
      'data_request:rejected': 'Request Rejected',
      'data_request:forwarded': 'New Assignment',
      'data_response:ready_for_review': 'Response Ready',
      'data_response:delivered': 'Data Delivered',
      'data_response:withheld': 'Response Withheld',
    };
    return map[type_] ?? type_.replace(/[_:]/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function formatNotifText(n: NotificationItem): string {
    const p = n.payload as Record<string, unknown>;
    switch (n.type_) {
      case 'data_request:submitted':
        return `New data request from ${p.requester_name ?? 'unknown'} — ${p.dataset_description ?? ''}`;
      case 'data_request:approved':
        return `Your data request was approved${p.reason ? `: ${p.reason}` : ''}`;
      case 'data_request:rejected':
        return `Your data request was rejected${p.reason ? `: ${p.reason}` : ''}`;
      case 'data_request:forwarded':
        return String(p.message ?? 'A new data request is ready for processing.');
      case 'data_response:ready_for_review':
        return `Response from ${p.analyst_name ?? 'analyst'} is ready for outbound review.`;
      case 'data_response:delivered':
        return String(p.message ?? 'Your data request has been fulfilled.');
      case 'data_response:withheld':
        return `Your response was withheld${p.reason ? `: ${p.reason}` : ''}. Please revise.`;
      default:
        return Object.entries(p).map(([k, v]) => `${k}: ${v}`).join(' | ') || 'No details';
    }
  }

  function isAnalyst(): boolean {
    return user?.role === 'DataAnalyst' || user?.role === 'Administrator';
  }

  interface NavLink { label: string; href: string; analystOnly: boolean; }

  const navLinks: NavLink[] = [
    { label: 'Analyst Inbox', href: '/data/analyst/inbox', analystOnly: true },
    { label: 'Submit Request', href: '/data/request/new', analystOnly: false },
    { label: 'My Requests', href: '/data/request/mine', analystOnly: false },
    { label: 'Messages', href: '/messaging/inbox?channel=general', analystOnly: false },
  ];

  function visibleLinks(): NavLink[] {
    return navLinks.filter((l) => !l.analystOnly || isAnalyst());
  }

  async function handleLogout() { await logout(); goto('/auth'); }
</script>

<div class="shell">
  <header class="top-bar">
    <div class="brand">
      <img src="/Logo.png" alt="RUSA" class="logo" />
      <h1>{isAnalyst() ? 'Data Analyst Hub' : 'Data Requests'}</h1>
    </div>
    <div class="user-section">
      <button class="notif-btn" onclick={() => (showNotifs = !showNotifs)}>
        🔔 {notifications.length > 0 ? `${notifications.length}` : ''}
      </button>
      {#if user}
        <span class="user-name">{user.full_name}</span>
        <span class="role-tag">{user.role}</span>
        <button class="btn-sm" onclick={handleLogout}>Log Out</button>
      {/if}
    </div>
  </header>

  {#if showNotifs && notifications.length > 0}
    <div class="notif-panel">
      {#each notifications as n}
        <div class="notif-item">
          <span class="notif-type">{formatNotifType(n.type_)}</span>
          <span class="notif-text">{formatNotifText(n)}</span>
          <button class="notif-dismiss" onclick={() => dismissNotif(n.id)}>×</button>
        </div>
      {/each}
    </div>
  {/if}

  <div class="body">
    <nav class="side-nav">
      {#each visibleLinks() as link}
        <a href={link.href} class:active={pathVal.startsWith(link.href.split('?')[0])}>
          {link.label}
        </a>
      {/each}
      {#if user}
        <a href={getDefaultRoute(user.role)} class="back-link">← Back to Dashboard</a>
      {/if}
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
  .notif-btn { background:none;border:none;color:#F59E0B;font-size:0.75rem;cursor:pointer;min-width:20px; }
  .btn-sm { background:transparent;color:#94A3B8;border:1px solid #475569;border-radius:4px;padding:0.2rem 0.45rem;cursor:pointer;font-size:0.7rem; }
  .btn-sm:hover { color:#E6EDF3;border-color:#3ABEFF; }

  .notif-panel { background:#1F2937;border-bottom:1px solid rgba(58,190,255,0.1);max-height:200px;overflow-y:auto;padding:0.5rem 1rem; }
  .notif-item { display:flex;align-items:center;gap:0.5rem;padding:0.3rem 0;font-size:0.75rem;border-bottom:1px solid rgba(255,255,255,0.05); }
  .notif-type { color:#8B5CF6;font-weight:600;min-width:120px;white-space:nowrap; }
  .notif-text { flex:1;color:#94A3B8;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .notif-dismiss { background:none;border:none;color:#EF4444;cursor:pointer;font-size:1rem; }

  .body { display:flex;flex:1;overflow:hidden; }
  .side-nav { width:190px;min-width:150px;background:#111827;border-right:1px solid rgba(58,190,255,0.1);overflow-y:auto;padding:0.5rem;display:flex;flex-direction:column; }
  .side-nav a { display:block;padding:0.55rem 0.75rem;margin-bottom:0.15rem;border-radius:6px;color:#94A3B8;text-decoration:none;font-size:0.8rem; }
  .side-nav a:hover { color:#E6EDF3;background:rgba(58,190,255,0.05); }
  .side-nav a.active { color:#3ABEFF;background:rgba(58,190,255,0.1); }
  .side-nav .back-link { margin-top:auto;border-top:1px solid rgba(58,190,255,0.1);padding-top:0.6rem;color:#F59E0B;font-size:0.75rem; }
  .side-nav .back-link:hover { color:#FBBF24;background:rgba(245,158,11,0.08); }
  .main-content { flex:1;overflow-y:auto;padding:1.25rem; }
</style>
