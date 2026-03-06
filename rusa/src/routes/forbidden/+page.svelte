<!--
  Forbidden page — shown when a user tries to access a route they don't have permission for.
-->
<script lang="ts">
  import { goto } from '$app/navigation';

  function goBack() {
    goto('/');
  }
</script>

<style>
  @import url('https://fonts.googleapis.com/css2?family=Orbitron:wght@500;700&family=Inter:wght@400;500;600&display=swap');

  .forbidden-container {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100vh;
    width: 100vw;
    background:
      radial-gradient(circle at 30% 20%, rgba(239, 68, 68, 0.1), transparent 40%),
      radial-gradient(circle at 70% 80%, rgba(58, 190, 255, 0.1), transparent 40%),
      linear-gradient(to bottom right, #0E1428, #1A1F3A);
    font-family: 'Inter', sans-serif;
    color: #E6EDF3;
  }

  .panel {
    text-align: center;
    background-color: rgba(17, 24, 39, 0.8);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 12px;
    padding: 3rem 2.5rem;
    max-width: 420px;
  }

  h1 {
    font-family: 'Orbitron', sans-serif;
    font-size: 3rem;
    color: #EF4444;
    margin: 0 0 0.5rem 0;
  }

  h2 {
    font-family: 'Orbitron', sans-serif;
    font-size: 1.2rem;
    color: #EF4444;
    margin: 0 0 1rem 0;
  }

  p {
    color: #94A3B8;
    margin-bottom: 2rem;
  }

  .btn-back {
    background: transparent;
    color: #3ABEFF;
    border: 1px solid #3ABEFF;
    border-radius: 6px;
    padding: 0.6rem 1.5rem;
    font-family: 'Inter', sans-serif;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.3s ease;
  }

  .btn-back:hover {
    background: rgba(58, 190, 255, 0.1);
    box-shadow: 0 0 12px rgba(58, 190, 255, 0.4);
  }
</style>

<div class="forbidden-container">
  <div class="panel">
    <h1>403</h1>
    <h2>Access Denied</h2>
    <p>Your clearance level does not authorize access to this sector.</p>
    <button class="btn-back" onclick={goBack}>Return to Command Center</button>
  </div>
</div>
