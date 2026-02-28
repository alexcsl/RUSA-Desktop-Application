<!--
  Admin area layout — shared nav for all /admin/* routes.
  Gives Administrator a consistent sidebar with access to all management pages.
  Includes real-time notification polling and emergency alert modal.
-->
<script lang="ts">
  import { currentUser, logout } from '$lib/stores/auth';
  import type { SessionUser } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { getNotifications, markNotificationRead, type NotificationItem } from '$lib/stores/directors';
  import { onMount, onDestroy } from 'svelte';

  let { children } = $props();
  let user: SessionUser | null = $state(null);
  currentUser.subscribe((v) => (user = v));

  let pathVal = $state('');
  page.subscribe((p) => (pathVal = p.url.pathname));

  let notifications: NotificationItem[] = $state([]);
  let showNotifs = $state(false);
  let emergencyAlert: NotificationItem | null = $state(null);
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let shownEmergencyIds = new Set<string>();

  async function pollNotifications() {
    try {
      const fresh = await getNotifications();
      notifications = fresh;
      const newEmergency = fresh.find(
        (n) =>
          !n.read_at &&
          (n.type_ === 'broadcast:emergency' || n.type_ === 'broadcast:security') &&
          !shownEmergencyIds.has(n.id)
      );
      if (newEmergency) {
        shownEmergencyIds.add(newEmergency.id);
        emergencyAlert = newEmergency;
      }
    } catch {}
  }

  onMount(() => {
    if (user && user.role !== 'Administrator') {
      goto('/forbidden');
      return;
    }
    pollNotifications();
    pollTimer = setInterval(pollNotifications, 10_000);
  });

  onDestroy(() => {
    if (pollTimer) clearInterval(pollTimer);
  });

  async function dismissNotif(id: string) {
    await markNotificationRead(id);
    notifications = notifications.filter((n) => n.id !== id);
    if (emergencyAlert?.id === id) emergencyAlert = null;
  }

  async function dismissEmergency() {
    if (emergencyAlert) {
      await markNotificationRead(emergencyAlert.id);
      notifications = notifications.filter((n) => n.id !== emergencyAlert!.id);
      emergencyAlert = null;
    }
  }

  function formatNotifType(type_: string): string {
    const map: Record<string, string> = {
      'vote:new': 'New Vote',
      'vote:decided': 'Vote Decided',
      'vote:overridden': 'Vote Overridden',
      'vote:terminated': 'Vote Terminated',
      'task:assigned': 'Task Assigned',
      'task:updated': 'Task Updated',
      'relocation:issued': 'Relocation Order',
      'event:invited': 'Event Invitation',
      'broadcast:emergency': 'EMERGENCY',
      'broadcast:security': 'SECURITY ALERT',
      'broadcast:informational': 'Broadcast',
      'request:approved': 'Request Approved',
      'request:denied': 'Request Denied',
      'meeting:scheduled': 'Meeting Scheduled',
    };
    return map[type_] ?? type_.replace(/[_:]/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function formatNotifText(n: NotificationItem): string {
    const p = n.payload as Record<string, unknown>;
    switch (n.type_) {
      case 'vote:new':
        return `New voting session: "${p.topic ?? 'Unknown'}"`;
      case 'vote:decided':
        return `Vote "${p.topic ?? ''}" decided: ${p.result ?? 'unknown'}`;
      case 'vote:overridden':
        return `Vote "${p.topic ?? ''}" overridden by admin: ${p.decision ?? ''}`;
      case 'vote:terminated':
        return `Vote "${p.topic ?? ''}" terminated by admin`;
      case 'task:assigned':
        return `New task assigned: "${p.title ?? ''}"${p.due_date ? ` — due ${p.due_date}` : ''}`;
      case 'task:updated':
        return `Task "${p.title ?? ''}" was updated: ${p.status ?? ''}`;
      case 'relocation:issued':
        return `Relocation to ${p.destination ?? 'unknown'} (${p.type ?? ''}) — effective ${p.effective_date ?? ''}`;
      case 'event:invited':
        return `You're invited to "${p.title ?? ''}"${p.event_date ? ` on ${new Date(p.event_date as string).toLocaleDateString()}` : ''}`;
      case 'broadcast:emergency':
      case 'broadcast:security':
        return `${p.subject ?? 'Emergency Alert'}: ${p.content ?? ''}`;
      case 'broadcast:informational':
        return `${p.subject ?? 'Broadcast'}: ${p.content ?? ''}`;
      case 'request:approved':
        return `Your request "${p.title ?? ''}" was approved`;
      case 'request:denied':
        return `Your request "${p.title ?? ''}" was denied${p.reason ? `: ${p.reason}` : ''}`;
      case 'meeting:scheduled':
        return `Meeting "${p.title ?? ''}" scheduled for ${p.scheduled_at ? new Date(p.scheduled_at as string).toLocaleString() : 'TBD'}`;
      default:
        return Object.entries(p).map(([k, v]) => `${k}: ${v}`).join(' | ') || 'No details';
    }
  }

  function isEmergencyNotif(n: NotificationItem): boolean {
    return n.type_ === 'broadcast:emergency' || n.type_ === 'broadcast:security';
  }

  interface NavLink { label: string; href: string; }

  const navLinks: NavLink[] = [
    { label: 'Vote Management', href: '/admin/votes' },
    { label: 'Directors Votes', href: '/admin/directors-votes' },
    { label: 'Accounts', href: '/admin/accounts' },
    { label: 'Relocate', href: '/admin/relocate' },
    { label: 'Data Request', href: '/data/request/new' },
    { label: 'Messages', href: '/messaging/inbox?channel=general' },
  ];

  async function handleLogout() { await logout(); goto('/auth'); }
</script>

<div class="shell">
  <header class="top-bar">
    <div class="brand">
      <img src="/Logo.png" alt="RUSA" class="logo" />
      <h1>Administrator Console</h1>
    </div>
    <div class="user-section">
      <button class="notif-btn" onclick={() => (showNotifs = !showNotifs)}>
        🔔 {notifications.length > 0 ? `${notifications.length}` : ''}
      </button>
      {#if user}
        <span class="user-name">{user.full_name}</span>
        <span class="role-tag">Administrator</span>
        <button class="btn-sm" onclick={handleLogout}>Log Out</button>
      {/if}
    </div>
  </header>

  {#if showNotifs && notifications.length > 0}
    <div class="notif-panel">
      {#each notifications as n}
        <div class="notif-item" class:notif-emergency={isEmergencyNotif(n)}>
          <span class="notif-type" class:emergency-type={isEmergencyNotif(n)}>{formatNotifType(n.type_)}</span>
          <span class="notif-text">{formatNotifText(n)}</span>
          <button class="notif-dismiss" onclick={() => dismissNotif(n.id)}>×</button>
        </div>
      {/each}
    </div>
  {/if}

  {#if emergencyAlert}
    <div class="emergency-overlay" onclick={dismissEmergency}>
      <div class="emergency-modal" onclick={(e) => e.stopPropagation()}>
        <div class="emergency-header">
          <span class="emergency-icon">⚠</span>
          <h2>{formatNotifType(emergencyAlert.type_)}</h2>
        </div>
        <div class="emergency-body">
          <p>{formatNotifText(emergencyAlert)}</p>
        </div>
        <button class="emergency-dismiss-btn" onclick={dismissEmergency}>Acknowledge</button>
      </div>
    </div>
  {/if}

  <div class="body">
    <nav class="side-nav">
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
  .top-bar { display:flex;justify-content:space-between;align-items:center;padding:0.6rem 1.25rem;background:#111827;border-bottom:1px solid rgba(239,68,68,0.2); }
  .brand { display:flex;align-items:center;gap:0.6rem; }
  .brand h1 { font-family:'Orbitron',sans-serif;font-size:1rem;color:#EF4444;margin:0; }
  .logo { width:28px;height:28px; }
  .user-section { display:flex;align-items:center;gap:0.6rem; }
  .user-name { font-size:0.8rem; }
  .role-tag { background:rgba(239,68,68,0.2);color:#EF4444;padding:0.1rem 0.4rem;border-radius:4px;font-size:0.7rem; }
  .notif-btn { background:none;border:none;color:#F59E0B;font-size:0.75rem;cursor:pointer;min-width:20px; }
  .btn-sm { background:transparent;color:#94A3B8;border:1px solid #475569;border-radius:4px;padding:0.2rem 0.45rem;cursor:pointer;font-size:0.7rem; }
  .btn-sm:hover { color:#E6EDF3;border-color:#EF4444; }

  .notif-panel { background:#1F2937;border-bottom:1px solid rgba(239,68,68,0.1);max-height:200px;overflow-y:auto;padding:0.5rem 1rem; }
  .notif-item { display:flex;align-items:center;gap:0.5rem;padding:0.3rem 0;font-size:0.75rem;border-bottom:1px solid rgba(255,255,255,0.05); }
  .notif-item.notif-emergency { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);border-radius:4px;padding:0.4rem 0.5rem;margin:0.15rem 0; }
  .notif-type { color:#EF4444;font-weight:600;min-width:100px;white-space:nowrap; }
  .notif-type.emergency-type { color:#EF4444;text-transform:uppercase;animation:pulse-red 2s ease-in-out infinite; }
  .notif-text { flex:1;color:#94A3B8;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .notif-dismiss { background:none;border:none;color:#EF4444;cursor:pointer;font-size:1rem; }

  .emergency-overlay { position:fixed;inset:0;background:rgba(0,0,0,0.7);display:flex;align-items:center;justify-content:center;z-index:1000;animation:fadeIn 0.2s ease; }
  .emergency-modal { background:#1F2937;border:2px solid #EF4444;border-radius:12px;padding:2rem;max-width:500px;width:90%;box-shadow:0 0 40px rgba(239,68,68,0.4);animation:slideUp 0.3s ease; }
  .emergency-header { display:flex;align-items:center;gap:0.75rem;margin-bottom:1rem; }
  .emergency-header h2 { font-family:'Orbitron',sans-serif;font-size:1.1rem;color:#EF4444;margin:0; }
  .emergency-icon { font-size:1.5rem;animation:pulse-red 1s ease-in-out infinite; }
  .emergency-body { margin-bottom:1.5rem; }
  .emergency-body p { color:#E6EDF3;font-size:0.9rem;line-height:1.5;margin:0; }
  .emergency-dismiss-btn { width:100%;padding:0.6rem;background:rgba(239,68,68,0.2);border:1px solid #EF4444;color:#EF4444;border-radius:6px;cursor:pointer;font-size:0.85rem;font-weight:600; }
  .emergency-dismiss-btn:hover { background:rgba(239,68,68,0.35); }
  @keyframes pulse-red { 0%,100%{opacity:1}50%{opacity:0.5} }
  @keyframes fadeIn { from{opacity:0}to{opacity:1} }
  @keyframes slideUp { from{transform:translateY(20px);opacity:0}to{transform:translateY(0);opacity:1} }

  .body { display:flex;flex:1;overflow:hidden; }
  .side-nav { width:200px;min-width:160px;background:#111827;border-right:1px solid rgba(239,68,68,0.1);overflow-y:auto;padding:0.5rem; }
  .side-nav a { display:block;padding:0.55rem 0.75rem;margin-bottom:0.15rem;border-radius:6px;color:#94A3B8;text-decoration:none;font-size:0.8rem; }
  .side-nav a:hover { color:#E6EDF3;background:rgba(239,68,68,0.05); }
  .side-nav a.active { color:#EF4444;background:rgba(239,68,68,0.1); }
  .main-content { flex:1;overflow-y:auto;padding:1.25rem; }
</style>
