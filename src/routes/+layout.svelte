<!--
  +layout.svelte — Root layout with session initialization and real-time setup
  Source of truth: AUTH_GUIDE.md §6.4 + 00_MASTER_GUIDE.md §3 (Real-Time Flow)

  Real-time event subscription (Event Pattern + Observer Pattern):
    Rust emits → Tauri event bus → listen() here → Svelte stores updated
  Events:
    "notification:new" — a notification was inserted; refresh notification count
    "vote:updated"     — a vote session changed; Directors refresh their queue
    "message:new"      — a new message arrived; refresh unread message counts
-->
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/core';
  import { initSession, currentUser } from '$lib/stores/auth';
  import { getDefaultRoute } from '$lib/routing';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { unreadCountsStore } from '$lib/stores/messaging';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  let loading = $state(true);

  // Public routes that do NOT require authentication
  const PUBLIC_ROUTES = ['/auth', '/forbidden', '/station/map'];

  // Store unlisteners at component scope so onDestroy can access them.
  const unlisteners: Array<() => void> = [];

  onMount(async () => {
    await initSession();

    const currentPath = $page.url.pathname;
    const user = $currentUser;
    const isPublic = PUBLIC_ROUTES.some((r) => currentPath.startsWith(r));

    if (user && currentPath === '/auth') {
      // Already logged in, redirect away from login to role page
      goto(getDefaultRoute(user.role));
    } else if (user && currentPath === '/') {
      // Landing on root — redirect to role-specific page
      goto(getDefaultRoute(user.role));
    } else if (!user && !isPublic) {
      // Not authenticated, redirect to login
      goto('/auth');
    }

    loading = false;

    // ── Real-time event listeners (Event Pattern + Observer Pattern) ──────────
    // These are set up after session init so we know whether a user is logged in.
    // Tauri events are emitted by the Rust realtime.rs background task whenever
    // the Supabase Realtime WebSocket fires a postgres_changes event.

    // notification:new — a new notification was inserted for any user.
    // We let the individual notification component query its own data on trigger.
    const unlistenNotification = await listen<Record<string, unknown>>(
      'notification:new',
      () => {
        if ($currentUser) {
          // Signal other components that notifications changed.
          // They can react by calling get_notifications themselves.
          window.dispatchEvent(new CustomEvent('rusa:notification-changed'));
        }
      }
    );
    unlisteners.push(unlistenNotification);

    // vote:updated — a vote session was created or changed.
    // Directors' voting pages listen for this to refresh their queue without polling.
    const unlistenVote = await listen<Record<string, unknown>>(
      'vote:updated',
      () => {
        if ($currentUser) {
          window.dispatchEvent(new CustomEvent('rusa:vote-changed'));
        }
      }
    );
    unlisteners.push(unlistenVote);

    // message:new — a new message was inserted.
    // Refresh unread counts in all messaging views.
    const unlistenMessage = await listen<Record<string, unknown>>(
      'message:new',
      async () => {
        if ($currentUser) {
          try {
            const counts = await invoke<Array<{ channel: string; count: number }>>(
              'get_unread_counts'
            );
            unreadCountsStore.set(counts);
          } catch {
            // Non-critical: unread badge will refresh on next manual load
          }
        }
      }
    );
    unlisteners.push(unlistenMessage);
  });

  // Cleanup: remove all Tauri event listeners when this layout is destroyed.
  onDestroy(() => {
    for (const unlisten of unlisteners) {
      unlisten();
    }
  });
</script>

{#if loading}
  <div class="loader">Establishing Uplink...</div>
{:else}
  {@render children()}
{/if}

<style>
  .loader {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100vh;
    background-color: #0B0F1A;
    color: #3ABEFF;
    font-family: 'Orbitron', sans-serif;
    font-size: 1.2rem;
  }
</style>
