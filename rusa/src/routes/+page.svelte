<!--
  Dashboard / landing page — redirects logged-in users to their role-specific page.
  Falls through to a generic welcome panel if no role route is defined.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { currentUser, logout } from '$lib/stores/auth';
  import { getDefaultRoute } from '$lib/routing';
  import type { SessionUser } from '$lib/stores/auth';
  import { get } from 'svelte/store';

  let user: SessionUser | null = $state(null);

  currentUser.subscribe((value) => {
    user = value;
  });

  onMount(() => {
    const u = get(currentUser);
    if (u) {
      const target = getDefaultRoute(u.role);
      if (target && target !== '/dashboard') {
        goto(target);
        return;
      }
    }
  });

  async function handleLogout() {
    try {
      await logout();
      goto('/auth');
    } catch (e) {
      console.error(e);
    }
  }
</script>

<div class="dashboard-container">
  <header>
    <div class="logo-area">
      <img src="/Logo.png" alt="RUSA Logo" class="logo" />
      <h1>RUSA Terminal</h1>
    </div>
    
    {#if user}
      <div class="user-info">
        <div>
          <span style="color: #94A3B8; font-size: 0.85rem; margin-right: 0.5rem;">User:</span>
          <span>{user.full_name}</span>
        </div>
        <div class="role-badge">{user.role}</div>
        <button class="btn-logout" onclick={handleLogout}>Log Out</button>
      </div>
    {/if}
  </header>

  <main>
    <div class="welcome-panel">
      <h2 style="margin-top: 0; font-family: 'Orbitron', sans-serif;">Welcome to the RUSA Internal Desktop</h2>
      <p style="color: #94A3B8;">Secure connection established. Awaiting directives...</p>
    </div>
  </main>
</div>

<style>
  @import url('https://fonts.googleapis.com/css2?family=Orbitron:wght@500;700&family=Inter:wght@400;500;600&display=swap');

  :global(body) {
    margin: 0;
    font-family: 'Inter', sans-serif;
    background-color: #0B0F1A;
    color: #E6EDF3;
    overflow: hidden;
  }

  .dashboard-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background: 
      radial-gradient(circle at 30% 20%, rgba(139, 92, 246, 0.15), transparent 40%),
      radial-gradient(circle at 70% 80%, rgba(58, 190, 255, 0.15), transparent 40%),
      linear-gradient(to bottom right, #0E1428, #1A1F3A);
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 2rem;
    background-color: rgba(17, 24, 39, 0.8);
    border-bottom: 1px solid rgba(58, 190, 255, 0.2);
    backdrop-filter: blur(10px);
  }

  .logo-area {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .logo {
    width: 40px;
    height: auto;
    filter: drop-shadow(0 0 8px rgba(58, 190, 255, 0.4));
  }

  h1 {
    font-family: 'Orbitron', sans-serif;
    font-size: 1.5rem;
    margin: 0;
    background: linear-gradient(135deg, #3ABEFF 0%, #8B5CF6 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .user-info {
    display: flex;
    align-items: center;
    gap: 1.5rem;
  }

  .role-badge {
    background-color: rgba(139, 92, 246, 0.2);
    color: #C084FC;
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    font-size: 0.85rem;
    font-weight: 600;
    border: 1px solid rgba(139, 92, 246, 0.4);
  }

  main {
    flex: 1;
    padding: 2rem;
    overflow-y: auto;
  }

  .btn-logout {
    background: transparent;
    color: #EF4444;
    border: 1px solid #EF4444;
    border-radius: 6px;
    padding: 0.5rem 1rem;
    font-family: 'Inter', sans-serif;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.3s ease;
  }

  .btn-logout:hover {
    background: rgba(239, 68, 68, 0.1);
    box-shadow: 0 0 12px rgba(239, 68, 68, 0.4);
  }

  .welcome-panel {
    background-color: rgba(17, 24, 39, 0.6);
    border: 1px solid rgba(58, 190, 255, 0.2);
    border-radius: 8px;
    padding: 2rem;
    margin-bottom: 2rem;
  }
</style>