<script lang="ts">
  import { Heading, P, Button, Spinner } from 'flowbite-svelte';
  import {
    WalletSolid,
    UsersSolid,
    LayersSolid,
    CalendarMonthSolid,
    ArrowUpOutline,
    ArrowDownOutline,
    ChartLineUpOutline,
    ArrowRightOutline,
    ExclamationCircleSolid,
    UserCircleSolid,
    ArrowsRepeatOutline
  } from 'flowbite-svelte-icons';
  import {
    kasaApi,
    hissedarApi,
    hisseApi,
    donemApi,
    gelirGiderApi,
    cuzdanApi,
    kasaTransferApi,
    formatBakiye,
    type Kasa,
    type Hissedar,
    type Hisse,
    type Donem,
    type GelirGiderKayit,
    type ParaBirimi,
    type CuzdanHareketi,
    type KasaTransfer
  } from '$lib/tauri-api';
  import { hissedarLabel } from '$lib/hissedarFormat';

  interface BorcluSatir {
    hissedar: Hissedar;
    borc: number;
  }

  let kasalar = $state<Kasa[]>([]);
  let hissedarlar = $state<Hissedar[]>([]);
  let hisseler = $state<Hisse[]>([]);
  let donemler = $state<Donem[]>([]);
  let gelirGider = $state<GelirGiderKayit[]>([]);
  let topBorclular = $state<BorcluSatir[]>([]);
  let sonTransferler = $state<KasaTransfer[]>([]);
  let yukleniyor = $state(true);
  let hata = $state('');

  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      [kasalar, hissedarlar, hisseler, donemler, gelirGider] = await Promise.all([
        kasaApi.getAll(),
        hissedarApi.getAll(),
        hisseApi.getAll(),
        donemApi.getAll(),
        gelirGiderApi.getAll()
      ]);

      // Borçlu hissedarları hesapla (paralel)
      const cuzdanSonuclari = await Promise.all(
        hissedarlar.map((h) =>
          cuzdanApi
            .getByHissedar(h.id)
            .then((list) => ({ h, bakiye: list[0]?.bakiye ?? 0 }))
            .catch(() => ({ h, bakiye: 0 }))
        )
      );
      topBorclular = cuzdanSonuclari
        .filter((x) => x.bakiye < -0.009)
        .map((x) => ({ hissedar: x.h, borc: -x.bakiye }))
        .sort((a, b) => b.borc - a.borc)
        .slice(0, 10);

      // Son transferler: tüm kasalardan çek, id'ye göre dedupe et
      const transferArr = await Promise.all(
        kasalar.map((k) => kasaTransferApi.getAll(k.id).catch(() => [] as KasaTransfer[]))
      );
      const map = new Map<number, KasaTransfer>();
      for (const list of transferArr) for (const t of list) map.set(t.id, t);
      sonTransferler = [...map.values()]
        .sort((a, b) => (b.created_at > a.created_at ? 1 : -1))
        .slice(0, 10);
    } catch (e: any) {
      hata = e?.toString() ?? 'Yükleme hatası';
    } finally {
      yukleniyor = false;
    }
  }

  $effect(() => { yukle(); });

  const aktifKasalar = $derived(kasalar.filter((k) => k.aktif));
  const aktifDonem = $derived(donemler.find((d) => d.aktif));

  const toplamHisse = $derived(hisseler.length);
  const musaitHisse = $derived(hisseler.filter((h) => h.durum === 'musait').length);
  const atanmisHisse = $derived(hisseler.filter((h) => h.durum === 'atanmis').length);
  const satilanHisse = $derived(hisseler.filter((h) => h.durum === 'satildi').length);
  const aktifHissedar = $derived(hissedarlar.filter((h) => h.aktif).length);

  const buAy = $derived(new Date().toISOString().slice(0, 7));
  const buAyGelir = $derived(
    gelirGider.filter((k) => k.kategori_tip === 'gelir' && k.tarih.startsWith(buAy)).reduce((s, k) => s + k.tutar, 0)
  );
  const buAyGider = $derived(
    gelirGider.filter((k) => k.kategori_tip === 'gider' && k.tarih.startsWith(buAy)).reduce((s, k) => s + k.tutar, 0)
  );
  const toplamGelir = $derived(
    gelirGider.filter((k) => k.kategori_tip === 'gelir').reduce((s, k) => s + k.tutar, 0)
  );
  const toplamGider = $derived(
    gelirGider.filter((k) => k.kategori_tip === 'gider').reduce((s, k) => s + k.tutar, 0)
  );

  const sonKayitlar = $derived(gelirGider.slice(0, 5));

  const kasaToplamlari = $derived.by(() => {
    const toplam: Record<string, number> = {};
    for (const k of aktifKasalar) {
      toplam[k.para_birimi] = (toplam[k.para_birimi] ?? 0) + k.bakiye;
    }
    return Object.entries(toplam);
  });

  function aySozel(ay: number): string {
    const aylar = ['Ocak', 'Şubat', 'Mart', 'Nisan', 'Mayıs', 'Haziran', 'Temmuz', 'Ağustos', 'Eylül', 'Ekim', 'Kasım', 'Aralık'];
    return aylar[ay - 1] ?? '';
  }

  function tarihFormat(t: string): string {
    return new Date(t).toLocaleDateString('tr-TR', { day: '2-digit', month: '2-digit', year: 'numeric' });
  }

  function tutarFormat(tutar: number): string {
    return tutar.toLocaleString('tr-TR', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }
</script>

<main class="min-h-screen bg-gray-50 p-6 dark:bg-gray-900">

  <div class="mb-6">
    <Heading tag="h1" class="text-2xl font-bold text-gray-900 dark:text-white">
      Kooperatif Paneli
    </Heading>
    <P class="text-sm text-gray-500 dark:text-gray-400">
      Hızlı genel bakış ve son işlemler
    </P>
  </div>

  {#if hata}
    <div class="mb-4 rounded-lg bg-red-50 p-4 text-sm text-red-700 dark:bg-red-900/20 dark:text-red-400">
      {hata}
    </div>
  {/if}

  {#if yukleniyor}
    <div class="flex h-64 items-center justify-center">
      <Spinner size="10" />
    </div>
  {:else}

    <div class="mb-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">

      <a href="/hissedar" class="group rounded-xl border border-gray-200 bg-white p-5 shadow-sm transition-shadow hover:shadow-md dark:border-gray-700 dark:bg-gray-800">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-500 dark:text-gray-400">Aktif Hissedar</p>
            <p class="text-2xl font-bold text-gray-900 dark:text-white">{aktifHissedar}</p>
            <p class="text-xs text-gray-400">Toplam: {hissedarlar.length}</p>
          </div>
          <div class="flex h-12 w-12 items-center justify-center rounded-full bg-blue-100 text-blue-600 dark:bg-blue-900/30 dark:text-blue-400">
            <UsersSolid class="h-6 w-6" />
          </div>
        </div>
      </a>

      <a href="/hisse" class="group rounded-xl border border-gray-200 bg-white p-5 shadow-sm transition-shadow hover:shadow-md dark:border-gray-700 dark:bg-gray-800">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-500 dark:text-gray-400">Toplam Hisse</p>
            <p class="text-2xl font-bold text-gray-900 dark:text-white">{toplamHisse}</p>
            <div class="mt-1 flex flex-wrap gap-1 text-xs">
              <span class="text-gray-500">Müsait: <b class="text-gray-700 dark:text-gray-300">{musaitHisse}</b></span>
              <span class="text-green-600">· Atanmış: <b>{atanmisHisse}</b></span>
              <span class="text-red-500">· Satılan: <b>{satilanHisse}</b></span>
            </div>
          </div>
          <div class="flex h-12 w-12 items-center justify-center rounded-full bg-purple-100 text-purple-600 dark:bg-purple-900/30 dark:text-purple-400">
            <LayersSolid class="h-6 w-6" />
          </div>
        </div>
      </a>

      <a href="/donem" class="group rounded-xl border border-gray-200 bg-white p-5 shadow-sm transition-shadow hover:shadow-md dark:border-gray-700 dark:bg-gray-800">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-500 dark:text-gray-400">Aktif Dönem</p>
            {#if aktifDonem}
              <p class="text-xl font-bold text-gray-900 dark:text-white">
                {aySozel(aktifDonem.ay)} {aktifDonem.yil}
              </p>
              <p class="text-xs text-gray-400">
                Aidat: {tutarFormat(aktifDonem.hisse_basi_aidat)} ₺ / hisse
              </p>
            {:else}
              <p class="text-lg font-semibold text-gray-400">Yok</p>
              <p class="text-xs text-gray-400">Toplam {donemler.length} dönem</p>
            {/if}
          </div>
          <div class="flex h-12 w-12 items-center justify-center rounded-full bg-amber-100 text-amber-600 dark:bg-amber-900/30 dark:text-amber-400">
            <CalendarMonthSolid class="h-6 w-6" />
          </div>
        </div>
      </a>

      <a href="/kasa" class="group rounded-xl border border-gray-200 bg-white p-5 shadow-sm transition-shadow hover:shadow-md dark:border-gray-700 dark:bg-gray-800">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-500 dark:text-gray-400">Aktif Kasa</p>
            <p class="text-2xl font-bold text-gray-900 dark:text-white">{aktifKasalar.length}</p>
            <p class="text-xs text-gray-400">Toplam: {kasalar.length}</p>
          </div>
          <div class="flex h-12 w-12 items-center justify-center rounded-full bg-teal-100 text-teal-600 dark:bg-teal-900/30 dark:text-teal-400">
            <WalletSolid class="h-6 w-6" />
          </div>
        </div>
      </a>
    </div>

    <div class="mb-6 grid grid-cols-1 gap-4 lg:grid-cols-3">

      <div class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-700 dark:bg-gray-800 lg:col-span-2">
        <div class="mb-4 flex items-center justify-between">
          <Heading tag="h3" class="text-lg font-semibold text-gray-900 dark:text-white">
            Kasa Bakiyeleri
          </Heading>
          <a href="/kasa" class="text-sm text-primary-600 hover:underline dark:text-primary-400">
            Tümünü gör →
          </a>
        </div>

        {#if aktifKasalar.length === 0}
          <P class="py-8 text-center text-gray-400">Henüz kasa yok</P>
        {:else}
          <div class="mb-4 flex flex-wrap gap-3">
            {#each kasaToplamlari as [birim, toplam]}
              <div class="rounded-lg bg-gray-50 px-4 py-2 dark:bg-gray-700">
                <p class="text-xs text-gray-500 dark:text-gray-400">{birim}</p>
                <p class="font-bold text-gray-900 dark:text-white">
                  {formatBakiye(toplam, birim as ParaBirimi)}
                </p>
              </div>
            {/each}
          </div>

          <div class="space-y-2">
            {#each aktifKasalar.slice(0, 5) as k}
              <a
                href={`/kasa/${k.id}`}
                class="flex items-center justify-between rounded-lg border border-gray-100 p-3 transition-colors hover:bg-gray-50 dark:border-gray-700 dark:hover:bg-gray-700"
              >
                <div class="flex items-center gap-3">
                  <div class="flex h-9 w-9 items-center justify-center rounded-full bg-teal-100 text-teal-600 dark:bg-teal-900/30 dark:text-teal-400">
                    <WalletSolid class="h-4 w-4" />
                  </div>
                  <div>
                    <p class="font-medium text-gray-900 dark:text-white">{k.ad}</p>
                    <p class="text-xs text-gray-500 dark:text-gray-400">{k.para_birimi}</p>
                  </div>
                </div>
                <p class="font-bold {k.bakiye >= 0 ? 'text-gray-900 dark:text-white' : 'text-red-600 dark:text-red-400'}">
                  {formatBakiye(k.bakiye, k.para_birimi)}
                </p>
              </a>
            {/each}
          </div>
        {/if}
      </div>

      <div class="flex flex-col gap-4">
        <div class="rounded-xl border border-green-200 bg-gradient-to-br from-green-50 to-white p-5 shadow-sm dark:border-green-900/40 dark:from-green-900/10 dark:to-gray-800">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm text-green-700 dark:text-green-400">Bu Ay Gelir</p>
              <p class="text-2xl font-bold text-green-700 dark:text-green-400">
                {tutarFormat(buAyGelir)} ₺
              </p>
              <p class="mt-1 text-xs text-gray-500">Toplam: {tutarFormat(toplamGelir)} ₺</p>
            </div>
            <ArrowUpOutline class="h-8 w-8 text-green-400" />
          </div>
        </div>

        <div class="rounded-xl border border-red-200 bg-gradient-to-br from-red-50 to-white p-5 shadow-sm dark:border-red-900/40 dark:from-red-900/10 dark:to-gray-800">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm text-red-700 dark:text-red-400">Bu Ay Gider</p>
              <p class="text-2xl font-bold text-red-700 dark:text-red-400">
                {tutarFormat(buAyGider)} ₺
              </p>
              <p class="mt-1 text-xs text-gray-500">Toplam: {tutarFormat(toplamGider)} ₺</p>
            </div>
            <ArrowDownOutline class="h-8 w-8 text-red-400" />
          </div>
        </div>

        <div class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-700 dark:bg-gray-800">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm text-gray-500 dark:text-gray-400">Bu Ay Net</p>
              <p class="text-2xl font-bold {buAyGelir - buAyGider >= 0 ? 'text-gray-900 dark:text-white' : 'text-red-600 dark:text-red-400'}">
                {tutarFormat(buAyGelir - buAyGider)} ₺
              </p>
            </div>
            <ChartLineUpOutline class="h-8 w-8 text-gray-400" />
          </div>
        </div>
      </div>
    </div>

    <div class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-700 dark:bg-gray-800">
      <div class="mb-4 flex items-center justify-between">
        <Heading tag="h3" class="text-lg font-semibold text-gray-900 dark:text-white">
          Son Gelir / Gider Hareketleri
        </Heading>
        <Button size="xs" color="alternative" href="/gelir-gider" class="gap-1">
          Tümü <ArrowRightOutline class="h-3 w-3" />
        </Button>
      </div>

      {#if sonKayitlar.length === 0}
        <P class="py-8 text-center text-gray-400">Henüz kayıt yok</P>
      {:else}
        <div class="divide-y divide-gray-100 dark:divide-gray-700">
          {#each sonKayitlar as k}
            <div class="flex items-center justify-between py-3">
              <div class="flex items-center gap-3">
                <div class="flex h-9 w-9 items-center justify-center rounded-full {k.kategori_tip === 'gelir' ? 'bg-green-100 text-green-600 dark:bg-green-900/30 dark:text-green-400' : 'bg-red-100 text-red-600 dark:bg-red-900/30 dark:text-red-400'}">
                  {#if k.kategori_tip === 'gelir'}
                    <ArrowUpOutline class="h-4 w-4" />
                  {:else}
                    <ArrowDownOutline class="h-4 w-4" />
                  {/if}
                </div>
                <div>
                  <p class="font-medium text-gray-900 dark:text-white">{k.aciklama}</p>
                  <p class="text-xs text-gray-500 dark:text-gray-400">
                    {k.kategori_ad} · {k.kasa_ad} · {tarihFormat(k.tarih)}
                  </p>
                </div>
              </div>
              <p class="font-bold {k.kategori_tip === 'gelir' ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}">
                {k.kategori_tip === 'gider' ? '-' : '+'}{tutarFormat(k.tutar)} ₺
              </p>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <div class="mt-6 grid grid-cols-1 gap-4 lg:grid-cols-2">

      <div class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-700 dark:bg-gray-800">
        <div class="mb-4 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <ExclamationCircleSolid class="h-5 w-5 text-red-500" />
            <Heading tag="h3" class="text-lg font-semibold text-gray-900 dark:text-white">
              En Çok Borçlu 10 Kişi
            </Heading>
          </div>
          <Button size="xs" color="alternative" href="/borclar" class="gap-1">
            Tümü <ArrowRightOutline class="h-3 w-3" />
          </Button>
        </div>

        {#if topBorclular.length === 0}
          <P class="py-8 text-center text-gray-400">Borçlu hissedar yok</P>
        {:else}
          <div class="divide-y divide-gray-100 dark:divide-gray-700">
            {#each topBorclular as b, i (b.hissedar.id)}
              <a
                href={`/hissedar/${b.hissedar.id}`}
                class="flex items-center justify-between py-2.5 transition-colors hover:bg-red-50 dark:hover:bg-red-900/10 -mx-5 px-5"
              >
                <div class="flex items-center gap-3 min-w-0">
                  <span class="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-red-100 text-xs font-bold text-red-600 dark:bg-red-900/30 dark:text-red-400">
                    {i + 1}
                  </span>
                  <div class="min-w-0 flex-1">
                    <p class="truncate font-medium text-gray-900 dark:text-white">
                      {hissedarLabel(b.hissedar)}
                    </p>
                    {#if b.hissedar.tel}
                      <p class="text-xs text-gray-400">{b.hissedar.tel}</p>
                    {/if}
                  </div>
                </div>
                <p class="ml-2 flex-shrink-0 font-bold text-red-600 dark:text-red-400">
                  -{tutarFormat(b.borc)} ₺
                </p>
              </a>
            {/each}
          </div>
        {/if}
      </div>

      <div class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-700 dark:bg-gray-800">
        <div class="mb-4 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <ArrowsRepeatOutline class="h-5 w-5 text-blue-500" />
            <Heading tag="h3" class="text-lg font-semibold text-gray-900 dark:text-white">
              Son 10 Kasa Transferi
            </Heading>
          </div>
        </div>

        {#if sonTransferler.length === 0}
          <P class="py-8 text-center text-gray-400">Henüz transfer yok</P>
        {:else}
          <div class="divide-y divide-gray-100 dark:divide-gray-700">
            {#each sonTransferler as t (t.id)}
              <div class="py-2.5">
                <div class="flex items-center justify-between gap-2">
                  <div class="flex items-center gap-2 min-w-0 text-sm">
                    <a href={`/kasa/${t.kaynak_kasa_id}`} class="truncate font-medium text-gray-700 hover:text-primary-600 dark:text-gray-200">
                      {t.kaynak_kasa_ad}
                    </a>
                    <ArrowRightOutline class="h-3 w-3 flex-shrink-0 text-gray-400" />
                    <a href={`/kasa/${t.hedef_kasa_id}`} class="truncate font-medium text-gray-700 hover:text-primary-600 dark:text-gray-200">
                      {t.hedef_kasa_ad}
                    </a>
                  </div>
                  <p class="flex-shrink-0 text-xs text-gray-400">{tarihFormat(t.tarih)}</p>
                </div>
                <div class="mt-1 flex items-center justify-between text-xs">
                  <p class="truncate text-gray-500 dark:text-gray-400">
                    {t.aciklama || '—'}
                  </p>
                  <p class="ml-2 flex-shrink-0 font-semibold text-gray-900 dark:text-white">
                    {#if t.kaynak_kasa_para_birimi === t.hedef_kasa_para_birimi}
                      {tutarFormat(t.hedef_miktar)} {t.hedef_kasa_para_birimi}
                    {:else}
                      {tutarFormat(t.kaynak_miktar)} {t.kaynak_kasa_para_birimi} → {tutarFormat(t.hedef_miktar)} {t.hedef_kasa_para_birimi}
                    {/if}
                  </p>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

    </div>

  {/if}
</main>
