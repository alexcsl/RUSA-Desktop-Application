<!--
  +layout.svelte — Root layout with session initialization
  Source of truth: AUTH_GUIDE.md §6.4
  Session lives in Rust AppState. This merely mirrors it for UI routing.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { initSession, currentUser } from '$lib/stores/auth';
  import { getDefaultRoute } from '$lib/routing';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet } = $props();

  let loading = $state(true);

  // Public routes that do NOT require authentication
  const PUBLIC_ROUTES = ['/auth', '/forbidden', '/station/map'];

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
