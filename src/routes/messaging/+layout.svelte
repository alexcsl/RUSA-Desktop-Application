<!--
  /messaging/+layout.svelte — Messaging layout with full app shell + channel sidebar
  Auth guard: Any authenticated user.
  Source of truth: 00_MASTER_GUIDE.md §6, AUTH_GUIDE.md
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { currentUser, logout } from '$lib/stores/auth';
  import { getDefaultRoute } from '$lib/routing';
  import { getUnreadCounts, type UnreadCount, CHANNEL_LABELS, type Channel, unreadCountsStore, refreshUnreadCounts } from '$lib/stores/messaging';
  import { getNotifications, markNotificationRead, type NotificationItem } from '$lib/stores/directors';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  let unreadCounts: UnreadCount[] = $state([]);
  let user = $state($currentUser);  let pathVal = $state('');
  page.subscribe((p) => (pathVal = p.url.pathname + p.url.search));

  // Subscribe to the shared unread counts store
  unreadCountsStore.subscribe((counts) => { unreadCounts = counts; });

  // Notifications (same pattern as admin/directors layouts)
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
          !shownEmergencyIds.has(n.id),
      );
      if (newEmergency) {
        shownEmergencyIds.add(newEmergency.id);
        emergencyAlert = newEmergency;
      }
    } catch {}
  }

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

  function isEmergencyNotif(n: NotificationItem): boolean {
    return n.type_ === 'broadcast:emergency' || n.type_ === 'broadcast:security';
  }

  function formatNotifType(type_: string): string {
    const map: Record<string, string> = {
      'vote:new': 'New Vote', 'vote:decided': 'Vote Decided',
      'broadcast:emergency': 'EMERGENCY', 'broadcast:security': 'SECURITY ALERT',
      'broadcast:informational': 'Broadcast', 'task:assigned': 'Task Assigned',
      'task:updated': 'Task Updated', 'event:invited': 'Event Invitation',
    };
    return map[type_] ?? type_.replace(/[_:]/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function formatNotifText(n: NotificationItem): string {
    const p = n.payload as Record<string, unknown>;
    if (n.type_ === 'broadcast:emergency' || n.type_ === 'broadcast:security') {
      return `${p.subject ?? 'Alert'}: ${p.content ?? ''}`;
    }
    return Object.entries(p).map(([k, v]) => `${k}: ${v}`).join(' | ') || 'No details';
  }

  // Channel helpers — only show channels that DON'T have dedicated pages
  // Security & Medical Heads have dedicated pages below the divider
  function getAccessibleChannels(): Channel[] {
    if (!user) return [];
    const channels: Channel[] = ['general'];
    channels.push('broadcast');
    return channels;
  }

  function unreadFor(channel: string): number {
    return unreadCounts.find((c) => c.channel === channel)?.count ?? 0;
  }

  function totalUnread(): number {
    return unreadCounts.reduce((sum, c) => sum + c.count, 0);
  }

  function isActive(href: string): boolean {
    return pathVal.startsWith(href) || pathVal === href;
  }

  function isChannelActive(ch: string): boolean {
    return $page.url.searchParams.get('channel') === ch ||
      $page.url.pathname.includes(ch);
  }

  function getBackRoute(): string {
    if (!user) return '/auth';
    if (user.role === 'Administrator') return '/admin/votes';
    if (user.role.startsWith('The') || user.role === 'GeneralDirector') return '/directors/votes';
    return getDefaultRoute(user.role);
  }

  onMount(async () => {
    if (!$currentUser) { goto('/auth'); return; }
    user = $currentUser;
    await refreshUnreadCounts();
    pollNotifications();
    pollTimer = setInterval(pollNotifications, 15_000);
  });

  onDestroy(() => { if (pollTimer) clearInterval(pollTimer); });

  currentUser.subscribe((u) => { user = u; });

  async function handleLogout() { await logout(); goto('/auth'); }
</script>

<div class="shell">
  <!-- ─── Top Bar ─── -->
  <header class="top-bar">
    <div class="brand">
      <img src="/Logo.png" alt="RUSA" class="logo" />
      <h1>Communications Hub</h1>
    </div>
    <div class="user-section">
      <a href={getBackRoute()} class="back-link">← Back to Dashboard</a>
      <button class="notif-btn" onclick={() => (showNotifs = !showNotifs)}>
        🔔 {notifications.filter((n) => !n.read_at).length || ''}
      </button>
      {#if user}
        <span class="user-name">{user.full_name}</span>
        <span class="role-tag">{user.role.replace(/([A-Z])/g, ' $1').trim()}</span>
        <button class="btn-sm" onclick={handleLogout}>Log Out</button>
      {/if}
    </div>
  </header>

  <!-- ─── Notification Panel ─── -->
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

  <!-- ─── Emergency Alert Modal ─── -->
  {#if emergencyAlert}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div class="emergency-overlay" role="dialog" aria-modal="true" tabindex="-1" onclick={dismissEmergency} onkeydown={(e) => { if (e.key === 'Escape') dismissEmergency(); }}>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div class="emergency-modal" onclick={(e) => e.stopPropagation()} onkeydown={() => {}}>
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

  <!-- ─── Body: Sidebar + Content ─── -->
  <div class="body">
    <aside class="channel-sidebar">
      <h2 class="sidebar-title">Channels</h2>
      <nav class="channel-nav">
        {#each getAccessibleChannels() as ch}
          <a
            href="/messaging/inbox?channel={ch}"
            class="channel-link"
            class:active={isChannelActive(ch)}
          >
            <span class="channel-name">{CHANNEL_LABELS[ch]}</span>
            {#if unreadFor(ch) > 0}
              <span class="unread-badge">{unreadFor(ch)}</span>
            {/if}
          </a>
        {/each}
      </nav>
      <hr class="divider" />
      <nav class="action-nav">
        <a href="/messaging/compose" class="compose-btn">+ Compose</a>
        <a href="/messaging/sent" class="action-link" class:active={pathVal.startsWith('/messaging/sent')}>Sent</a>
        <a href="/messaging/groups" class="action-link" class:active={pathVal.startsWith('/messaging/groups')}>Groups</a>
      </nav>

      {#if user && ['GalacticSecurityHead', 'GalacticSecurityStaff', 'TheGuardian', 'TheOverseer', 'Administrator'].includes(user.role)}
        <hr class="divider" />
        <a href="/messaging/channels/security" class="action-link sec-link" class:active={pathVal.includes('/channels/security')}>
          🔒 Security Line
        </a>
      {/if}
      {#if user && ['HeadOfMedicine', 'Administrator'].includes(user.role)}
        <a href="/messaging/channels/medical-heads" class="action-link med-link" class:active={pathVal.includes('/channels/medical-heads')}>
          🏥 Medical Heads
        </a>
      {/if}

      <!-- Unread summary -->
      {#if totalUnread() > 0}
        <div class="unread-summary">
          {totalUnread()} unread message{totalUnread() > 1 ? 's' : ''}
        </div>
      {/if}
    </aside>

    <main class="messaging-content">
      {@render children()}
    </main>
  </div>
</div>

<style>
  /* ─── Shell ─── */
  .shell { display:flex;flex-direction:column;height:100vh;background:#0B0F1A;color:#E6EDF3;font-family:'Inter',sans-serif; }

  /* ─── Top Bar ─── */
  .top-bar { display:flex;justify-content:space-between;align-items:center;padding:0.6rem 1.25rem;background:#111827;border-bottom:1px solid rgba(58,190,255,0.15); }
  .brand { display:flex;align-items:center;gap:0.6rem; }
  .brand h1 { font-family:'Orbitron',sans-serif;font-size:1rem;color:#3ABEFF;margin:0; }
  .logo { width:28px;height:28px; }
  .user-section { display:flex;align-items:center;gap:0.6rem; }
  .user-name { font-size:0.8rem; }
  .role-tag { background:rgba(139,92,246,0.2);color:#C084FC;padding:0.1rem 0.4rem;border-radius:4px;font-size:0.7rem; }
  .back-link { color:#94A3B8;font-size:0.75rem;text-decoration:none;padding:0.2rem 0.5rem;border:1px solid #475569;border-radius:4px;transition:color 0.15s,border-color 0.15s; }
  .back-link:hover { color:#3ABEFF;border-color:#3ABEFF; }
  .notif-btn { background:none;border:none;color:#F59E0B;font-size:0.75rem;cursor:pointer;min-width:20px; }
  .btn-sm { background:transparent;color:#94A3B8;border:1px solid #475569;border-radius:4px;padding:0.2rem 0.45rem;cursor:pointer;font-size:0.7rem; }
  .btn-sm:hover { color:#E6EDF3;border-color:#3ABEFF; }

  /* ─── Notifications ─── */
  .notif-panel { background:#1F2937;border-bottom:1px solid rgba(58,190,255,0.1);max-height:200px;overflow-y:auto;padding:0.5rem 1rem; }
  .notif-item { display:flex;align-items:center;gap:0.5rem;padding:0.3rem 0;font-size:0.75rem;border-bottom:1px solid rgba(255,255,255,0.05); }
  .notif-item.notif-emergency { background:rgba(239,68,68,0.1);border:1px solid rgba(239,68,68,0.3);border-radius:4px;padding:0.4rem 0.5rem;margin:0.15rem 0; }
  .notif-type { color:#8B5CF6;font-weight:600;min-width:100px;white-space:nowrap; }
  .notif-type.emergency-type { color:#EF4444;text-transform:uppercase;animation:pulse-red 2s ease-in-out infinite; }
  .notif-text { flex:1;color:#94A3B8;overflow:hidden;text-overflow:ellipsis;white-space:nowrap; }
  .notif-dismiss { background:none;border:none;color:#EF4444;cursor:pointer;font-size:1rem; }

  /* ─── Emergency Modal ─── */
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

  /* ─── Body ─── */
  .body { display:flex;flex:1;overflow:hidden; }

  /* ─── Channel Sidebar ─── */
  .channel-sidebar {
    width: 210px;
    min-width: 180px;
    background: #111827;
    border-right: 1px solid rgba(58, 190, 255, 0.1);
    padding: 0.75rem 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    overflow-y: auto;
  }

  .sidebar-title {
    font-family: 'Orbitron', sans-serif;
    font-size: 0.8rem;
    color: #3abeff;
    margin: 0 0 0.5rem;
    padding: 0 0.5rem;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  .channel-nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .channel-link {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.45rem 0.65rem;
    border-radius: 6px;
    color: #94a3b8;
    text-decoration: none;
    font-size: 0.8rem;
    transition: background 0.15s, color 0.15s;
  }

  .channel-link:hover { background: rgba(58, 190, 255, 0.08); color: #e6edf3; }
  .channel-link.active { background: rgba(58, 190, 255, 0.15); color: #3abeff; font-weight: 600; }

  .channel-name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .unread-badge {
    background: #3abeff;
    color: #0b0f1a;
    font-size: 0.65rem;
    font-weight: 700;
    padding: 0.05rem 0.35rem;
    border-radius: 10px;
    min-width: 1rem;
    text-align: center;
  }

  .divider { border: none; border-top: 1px solid rgba(58, 190, 255, 0.08); margin: 0.4rem 0; }

  .action-nav { display: flex; flex-direction: column; gap: 2px; }

  .compose-btn {
    display: block;
    text-align: center;
    padding: 0.5rem 0.65rem;
    background: linear-gradient(135deg, #3abeff 0%, #8b5cf6 100%);
    color: #0b0f1a;
    font-weight: 700;
    font-size: 0.8rem;
    border-radius: 6px;
    text-decoration: none;
    margin-bottom: 0.15rem;
    transition: box-shadow 0.15s;
  }

  .compose-btn:hover { box-shadow: 0 0 12px rgba(58, 190, 255, 0.6); }

  .action-link {
    display: block;
    padding: 0.45rem 0.65rem;
    color: #94a3b8;
    text-decoration: none;
    font-size: 0.8rem;
    border-radius: 6px;
    transition: background 0.15s, color 0.15s;
  }

  .action-link:hover { background: rgba(58, 190, 255, 0.08); color: #e6edf3; }
  .action-link.active { background: rgba(58, 190, 255, 0.12); color: #3abeff; }

  .sec-link.active { background: rgba(245, 158, 11, 0.12); color: #f59e0b; }
  .med-link.active { background: rgba(16, 185, 129, 0.12); color: #10b981; }

  .unread-summary {
    margin-top: auto;
    padding: 0.45rem 0.65rem;
    font-size: 0.7rem;
    color: #3abeff;
    background: rgba(58, 190, 255, 0.06);
    border-radius: 6px;
    text-align: center;
  }

  /* ─── Main Content ─── */
  .messaging-content { flex: 1; overflow-y: auto; padding: 1.25rem; }
</style>
