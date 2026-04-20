<script lang="ts">
  import {
    DarkMode,
    NavBrand,
    Navbar,
    Avatar,
    Dropdown,
    DropdownHeader,
    DropdownItem,
    DropdownDivider
  } from 'flowbite-svelte';
  import {
    BuildingOutline,
    ArrowRightToBracketOutline,
    UserSettingsSolid,
    UserCircleSolid
  } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import { getCurrentUser, logout } from '$lib/api-client';
  import '../../app.css';

  interface Props {
    drawerHidden?: boolean;
  }

  let { drawerHidden = $bindable(false) }: Props = $props();

  const user = getCurrentUser();
  const isAdmin = user?.rol === 'admin';

  const rolEtiket: Record<string, string> = {
    admin: 'Yönetici',
    muhasebe: 'Muhasebe',
    uye: 'Üye',
    izleyici: 'İzleyici'
  };

  let dropdownOpen = $state(false);

  function cikisYap() {
    dropdownOpen = false;
    logout();
  }
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
    <DarkMode class="text-lg" />

    {#if user}
      <button
        type="button"
        class="ms-2 flex items-center gap-2 rounded-full p-1 ring-gray-200 hover:ring-2 focus:ring-2 focus:outline-none dark:ring-gray-600"
        aria-label="Kullanıcı menüsü"
      >
        <Avatar size="sm" class="bg-primary-100 text-primary-700">
          <UserCircleSolid class="h-6 w-6" />
        </Avatar>
        <span class="hidden pr-2 text-sm font-medium text-gray-700 md:inline dark:text-gray-200">
          {user.email}
        </span>
      </button>
      <Dropdown bind:open={dropdownOpen} placement="bottom-end" simple>
        <DropdownHeader>
          <span class="block text-sm font-semibold">{user.email}</span>
          <span class="block truncate text-xs text-gray-500 dark:text-gray-400">
            {rolEtiket[user.rol] ?? user.rol}
          </span>
        </DropdownHeader>
        {#if isAdmin}
          <DropdownItem onclick={() => { dropdownOpen = false; goto('/kullanicilar'); }}>
            <div class="flex items-center gap-2">
              <UserSettingsSolid class="h-4 w-4" />
              Kullanıcı Yönetimi
            </div>
          </DropdownItem>
          <DropdownItem onclick={() => { dropdownOpen = false; goto('/roller'); }}>
            <div class="flex items-center gap-2">
              <UserSettingsSolid class="h-4 w-4" />
              Rol ve İzinler
            </div>
          </DropdownItem>
          <DropdownDivider />
        {/if}
        <DropdownItem onclick={cikisYap}>
          <div class="flex items-center gap-2 text-red-600 dark:text-red-400">
            <ArrowRightToBracketOutline class="h-4 w-4" />
            Çıkış Yap
          </div>
        </DropdownItem>
      </Dropdown>
    {/if}
  </div>
</Navbar>
