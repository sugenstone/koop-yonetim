<script lang="ts">
  import '../../app.css';
  import { onMount } from 'svelte';
  import Navbar from './Navbar.svelte';
  import Sidebar from './Sidebar.svelte';
  import type { LayoutProps } from './$types';
  import { loadMyPermissions } from '$lib/permissions';

  let { children }: LayoutProps = $props();

  let drawerHidden = $state(false);

  // Sayfa yenilenmesinde izinleri arka planda tazele (cache anlik olarak hazir olur)
  onMount(() => {
    loadMyPermissions();
  });
</script>

<header class="fixed top-0 z-40 mx-auto w-full flex-none border-b border-gray-200 bg-white dark:border-gray-600 dark:bg-gray-800">
  <Navbar bind:drawerHidden />
</header>
<div class="overflow-hidden lg:flex">
  <Sidebar bind:drawerHidden />
  <div class="relative h-full w-full overflow-y-auto pt-[70px] lg:ml-64">
    {@render children()}
  </div>
</div>
