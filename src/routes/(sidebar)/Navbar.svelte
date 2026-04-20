<script lang="ts">
  import { DarkMode, NavBrand, Navbar } from 'flowbite-svelte';
  import { BuildingOutline } from 'flowbite-svelte-icons';
  import '../../app.css';

  interface Props {
    drawerHidden?: boolean;
  }

  let { drawerHidden = $bindable(false) }: Props = $props();
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
  <div class="flex items-center lg:order-2">
    <DarkMode class="text-lg" />
  </div>
</Navbar>
