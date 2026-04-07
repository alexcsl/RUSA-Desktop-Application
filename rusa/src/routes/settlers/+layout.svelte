<!--
  Settlers area layout — role-filtered sidebar for /settlers/* routes.
  Source of truth: 06_EXOPLANET_SETTLERS.md

  Roles and their nav:
  - SettlerCommander: Dashboard, Assign Task, My Tasks, Incoming Queue, Commander Anomaly,
      Abandonment, Repatriation, Commander Supply, Manage Inventory,
      Progress Reports, Building Logs, Farm Logs,
      Anomaly Report, Settler Report, Data Request, My Requests, Security Report
  - CivilEngineer: My Tasks, Anomaly Report, Settler Report, Material Request,
      Construction Report, Building Health, Residence, Inventory,
      Data Request, My Requests, Security Report
  - Farmer: My Tasks, Anomaly Report, Settler Report, Farm Supplies,
      Farm Health Log, Inventory, Data Request, My Requests, Security Report
  - TemporarySetter: Anomaly Report, Settler Report ONLY
-->
<script lang="ts">
  import { currentUser, logout } from '$lib/stores/auth';
  import type { SessionUser } from '$lib/stores/auth';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { onMount, onDestroy } from 'svelte';
  import { getNotifications, markNotificationRead, type NotificationItem } from '$lib/stores/directors';

  let { children } = $props();
  let user: SessionUser | null = $state(null);
  const unsubUser = currentUser.subscribe((v) => (user = v));

  let pathVal = $state('');
  const unsubPage = page.subscribe((p) => (pathVal = p.url.pathname));

  let notifications: NotificationItem[] = $state([]);
  let showNotifs = $state(false);
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  async function pollNotifications() {
    try { notifications = await getNotifications(); } catch {}
  }

  onMount(() => { pollNotifications(); pollTimer = setInterval(pollNotifications, 10_000); });
  onDestroy(() => { unsubUser(); unsubPage(); if (pollTimer) clearInterval(pollTimer); });

  async function dismissNotif(id: string) {
    await markNotificationRead(id);
    notifications = notifications.filter((n) => n.id !== id);
  }

  function formatNotifType(type_: string): string {
    const map: Record<string, string> = {
      'task:assigned': 'Task Assigned',
      'report:received': 'Report Received',
      'request:rejected': 'Request Rejected',
      'request:forwarded': 'Request Forwarded',
      'request:outcome': 'Request Outcome',
      'vote:new': 'New Vote',
      'broadcast:emergency': 'EMERGENCY',
      'broadcast:security': 'SECURITY',
    };
    return map[type_] ?? type_.replace(/[_:]/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function formatNotifText(n: NotificationItem): string {
    const p = n.payload as Record<string, unknown>;
    switch (n.type_) {
      case 'task:assigned': return `New task: "${p.title ?? ''}"`;
      case 'report:received': return `${String(p.report_type ?? 'report').replace(/_/g, ' ')} received`;
      case 'request:rejected': return `Your request was rejected: ${p.reason ?? ''}`;
      case 'request:forwarded': return `Your request was forwarded to Directors`;
      case 'broadcast:emergency':
      case 'broadcast:security': return `${p.subject ?? 'Alert'}: ${p.content ?? ''}`;
      default: return Object.entries(p).map(([k, v]) => `${k}: ${v}`).join(' | ') || 'No details';
    }
  }

  interface NavLink { label: string; href: string; exact?: boolean; }

  function navLinks(role: string | undefined): NavLink[] {
    if (!role) return [];

    // Administrator sees all links
    if (role === 'Administrator') {
      return [
        { label: 'Dashboard',         href: '/settlers/commander/dashboard' },
        { label: 'Assign Task',        href: '/settlers/commander/tasks/new' },
        { label: 'My Tasks',           href: '/settlers/tasks' },
        { label: 'Incoming Queue',     href: '/settlers/commander/queue' },
        { label: 'Cmdr Anomaly',       href: '/settlers/commander/anomaly' },
        { label: 'Abandonment',        href: '/settlers/commander/abandonment' },
        { label: 'Repatriation',       href: '/settlers/commander/repatriation' },
        { label: 'Cmdr Supply',        href: '/settlers/commander/supply' },
        { label: 'Manage Inventory',   href: '/settlers/commander/inventory' },
        { label: 'Progress Reports',   href: '/settlers/commander/progress-reports' },
        { label: 'Building Logs',      href: '/settlers/commander/building-logs' },
        { label: 'Farm Logs',          href: '/settlers/commander/farm-logs' },
        { label: 'Material Request',   href: '/settlers/engineer/materials' },
        { label: 'Construction Report',href: '/settlers/engineer/construction-report' },
        { label: 'Building Health',    href: '/settlers/engineer/building-health' },
        { label: 'Residence',          href: '/settlers/engineer/residence' },
        { label: 'Farm Supplies',      href: '/settlers/farmer/supplies' },
        { label: 'Farm Health Log',    href: '/settlers/farmer/health-check' },
        { label: 'Inventory',          href: '/settlers/inventory' },
        { label: 'Anomaly Report',     href: '/settlers/reports/anomaly' },
        { label: 'Settler Report',     href: '/settlers/reports/complaint' },
        { label: 'Data Request',       href: '/data/request/new', exact: true },
        { label: 'My Requests',        href: '/data/request/mine' },
        { label: 'Security Report',    href: '/settlers/security-report' },
        { label: 'My Profile',         href: '/me/profile' },
      ];
    }

    // TemporarySetter: ONLY these two
    if (role === 'TemporarySetter') {
      return [
        { label: 'Anomaly Report', href: '/settlers/reports/anomaly' },
        { label: 'Settler Report',  href: '/settlers/reports/complaint' },
        { label: 'My Profile',     href: '/me/profile' },
      ];
    }

    // Settler Commander
    if (role === 'SettlerCommander') {
      return [
        { label: 'Dashboard',         href: '/settlers/commander/dashboard' },
        { label: 'Assign Task',        href: '/settlers/commander/tasks/new' },
        { label: 'My Tasks',           href: '/settlers/tasks' },
        { label: 'Incoming Queue',     href: '/settlers/commander/queue' },
        { label: 'Cmdr Anomaly',       href: '/settlers/commander/anomaly' },
        { label: 'Abandonment',        href: '/settlers/commander/abandonment' },
        { label: 'Repatriation',       href: '/settlers/commander/repatriation' },
        { label: 'Cmdr Supply',        href: '/settlers/commander/supply' },
        { label: 'Manage Inventory',   href: '/settlers/commander/inventory' },
        { label: 'Progress Reports',   href: '/settlers/commander/progress-reports' },
        { label: 'Building Logs',      href: '/settlers/commander/building-logs' },
        { label: 'Farm Logs',          href: '/settlers/commander/farm-logs' },
        { label: 'Anomaly Report',     href: '/settlers/reports/anomaly' },
        { label: 'Settler Report',     href: '/settlers/reports/complaint' },
        { label: 'Data Request',       href: '/data/request/new', exact: true },
        { label: 'My Requests',        href: '/data/request/mine' },
        { label: 'Security Report',    href: '/settlers/security-report' },
        { label: 'My Profile',         href: '/me/profile' },
      ];
    }

    // Civil Engineer
    if (role === 'CivilEngineer') {
      return [
        { label: 'My Tasks',           href: '/settlers/tasks' },
        { label: 'Anomaly Report',     href: '/settlers/reports/anomaly' },
        { label: 'Settler Report',     href: '/settlers/reports/complaint' },
        { label: 'Material Request',   href: '/settlers/engineer/materials' },
        { label: 'Construction Report',href: '/settlers/engineer/construction-report' },
        { label: 'Building Health',    href: '/settlers/engineer/building-health' },
        { label: 'Residence',          href: '/settlers/engineer/residence' },
        { label: 'Inventory',          href: '/settlers/inventory' },
        { label: 'Data Request',       href: '/data/request/new', exact: true },
        { label: 'My Requests',        href: '/data/request/mine' },
        { label: 'Security Report',    href: '/settlers/security-report' },
        { label: 'My Profile',         href: '/me/profile' },
      ];
    }

    // Farmer
    if (role === 'Farmer') {
      return [
        { label: 'My Tasks',           href: '/settlers/tasks' },
        { label: 'Anomaly Report',     href: '/settlers/reports/anomaly' },
        { label: 'Settler Report',     href: '/settlers/reports/complaint' },
        { label: 'Farm Supplies',      href: '/settlers/farmer/supplies' },
        { label: 'Farm Health Log',    href: '/settlers/farmer/health-check' },
        { label: 'Inventory',          href: '/settlers/inventory' },
        { label: 'Data Request',       href: '/data/request/new', exact: true },
        { label: 'My Requests',        href: '/data/request/mine' },
        { label: 'Security Report',    href: '/settlers/security-report' },
        { label: 'My Profile',         href: '/me/profile' },
      ];
    }

    return [];
  }

  function isActive(link: NavLink, path: string): boolean {
    if (link.exact) return path === link.href;
    return path.startsWith(link.href);
  }

  async function handleLogout() { await logout(); goto('/auth'); }
</script>

<div class="shell">
  <header class="top-bar">
    <div class="brand">
      <img src="/Logo.png" alt="RUSA" class="logo" />
      <h1>Exoplanet Settlers</h1>
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
      {#if user?.role === 'Administrator'}
        <a href="/admin" class="back-link">← Dashboard</a>
      {/if}
      {#each navLinks(user?.role) as link}
        <a href={link.href} class:active={isActive(link, pathVal)}>
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
  .notif-btn { background:none;border:none;color:#F59E0B;font-size:0.75rem;cursor:pointer;min-width:20px; }
  .btn-sm { background:transparent;color:#94A3B8;border:1px solid #475569;border-radius:4px;padding:0.2rem 0.45rem;cursor:pointer;font-size:0.7rem; }
  .btn-sm:hover { color:#E6EDF3;border-color:#3ABEFF; }
  .notif-panel { background:#1F2937;border-bottom:1px solid rgba(58,190,255,0.1);max-height:200px;overflow-y:auto;padding:0.5rem 1rem; }
  .notif-item { display:flex;align-items:center;gap:0.5rem;padding:0.3rem 0;font-size:0.75rem;border-bottom:1px solid rgba(255,255,255,0.05); }
  .notif-type { color:#8B5CF6;font-weight:600;min-width:100px;white-space:nowrap; }
  .notif-text { flex:1;color:#94A3B8;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .notif-dismiss { background:none;border:none;color:#EF4444;cursor:pointer;font-size:1rem; }
  .body { display:flex;flex:1;overflow:hidden; }
  .side-nav { width:200px;min-width:160px;background:#111827;border-right:1px solid rgba(58,190,255,0.1);overflow-y:auto;padding:0.5rem; }
  .side-nav a { display:block;padding:0.55rem 0.75rem;margin-bottom:0.15rem;border-radius:6px;color:#94A3B8;text-decoration:none;font-size:0.8rem; }
  .side-nav a:hover { color:#E6EDF3;background:rgba(58,190,255,0.05); }
  .side-nav a.active { color:#3ABEFF;background:rgba(58,190,255,0.1); }
  .main-content { flex:1;overflow-y:auto;padding:1.25rem; }
  .back-link { display:block;padding:0.45rem 0.75rem;margin-bottom:0.4rem;border-radius:6px;color:#EF4444;text-decoration:none;font-size:0.75rem;border:1px solid rgba(239,68,68,0.2);background:rgba(239,68,68,0.05); }
  .back-link:hover { background:rgba(239,68,68,0.12);border-color:rgba(239,68,68,0.4); }
</style>
