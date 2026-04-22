<script lang="ts">
  import {
    Badge,
    Spinner,
    Heading,
    P,
    Select
  } from 'flowbite-svelte';
  import { ClipboardListSolid } from 'flowbite-svelte-icons';
  import { logApi, type AuditLog } from '$lib/tauri-api';
  import { getCurrentUser } from '$lib/api-client';
  import DataTable from '$lib/components/DataTable.svelte';
  import type { DataTableColumn } from '$lib/components/dataTableUtils';

  // ─── Yetki ──────────────────────────────────────────────────────────────────

  const currentUser = getCurrentUser();
  const isAdmin = currentUser?.rol === 'admin';

  // ─── State ──────────────────────────────────────────────────────────────────

  let loglar = $state<AuditLog[]>([]);
  let yukleniyor = $state(true);
  let hata = $state('');
  let limitSec = $state('200');

  const limitSecenekleri = [
    { value: '50', name: 'Son 50 kayıt' },
    { value: '100', name: 'Son 100 kayıt' },
    { value: '200', name: 'Son 200 kayıt' },
    { value: '500', name: 'Son 500 kayıt' },
    { value: '1000', name: 'Son 1000 kayıt' },
  ];

  // ─── Yükle ──────────────────────────────────────────────────────────────────

  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      loglar = await logApi.getAll(parseInt(limitSec));
    } catch (e: any) {
      hata = e?.message ?? 'Loglar yüklenemedi';
    } finally {
      yukleniyor = false;
    }
  }

  $effect(() => {
    limitSec;
    yukle();
  });

  // ─── Yardımcılar ────────────────────────────────────────────────────────────

  function durumRengi(kod: number): 'green' | 'yellow' | 'red' | 'dark' {
    if (kod >= 500) return 'red';
    if (kod >= 400) return 'yellow';
    if (kod >= 200 && kod < 300) return 'green';
    return 'dark';
  }

  function yontemRengi(y: string): string {
    const renkler: Record<string, string> = {
      GET: 'text-blue-600 dark:text-blue-400',
      POST: 'text-green-600 dark:text-green-400',
      PUT: 'text-yellow-600 dark:text-yellow-400',
      PATCH: 'text-orange-600 dark:text-orange-400',
      DELETE: 'text-red-600 dark:text-red-400',
    };
    return renkler[y] ?? 'text-gray-600 dark:text-gray-400';
  }

  function tarihFormat(t: string): string {
    const d = new Date(t);
    return d.toLocaleDateString('tr-TR') + ' ' + d.toLocaleTimeString('tr-TR', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  }

  // ─── Kolonlar ───────────────────────────────────────────────────────────────

  const kolonlar: DataTableColumn<AuditLog>[] = [
    { id: 'id', header: '#', accessor: 'id', align: 'left', hiddenByDefault: true },
    { id: 'tarih', header: 'Tarih', accessor: (l) => tarihFormat(l.tarih) },
    { id: 'kullanici', header: 'Kullanıcı', accessor: (l) => l.kullanici_email ?? '-' },
    { id: 'rol', header: 'Rol', accessor: (l) => l.rol ?? '-' },
    { id: 'yontem', header: 'Yöntem', accessor: 'yontem', align: 'center' },
    { id: 'yol', header: 'Endpoint', accessor: 'yol' },
    { id: 'durum', header: 'Durum', accessor: (l) => String(l.durum_kodu), align: 'center' },
    { id: 'sure', header: 'Süre (ms)', accessor: 'sure_ms', align: 'right' },
    { id: 'ip', header: 'IP', accessor: (l) => l.ip ?? '-' },
    { id: 'hata', header: 'Hata', accessor: (l) => l.hata ?? '', hiddenByDefault: true },
  ];
</script>

<main class="min-h-screen bg-gray-50 p-6 dark:bg-gray-900">
  <!-- Başlık -->
  <div class="mb-6 flex items-center justify-between">
    <div>
      <Heading tag="h1" class="text-2xl font-bold text-gray-900 dark:text-white">
        <ClipboardListSolid class="mr-2 inline h-6 w-6" />
        Sistem Logları
      </Heading>
      <P class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Toplam {loglar.length} kayıt
      </P>
    </div>
    <div class="w-44">
      <Select
        items={limitSecenekleri}
        bind:value={limitSec}
        class="text-sm"
      />
    </div>
  </div>

  {#if !isAdmin}
    <div class="mb-4 rounded-lg bg-yellow-50 p-4 text-sm text-yellow-700 dark:bg-yellow-900/20 dark:text-yellow-400">
      Bu sayfayı görüntülemek için yönetici yetkisi gereklidir.
    </div>
  {/if}

  <!-- Hata -->
  {#if hata}
    <div class="mb-4 rounded-lg bg-red-50 p-4 text-sm text-red-700 dark:bg-red-900/20 dark:text-red-400">
      {hata}
    </div>
  {/if}

  <!-- Tablo -->
  {#if yukleniyor}
    <div class="flex h-48 items-center justify-center">
      <Spinner size="10" />
    </div>
  {:else}
    <DataTable
      data={loglar}
      columns={kolonlar}
      searchPlaceholder="Kullanıcı, endpoint, IP ara..."
      exportFileName="loglar"
      emptyMessage="Log kaydı bulunamadı"
    >
      {#snippet row(l, _i, visibleCols)}
        <tr class="border-b bg-white hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:hover:bg-gray-700">
          {#if visibleCols.has('id')}
            <td class="px-4 py-2 text-xs text-gray-400">{l.id}</td>
          {/if}
          {#if visibleCols.has('tarih')}
            <td class="whitespace-nowrap px-4 py-2 font-mono text-xs text-gray-600 dark:text-gray-400">
              {tarihFormat(l.tarih)}
            </td>
          {/if}
          {#if visibleCols.has('kullanici')}
            <td class="px-4 py-2 text-sm">
              {#if l.kullanici_email}
                <span class="font-medium text-gray-900 dark:text-white">{l.kullanici_email}</span>
              {:else}
                <span class="text-gray-400">-</span>
              {/if}
            </td>
          {/if}
          {#if visibleCols.has('rol')}
            <td class="px-4 py-2 text-sm text-gray-600 dark:text-gray-400">
              {l.rol ?? '-'}
            </td>
          {/if}
          {#if visibleCols.has('yontem')}
            <td class="px-4 py-2 text-center">
              <span class="font-mono text-xs font-bold {yontemRengi(l.yontem)}">
                {l.yontem}
              </span>
            </td>
          {/if}
          {#if visibleCols.has('yol')}
            <td class="max-w-xs truncate px-4 py-2 font-mono text-xs text-gray-700 dark:text-gray-300" title={l.yol}>
              {l.yol}
            </td>
          {/if}
          {#if visibleCols.has('durum')}
            <td class="px-4 py-2 text-center">
              <Badge color={durumRengi(l.durum_kodu)} class="font-mono text-xs">
                {l.durum_kodu}
              </Badge>
            </td>
          {/if}
          {#if visibleCols.has('sure')}
            <td class="px-4 py-2 text-right font-mono text-xs text-gray-600 dark:text-gray-400">
              {l.sure_ms}ms
            </td>
          {/if}
          {#if visibleCols.has('ip')}
            <td class="px-4 py-2 font-mono text-xs text-gray-500 dark:text-gray-400">
              {l.ip ?? '-'}
            </td>
          {/if}
          {#if visibleCols.has('hata')}
            <td class="max-w-xs truncate px-4 py-2 text-xs text-red-600 dark:text-red-400" title={l.hata ?? ''}>
              {l.hata ?? ''}
            </td>
          {/if}
        </tr>
      {/snippet}
    </DataTable>
  {/if}
</main>
