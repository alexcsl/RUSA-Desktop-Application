<!--
  RouteGuard.svelte — Frontend route protection component
  Source of truth: AUTH_GUIDE.md §6.2

  This is UX polish only. The real security boundary is the Rust backend guard.
  Never rely on this component alone for access control.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { get } from 'svelte/store';
  import { currentUser } from '$lib/stores/auth';

  /** The role (or roles) required for this route. Pass empty array to allow any authenticated user. */
  interface Props {
    requiredRoles?: string[];
    children?: import('svelte').Snippet;
  }

  let { requiredRoles = [], children }: Props = $props();

  let authorized = $state(false);

  onMount(() => {
    const user = get(currentUser);

    if (!user) {
      goto('/auth');
      return;
    }

    // Administrator passes all role checks
    if (user.role === 'Administrator') {
      authorized = true;
      return;
    }

    // No role restriction: any authenticated user passes
    if (requiredRoles.length === 0) {
      authorized = true;
      return;
    }

    if (requiredRoles.includes(user.role)) {
      authorized = true;
    } else {
      goto('/forbidden');
    }
  });
</script>

{#if authorized}
  {@render children?.()}
{/if}
