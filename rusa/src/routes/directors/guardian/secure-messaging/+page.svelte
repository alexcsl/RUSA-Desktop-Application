<!--
  /directors/guardian/secure-messaging — DEPRECATED: Redirects to Communications Hub
  The Guardian's secure messaging is now unified under /messaging/channels/security.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  onMount(() => { goto('/messaging/channels/security'); });
</script>
<p style="color:#94A3B8;padding:2rem;">Redirecting to Communications Hub…</p>
