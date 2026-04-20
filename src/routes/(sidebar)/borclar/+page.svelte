<script lang="ts">
  import { goto } from '$app/navigation';
  import {
    Heading,
    P,
    Button,
    Spinner,
    Input,
    Table,
    TableBody,
    TableBodyCell,
    TableBodyRow,
    TableHead,
    TableHeadCell,
    Badge
  } from 'flowbite-svelte';
  import {
    ArrowLeftOutline,
    SearchOutline,
    UserCircleSolid,
    ExclamationCircleSolid
  } from 'flowbite-svelte-icons';
  import {
    hissedarApi,
    cuzdanApi,
    type Hissedar,
    type CuzdanHareketi
  } from '$lib/tauri-api';
  import { hissedarLabel } from '$lib/hissedarFormat';

  interface BorcluSatir {
    hissedar: Hissedar;
    bakiye: number;
    borc: number; // pozitif sayı: toplam borç miktarı
    sonIslemTarih?: string;
    sonIslemBilgi?: string;
  }

  let yukleniyor = $state(true);
  let hata = $state('');
  let borclular = $state<BorcluSatir[]>([]);
  let arama = $state('');
  let sirala = $state<'borc_desc' | 'borc_asc' | 'isim'>('borc_desc');

  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      const hissedarlar = await hissedarApi.getAll();
      const cuzdanlar = await Promise.all(
        hissedarlar.map((h) =>
          cuzdanApi
            .getByHissedar(h.id)
            .then((list) => ({ h, list }))
            .catch(() => ({ h, list: [] as CuzdanHareketi[] }))
        )
      );

      const sonuc: BorcluSatir[] = [];
      for (const { h, list } of cuzdanlar) {
        // list DESC sıralı, ilk satır son bakiye
        const sonHareket = list[0];
        const bakiye = sonHareket?.bakiye ?? 0;
        if (bakiye < -0.009) {
          sonuc.push({
            hissedar: h,
            bakiye,
            borc: -bakiye,
            sonIslemTarih: sonHareket?.tarih,
            sonIslemBilgi: sonHareket?.bilgi
          });
        }
      }
      borclular = sonuc;
    } catch (e: any) {
      hata = e?.toString() ?? 'Yükleme hatası';
    } finally {
      yukleniyor = false;
    }
  }

  $effect(() => { yukle(); });

  const filtreli = $derived.by(() => {
    const q = arama.trim().toLocaleLowerCase('tr-TR');
    let arr = borclular;
    if (q) {
      arr = arr.filter((b) => {
        const ad = `${b.hissedar.ad} ${b.hissedar.soyad}`.toLocaleLowerCase('tr-TR');
        const yakin = (b.hissedar.yakin_adi ?? '').toLocaleLowerCase('tr-TR');
        return ad.includes(q) || yakin.includes(q);
      });
    }
    const kopya = [...arr];
    if (sirala === 'borc_desc') kopya.sort((a, b) => b.borc - a.borc);
    else if (sirala === 'borc_asc') kopya.sort((a, b) => a.borc - b.borc);
    else
      kopya.sort((a, b) =>
        `${a.hissedar.ad} ${a.hissedar.soyad}`.localeCompare(
          `${b.hissedar.ad} ${b.hissedar.soyad}`,
          'tr-TR'
        )
      );
    return kopya;
  });

  const toplamBorc = $derived(borclular.reduce((s, b) => s + b.borc, 0));

  function tutarFormat(n: number): string {
    return n.toLocaleString('tr-TR', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  function tarihFormat(t?: string): string {
    if (!t) return '';
    return new Date(t).toLocaleDateString('tr-TR', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric'
    });
  }

  function hissedaraGit(id: number) {
    goto(`/hissedar/${id}`);
  }

  function geri() {
    if (typeof window !== 'undefined' && window.history.length > 1) {
      history.back();
    } else {
      goto('/dashboard');
    }
  }
</script>

<main class="min-h-screen bg-gray-50 p-6 dark:bg-gray-900">

  <div class="mb-4 flex items-center gap-3">
    <Button color="alternative" size="sm" onclick={geri} class="gap-2">
      <ArrowLeftOutline class="h-4 w-4" /> Geri
    </Button>
    <div>
      <Heading tag="h1" class="text-2xl font-bold text-gray-900 dark:text-white">
        Borç Listesi
      </Heading>
      <P class="text-sm text-gray-500 dark:text-gray-400">
        Aidat ve hisse atama ücreti borcu olan hissedarlar
      </P>
    </div>
  </div>

  {#if hata}
    <div class="mb-4 rounded-lg bg-red-50 p-4 text-sm text-red-700 dark:bg-red-900/20 dark:text-red-400">
      {hata}
    </div>
  {/if}

  <div class="mb-4 grid grid-cols-1 gap-3 sm:grid-cols-3">
    <div class="rounded-xl border border-red-200 bg-gradient-to-br from-red-50 to-white p-4 shadow-sm dark:border-red-900/40 dark:from-red-900/10 dark:to-gray-800">
      <div class="flex items-center justify-between">
        <div>
          <p class="text-xs text-red-700 dark:text-red-400">Borçlu Hissedar</p>
          <p class="text-2xl font-bold text-red-700 dark:text-red-400">{borclular.length}</p>
        </div>
        <ExclamationCircleSolid class="h-8 w-8 text-red-400" />
      </div>
    </div>
    <div class="rounded-xl border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800 sm:col-span-2">
      <p class="text-xs text-gray-500 dark:text-gray-400">Toplam Borç</p>
      <p class="text-2xl font-bold text-red-600 dark:text-red-400">
        {tutarFormat(toplamBorc)} ₺
      </p>
    </div>
  </div>

  <div class="rounded-xl border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800">
    <div class="mb-4 flex flex-wrap items-center gap-3">
      <div class="relative flex-1 min-w-[200px]">
        <div class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
          <SearchOutline class="h-4 w-4 text-gray-400" />
        </div>
        <Input
          placeholder="Ad, soyad veya yakın adı ile ara..."
          bind:value={arama}
          class="pl-9"
        />
      </div>
      <select
        bind:value={sirala}
        class="rounded-lg border border-gray-300 bg-gray-50 p-2 text-sm text-gray-900 focus:border-primary-500 focus:ring-primary-500 dark:border-gray-600 dark:bg-gray-700 dark:text-white"
      >
        <option value="borc_desc">Borca göre (çok → az)</option>
        <option value="borc_asc">Borca göre (az → çok)</option>
        <option value="isim">İsme göre</option>
      </select>
    </div>

    {#if yukleniyor}
      <div class="flex h-40 items-center justify-center">
        <Spinner size="8" />
      </div>
    {:else if filtreli.length === 0}
      <P class="py-10 text-center text-gray-400">
        {borclular.length === 0 ? 'Harika! Hiç borçlu hissedar yok.' : 'Kayıt bulunamadı.'}
      </P>
    {:else}
      <div class="overflow-x-auto">
        <Table hoverable shadow class="text-sm">
          <TableHead>
            <TableHeadCell>#</TableHeadCell>
            <TableHeadCell>Hissedar</TableHeadCell>
            <TableHeadCell>Telefon</TableHeadCell>
            <TableHeadCell>Son İşlem</TableHeadCell>
            <TableHeadCell class="text-right">Borç (₺)</TableHeadCell>
          </TableHead>
          <TableBody>
            {#each filtreli as b, i (b.hissedar.id)}
              <TableBodyRow
                class="cursor-pointer hover:bg-red-50 dark:hover:bg-red-900/10"
                onclick={() => hissedaraGit(b.hissedar.id)}
              >
                <TableBodyCell class="font-medium text-gray-500">{i + 1}</TableBodyCell>
                <TableBodyCell>
                  <div class="flex items-center gap-3">
                    <div class="flex h-9 w-9 items-center justify-center rounded-full bg-red-100 text-red-600 dark:bg-red-900/30 dark:text-red-400">
                      <UserCircleSolid class="h-5 w-5" />
                    </div>
                    <div>
                      <p class="font-medium text-gray-900 dark:text-white">
                        {hissedarLabel(b.hissedar)}
                      </p>
                      {#if !b.hissedar.aktif}
                        <Badge color="dark" class="mt-0.5">Pasif</Badge>
                      {/if}
                    </div>
                  </div>
                </TableBodyCell>
                <TableBodyCell class="text-gray-600 dark:text-gray-300">
                  {b.hissedar.tel ?? '—'}
                </TableBodyCell>
                <TableBodyCell class="text-gray-600 dark:text-gray-300">
                  {#if b.sonIslemTarih}
                    <p class="text-xs">{tarihFormat(b.sonIslemTarih)}</p>
                    <p class="truncate text-xs text-gray-400 max-w-[220px]">{b.sonIslemBilgi ?? ''}</p>
                  {:else}
                    —
                  {/if}
                </TableBodyCell>
                <TableBodyCell class="text-right font-bold text-red-600 dark:text-red-400">
                  -{tutarFormat(b.borc)}
                </TableBodyCell>
              </TableBodyRow>
            {/each}
          </TableBody>
        </Table>
      </div>
    {/if}
  </div>
</main>
