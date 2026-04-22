<script lang="ts">
  import '../../app.css';
  import Navbar from './Navbar.svelte';
  import Sidebar from './Sidebar.svelte';
  import type { LayoutProps } from './$types';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { isLoggedIn } from '$lib/auth';

  let { children }: LayoutProps = $props();

  let drawerHidden = $state(false);
  let hazir = $state(false);

  // Browser'da (Tauri dışında) oturum kontrolü yap
  function isTauriEnv(): boolean {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  }

  onMount(() => {
    if (!isTauriEnv() && !isLoggedIn()) {
      goto('/login');
    } else {
      hazir = true;
    }
  });
</script>

{#if hazir || isTauriEnv()}
  <header class="fixed top-0 z-40 mx-auto w-full flex-none border-b border-gray-200 bg-white dark:border-gray-600 dark:bg-gray-800">
    <Navbar bind:drawerHidden />
  </header>
  <div class="overflow-hidden lg:flex">
    <Sidebar bind:drawerHidden />
    <div class="relative h-full w-full overflow-y-auto pt-[70px] lg:ml-64">
      {@render children()}
    </div>
  </div>
{/if}
