<script lang="ts">
  import { afterNavigate } from '$app/navigation';
  import { Sidebar, SidebarGroup, SidebarItem, SidebarWrapper, SidebarButton, uiHelpers } from 'flowbite-svelte';
  import {
    LayersSolid,
    ChartPieOutline,
    WalletSolid,
    UsersSolid,
    CalendarMonthSolid,
    ChartLineUpOutline,
    ExclamationCircleOutline,
    UserSettingsSolid,
    ClipboardListSolid
  } from 'flowbite-svelte-icons';
  import { getCurrentUser } from '$lib/api-client';
  import { myPermissions } from '$lib/permissions';

  interface Props {
    drawerHidden: boolean;
  }
  let { drawerHidden = $bindable(false) }: Props = $props();
  const closeDrawer = () => {
    drawerHidden = true;
  };

  let iconClass = 'flex-shrink-0 w-6 h-6 text-gray-500 transition duration-75 group-hover:text-gray-900 dark:text-gray-300 dark:group-hover:text-white';
  let itemClass = 'flex items-center p-2 text-base text-gray-900 transition duration-75 rounded-lg hover:bg-gray-100 group dark:text-gray-200 dark:hover:bg-gray-700 w-full';
  let groupClass = 'pt-2 space-y-2 mb-3';

  const sidebarUi = uiHelpers();
  let isOpen = $state(false);
  const closeSidebar = sidebarUi.close;
  $effect(() => {
    isOpen = sidebarUi.isOpen;
  });

  afterNavigate(() => {
    document.getElementById('svelte')?.scrollTo({ top: 0 });
    closeDrawer();
  });

  const currentUser = getCurrentUser();
  const isAdmin = currentUser?.rol === 'admin';

  // Izin bazli filtreleme: admin her seyi gorur, diger roller izin anahtarina gore
  function izinVar(anahtar: string | null): boolean {
    if (!anahtar) return true; // izin gerektirmeyen menu (dashboard vs.)
    if (isAdmin) return true;
    return $myPermissions.has(anahtar);
  }

  type MenuItem = { name: string; Icon: any; href: string; izin: string | null };

  const tumMenuler: MenuItem[] = [
    { name: 'Dashboard', Icon: ChartPieOutline, href: '/dashboard', izin: null },
    { name: 'Kasa', Icon: WalletSolid, href: '/kasa', izin: 'kasa.goruntule' },
    { name: 'Gelir / Gider', Icon: ChartLineUpOutline, href: '/gelir-gider', izin: 'gelir_gider.goruntule' },
    { name: 'Borçlar', Icon: ExclamationCircleOutline, href: '/borclar', izin: 'borc.goruntule' },
    { name: 'Hissedar', Icon: UsersSolid, href: '/hissedar', izin: 'hissedar.goruntule' },
    { name: 'Dönem', Icon: CalendarMonthSolid, href: '/donem', izin: 'donem.goruntule' },
    { name: 'Hisse', Icon: LayersSolid, href: '/hisse', izin: 'hisse.goruntule' },
    { name: 'Kullanıcılar', Icon: UserSettingsSolid, href: '/kullanicilar', izin: 'kullanici.goruntule' },
    { name: 'Rol & İzinler', Icon: UserSettingsSolid, href: '/roller', izin: 'rol.yonet' },
    { name: 'Sistem Logları', Icon: ClipboardListSolid, href: '/loglar', izin: 'log.goruntule' }
  ];

  let posts = $derived(tumMenuler.filter((m) => izinVar(m.izin)));
</script>

<SidebarButton breakpoint="lg" onclick={sidebarUi.toggle} class="fixed top-[22px] z-40 mb-2" />
<Sidebar
  breakpoint="lg"
  backdrop={false}
  {isOpen}
  {closeSidebar}
  params={{ x: -50, duration: 50 }}
  class="top-0 left-0 mt-[69px] h-screen w-64 bg-gray-50 transition-transform lg:block dark:bg-gray-800"
  classes={{ div: 'h-full px-1 py-1 overflow-y-auto bg-gray-50 dark:bg-gray-800', nonactive: 'p-2', active: 'p-2' }}
>
  <h4 class="sr-only">Ana menü</h4>
  <SidebarWrapper class="scrolling-touch h-full max-w-2xs overflow-y-auto bg-white px-3 pt-20 lg:sticky lg:me-0 lg:block lg:h-[calc(100vh-4rem)] lg:pt-5 dark:bg-gray-800">
    <SidebarGroup class={groupClass}>
      {#each posts as { name, Icon, href } (name)}
        <SidebarItem label={name} {href} spanClass="ml-3" class={itemClass} aClass="w-full p-0 py-2">
          {#snippet icon()}
            <Icon class={iconClass} />
          {/snippet}
        </SidebarItem>
      {/each}
    </SidebarGroup>
  </SidebarWrapper>
</Sidebar>
