<script lang="ts">
  import {
    Button,
    Badge,
    Spinner,
    Heading,
    P,
    Select,
    Input,
    TableBodyRow,
    TableBodyCell
  } from 'flowbite-svelte';
  import DataTable from '$lib/components/DataTable.svelte';
  import type { DataTableColumn } from '$lib/components/dataTableUtils';
  import { logApi, type IslemLog, type LogOzet, type LogFiltre } from '$lib/tauri-api';
  import { notify } from '$lib/toast';

  let loglar = $state<IslemLog[]>([]);
  let ozet = $state<LogOzet | null>(null);
  let yukleniyor = $state(true);
  let hata = $state('');

  // Filtre state
  let fYontem = $state<string>('');
  let fMinDurum = $state<string>(''); // '', '400', '500'
  let fQ = $state<string>('');
  let fLimit = $state<number>(100);

  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      const filtre: LogFiltre = { limit: fLimit };
      if (fYontem) filtre.yontem = fYontem;
      if (fMinDurum) filtre.min_durum = Number(fMinDurum);
      if (fQ.trim()) filtre.q = fQ.trim();
      [loglar, ozet] = await Promise.all([logApi.getAll(filtre), logApi.ozet()]);
    } catch (e: any) {
      hata = e?.message ?? 'Yükleme hatası';
      notify.apiError(e, 'Loglar yüklenemedi');
    } finally {
      yukleniyor = false;
    }
  }

  $effect(() => { yukle(); });

  function tarihFormat(t: string): string {
    const d = new Date(t);
    return d.toLocaleString('tr-TR', {
      day: '2-digit', month: '2-digit', year: 'numeric',
      hour: '2-digit', minute: '2-digit', second: '2-digit'
    });
  }

  function durumColor(k: number): 'green' | 'yellow' | 'red' | 'primary' {
    if (k >= 500) return 'red';
    if (k >= 400) return 'yellow';
    if (k >= 300) return 'primary';
    return 'green';
  }

  function yontemColor(y: string): 'green' | 'yellow' | 'red' | 'primary' | 'dark' {
    switch (y) {
      case 'POST': return 'green';
      case 'PUT': case 'PATCH': return 'yellow';
      case 'DELETE': return 'red';
      default: return 'dark';
    }
  }

  const kolonlar: DataTableColumn<IslemLog>[] = [
    { id: 'tarih', header: 'Tarih', accessor: 'tarih' },
    { id: 'kullanici', header: 'Kullanıcı', accessor: 'kullanici_email' },
    { id: 'rol', header: 'Rol', accessor: 'rol' },
    { id: 'yontem', header: 'Yöntem', accessor: 'yontem' },
    { id: 'yol', header: 'Endpoint', accessor: 'yol' },
    { id: 'durum', header: 'Durum', accessor: 'durum_kodu', align: 'right' },
    { id: 'sure', header: 'Süre (ms)', accessor: 'sure_ms', align: 'right' },
    { id: 'ua', header: 'User-Agent', accessor: 'user_agent' }
  ];
</script>

<main class="min-h-screen bg-gray-50 p-6 dark:bg-gray-900">
  <div class="mb-6">
    <Heading tag="h1" class="text-2xl font-bold">İşlem Logları</Heading>
    <P class="text-sm text-gray-500 dark:text-gray-400">
      Tüm POST / PUT / PATCH / DELETE istekleri otomatik olarak kaydedilir.
    </P>
  </div>

  {#if ozet}
    <div class="mb-6 grid grid-cols-1 gap-4 sm:grid-cols-3">
      <div class="rounded-xl border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800">
        <p class="text-sm text-gray-500 dark:text-gray-400">Toplam</p>
        <p class="text-xl font-bold">{ozet.toplam.toLocaleString('tr-TR')}</p>
      </div>
      <div class="rounded-xl border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800">
        <p class="text-sm text-gray-500 dark:text-gray-400">Son 24 saat</p>
        <p class="text-xl font-bold">{ozet.son_24_saat.toLocaleString('tr-TR')}</p>
      </div>
      <div class="rounded-xl border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800">
        <p class="text-sm text-gray-500 dark:text-gray-400">Son 24s hata (≥400)</p>
        <p class="text-xl font-bold {ozet.hata_24_saat > 0 ? 'text-red-600 dark:text-red-400' : ''}">
          {ozet.hata_24_saat.toLocaleString('tr-TR')}
        </p>
      </div>
    </div>
  {/if}

  {#if hata}
    <div class="mb-4 rounded-lg bg-red-50 p-4 text-sm text-red-700 dark:bg-red-900/20 dark:text-red-400">{hata}</div>
  {/if}

  <!-- Filtreler -->
  <div class="mb-4 flex flex-wrap items-end gap-3">
    <div>
      <label for="fYontem" class="mb-1 block text-xs text-gray-500 dark:text-gray-400">Yöntem</label>
      <Select id="fYontem" bind:value={fYontem} class="w-32">
        <option value="">Tümü</option>
        <option value="POST">POST</option>
        <option value="PUT">PUT</option>
        <option value="PATCH">PATCH</option>
        <option value="DELETE">DELETE</option>
      </Select>
    </div>
    <div>
      <label for="fMinDurum" class="mb-1 block text-xs text-gray-500 dark:text-gray-400">Durum</label>
      <Select id="fMinDurum" bind:value={fMinDurum} class="w-40">
        <option value="">Hepsi</option>
        <option value="400">≥ 400 (hata)</option>
        <option value="500">≥ 500 (sunucu)</option>
      </Select>
    </div>
    <div class="flex-1 min-w-[200px]">
      <label for="fQ" class="mb-1 block text-xs text-gray-500 dark:text-gray-400">Ara (endpoint / email)</label>
      <Input id="fQ" bind:value={fQ} placeholder="/kasalar veya admin@" />
    </div>
    <div>
      <label for="fLimit" class="mb-1 block text-xs text-gray-500 dark:text-gray-400">Limit</label>
      <Select id="fLimit" bind:value={fLimit} class="w-28">
        <option value={50}>50</option>
        <option value={100}>100</option>
        <option value={250}>250</option>
        <option value={500}>500</option>
      </Select>
    </div>
    <Button color="primary" onclick={yukle}>Uygula</Button>
    <Button color="alternative" onclick={() => { fYontem=''; fMinDurum=''; fQ=''; fLimit=100; yukle(); }}>Temizle</Button>
  </div>

  {#if yukleniyor}
    <div class="flex h-48 items-center justify-center"><Spinner size="10" /></div>
  {:else}
    <DataTable
      data={loglar}
      columns={kolonlar}
      searchPlaceholder="Listede ara..."
      exportFileName="islem-loglari"
      emptyMessage="Kayıt yok"
    >
      {#snippet row(l, _i, visibleCols)}
        <TableBodyRow>
          {#if visibleCols.has('tarih')}
            <TableBodyCell class="whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
              {tarihFormat(l.tarih)}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('kullanici')}
            <TableBodyCell class="text-sm text-gray-700 dark:text-gray-300">
              {l.kullanici_email ?? '—'}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('rol')}
            <TableBodyCell class="text-xs">
              {#if l.rol}<Badge color="dark">{l.rol}</Badge>{:else}—{/if}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('yontem')}
            <TableBodyCell>
              <Badge color={yontemColor(l.yontem)}>{l.yontem}</Badge>
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('yol')}
            <TableBodyCell class="font-mono text-xs text-gray-800 dark:text-gray-200">
              {l.yol}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('durum')}
            <TableBodyCell class="text-right">
              <Badge color={durumColor(l.durum_kodu)}>{l.durum_kodu}</Badge>
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('sure')}
            <TableBodyCell class="text-right text-xs text-gray-500">
              {l.sure_ms ?? '—'}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('ua')}
            <TableBodyCell class="max-w-[200px] truncate text-xs text-gray-400" title={l.user_agent ?? ''}>
              {l.user_agent ?? '—'}
            </TableBodyCell>
          {/if}
        </TableBodyRow>
      {/snippet}
    </DataTable>
  {/if}
</main>
