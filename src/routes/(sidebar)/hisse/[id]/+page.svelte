<script lang="ts">
  import {
    Button,
    Badge,
    Modal,
    Label,
    Input,
    Textarea,
    Select,
    Spinner,
    TableBodyRow,
    TableBodyCell,
    Breadcrumb,
    BreadcrumbItem,
    Heading,
    P
  } from 'flowbite-svelte';
  import DataTable from '$lib/components/DataTable.svelte';
  import type { DataTableColumn } from '$lib/components/dataTableUtils';
  import {
    TrashBinSolid,
    ArrowLeftOutline,
    UsersSolid,
    LayersSolid,
    PlusOutline,
    CheckCircleSolid,
    CloseCircleSolid,
    MinusOutline,
    DollarOutline,
    FileLinesSolid
  } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import {
    hisseApi,
    hisseAtamaApi,
    hissedarApi,
    aidatApi,
    kasaApi,
    hisseSatisApi,
    type Hisse,
    type HisseAtama,
    type Hissedar,
    type AtamaInput,
    type HisseTransferInput,
    type AidatBorcu,
    type Kasa,
    type HisseSatis,
    type HisseSatisOdeme,
    type HisseSatisBaslatInput,
    type HisseSatisOdemeInput
  } from '$lib/tauri-api';
  import { hissedarLabel, hissedarLabelFromFields } from '$lib/hissedarFormat';
  import { exportPdf, formatTL, formatTarih } from '$lib/pdf';

  // ─── State ──────────────────────────────────────────────────────────────────

  let hisse = $state<Hisse | null>(null);
  let atamalar = $state<HisseAtama[]>([]);
  let hissedarlar = $state<Hissedar[]>([]);
  let borclar = $state<AidatBorcu[]>([]);
  let kasalar = $state<Kasa[]>([]);
  let aktifSatis = $state<HisseSatis | null>(null);
  let satisOdemeleri = $state<HisseSatisOdeme[]>([]);
  let yukleniyor = $state(true);
  let hata = $state('');

  const hisseId = $derived(Number($page.params.id));

  // Atama modal
  let atamaModalAcik = $state(false);
  let atamaHissedarId = $state<number | ''>('');
  let atamaTarih = $state(bugun());
  let atamaUcret = $state('0');
  let atamaAciklama = $state('');
  let atamaKaydediliyor = $state(false);

  // Silme modal
  let silModalAcik = $state(false);
  let silinecekAtamaId = $state<number | null>(null);
  let silKaydediliyor = $state(false);

  // Transfer modal
  let transferModalAcik = $state(false);
  let transferHissedarId = $state<number | ''>('');
  let transferTarih = $state(bugun());
  let transferUcret = $state('0');
  let transferAciklama = $state('');
  let transferKaydediliyor = $state(false);

  // Satış başlat modal
  let satisModalAcik = $state(false);
  let satisKasaId = $state<number | ''>('');
  let satisTutar = $state('0');
  let satisTarih = $state(bugun());
  let satisAciklama = $state('');
  let satisKaydediliyor = $state(false);

  // Satış ödeme modal
  let odemeModalAcik = $state(false);
  let odemeTutar = $state('0');
  let odemeTarih = $state(bugun());
  let odemeAciklama = $state('');
  let odemeKaydediliyor = $state(false);

  // ─── Yükle ─────────────────────────────────────────────────────────────────

  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      const [h, a, hs, b, k, s] = await Promise.all([
        hisseApi.get(hisseId),
        hisseAtamaApi.getByHisse(hisseId),
        hissedarApi.getAll(),
        aidatApi.getByHisse(hisseId),
        kasaApi.getAll(),
        hisseSatisApi.getAktif(hisseId)
      ]);
      hisse = h;
      atamalar = a;
      hissedarlar = hs;
      borclar = b;
      kasalar = k;
      aktifSatis = s;
      satisOdemeleri = s ? await hisseSatisApi.getOdemeler(s.id) : [];
    } catch (e: any) {
      hata = e?.toString() ?? 'Yükleme hatası';
    } finally {
      yukleniyor = false;
    }
  }

  $effect(() => {
    if (hisseId) yukle();
  });

  // ─── Atama Modal ────────────────────────────────────────────────────────────

  const aktifHissedarlar = $derived(hissedarlar.filter((h) => h.aktif));

  function atamaAc() {
    atamaHissedarId = '';
    atamaTarih = bugun();
    atamaUcret = '0';
    atamaAciklama = '';
    atamaModalAcik = true;
  }

  async function atamaKaydet() {
    if (!atamaHissedarId) return;
    const ucret = parseFloat(String(atamaUcret).replace(',', '.'));
    if (isNaN(ucret) || ucret < 0) return;

    atamaKaydediliyor = true;
    hata = '';
    try {
      const input: AtamaInput = {
        hisse_id: hisseId,
        hissedar_id: Number(atamaHissedarId),
        tarih: atamaTarih,
        ucret,
        aciklama: atamaAciklama.trim() || undefined
      };
      await hisseAtamaApi.ata(input);
      atamaModalAcik = false;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Atama hatası';
    } finally {
      atamaKaydediliyor = false;
    }
  }

  // ─── Atama Sil ──────────────────────────────────────────────────────────────

  function silAc(id: number) {
    silinecekAtamaId = id;
    silModalAcik = true;
  }

  async function atamaSil() {
    if (silinecekAtamaId === null) return;
    silKaydediliyor = true;
    hata = '';
    try {
      await hisseAtamaApi.sil(silinecekAtamaId);
      silModalAcik = false;
      silinecekAtamaId = null;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Silme hatası';
    } finally {
      silKaydediliyor = false;
    }
  }

  // ─── Transfer ──────────────────────────────────────────────────────────────

  // En güncel atama = hissenin mevcut sahibi (atamalar created_at DESC sıralı gelir)
  const mevcutSahip = $derived(atamalar.length > 0 ? atamalar[0] : null);

  // Transfer'de mevcut sahip listeden hariç tutulur
  const transferHedefAdaylari = $derived(
    aktifHissedarlar.filter((h) => h.id !== mevcutSahip?.hissedar_id)
  );

  function transferAc() {
    transferHissedarId = '';
    transferTarih = bugun();
    transferUcret = '0';
    transferAciklama = '';
    transferModalAcik = true;
  }

  async function transferKaydet() {
    if (!transferHissedarId || !mevcutSahip) return;
    const ucret = parseFloat(String(transferUcret).replace(',', '.'));
    if (isNaN(ucret) || ucret < 0) return;

    transferKaydediliyor = true;
    hata = '';
    try {
      const input: HisseTransferInput = {
        hisse_id: hisseId,
        yeni_hissedar_id: Number(transferHissedarId),
        tarih: transferTarih,
        ucret,
        aciklama: transferAciklama.trim() || undefined
      };
      await hisseAtamaApi.transfer(input);
      transferModalAcik = false;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Transfer hatası';
    } finally {
      transferKaydediliyor = false;
    }
  }

  // ─── Satış ──────────────────────────────────────────────────────────────────

  function satisModalAc() {
    satisKasaId = kasalar[0]?.id ?? '';
    satisTutar = '0';
    satisTarih = bugun();
    satisAciklama = '';
    hata = '';
    satisModalAcik = true;
  }

  async function satisiBaslat() {
    if (satisKasaId === '') return;
    const tutar = parseFloat(satisTutar);
    if (isNaN(tutar) || tutar <= 0) {
      hata = 'Satış tutarı pozitif olmalıdır';
      return;
    }
    satisKaydediliyor = true;
    hata = '';
    try {
      const input: HisseSatisBaslatInput = {
        hisse_id: hisseId,
        kasa_id: Number(satisKasaId),
        satis_tutari: tutar,
        tarih: satisTarih,
        aciklama: satisAciklama.trim() || undefined
      };
      await hisseSatisApi.baslat(input);
      satisModalAcik = false;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Satış başlatma hatası';
    } finally {
      satisKaydediliyor = false;
    }
  }

  function odemeModalAc() {
    if (!aktifSatis) return;
    odemeTutar = aktifSatis.kalan_tutar.toFixed(2);
    odemeTarih = bugun();
    odemeAciklama = '';
    hata = '';
    odemeModalAcik = true;
  }

  async function odemeyiEkle() {
    if (!aktifSatis) return;
    const tutar = parseFloat(odemeTutar);
    if (isNaN(tutar) || tutar <= 0) {
      hata = 'Ödeme tutarı pozitif olmalıdır';
      return;
    }
    if (tutar > aktifSatis.kalan_tutar + 0.001) {
      hata = `Ödeme kalan tutardan büyük olamaz (kalan: ${formatAidat(aktifSatis.kalan_tutar)})`;
      return;
    }
    odemeKaydediliyor = true;
    hata = '';
    try {
      const input: HisseSatisOdemeInput = {
        satis_id: aktifSatis.id,
        tutar,
        tarih: odemeTarih,
        aciklama: odemeAciklama.trim() || undefined
      };
      await hisseSatisApi.odemeEkle(input);
      odemeModalAcik = false;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Ödeme ekleme hatası';
    } finally {
      odemeKaydediliyor = false;
    }
  }

  async function satisiIptalEt() {
    if (!aktifSatis) return;
    if (!confirm('Bu satış iptal edilecek. Devam etmek istiyor musunuz?')) return;
    try {
      await hisseSatisApi.iptal(aktifSatis.id);
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'İptal hatası';
    }
  }

  // ─── Yardımcılar ────────────────────────────────────────────────────────────

  function bugun(): string {
    return new Date().toISOString().slice(0, 10);
  }

  function tarihFormat(t: string): string {
    return new Date(t).toLocaleDateString('tr-TR', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric'
    });
  }

  function paraBirim(id: number): string {
    const h = hissedarlar.find((hs) => hs.id === id);
    return h ? `(kasa_id: ${h.kasa_id})` : '';
  }

  function formatAidat(aidat: number): string {
    return new Intl.NumberFormat('tr-TR', { style: 'currency', currency: 'TRY', minimumFractionDigits: 2 }).format(aidat);
  }

  // ─── Kolon tanımları ────────────────────────────────────────────────
  const atamaKolonlar: DataTableColumn<HisseAtama>[] = [
    { id: 'tarih', header: 'Tarih', accessor: 'tarih' },
    { id: 'hissedar', header: 'Hissedar', accessor: (a) => hissedarLabelFromFields(a.hissedar_id, a.hissedar_ad, a.hissedar_soyad, hissedarlar) },
    { id: 'ucret', header: 'Ücret', accessor: 'ucret', align: 'right' },
    { id: 'aciklama', header: 'Açıklama', accessor: (a) => a.aciklama ?? '' },
    { id: 'islemler', header: '', accessor: () => '', sortable: false, searchable: false }
  ];

  const borcKolonlar: DataTableColumn<AidatBorcu>[] = [
    { id: 'donem', header: 'Dönem', accessor: (b) => b.donem_adi ?? '' },
    { id: 'hissedar', header: 'Hissedar', accessor: (b) => hissedarLabelFromFields(b.hissedar_id, b.hissedar_ad, b.hissedar_soyad, hissedarlar) },
    { id: 'tutar', header: 'Tutar', accessor: 'tutar', align: 'right' },
    { id: 'durum', header: 'Durum', accessor: (b) => (b.odendi ? 'Tahsil Edildi' : 'Ödenmedi') },
    { id: 'odeme_tarihi', header: 'Ödeme Tarihi', accessor: (b) => b.odeme_tarihi ?? '' }
  ];

  // ─── PDF ────────────────────────────────────────────────────────────────────

  function pdfIndir() {
    if (!hisse) return;
    const h = hisse;
    const sahip = mevcutSahip
      ? `${mevcutSahip.hissedar_ad ?? ''} ${mevcutSahip.hissedar_soyad ?? ''}`.trim()
      : '-';
    const durumMetni =
      h.durum === 'musait' ? 'Müsait' : h.durum === 'atanmis' ? 'Atanmış' : 'Satıldı';
    const toplamUcret = atamalar.reduce((s, a) => s + (a.ucret ?? 0), 0);
    const toplamBorc = borclar.reduce((s, b) => s + b.tutar, 0);
    const tahsilBorc = borclar.filter((b) => b.odendi).reduce((s, b) => s + b.tutar, 0);

    const sections: any[] = [
      {
        kind: 'kv',
        heading: 'Hisse Bilgileri',
        columns: 2,
        items: [
          { label: 'Kod', value: h.kod },
          { label: 'Durum', value: durumMetni },
          { label: 'Mevcut Sahip', value: sahip },
          { label: 'Toplam Atama', value: atamalar.length },
          { label: 'Toplam Ücret', value: formatTL(toplamUcret) },
          { label: 'Açıklama', value: h.aciklama ?? '-' }
        ]
      }
    ];

    if (aktifSatis) {
      sections.push({
        kind: 'kv',
        heading: 'Aktif Satış',
        columns: 2,
        items: [
          { label: 'Tarih', value: formatTarih(aktifSatis.tarih) },
          { label: 'Kasa', value: aktifSatis.kasa_ad ?? '-' },
          { label: 'Satış Tutarı', value: formatTL(aktifSatis.satis_tutari) },
          { label: 'Ödenen', value: formatTL(aktifSatis.odenen_tutar) },
          { label: 'Kalan', value: formatTL(aktifSatis.kalan_tutar) },
          { label: 'Açıklama', value: aktifSatis.aciklama ?? '-' }
        ]
      });
      if (satisOdemeleri.length > 0) {
        sections.push({
          kind: 'table',
          heading: 'Satış Ödemeleri',
          columns: ['Tarih', 'Tutar', 'Açıklama'],
          widths: ['auto', 'auto', '*'],
          rows: satisOdemeleri.map((o) => [formatTarih(o.tarih), formatTL(o.tutar), o.aciklama ?? ''])
        });
      }
    }

    if (atamalar.length > 0) {
      sections.push({
        kind: 'table',
        heading: 'Atamalar',
        columns: ['Tarih', 'Hissedar', 'Ücret', 'Açıklama'],
        widths: ['auto', '*', 'auto', '*'],
        rows: atamalar.map((a) => [
          formatTarih(a.tarih),
          `${a.hissedar_ad ?? ''} ${a.hissedar_soyad ?? ''}`.trim(),
          formatTL(a.ucret ?? 0),
          a.aciklama ?? ''
        ])
      });
    }

    if (borclar.length > 0) {
      sections.push({
        kind: 'kv',
        heading: 'Aidat Özeti',
        columns: 2,
        items: [
          { label: 'Toplam Borç', value: formatTL(toplamBorc) },
          { label: 'Tahsil Edilen', value: formatTL(tahsilBorc) },
          { label: 'Kalan', value: formatTL(toplamBorc - tahsilBorc) },
          { label: 'Kayıt Sayısı', value: borclar.length }
        ]
      });
      sections.push({
        kind: 'table',
        heading: 'Aidat Borçları',
        columns: ['Dönem', 'Hissedar', 'Tutar', 'Durum', 'Ödeme Tarihi'],
        widths: ['auto', '*', 'auto', 'auto', 'auto'],
        rows: borclar.map((b) => [
          b.donem_adi ?? '-',
          `${b.hissedar_ad ?? ''} ${b.hissedar_soyad ?? ''}`.trim(),
          formatTL(b.tutar),
          b.odendi ? 'Tahsil Edildi' : 'Ödenmedi',
          b.odeme_tarihi ? formatTarih(b.odeme_tarihi) : '-'
        ])
      });
    }

    exportPdf({
      title: `Hisse: ${h.kod}`,
      subtitle: durumMetni,
      fileName: `hisse-${h.kod}`,
      sections
    });
  }
</script>

<main class="min-h-screen bg-gray-50 p-6 dark:bg-gray-900">

  <!-- Breadcrumb -->
  <div class="mb-5 flex items-center justify-between gap-3">
    <Breadcrumb>
      <BreadcrumbItem href="/hisse">Hisseler</BreadcrumbItem>
      <BreadcrumbItem>{hisse?.kod ?? '...'}</BreadcrumbItem>
    </Breadcrumb>
    {#if hisse}
      <Button size="sm" color="alternative" class="gap-2" onclick={pdfIndir}>
        <FileLinesSolid class="h-4 w-4" /> PDF
      </Button>
    {/if}
  </div>

  <!-- Hata -->
  {#if hata}
    <div class="mb-4 rounded-lg bg-red-50 p-4 text-sm text-red-700 dark:bg-red-900/20 dark:text-red-400">
      {hata}
    </div>
  {/if}

  {#if yukleniyor}
    <div class="flex h-48 items-center justify-center">
      <Spinner size="10" />
    </div>

  {:else if hisse}

    <!-- Hisse Bilgi Kartı -->
    <div class="mb-6 rounded-xl border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800">
      <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div class="flex items-center gap-4">
          <div class="flex h-12 w-12 items-center justify-center rounded-full bg-indigo-100 dark:bg-indigo-900">
            <LayersSolid class="h-6 w-6 text-indigo-600 dark:text-indigo-300" />
          </div>
          <div>
            <Heading tag="h1" class="font-mono text-2xl font-bold text-gray-900 dark:text-white">
              {hisse.kod}
            </Heading>
            {#if hisse.aciklama}
              <P class="text-sm text-gray-500 dark:text-gray-400">{hisse.aciklama}</P>
            {/if}
          </div>
          <Badge
            color={hisse.durum === 'musait' ? 'green' : hisse.durum === 'atanmis' ? 'yellow' : 'dark'}
            class="ml-2"
          >
            {hisse.durum === 'musait' ? 'Müsait' : hisse.durum === 'atanmis' ? 'Atanmış' : 'Satıldı'}
          </Badge>
        </div>

        <div class="flex items-center gap-3">
          <div class="rounded-lg bg-indigo-50 px-4 py-3 dark:bg-indigo-900/20">
            <p class="text-xs text-indigo-600 dark:text-indigo-400">Toplam Atama</p>
            <p class="text-lg font-bold text-indigo-700 dark:text-indigo-300">{atamalar.length}</p>
          </div>
          <div class="rounded-lg bg-gray-100 px-4 py-3 dark:bg-gray-700">
            <p class="text-xs text-gray-500 dark:text-gray-400">Toplam Ücret</p>
            <p class="text-lg font-bold text-gray-900 dark:text-white">
              {atamalar.reduce((s, a) => s + a.ucret, 0).toLocaleString('tr-TR', { minimumFractionDigits: 2 })} ₺
            </p>
          </div>
        </div>
      </div>
    </div>

    <!-- Satıldı Banner -->
    {#if hisse.durum === 'satildi'}
      <div
        class="mb-4 rounded-lg border border-gray-300 bg-gray-100 p-4 dark:border-gray-600 dark:bg-gray-800"
      >
        <div class="flex items-center gap-3">
          <CloseCircleSolid class="h-6 w-6 text-gray-700 dark:text-gray-300" />
          <div>
            <p class="font-semibold text-gray-900 dark:text-white">Bu hisse sisteme satılmış</p>
            <p class="text-sm text-gray-600 dark:text-gray-400">
              Satılmış hisseler yeni atama veya transfer için kullanılamaz.
            </p>
          </div>
        </div>
      </div>
    {/if}

    <!-- Aktif Satış Paneli -->
    {#if aktifSatis && !aktifSatis.tamamlandi && !aktifSatis.iptal}
      {@const yuzde = Math.min(
        100,
        (aktifSatis.odenen_tutar / Math.max(aktifSatis.satis_tutari, 0.01)) * 100
      )}
      <div
        class="mb-4 rounded-lg border border-red-200 bg-red-50 p-4 dark:border-red-800 dark:bg-red-900/20"
      >
        <div class="mb-3 flex items-center justify-between">
          <div class="flex items-center gap-3">
            <DollarOutline class="h-6 w-6 text-red-600 dark:text-red-400" />
            <div>
              <p class="font-semibold text-gray-900 dark:text-white">
                Devam Eden Satış — {aktifSatis.hissedar_ad} {aktifSatis.hissedar_soyad}
              </p>
              <p class="text-xs text-gray-600 dark:text-gray-400">
                {tarihFormat(aktifSatis.tarih)} • Kasa: {aktifSatis.kasa_ad}
                {#if aktifSatis.aciklama}• {aktifSatis.aciklama}{/if}
              </p>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <Button size="sm" color="primary" class="gap-2" onclick={odemeModalAc}>
              <PlusOutline class="h-4 w-4" /> Ödeme Ekle
            </Button>
            {#if aktifSatis.odenen_tutar === 0}
              <Button size="xs" color="alternative" onclick={satisiIptalEt}>İptal</Button>
            {/if}
          </div>
        </div>

        <div class="mb-2 grid grid-cols-3 gap-3">
          <div class="rounded bg-white p-2 text-center dark:bg-gray-800">
            <p class="text-xs text-gray-500 dark:text-gray-400">Satış Tutarı</p>
            <p class="text-base font-bold text-gray-900 dark:text-white">
              {formatAidat(aktifSatis.satis_tutari)}
            </p>
          </div>
          <div class="rounded bg-white p-2 text-center dark:bg-gray-800">
            <p class="text-xs text-green-600 dark:text-green-400">Ödenen</p>
            <p class="text-base font-bold text-green-700 dark:text-green-400">
              {formatAidat(aktifSatis.odenen_tutar)}
            </p>
          </div>
          <div class="rounded bg-white p-2 text-center dark:bg-gray-800">
            <p class="text-xs text-red-600 dark:text-red-400">Kalan</p>
            <p class="text-base font-bold text-red-700 dark:text-red-400">
              {formatAidat(aktifSatis.kalan_tutar)}
            </p>
          </div>
        </div>

        <div class="h-2 w-full overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700">
          <div
            class="h-full bg-gradient-to-r from-green-500 to-emerald-500 transition-all"
            style="width: {yuzde}%"
          ></div>
        </div>

        {#if satisOdemeleri.length > 0}
          <div class="mt-3">
            <p class="mb-1 text-xs font-semibold uppercase tracking-wide text-gray-600 dark:text-gray-400">
              Ödemeler ({satisOdemeleri.length})
            </p>
            <div class="max-h-40 overflow-y-auto rounded border border-red-100 dark:border-red-800">
              <table class="w-full text-sm">
                <thead class="bg-red-100/50 text-xs uppercase text-gray-600 dark:bg-red-900/30 dark:text-gray-300">
                  <tr>
                    <th class="px-3 py-1.5 text-left">Tarih</th>
                    <th class="px-3 py-1.5 text-right">Tutar</th>
                    <th class="px-3 py-1.5 text-left">Açıklama</th>
                  </tr>
                </thead>
                <tbody>
                  {#each satisOdemeleri as o (o.id)}
                    <tr class="border-t border-red-100 dark:border-red-800">
                      <td class="px-3 py-1.5 text-gray-700 dark:text-gray-300">{tarihFormat(o.tarih)}</td>
                      <td class="px-3 py-1.5 text-right font-medium text-gray-900 dark:text-white">
                        {formatAidat(o.tutar)}
                      </td>
                      <td class="px-3 py-1.5 text-gray-600 dark:text-gray-400">{o.aciklama ?? ''}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Araç Çubuğu -->
    <div class="mb-4 flex items-center justify-between">
      <Heading tag="h2" class="text-lg font-semibold text-gray-900 dark:text-white">
        Atamalar
        <span class="ml-2 text-sm font-normal text-gray-500">({atamalar.length} kayıt)</span>
      </Heading>
      <div class="flex items-center gap-2">
        {#if hisse?.durum === 'atanmis' && !aktifSatis}
          <Button size="sm" color="red" class="gap-2" onclick={satisModalAc}>
            <MinusOutline class="h-4 w-4" /> Sisteme Sat
          </Button>
        {/if}
        {#if mevcutSahip && !aktifSatis && hisse?.durum !== 'satildi'}
          <Button size="sm" color="alternative" class="gap-2" onclick={transferAc}>
            <UsersSolid class="h-4 w-4" /> Transfer Et
          </Button>
        {/if}
        {#if hisse?.durum === 'musait'}
          <Button size="sm" color="primary" class="gap-2" onclick={atamaAc}>
            <PlusOutline class="h-4 w-4" /> Hissedara Ata
          </Button>
        {/if}
      </div>
    </div>

    <!-- Atamalar Tablosu -->
    <DataTable
      data={atamalar}
      columns={atamaKolonlar}
      searchPlaceholder="Hissedar veya açıklama ara..."
      exportFileName="hisse-atamalar-{hisse?.kod ?? ''}"
      emptyMessage="Bu hisse henüz hiç hissedara atanmamış"
    >
      {#snippet row(atama, _i, visibleCols)}
        <TableBodyRow>
          {#if visibleCols.has('tarih')}
            <TableBodyCell class="text-sm text-gray-500 dark:text-gray-400">
              {tarihFormat(atama.tarih)}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('hissedar')}
            <TableBodyCell>
              <div class="flex items-center gap-2">
                <div class="flex h-8 w-8 items-center justify-center rounded-full bg-gray-100 dark:bg-gray-700">
                  <UsersSolid class="h-4 w-4 text-gray-500" />
                </div>
                <span class="font-medium text-gray-900 dark:text-white">
                  {hissedarLabelFromFields(atama.hissedar_id, atama.hissedar_ad, atama.hissedar_soyad, hissedarlar)}
                </span>
              </div>
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('ucret')}
            <TableBodyCell class="text-right font-semibold {atama.ucret > 0 ? 'text-red-600 dark:text-red-400' : 'text-gray-500 dark:text-gray-400'}">
              {atama.ucret > 0
                ? atama.ucret.toLocaleString('tr-TR', { minimumFractionDigits: 2 }) + ' ₺'
                : '—'}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('aciklama')}
            <TableBodyCell class="text-sm text-gray-500 dark:text-gray-400">
              {#if atama.aciklama}
                {atama.aciklama}
              {:else}
                <span class="text-gray-300 dark:text-gray-600">—</span>
              {/if}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('islemler')}
            <TableBodyCell class="text-center">
              <button
                class="rounded p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-gray-700"
                onclick={() => silAc(atama.id)}
                title="Atamayı Sil"
              >
                <TrashBinSolid class="h-4 w-4" />
              </button>
            </TableBodyCell>
          {/if}
        </TableBodyRow>
      {/snippet}
      {#snippet empty()}
        <div class="flex flex-col items-center justify-center py-6">
          <UsersSolid class="mb-3 h-10 w-10 text-gray-400" />
          <P class="text-gray-500 dark:text-gray-400">Bu hisse henüz hiç hissedara atanmamış</P>
          <Button size="sm" class="mt-4 gap-2" onclick={atamaAc}>
            <PlusOutline class="h-4 w-4" /> Hissedara Ata
          </Button>
        </div>
      {/snippet}
    </DataTable>

    <!-- Aidat Borçları -->
    <div class="mb-4 mt-8 flex items-center justify-between">
      <Heading tag="h2" class="text-lg font-semibold text-gray-900 dark:text-white">
        Aidat Borçları
        <span class="ml-2 text-sm font-normal text-gray-500">({borclar.length} kayıt)</span>
      </Heading>
    </div>

    <DataTable
      data={borclar}
      columns={borcKolonlar}
      searchPlaceholder="Dönem veya hissedar ara..."
      exportFileName="hisse-borclar-{hisse?.kod ?? ''}"
      emptyMessage="Bu hisse için aidat borcu bulunmuyor"
    >
      {#snippet row(b, _i, visibleCols)}
        <TableBodyRow>
          {#if visibleCols.has('donem')}
            <TableBodyCell>
              <span class="font-medium text-gray-900 dark:text-white">{b.donem_adi}</span>
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('hissedar')}
            <TableBodyCell>
              {hissedarLabelFromFields(b.hissedar_id, b.hissedar_ad, b.hissedar_soyad, hissedarlar)}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('tutar')}
            <TableBodyCell class="text-right">
              <span class="font-semibold">{formatAidat(b.tutar)}</span>
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('durum')}
            <TableBodyCell>
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
        </TableBodyRow>
      {/snippet}
    </DataTable>

    <!-- Geri Butonu -->
    <div class="mt-6">
      <button
        class="flex items-center gap-2 text-sm text-gray-500 hover:text-primary-600 dark:text-gray-400 dark:hover:text-primary-400"
        onclick={() => goto('/hisse')}
      >
        <ArrowLeftOutline class="h-4 w-4" /> Tüm Hisselere Dön
      </button>
    </div>

  {:else}
    <p class="text-gray-500">Hisse bulunamadı.</p>
  {/if}
</main>

<!-- Hissedara Ata Modal -->
<Modal bind:open={atamaModalAcik} title="Hissedara Ata" size="md" autoclose={false}>

  <div class="flex flex-col gap-4">

    {#if hisse}
      <div class="rounded-lg border border-indigo-200 bg-indigo-50 p-3 dark:border-indigo-700 dark:bg-indigo-900/20">
        <p class="text-xs font-semibold uppercase tracking-wide text-indigo-600 dark:text-indigo-400">Hisse</p>
        <p class="font-mono font-bold text-indigo-700 dark:text-indigo-300">{hisse.kod}</p>
      </div>
    {/if}

    <div>
      <Label for="atamaHissedar" class="mb-2">Hissedar *</Label>
      <Select
        id="atamaHissedar"
        bind:value={atamaHissedarId}
        items={aktifHissedarlar.map((h) => ({
          value: h.id,
          name: hissedarLabel(h)
        }))}
        placeholder="Hissedar seçin..."
      />
    </div>

    <div>
      <Label for="atamaTarih" class="mb-2">Tarih *</Label>
      <Input id="atamaTarih" type="date" bind:value={atamaTarih} required />
    </div>

    <div>
      <Label for="atamaUcret" class="mb-2">Ücret (₺) — 0 girilebilir</Label>
      <Input
        id="atamaUcret"
        type="number"
        step="0.01"
        min="0"
        bind:value={atamaUcret}
        placeholder="0.00"
      />
      {#if parseFloat(String(atamaUcret).replace(',', '.')) > 0}
        <p class="mt-1 text-xs text-orange-600 dark:text-orange-400">
          Bu tutar hissedarın cüzdanına borç olarak işlenecektir.
        </p>
      {/if}
    </div>

    <div>
      <Label for="atamaAciklama" class="mb-2">Açıklama (isteğe bağlı)</Label>
      <Textarea
        id="atamaAciklama"
        bind:value={atamaAciklama}
        rows={2}
        placeholder="Atama açıklaması..."
      />
    </div>
  </div>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button
        color="primary"
        onclick={atamaKaydet}
        disabled={atamaKaydediliyor || !atamaHissedarId}
      >
        {#if atamaKaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Atamayı Kaydet
      </Button>
      <Button color="alternative" onclick={() => (atamaModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Transfer Modal -->
<Modal bind:open={transferModalAcik} title="Hisseyi Transfer Et" size="md" autoclose={false}>
  <div class="flex flex-col gap-4">
    {#if hisse && mevcutSahip}
      <div class="rounded-lg border border-amber-200 bg-amber-50 p-3 dark:border-amber-700 dark:bg-amber-900/20">
        <p class="text-xs font-semibold uppercase tracking-wide text-amber-700 dark:text-amber-400">
          Hisse Transferi
        </p>
        <p class="mt-1 text-sm text-gray-700 dark:text-gray-300">
          <span class="font-mono font-bold text-amber-800 dark:text-amber-300">{hisse.kod}</span>
          <span class="mx-2 text-gray-500">•</span>
          Mevcut sahip:
          <span class="font-medium text-gray-900 dark:text-white">
            {mevcutSahip.hissedar_ad} {mevcutSahip.hissedar_soyad}
          </span>
        </p>
        <p class="mt-2 text-xs text-amber-700 dark:text-amber-400">
          Transfer sonrası önceki atama kayıtları silinmez; hisse geçmişi korunur.
        </p>
      </div>
    {/if}

    <div>
      <Label for="transferHissedar" class="mb-2">Yeni Hissedar <span class="text-red-500">*</span></Label>
      <Select
        id="transferHissedar"
        bind:value={transferHissedarId}
        items={transferHedefAdaylari.map((h) => ({
          value: h.id,
          name: hissedarLabel(h)
        }))}
        placeholder="Hissedar seçin..."
      />
    </div>

    <div>
      <Label for="transferTarih" class="mb-2">Tarih <span class="text-red-500">*</span></Label>
      <Input id="transferTarih" type="date" bind:value={transferTarih} required />
    </div>

    <div>
      <Label for="transferUcret" class="mb-2">Transfer Ücreti (₺) — 0 girilebilir</Label>
      <Input
        id="transferUcret"
        type="number"
        step="0.01"
        min="0"
        bind:value={transferUcret}
        placeholder="0.00"
      />
      <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
        Ücret &gt; 0 ise alıcı cüzdanına borç, satıcı cüzdanına alacak işlenir. Alıcı bakiyesi yeterse
        kasalar arası ödeme de otomatik yapılır.
      </p>
    </div>

    <div>
      <Label for="transferAciklama" class="mb-2">Açıklama</Label>
      <Textarea id="transferAciklama" bind:value={transferAciklama} rows={2} placeholder="Opsiyonel not" />
    </div>

    {#if hata}
      <div class="rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700 dark:border-red-700 dark:bg-red-900/20 dark:text-red-400">
        {hata}
      </div>
    {/if}
  </div>

  {#snippet footer()}
    <div class="flex justify-end gap-2">
      <Button
        color="primary"
        onclick={transferKaydet}
        disabled={transferKaydediliyor || !transferHissedarId}
      >
        {#if transferKaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Transferi Onayla
      </Button>
      <Button color="alternative" onclick={() => (transferModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Atama Sil Onay Modal -->
<Modal bind:open={silModalAcik} title="Atamayı Sil" size="sm" autoclose={false}>
  <p class="text-gray-600 dark:text-gray-400">
    Bu atamayı silmek istediğinize emin misiniz? Kasa bakiyesi geri alınmaz.
  </p>
  {#snippet footer()}
    <div class="flex gap-3">
      <Button color="red" onclick={atamaSil} disabled={silKaydediliyor}>
        {#if silKaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Evet, Sil
      </Button>
      <Button color="alternative" onclick={() => (silModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Satış Başlat Modal -->
<Modal bind:open={satisModalAcik} title="Hisseyi Sisteme Sat" size="md" autoclose={false}>
  <div class="flex flex-col gap-4">
    {#if hisse && mevcutSahip}
      <div class="rounded-lg border border-red-200 bg-red-50 p-3 dark:border-red-800 dark:bg-red-900/20">
        <p class="text-xs font-semibold uppercase tracking-wide text-red-700 dark:text-red-400">
          Satış Başlatılıyor
        </p>
        <p class="mt-1 text-sm text-gray-700 dark:text-gray-300">
          <span class="font-mono font-bold text-red-800 dark:text-red-300">{hisse.kod}</span>
          <span class="mx-2 text-gray-500">•</span>
          Satıcı:
          <span class="font-medium text-gray-900 dark:text-white">
            {mevcutSahip.hissedar_ad} {mevcutSahip.hissedar_soyad}
          </span>
        </p>
        <p class="mt-2 text-xs text-red-700 dark:text-red-400">
          Satış başladıktan sonra transfer ve yeniden atama kilitlenir. Ödemeler kısmi yapılabilir;
          toplam ödeme satış tutarına ulaşınca hisse kullanılamaz hale gelir.
        </p>
      </div>
    {/if}

    <div>
      <Label for="satisTutar" class="mb-2">Satış Tutarı (₺) <span class="text-red-500">*</span></Label>
      <Input
        id="satisTutar"
        type="number"
        step="0.01"
        min="0.01"
        bind:value={satisTutar}
        placeholder="0.00"
      />
    </div>

    <div>
      <Label for="satisKasa" class="mb-2">Ödeme Kasası <span class="text-red-500">*</span></Label>
      <Select
        id="satisKasa"
        bind:value={satisKasaId}
        items={kasalar.map((k) => ({
          value: k.id,
          name: `${k.ad} (${k.bakiye.toLocaleString('tr-TR', { minimumFractionDigits: 2 })} ₺)`
        }))}
        placeholder="Kasa seçin..."
      />
    </div>

    <div>
      <Label for="satisTarih" class="mb-2">Tarih <span class="text-red-500">*</span></Label>
      <Input id="satisTarih" type="date" bind:value={satisTarih} required />
    </div>

    <div>
      <Label for="satisAciklama" class="mb-2">Açıklama</Label>
      <Textarea id="satisAciklama" bind:value={satisAciklama} rows={2} placeholder="Opsiyonel not" />
    </div>

    {#if hata}
      <div class="rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700 dark:border-red-700 dark:bg-red-900/20 dark:text-red-400">
        {hata}
      </div>
    {/if}
  </div>

  {#snippet footer()}
    <div class="flex justify-end gap-2">
      <Button
        color="red"
        onclick={satisiBaslat}
        disabled={satisKaydediliyor || satisKasaId === '' || parseFloat(satisTutar) <= 0}
      >
        {#if satisKaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Satışı Başlat
      </Button>
      <Button color="alternative" onclick={() => (satisModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Satış Ödeme Ekle Modal -->
<Modal bind:open={odemeModalAcik} title="Satış Ödemesi Ekle" size="md" autoclose={false}>
  <div class="flex flex-col gap-4">
    {#if aktifSatis}
      <div class="rounded-lg border border-amber-200 bg-amber-50 p-3 dark:border-amber-700 dark:bg-amber-900/20">
        <p class="text-sm text-gray-700 dark:text-gray-300">
          Kalan borç:
          <span class="font-bold text-amber-800 dark:text-amber-300">
            {formatAidat(aktifSatis.kalan_tutar)}
          </span>
          <span class="mx-2 text-gray-500">•</span>
          Kasa: <span class="font-medium">{aktifSatis.kasa_ad}</span>
        </p>
      </div>
    {/if}

    <div>
      <Label for="odemeTutar" class="mb-2">Ödeme Tutarı (₺) <span class="text-red-500">*</span></Label>
      <Input
        id="odemeTutar"
        type="number"
        step="0.01"
        min="0.01"
        max={aktifSatis?.kalan_tutar ?? undefined}
        bind:value={odemeTutar}
        placeholder="0.00"
      />
    </div>

    <div>
      <Label for="odemeTarih" class="mb-2">Tarih <span class="text-red-500">*</span></Label>
      <Input id="odemeTarih" type="date" bind:value={odemeTarih} required />
    </div>

    <div>
      <Label for="odemeAciklama" class="mb-2">Açıklama</Label>
      <Textarea id="odemeAciklama" bind:value={odemeAciklama} rows={2} placeholder="Opsiyonel not" />
    </div>

    {#if hata}
      <div class="rounded-lg border border-red-200 bg-red-50 p-3 text-sm text-red-700 dark:border-red-700 dark:bg-red-900/20 dark:text-red-400">
        {hata}
      </div>
    {/if}
  </div>

  {#snippet footer()}
    <div class="flex justify-end gap-2">
      <Button
        color="primary"
        onclick={odemeyiEkle}
        disabled={odemeKaydediliyor || parseFloat(odemeTutar) <= 0}
      >
        {#if odemeKaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Ödemeyi Kaydet
      </Button>
      <Button color="alternative" onclick={() => (odemeModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>
