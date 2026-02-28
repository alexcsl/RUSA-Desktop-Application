<!--
  Login page — Source of truth: AUTH_GUIDE.md §6.3
  All auth goes through Rust Tauri commands — session stored in Tauri AppState.
-->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { login, currentUser } from '$lib/stores/auth';
  import { getDefaultRoute } from '$lib/routing';
  import { get } from 'svelte/store';

  let username = $state('');
  let password = $state('');
  let errorMessage = $state('');
  let loading = $state(false);

  async function handleLogin(e: SubmitEvent) {
    e.preventDefault();
    errorMessage = '';
    loading = true;
    try {
      await login(username, password);
      // Route to the role-specific landing page
      const user = get(currentUser);
      const target = user ? getDefaultRoute(user.role) : '/';
      goto(target);
    } catch (err: unknown) {
      // Display the user-facing message from AppError — never the raw error
      errorMessage =
        typeof err === 'string'
          ? err
          : (err as { message?: string })?.message ??
            'Login failed. Please try again.';
    } finally {
      loading = false;
    }
  }
</script>

<style>
  @import url('https://fonts.googleapis.com/css2?family=Orbitron:wght@500;700&family=Inter:wght@400;500;600&display=swap');

  :global(body) {
    margin: 0;
    font-family: 'Inter', sans-serif;
    background-color: #0B0F1A;
    color: #E6EDF3;
    overflow: hidden;
  }

  .auth-container {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100vh;
    width: 100vw;
    background: 
      radial-gradient(circle at 30% 20%, rgba(139, 92, 246, 0.15), transparent 40%),
      radial-gradient(circle at 70% 80%, rgba(58, 190, 255, 0.15), transparent 40%),
      linear-gradient(to bottom right, #0E1428, #1A1F3A);
  }

  .login-panel {
    background-color: #111827;
    border: 1px solid rgba(139, 92, 246, 0.3);
    border-radius: 12px;
    padding: 3rem 2.5rem;
    width: 100%;
    max-width: 400px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(10px);
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .header-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .logo {
    width: 80px;
    height: auto;
    filter: drop-shadow(0 0 8px rgba(58, 190, 255, 0.4));
  }

  h1 {
    font-family: 'Orbitron', sans-serif;
    font-size: 1.8rem;
    text-align: center;
    margin: 0 0 0.5rem 0;
    background: linear-gradient(135deg, #3ABEFF 0%, #8B5CF6 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  p.subtitle {
    text-align: center;
    color: #94A3B8;
    margin-top: -1.5rem;
    margin-bottom: 1.5rem;
    font-size: 0.9rem;
  }

  .input-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  label {
    font-size: 0.85rem;
    color: #94A3B8;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  input {
    background-color: rgba(11, 15, 26, 0.8);
    border: 1px solid rgba(58, 190, 255, 0.2);
    border-radius: 6px;
    padding: 0.75rem 1rem;
    color: #E6EDF3;
    font-family: 'Inter', sans-serif;
    font-size: 1rem;
    transition: all 0.2s ease;
    outline: none;
  }

  input:focus {
    border-color: #3ABEFF;
    box-shadow: 0 0 12px rgba(58, 190, 255, 0.6);
  }

  .btn-primary {
    background: linear-gradient(135deg, #3ABEFF 0%, #8B5CF6 100%);
    color: #ffffff;
    border: none;
    border-radius: 6px;
    padding: 0.85rem;
    font-family: 'Orbitron', sans-serif;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.3s ease;
    margin-top: 1rem;
    box-shadow: 0 0 12px rgba(58, 190, 255, 0.4);
  }

  .btn-primary:hover:not(:disabled) {
    background: linear-gradient(135deg, #1DA1F2 0%, #C084FC 100%);
    box-shadow: 0 0 16px rgba(58, 190, 255, 0.8);
    transform: translateY(-1px);
  }

  .btn-primary:active:not(:disabled) {
    transform: translateY(1px);
  }

  .btn-primary:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }

  .error-message {
    color: #EF4444;
    font-size: 0.85rem;
    text-align: center;
    background-color: rgba(239, 68, 68, 0.1);
    padding: 0.5rem;
    border-radius: 4px;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }
</style>

<div class="auth-container">
  <div class="login-panel">
    <div class="header-container">
      <img src="/Logo.png" alt="RUSA Logo" class="logo" />
      <h1>RUSA</h1>
    </div>
    <p class="subtitle">Secure Internal Desktop</p>

    {#if errorMessage}
      <div class="error-message">{errorMessage}</div>
    {/if}

    <form onsubmit={handleLogin} style="display: flex; flex-direction: column; gap: 1.25rem;">
      <div class="input-group">
        <label for="username">Username</label>
        <input
          type="text"
          id="username"
          bind:value={username}
          placeholder="agent.callsign"
          required
        />
      </div>

      <div class="input-group">
        <label for="password">Access Phrase</label>
        <input
          type="password"
          id="password"
          bind:value={password}
          placeholder="••••••••"
          required
        />
      </div>

      <button type="submit" class="btn-primary" disabled={loading}>
        {loading ? 'Authenticating...' : 'Engage Uplink'}
      </button>
    </form>
  </div>
</div>
