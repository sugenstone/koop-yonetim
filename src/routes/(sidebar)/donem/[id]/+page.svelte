<script lang="ts">
  import {
    Button,
    Badge,
    Modal,
    Label,
    Input,
    Select,
    Textarea,
    Spinner,
    Breadcrumb,
    BreadcrumbItem,
    Heading,
    Toggle,
    Table,
    TableHead,
    TableHeadCell,
    TableBody,
    TableBodyRow,
    TableBodyCell
  } from 'flowbite-svelte';
  import {
    CalendarMonthSolid,
    CalendarPlusOutline,
    PlusOutline,
    EditOutline,
    TrashBinSolid,
    ArrowLeftOutline,
    ClipboardCheckSolid,
    ChevronDownOutline,
    ChevronRightOutline,
    ClockOutline,
    CashOutline,
    CheckCircleSolid,
    CloseCircleSolid,
    FileLinesSolid
  } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import {
    donemApi,
    toplantIApi,
    kararApi,
    aidatApi,
    cuzdanApi,
    hissedarApi,
    donemAdi,
    donemYillari,
    AY_ADLARI,
    type Donem,
    type Toplanti,
    type Karar,
    type AidatBorcu,
    type BorcOlusturSonuc,
    type CuzdanParaEkleSonuc,
    type CuzdanHareketi,
    type Hissedar
  } from '$lib/tauri-api';
  import { hissedarLabelFromFields } from '$lib/hissedarFormat';
  import DataTable from '$lib/components/DataTable.svelte';
  import type { DataTableColumn } from '$lib/components/dataTableUtils';
  import { exportPdf, formatTL, formatTarih } from '$lib/pdf';

  // ─── State ──────────────────────────────────────────────────────────────────

  const donemId = $derived(Number($page.params.id));

  let donem = $state<Donem | null>(null);
  let toplantilar = $state<(Toplanti & { kararlar: Karar[]; acik: boolean })[]>([]);
  let yukleniyor = $state(true);
  let hata = $state('');

  // Dönem düzenle modal
  let donemModalAcik = $state(false);
  let fDonemAy = $state('');
  let fDonemYil = $state('');
  let fDonemAidat = $state('');
  let fDonemAktif = $state(true);

  // Toplantı modal
  let toplantIModalAcik = $state(false);
  let duzenlenecekToplanti = $state<Toplanti | null>(null);
  let fTarih = $state('');
  let fKonu = $state('');
  let fYer = $state('');

  // Sil modalleri
  let silDonemModal = $state(false);
  let silToplantIModal = $state(false);
  let silinecekToplantI = $state<Toplanti | null>(null);

  // Karar modal
  let kararModalAcik = $state(false);
  let kararToplantIId = $state(0);
  let duzenlenecekKarar = $state<Karar | null>(null);
  let fKararNo = $state('');
  let fAciklama = $state('');
  let silKararModal = $state(false);
  let silinecekKarar = $state<Karar | null>(null);

  let kaydediliyor = $state(false);

  // Aidat borçları
  let borclar = $state<AidatBorcu[]>([]);
  let hissedarlar = $state<Hissedar[]>([]);
  let borcOlusturModal = $state(false);
  let borcOlusturuluyor = $state(false);
  let borcSonuc = $state<BorcOlusturSonuc | null>(null);

  // ─── Yükle ─────────────────────────────────────────────────────────────────

  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      donem = await donemApi.get(donemId);
      const [tList, bList, hList] = await Promise.all([
        toplantIApi.getAll(donemId),
        aidatApi.getByDonem(donemId),
        hissedarApi.getAll()
      ]);
      borclar = bList;
      hissedarlar = hList;
      // Her toplantının kararlarını paralel yükle
      const kararListesi = await Promise.all(
        tList.map((t) => kararApi.getAll(t.id))
      );
      toplantilar = tList.map((t, i) => ({ ...t, kararlar: kararListesi[i], acik: true }));
    } catch (e: any) {
      hata = e?.toString() ?? 'Yükleme hatası';
    } finally {
      yukleniyor = false;
    }
  }

  $effect(() => { if (donemId) yukle(); });

  // ─── Dönem Düzenle ──────────────────────────────────────────────────────────

  function donemDuzenleAc() {
    if (!donem) return;
    fDonemAy = donem.ay.toString();
    fDonemYil = donem.yil.toString();
    fDonemAidat = donem.hisse_basi_aidat.toString();
    fDonemAktif = donem.aktif;
    donemModalAcik = true;
  }

  async function donemKaydet() {
    if (!donem || !fDonemAy || !fDonemYil) return;
    kaydediliyor = true;
    try {
      await donemApi.update({
        id: donem.id,
        ay: parseInt(fDonemAy),
        yil: parseInt(fDonemYil),
        hisse_basi_aidat: parseFloat(fDonemAidat) || 0,
        aktif: fDonemAktif
      });
      donemModalAcik = false;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Güncelleme hatası';
    } finally {
      kaydediliyor = false;
    }
  }

  async function donemSil() {
    if (!donem) return;
    kaydediliyor = true;
    try {
      await donemApi.delete(donem.id);
      goto('/donem');
    } catch (e: any) {
      hata = e?.toString() ?? 'Silme hatası';
      kaydediliyor = false;
    }
  }

  // ─── Toplantı CRUD ──────────────────────────────────────────────────────────

  function toplantIYeniAc() {
    duzenlenecekToplanti = null;
    fTarih = new Date().toISOString().split('T')[0];
    fKonu = '';
    fYer = '';
    toplantIModalAcik = true;
  }

  function toplantIDuzenleAc(t: Toplanti) {
    duzenlenecekToplanti = t;
    fTarih = t.tarih;
    fKonu = t.konu;
    fYer = t.yer ?? '';
    toplantIModalAcik = true;
  }

  async function toplantIKaydet() {
    if (!fTarih.trim() || !fKonu.trim()) return;
    kaydediliyor = true;
    try {
      if (duzenlenecekToplanti) {
        await toplantIApi.update({
          id: duzenlenecekToplanti.id,
          tarih: fTarih,
          konu: fKonu.trim(),
          yer: fYer.trim() || undefined
        });
      } else {
        await toplantIApi.create({
          donem_id: donemId,
          tarih: fTarih,
          konu: fKonu.trim(),
          yer: fYer.trim() || undefined
        });
      }
      toplantIModalAcik = false;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Kayıt hatası';
    } finally {
      kaydediliyor = false;
    }
  }

  async function toplantISil() {
    if (!silinecekToplantI) return;
    kaydediliyor = true;
    try {
      await toplantIApi.delete(silinecekToplantI.id);
      silToplantIModal = false;
      silinecekToplantI = null;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Silme hatası';
    } finally {
      kaydediliyor = false;
    }
  }

  // ─── Karar CRUD ─────────────────────────────────────────────────────────────

  function kararYeniAc(toplantIId: number) {
    kararToplantIId = toplantIId;
    duzenlenecekKarar = null;
    fKararNo = '';
    fAciklama = '';
    kararModalAcik = true;
  }

  function kararDuzenleAc(k: Karar) {
    kararToplantIId = k.toplanti_id;
    duzenlenecekKarar = k;
    fKararNo = k.karar_no?.toString() ?? '';
    fAciklama = k.aciklama;
    kararModalAcik = true;
  }

  async function kararKaydet() {
    if (!fAciklama.trim()) return;
    kaydediliyor = true;
    try {
      if (duzenlenecekKarar) {
        await kararApi.update({
          id: duzenlenecekKarar.id,
          karar_no: fKararNo ? parseInt(fKararNo) : undefined,
          aciklama: fAciklama.trim()
        });
      } else {
        await kararApi.create({
          toplanti_id: kararToplantIId,
          karar_no: fKararNo ? parseInt(fKararNo) : undefined,
          aciklama: fAciklama.trim()
        });
      }
      kararModalAcik = false;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Kayıt hatası';
    } finally {
      kaydediliyor = false;
    }
  }

  async function kararSil() {
    if (!silinecekKarar) return;
    kaydediliyor = true;
    try {
      await kararApi.delete(silinecekKarar.id);
      silKararModal = false;
      silinecekKarar = null;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Silme hatası';
    } finally {
      kaydediliyor = false;
    }
  }

  // ─── Borç İşlemleri ──────────────────────────────────────────────────────────

  async function borcOlustur() {
    borcOlusturuluyor = true;
    hata = '';
    borcSonuc = null;
    try {
      borcSonuc = await aidatApi.borcOlustur(donemId);
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Borç oluşturma hatası';
    } finally {
      borcOlusturuluyor = false;
    }
  }

  // ─── Tahsilat ────────────────────────────────────────────────────────────────

  let tahsilatModal = $state(false);
  let tahsilatBorcRef = $state<AidatBorcu | null>(null);
  let tahsilatTutar = $state('');
  let tahsilEdiliyor = $state(false);
  let tahsilatSonuc = $state<CuzdanParaEkleSonuc | null>(null);
  let tahsilatHissedarBorclar = $state<AidatBorcu[]>([]);
  let tahsilatHisseBorclar = $state<CuzdanHareketi[]>([]);  // hisse satın alma borçları
  let tahsilatCuzdanBakiye = $state(0);
  let tahsilatYukleniyor = $state(false);

  async function tahsilatAc(b: AidatBorcu) {
    tahsilatBorcRef = b;
    tahsilatTutar = b.tutar.toFixed(2);
    tahsilatSonuc = null;
    tahsilatHissedarBorclar = [];
    tahsilatHisseBorclar = [];
    tahsilatCuzdanBakiye = 0;
    tahsilatModal = true;
    tahsilatYukleniyor = true;
    try {
      const [tumBorclar, hareketler] = await Promise.all([
        aidatApi.getByHissedar(b.hissedar_id),
        cuzdanApi.getByHissedar(b.hissedar_id)
      ]);
      // Ödenmemiş aidat borçlarını al, en eskiden yeniye sırala
      tahsilatHissedarBorclar = tumBorclar
        .filter((x) => !x.odendi)
        .sort((a, b) => a.donem_id - b.donem_id);
      tahsilatCuzdanBakiye = hareketler.length > 0 ? hareketler[0].bakiye : 0;
      // Cüzdandaki hisse satın alma borçlarını bul
      // borc > 0, alacak = 0, bilgi 'Hisse satın alma' ile başlıyor
      tahsilatHisseBorclar = hareketler.filter(
        (h) => h.borc > 0 && h.alacak === 0 && h.bilgi.startsWith('Hisse satın alma')
      );
    } catch {
      // sessiz hata - modal yine de açık
    } finally {
      tahsilatYukleniyor = false;
    }
  }

  async function tahsilatYap() {
    if (!tahsilatBorcRef) return;
    const tutar = parseFloat(tahsilatTutar);
    if (!tutar || tutar <= 0) return;
    tahsilEdiliyor = true;
    try {
      const sonuc = await cuzdanApi.paraEkle({
        hissedar_id: tahsilatBorcRef.hissedar_id,
        tutar,
        aciklama: `Aidat tahsilatı: ${tahsilatBorcRef.donem_adi} - ${tahsilatBorcRef.hissedar_ad} ${tahsilatBorcRef.hissedar_soyad}`
      });
      await yukle();          // ← önce tabloyu yenile
      tahsilatSonuc = sonuc;  // ← sonra başarı ekranını göster
    } catch (e: any) {
      hata = e?.toString() ?? 'Tahsilat hatası';
      tahsilatModal = false;
    } finally {
      tahsilEdiliyor = false;
    }
  }

  // Tümünü öde = cüzdandaki toplam borç (bakiye negatifse o kadar ödenmeli)
  const tahsilatTumunuOdeTutar = $derived(
    Math.max(0, -tahsilatCuzdanBakiye)
  );

  // ─── Yardımcılar ────────────────────────────────────────────────────────────

  const yillar = donemYillari();
  const aySecenekleri = AY_ADLARI.map((ad, i) => ({ value: (i + 1).toString(), name: ad }));
  const yilSecenekleri = yillar.map((y) => ({ value: y.toString(), name: y.toString() }));

  function tarihFormat(t: string): string {
    return new Date(t).toLocaleDateString('tr-TR', { day: '2-digit', month: 'long', year: 'numeric' });
  }

  function formatAidat(aidat: number): string {
    return new Intl.NumberFormat('tr-TR', { style: 'currency', currency: 'TRY', minimumFractionDigits: 2 }).format(aidat);
  }

  function toggleToplanti(id: number) {
    toplantilar = toplantilar.map((t) => t.id === id ? { ...t, acik: !t.acik } : t);
  }

  const toplamBorc = $derived(borclar.reduce((s, b) => s + b.tutar, 0));
  const tahsilEdilen = $derived(borclar.filter((b) => b.odendi).reduce((s, b) => s + b.tutar, 0));
  const kalanBorc = $derived(toplamBorc - tahsilEdilen);

  // ─── Borç tablosu kolonları ────────────────────────────────────────────
  const borcKolonlar: DataTableColumn<AidatBorcu>[] = [
    {
      id: 'hissedar',
      header: 'Hissedar',
      accessor: (b) => `${b.hissedar_ad ?? ''} ${b.hissedar_soyad ?? ''}`
    },
    { id: 'hisse_sayisi', header: 'Hisse Sayısı', accessor: 'hisse_sayisi', align: 'center' },
    { id: 'tutar', header: 'Tutar', accessor: 'tutar', align: 'right' },
    { id: 'durum', header: 'Durum', accessor: (b) => (b.odendi ? 'Tahsil Edildi' : 'Ödenmedi'), align: 'center' },
    { id: 'odeme_tarihi', header: 'Ödeme Tarihi', accessor: (b) => b.odeme_tarihi ?? '' },
    { id: 'islemler', header: '', accessor: () => '', sortable: false, searchable: false }
  ];

  // ─── PDF ────────────────────────────────────────────────────────────────────

  function pdfIndir() {
    if (!donem) return;
    const d = donem;
    const toplam = borclar.reduce((s, b) => s + b.tutar, 0);
    const tahsil = borclar.filter((b) => b.odendi).reduce((s, b) => s + b.tutar, 0);
    const kalan = toplam - tahsil;
    const toplamKarar = toplantilar.reduce((s, t) => s + t.kararlar.length, 0);

    const sections: any[] = [
      {
        kind: 'kv',
        heading: 'Dönem Bilgileri',
        columns: 2,
        items: [
          { label: 'Dönem', value: donemAdi(d.ay, d.yil) },
          { label: 'Yıl', value: d.yil },
          { label: 'Ay', value: AY_ADLARI[d.ay - 1] ?? d.ay },
          { label: 'Hisse Başı Aidat', value: formatTL(d.hisse_basi_aidat) },
          { label: 'Toplantı Sayısı', value: d.toplanti_sayisi },
          { label: 'Karar Sayısı', value: toplamKarar },
          { label: 'Durum', value: d.aktif ? 'Aktif' : 'Pasif' }
        ]
      },
      {
        kind: 'kv',
        heading: 'Aidat Özeti',
        columns: 2,
        items: [
          { label: 'Toplam Borç', value: formatTL(toplam) },
          { label: 'Tahsil Edilen', value: formatTL(tahsil) },
          { label: 'Kalan', value: formatTL(kalan) },
          { label: 'Kayıt Sayısı', value: borclar.length }
        ]
      }
    ];

    if (toplantilar.length > 0) {
      sections.push({
        kind: 'table',
        heading: 'Toplantılar',
        columns: ['Tarih', 'Konu', 'Yer', 'Karar'],
        widths: ['auto', '*', 'auto', 'auto'],
        rows: toplantilar.map((t) => [
          formatTarih(t.tarih),
          t.konu,
          t.yer ?? '-',
          t.kararlar.length
        ])
      });
      for (const t of toplantilar) {
        if (t.kararlar.length === 0) continue;
        sections.push({
          kind: 'table',
          heading: `Kararlar — ${formatTarih(t.tarih)} · ${t.konu}`,
          columns: ['Karar No', 'Açıklama'],
          widths: ['auto', '*'],
          rows: t.kararlar.map((k) => [k.karar_no, k.aciklama])
        });
      }
    }

    if (borclar.length > 0) {
      sections.push({
        kind: 'table',
        heading: 'Aidat Borçları',
        columns: ['Hissedar', 'Hisse', 'Tutar', 'Durum', 'Ödeme Tarihi'],
        widths: ['*', 'auto', 'auto', 'auto', 'auto'],
        rows: borclar.map((b) => [
          `${b.hissedar_soyad ?? ''} ${b.hissedar_ad ?? ''}`,
          b.hisse_sayisi,
          formatTL(b.tutar),
          b.odendi ? 'Tahsil Edildi' : 'Ödenmedi',
          b.odeme_tarihi ? formatTarih(b.odeme_tarihi) : '-'
        ])
      });
    }

    exportPdf({
      title: `Dönem Raporu — ${donemAdi(d.ay, d.yil)}`,
      subtitle: `${donemAdi(d.ay, d.yil)} — ${d.aktif ? 'Aktif' : 'Pasif'}`,
      fileName: `donem-${d.yil}-${String(d.ay).padStart(2, '0')}`,
      sections
    });
  }
</script>

<main class="min-h-screen bg-gray-50 p-6 dark:bg-gray-900">

  <!-- Breadcrumb -->
  <Breadcrumb class="mb-5">
    <BreadcrumbItem href="/donem">Dönemler</BreadcrumbItem>
    <BreadcrumbItem>{donem ? donemAdi(donem.ay, donem.yil) : '...'}</BreadcrumbItem>
  </Breadcrumb>

  <!-- Hata -->
  {#if hata}
    <div class="mb-4 rounded-lg bg-red-50 p-4 text-sm text-red-700 dark:bg-red-900/20 dark:text-red-400">
      {hata}
    </div>
  {/if}

  {#if yukleniyor}
    <div class="flex h-48 items-center justify-center"><Spinner size="10" /></div>

  {:else if donem}

    <!-- Dönem Başlık Kartı -->
    <div class="mb-6 overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm dark:border-gray-700 dark:bg-gray-800">
      <div class="bg-primary-600 px-6 py-5">
        <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <div class="flex items-center gap-3">
            <CalendarMonthSolid class="h-8 w-8 text-primary-200" />
            <div>
              <Heading tag="h1" class="text-2xl font-bold text-white">
                {donemAdi(donem.ay, donem.yil)}
              </Heading>
              <p class="text-sm text-primary-200">
                Hisse Başı Aidat: <strong>{formatAidat(donem.hisse_basi_aidat)}</strong>
                &nbsp;·&nbsp;
                <Badge color={donem.aktif ? 'green' : 'red'} class="text-xs">
                  {donem.aktif ? 'Aktif' : 'Pasif'}
                </Badge>
              </p>
            </div>
          </div>
          <div class="flex gap-2">
            <Button size="sm" color="alternative" class="gap-2" onclick={pdfIndir}>
              <FileLinesSolid class="h-4 w-4" /> PDF
            </Button>
            <Button size="sm" color="light" class="gap-2" onclick={donemDuzenleAc}>
              <EditOutline class="h-4 w-4" /> Düzenle
            </Button>
            <Button size="sm" color="red" class="gap-2" onclick={() => (silDonemModal = true)}>
              <TrashBinSolid class="h-4 w-4" /> Sil
            </Button>
          </div>
        </div>
      </div>

      <!-- İstatistikler -->
      <div class="grid grid-cols-3 divide-x divide-gray-100 dark:divide-gray-700 sm:grid-cols-3">
        <div class="p-4 text-center">
          <p class="text-2xl font-bold text-gray-900 dark:text-white">{donem.toplanti_sayisi}</p>
          <p class="text-xs text-gray-500 dark:text-gray-400">Toplantı</p>
        </div>
        <div class="p-4 text-center">
          <p class="text-2xl font-bold text-gray-900 dark:text-white">
            {toplantilar.reduce((s, t) => s + t.kararlar.length, 0)}
          </p>
          <p class="text-xs text-gray-500 dark:text-gray-400">Karar</p>
        </div>
        <div class="p-4 text-center">
          <p class="text-2xl font-bold text-gray-900 dark:text-white">{borclar.length}</p>
          <p class="text-xs text-gray-500 dark:text-gray-400">Aidat Borcu</p>
        </div>
      </div>
    </div>

    <!-- Toplantılar -->
    <div class="mb-4 flex items-center justify-between">
      <Heading tag="h2" class="text-lg font-semibold text-gray-900 dark:text-white">Toplantılar</Heading>
      <Button size="sm" class="gap-2" onclick={toplantIYeniAc}>
        <CalendarPlusOutline class="h-4 w-4" /> Toplantı Ekle
      </Button>
    </div>

    {#if toplantilar.length === 0}
      <div class="flex h-36 items-center justify-center rounded-xl border-2 border-dashed border-gray-300 dark:border-gray-700">
        <p class="text-sm text-gray-500">Henüz toplantı eklenmemiş</p>
      </div>

    {:else}
      <div class="space-y-4">
        {#each toplantilar as t (t.id)}
          <div class="overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm dark:border-gray-700 dark:bg-gray-800">

            <!-- Toplantı Başlığı -->
            <div class="flex items-start justify-between p-5">
              <button
                class="flex flex-1 items-start gap-3 text-left"
                onclick={() => toggleToplanti(t.id)}
              >
                <div class="mt-0.5 text-primary-500">
                  {#if t.acik}
                    <ChevronDownOutline class="h-5 w-5" />
                  {:else}
                    <ChevronRightOutline class="h-5 w-5" />
                  {/if}
                </div>
                <div>
                  <p class="font-semibold text-gray-900 dark:text-white">{t.konu}</p>
                  <div class="mt-1 flex flex-wrap items-center gap-3 text-xs text-gray-500 dark:text-gray-400">
                    <span class="flex items-center gap-1">
                      <ClockOutline class="h-3.5 w-3.5" />
                      {tarihFormat(t.tarih)}
                    </span>
                    {#if t.yer}
                      <span>📍 {t.yer}</span>
                    {/if}
                    <Badge color="blue">{t.kararlar.length} karar</Badge>
                  </div>
                </div>
              </button>
              <div class="ml-3 flex shrink-0 gap-1">
                <button
                  class="rounded p-1.5 text-gray-400 hover:bg-gray-100 hover:text-primary-600 dark:hover:bg-gray-700"
                  onclick={() => toplantIDuzenleAc(t)}
                  title="Düzenle"
                >
                  <EditOutline class="h-4 w-4" />
                </button>
                <button
                  class="rounded p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-gray-700"
                  onclick={() => { silinecekToplantI = t; silToplantIModal = true; }}
                  title="Sil"
                >
                  <TrashBinSolid class="h-4 w-4" />
                </button>
              </div>
            </div>

            <!-- Kararlar (açılır kapanır) -->
            {#if t.acik}
              <div class="border-t border-gray-100 dark:border-gray-700">
                <!-- Kararlar başlık -->
                <div class="flex items-center justify-between px-5 py-3">
                  <div class="flex items-center gap-2 text-sm font-semibold text-gray-600 dark:text-gray-400">
                    <ClipboardCheckSolid class="h-4 w-4" />
                    Alınan Kararlar
                  </div>
                  <button
                    class="flex items-center gap-1 text-xs font-medium text-primary-600 hover:underline dark:text-primary-400"
                    onclick={() => kararYeniAc(t.id)}
                  >
                    <PlusOutline class="h-3.5 w-3.5" /> Karar Ekle
                  </button>
                </div>

                {#if t.kararlar.length === 0}
                  <div class="px-5 pb-4 text-sm text-gray-400 italic">Henüz karar eklenmemiş</div>

                {:else}
                  <ul class="divide-y divide-gray-100 dark:divide-gray-700">
                    {#each t.kararlar as k (k.id)}
                      <li class="flex items-start gap-3 px-5 py-3">
                        {#if k.karar_no}
                          <span class="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-primary-100 text-xs font-bold text-primary-700 dark:bg-primary-900 dark:text-primary-300">
                            {k.karar_no}
                          </span>
                        {:else}
                          <span class="mt-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-gray-100 text-xs text-gray-500 dark:bg-gray-700">
                            —
                          </span>
                        {/if}
                        <p class="flex-1 text-sm text-gray-800 dark:text-gray-200">{k.aciklama}</p>
                        <div class="flex shrink-0 gap-1">
                          <button
                            class="rounded p-1 text-gray-400 hover:text-primary-600"
                            onclick={() => kararDuzenleAc(k)}
                            title="Düzenle"
                          >
                            <EditOutline class="h-3.5 w-3.5" />
                          </button>
                          <button
                            class="rounded p-1 text-gray-400 hover:text-red-600"
                            onclick={() => { silinecekKarar = k; silKararModal = true; }}
                            title="Sil"
                          >
                            <TrashBinSolid class="h-3.5 w-3.5" />
                          </button>
                        </div>
                      </li>
                    {/each}
                  </ul>
                {/if}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    <!-- Aidat Borçları -->
    <div class="mb-4 mt-8 flex items-center justify-between">
      <Heading tag="h2" class="text-lg font-semibold text-gray-900 dark:text-white">Aidat Borçları</Heading>
      <Button size="sm" color="green" class="gap-2" onclick={() => (borcOlusturModal = true)}>
        <CashOutline class="h-4 w-4" /> Borç Kaydı Oluştur
      </Button>
    </div>

    {#if borclar.length === 0}
      <div class="flex h-36 items-center justify-center rounded-xl border-2 border-dashed border-gray-300 dark:border-gray-700">
        <p class="text-sm text-gray-500">Henüz aidat borcu oluşturulmamış</p>
      </div>
    {:else}
      <!-- Borç Özet Kartları -->
      <div class="mb-4 grid grid-cols-3 gap-4">
        <div class="rounded-xl border border-gray-200 bg-white p-4 text-center dark:border-gray-700 dark:bg-gray-800">
          <p class="text-2xl font-bold text-gray-900 dark:text-white">{formatAidat(toplamBorc)}</p>
          <p class="text-xs text-gray-500 dark:text-gray-400">Toplam Borç</p>
        </div>
        <div class="rounded-xl border border-green-200 bg-green-50 p-4 text-center dark:border-green-800 dark:bg-green-900/20">
          <p class="text-2xl font-bold text-green-700 dark:text-green-400">{formatAidat(tahsilEdilen)}</p>
          <p class="text-xs text-green-600 dark:text-green-500">Tahsil Edilen</p>
        </div>
        <div class="rounded-xl border border-red-200 bg-red-50 p-4 text-center dark:border-red-800 dark:bg-red-900/20">
          <p class="text-2xl font-bold text-red-700 dark:text-red-400">{formatAidat(kalanBorc)}</p>
          <p class="text-xs text-red-600 dark:text-red-500">Kalan Borç</p>
        </div>
      </div>

      <!-- Borç Tablosu -->
      <DataTable
        data={borclar}
        columns={borcKolonlar}
        searchPlaceholder="Hissedar ara..."
        exportFileName="donem-borclari-{donemId}"
        emptyMessage="Aidat borcu bulunmuyor"
      >
        {#snippet row(b, _i, visibleCols)}
          <TableBodyRow>
            {#if visibleCols.has('hissedar')}
              <TableBodyCell>
                <span class="font-medium text-gray-900 dark:text-white">
                  {hissedarLabelFromFields(b.hissedar_id, b.hissedar_ad, b.hissedar_soyad, hissedarlar)}
                </span>
              </TableBodyCell>
            {/if}
            {#if visibleCols.has('hisse_sayisi')}
              <TableBodyCell class="text-center">
                <Badge color="blue">{b.hisse_sayisi} Hisse</Badge>
              </TableBodyCell>
            {/if}
            {#if visibleCols.has('tutar')}
              <TableBodyCell class="text-right">
                <span class="font-semibold">{formatAidat(b.tutar)}</span>
              </TableBodyCell>
            {/if}
            {#if visibleCols.has('durum')}
              <TableBodyCell class="text-center">
                {#if b.odendi}
                  <Badge color="green" class="gap-1">
                    <CheckCircleSolid class="h-3 w-3" /> Tahsil Edildi
                  </Badge>
                {:else}
                  <Badge color="red" class="gap-1">
                    <CloseCircleSolid class="h-3 w-3" /> Ödenmedi
                  </Badge>
                {/if}
              </TableBodyCell>
            {/if}
            {#if visibleCols.has('odeme_tarihi')}
              <TableBodyCell>
                {#if b.odeme_tarihi}
                  {tarihFormat(b.odeme_tarihi)}
                {:else}
                  <span class="text-gray-400">—</span>
                {/if}
              </TableBodyCell>
            {/if}
            {#if visibleCols.has('islemler')}
              <TableBodyCell>
                {#if !b.odendi}
                  <Button size="xs" color="green" class="gap-1" onclick={() => tahsilatAc(b)}>
                    <CashOutline class="h-3.5 w-3.5" /> Tahsilat
                  </Button>
                {/if}
              </TableBodyCell>
            {/if}
          </TableBodyRow>
        {/snippet}
      </DataTable>
    {/if}

    <!-- Geri -->
    <div class="mt-6">
      <button
        class="flex items-center gap-2 text-sm text-gray-500 hover:text-primary-600 dark:text-gray-400 dark:hover:text-primary-400"
        onclick={() => goto('/donem')}
      >
        <ArrowLeftOutline class="h-4 w-4" /> Tüm Dönemlere Dön
      </button>
    </div>

  {:else}
    <p class="text-gray-500">Dönem bulunamadı.</p>
  {/if}
</main>

<!-- Dönem Düzenle Modal -->
<Modal bind:open={donemModalAcik} title="Dönemi Düzenle" size="sm" autoclose={false}>
  <div class="space-y-4">
    <div>
      <Label for="dAy" class="mb-2">Ay <span class="text-red-500">*</span></Label>
      <Select id="dAy" bind:value={fDonemAy} items={aySecenekleri} />
    </div>
    <div>
      <Label for="dYil" class="mb-2">Yıl <span class="text-red-500">*</span></Label>
      <Select id="dYil" bind:value={fDonemYil} items={yilSecenekleri} />
    </div>
    <div>
      <Label for="dAidat" class="mb-2">Hisse Başı Aidat (₺)</Label>
      <Input id="dAidat" type="number" min="0" step="0.01" bind:value={fDonemAidat} placeholder="0.00" />
    </div>
    <div class="flex items-center gap-3">
      <Toggle bind:checked={fDonemAktif} />
      <span class="text-sm text-gray-700 dark:text-gray-300">{fDonemAktif ? 'Aktif' : 'Pasif'}</span>
    </div>
  </div>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button onclick={donemKaydet} disabled={kaydediliyor || !fDonemAy || !fDonemYil}>
        {#if kaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Güncelle
      </Button>
      <Button color="alternative" onclick={() => (donemModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Dönem Sil Modal -->
<Modal bind:open={silDonemModal} title="Dönemi Sil" size="sm" autoclose={false}>
  <p class="text-gray-600 dark:text-gray-400">
    Bu dönem ve tüm toplantı/kararları kalıcı olarak silinecek. Emin misiniz?
  </p>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button color="red" onclick={donemSil} disabled={kaydediliyor}>
        {#if kaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Evet, Sil
      </Button>
      <Button color="alternative" onclick={() => (silDonemModal = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Toplantı Ekle/Düzenle Modal -->
<Modal
  bind:open={toplantIModalAcik}
  title={duzenlenecekToplanti ? 'Toplantıyı Düzenle' : 'Yeni Toplantı Ekle'}
  size="sm"
  autoclose={false}
>
  <div class="space-y-4">
    <div>
      <Label for="tTarih" class="mb-2">Tarih <span class="text-red-500">*</span></Label>
      <Input id="tTarih" type="date" bind:value={fTarih} required />
    </div>
    <div>
      <Label for="tKonu" class="mb-2">Konu <span class="text-red-500">*</span></Label>
      <Input id="tKonu" bind:value={fKonu} placeholder="Toplantı konusu" required />
    </div>
    <div>
      <Label for="tYer" class="mb-2">Yer</Label>
      <Input id="tYer" bind:value={fYer} placeholder="Toplantı yeri" />
    </div>
  </div>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button onclick={toplantIKaydet} disabled={kaydediliyor || !fTarih || !fKonu.trim()}>
        {#if kaydediliyor}<Spinner class="me-2" size="4" />{/if}
        {duzenlenecekToplanti ? 'Güncelle' : 'Ekle'}
      </Button>
      <Button color="alternative" onclick={() => (toplantIModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Toplantı Sil Modal -->
<Modal bind:open={silToplantIModal} title="Toplantıyı Sil" size="sm" autoclose={false}>
  <p class="text-gray-600 dark:text-gray-400">
    <strong class="text-gray-900 dark:text-white">{silinecekToplantI?.konu}</strong>
    toplantısı ve tüm kararları silinecek. Emin misiniz?
  </p>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button color="red" onclick={toplantISil} disabled={kaydediliyor}>
        {#if kaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Evet, Sil
      </Button>
      <Button color="alternative" onclick={() => (silToplantIModal = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Karar Ekle/Düzenle Modal -->
<Modal
  bind:open={kararModalAcik}
  title={duzenlenecekKarar ? 'Kararı Düzenle' : 'Yeni Karar Ekle'}
  size="sm"
  autoclose={false}
>
  <div class="space-y-4">
    <div>
      <Label for="kNo" class="mb-2">Karar No</Label>
      <Input id="kNo" type="number" min="1" bind:value={fKararNo} placeholder="örn. 1" />
    </div>
    <div>
      <Label for="kAciklama" class="mb-2">Karar Açıklaması <span class="text-red-500">*</span></Label>
      <Textarea id="kAciklama" bind:value={fAciklama} rows={4} placeholder="Kararın detayını yazın..." />
    </div>
  </div>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button onclick={kararKaydet} disabled={kaydediliyor || !fAciklama.trim()}>
        {#if kaydediliyor}<Spinner class="me-2" size="4" />{/if}
        {duzenlenecekKarar ? 'Güncelle' : 'Ekle'}
      </Button>
      <Button color="alternative" onclick={() => (kararModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Karar Sil Modal -->
<Modal bind:open={silKararModal} title="Kararı Sil" size="sm" autoclose={false}>
  <p class="text-gray-600 dark:text-gray-400">Bu kararı kalıcı olarak silmek istediğinize emin misiniz?</p>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button color="red" onclick={kararSil} disabled={kaydediliyor}>
        {#if kaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Evet, Sil
      </Button>
      <Button color="alternative" onclick={() => (silKararModal = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Borç Oluştur Onay Modal -->
<Modal bind:open={borcOlusturModal} title="Aidat Borcu Oluştur" size="sm" autoclose={false}>
  {#if borcSonuc}
    <div class="space-y-3">
      <div class="rounded-lg bg-green-50 p-4 dark:bg-green-900/20">
        <p class="text-sm font-medium text-green-800 dark:text-green-300">Borç kaydı başarıyla oluşturuldu!</p>
        <ul class="mt-2 space-y-1 text-sm text-green-700 dark:text-green-400">
          <li>• <strong>{borcSonuc.olusturulan}</strong> yeni borç kaydı oluşturuldu</li>
          <li>• <strong>{borcSonuc.otomatik_tahsil}</strong> borç otomatik tahsil edildi</li>
          <li>• <strong>{borcSonuc.tahsil_edilemeyen}</strong> borç tahsil edilemedi (bakiye yetersiz)</li>
        </ul>
      </div>
    </div>
  {:else}
    <div class="space-y-3">
      <p class="text-gray-600 dark:text-gray-400">
        Bu dönem için atanmış tüm hisselere <strong class="text-gray-900 dark:text-white">{donem ? formatAidat(donem.hisse_basi_aidat) : ''}</strong> tutarında aidat borcu oluşturulacak.
      </p>
      <div class="rounded-lg bg-blue-50 p-3 text-sm text-blue-700 dark:bg-blue-900/20 dark:text-blue-400">
        💡 Hissedarın cüzdanında yeterli bakiye varsa borç otomatik tahsil edilecektir.
      </div>
    </div>
  {/if}

  {#snippet footer()}
    <div class="flex gap-3">
      {#if borcSonuc}
        <Button onclick={() => { borcOlusturModal = false; borcSonuc = null; }}>Tamam</Button>
      {:else}
        <Button color="green" onclick={borcOlustur} disabled={borcOlusturuluyor}>
          {#if borcOlusturuluyor}<Spinner class="me-2" size="4" />{/if}
          Evet, Oluştur
        </Button>
        <Button color="alternative" onclick={() => (borcOlusturModal = false)}>İptal</Button>
      {/if}
    </div>
  {/snippet}
</Modal>

<!-- Tahsilat Modal -->
<Modal bind:open={tahsilatModal} title="Aidat Tahsilatı" size="lg" autoclose={false}>
  {#if tahsilatSonuc}
    <div class="space-y-3">
      <div class="rounded-lg bg-green-50 p-4 dark:bg-green-900/20">
        <p class="text-sm font-medium text-green-800 dark:text-green-300">Tahsilat başarıyla gerçekleşti!</p>
        <ul class="mt-2 space-y-1 text-sm text-green-700 dark:text-green-400">
          <li>• <strong>{tahsilatSonuc.tahsil_edilen_borc_sayisi}</strong> borç kaydı ödendi</li>
          <li>• Toplam <strong>{formatAidat(tahsilatSonuc.tahsil_edilen_toplam)}</strong> tahsil edildi</li>
          <li>• Güncel cüzdan bakiyesi: <strong>{formatAidat(tahsilatSonuc.yeni_bakiye)}</strong></li>
        </ul>
      </div>
    </div>
  {:else if tahsilatBorcRef}
    <div class="space-y-4">

      <!-- Hissedar & cüzdan bilgisi -->
      <div class="flex items-start justify-between rounded-lg bg-gray-50 p-3 dark:bg-gray-700">
        <div>
          <p class="font-semibold text-gray-900 dark:text-white">
            {hissedarLabelFromFields(tahsilatBorcRef.hissedar_id, tahsilatBorcRef.hissedar_ad, tahsilatBorcRef.hissedar_soyad, hissedarlar)}
          </p>
          <p class="text-xs text-gray-500 dark:text-gray-400">İşlem yapılan dönem: {tahsilatBorcRef.donem_adi}</p>
        </div>
        <div class="text-right">
          <p class="text-xs text-gray-500 dark:text-gray-400">Mevcut cüzdan bakiyesi</p>
          {#if tahsilatYukleniyor}
            <Spinner size="4" />
          {:else}
            <p class="font-semibold {tahsilatCuzdanBakiye >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}">
              {formatAidat(tahsilatCuzdanBakiye)}
            </p>
          {/if}
        </div>
      </div>

      <!-- Tüm ödenmemiş borçlar -->
      {#if tahsilatYukleniyor}
        <div class="flex justify-center py-3"><Spinner size="6" /></div>
      {:else if tahsilatHissedarBorclar.length > 0 || tahsilatHisseBorclar.length > 0}
        <div class="flex flex-col gap-3">
          <!-- Hisse satın alma borçları -->
          {#if tahsilatHisseBorclar.length > 0}
            <div>
              <p class="mb-2 text-sm font-medium text-gray-700 dark:text-gray-300">
                Hisse Satın Alma Borçları
              </p>
              <div class="rounded-lg border border-orange-200 dark:border-orange-700">
                {#each tahsilatHisseBorclar as h (h.id)}
                  <div class="flex items-center justify-between px-3 py-2 text-sm odd:bg-white even:bg-gray-50 dark:odd:bg-gray-800 dark:even:bg-gray-750">
                    <div class="flex items-center gap-2">
                      <span class="h-1.5 w-1.5 rounded-full bg-orange-400"></span>
                      <span class="font-medium text-gray-800 dark:text-gray-200">{h.bilgi}</span>
                    </div>
                    <span class="font-semibold text-red-600 dark:text-red-400">{formatAidat(h.borc)}</span>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Aidat borçları -->
          {#if tahsilatHissedarBorclar.length > 0}
            <div>
              <p class="mb-2 text-sm font-medium text-gray-700 dark:text-gray-300">
                Ödenmemiş Aidat Borçları
                <span class="ml-1 text-xs text-gray-400">(en eskiden yeniye — para bu sırayla uygulanır)</span>
              </p>
              <div class="max-h-48 overflow-y-auto rounded-lg border border-gray-200 dark:border-gray-600">
                {#each tahsilatHissedarBorclar as b (b.id)}
                  <div class="flex items-center justify-between px-3 py-2 text-sm
                    {b.id === tahsilatBorcRef.id
                      ? 'bg-yellow-50 dark:bg-yellow-900/20'
                      : 'odd:bg-white even:bg-gray-50 dark:odd:bg-gray-800 dark:even:bg-gray-750'}
                  ">
                    <div class="flex items-center gap-2">
                      {#if b.id === tahsilatBorcRef.id}
                        <span class="h-1.5 w-1.5 rounded-full bg-yellow-500"></span>
                      {:else}
                        <span class="h-1.5 w-1.5 rounded-full bg-gray-300 dark:bg-gray-600"></span>
                      {/if}
                      <span class="font-medium text-gray-800 dark:text-gray-200">{b.donem_adi}</span>
                      <Badge color="blue" class="text-xs">{b.hisse_sayisi} hisse</Badge>
                    </div>
                    <span class="font-semibold text-red-600 dark:text-red-400">{formatAidat(b.tutar)}</span>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <div class="flex justify-between text-xs text-gray-500 dark:text-gray-400 px-1">
            <span>{tahsilatHissedarBorclar.length + tahsilatHisseBorclar.length} borç kaydı</span>
            <span>Toplam: <strong class="text-gray-700 dark:text-gray-300">{formatAidat(tahsilatTumunuOdeTutar)}</strong></span>
          </div>
        </div>
      {:else}
        <p class="text-sm text-gray-400 italic">Başka ödenmemiş borç kaydı bulunamadı.</p>
      {/if}

      <!-- Tahsilat tutarı -->
      <div>
        <div class="mb-2 flex items-center justify-between">
          <Label for="tahsilatTutar">Tahsilat Tutarı (₺) <span class="text-red-500">*</span></Label>
          {#if tahsilatTumunuOdeTutar > 0}
            <button
              class="text-xs text-primary-600 hover:underline dark:text-primary-400"
              onclick={() => (tahsilatTutar = tahsilatTumunuOdeTutar.toFixed(2))}
            >
              Tümünü öde: {formatAidat(tahsilatTumunuOdeTutar)}
            </button>
          {/if}
        </div>
        <Input
          id="tahsilatTutar"
          type="number"
          min="0.01"
          step="0.01"
          bind:value={tahsilatTutar}
          placeholder="0.00"
        />
      </div>

      <div class="rounded-lg bg-blue-50 p-3 text-sm text-blue-700 dark:bg-blue-900/20 dark:text-blue-400">
        💡 Para önce hissedarın cüzdanına eklenir, ardından en eski borçtan başlayarak otomatik tahsil edilir.
      </div>
    </div>
  {/if}

  {#snippet footer()}
    <div class="flex gap-3">
      {#if tahsilatSonuc}
        <Button onclick={() => { tahsilatModal = false; tahsilatSonuc = null; }}>Tamam</Button>
      {:else}
        <Button color="green" onclick={tahsilatYap} disabled={tahsilEdiliyor || tahsilatYukleniyor || !tahsilatTutar || parseFloat(tahsilatTutar) <= 0}>
          {#if tahsilEdiliyor}<Spinner class="me-2" size="4" />{/if}
          Tahsilatı Kaydet
        </Button>
        <Button color="alternative" onclick={() => (tahsilatModal = false)}>İptal</Button>
      {/if}
    </div>
  {/snippet}
</Modal>
