<script lang="ts">
  import { DarkMode, NavBrand, Navbar } from 'flowbite-svelte';
  import { BuildingOutline, ArrowRightToBracketOutline } from 'flowbite-svelte-icons';
  import '../../app.css';
  import { getUser, logout, isLoggedIn } from '$lib/auth';

  interface Props {
    drawerHidden?: boolean;
  }

  let { drawerHidden = $bindable(false) }: Props = $props();

  const kullanici = getUser();
  const tarayici = typeof window !== 'undefined' && !('__TAURI_INTERNALS__' in window);
</script>

<Navbar
  class="bg-white px-4 py-2.5 text-gray-500 sm:px-4 dark:bg-gray-800 dark:text-gray-400"
  classes={{ navbarDivClass: 'flex flex-wrap justify-between items-center' }}
>
  <div class="flex items-center">
    <NavBrand href="/dashboard" class="ml-12 lg:ml-0">
      <BuildingOutline class="me-2 h-6 w-6 text-primary-700 dark:text-primary-400" />
      <span class="self-center text-lg font-semibold whitespace-nowrap dark:text-white">
        Kooperatif Yönetim
      </span>
    </NavBrand>
  </div>
  <div class="flex items-center gap-2 lg:order-2">
    {#if tarayici && kullanici}
      <span class="hidden text-sm text-gray-600 dark:text-gray-300 sm:block">
        {kullanici.ad}
      </span>
    {/if}
    <DarkMode class="text-lg" />
    {#if tarayici && isLoggedIn()}
      <button
        onclick={logout}
        title="Çıkış Yap"
        class="rounded-lg p-2 text-gray-500 hover:bg-gray-100 hover:text-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white"
      >
        <ArrowRightToBracketOutline class="h-5 w-5" />
      </button>
    {/if}
  </div>
</Navbar>
