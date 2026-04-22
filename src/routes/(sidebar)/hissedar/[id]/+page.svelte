<script lang="ts">
  import {
    Button,
    Badge,
    Modal,
    Label,
    Input,
    Select,
    Spinner,
    Breadcrumb,
    BreadcrumbItem,
    Heading,
    P,
    Toggle,
    TableBodyRow,
    TableBodyCell
  } from 'flowbite-svelte';
  import {
    EditOutline,
    TrashBinSolid,
    ArrowLeftOutline,
    UserCircleSolid,
    PhoneSolid,
    ProfileCardSolid,
    WalletSolid,
    UsersGroupSolid,
    PlusOutline,
    FileLinesSolid
  } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import {
    hissedarApi,
    kasaApi,
    cuzdanApi,
    type Hissedar,
    type Kasa,
    type CuzdanHareketi,
    type CuzdanParaEkleSonuc
  } from '$lib/tauri-api';
  import DataTable from '$lib/components/DataTable.svelte';
  import type { DataTableColumn } from '$lib/components/dataTableUtils';
  import { exportPdf, formatTL, formatTarih } from '$lib/pdf';
  import { hasPermission } from '$lib/permissions';

  // ─── State ──────────────────────────────────────────────────────────────────

  let hissedar = $state<Hissedar | null>(null);
  let kasalar = $state<Kasa[]>([]);
  let cuzdanHareketleri = $state<CuzdanHareketi[]>([]);
  let yukleniyor = $state(true);
  let hata = $state('');

  let duzenleModalAcik = $state(false);
  let silModalAcik = $state(false);
  let paraEkleModalAcik = $state(false);
  let kaydediliyor = $state(false);
  let paraEkleniyor = $state(false);
  let paraEkleSonuc = $state<CuzdanParaEkleSonuc | null>(null);

  // Form
  let fAd = $state('');
  let fSoyad = $state('');
  let fKasaId = $state('');
  let fAileSiraNo = $state('');
  let fTcno = $state('');
  let fTel = $state('');
  let fYakinAdi = $state('');
  let fYakinlik = $state('');
  let fAktif = $state(true);

  // Para ekleme formu
  let fParaTutar = $state('');
  let fParaAciklama = $state('');

  // Tahsilat iptal
  let iptalModalAcik = $state(false);
  let iptalHedefi = $state<CuzdanHareketi | null>(null);
  let iptalEdiliyor = $state(false);
  const canIptal = $derived(hasPermission('hissedar.cuzdan.iptal'));
  function iptalAc(h: CuzdanHareketi) {
    iptalHedefi = h;
    iptalModalAcik = true;
  }
  async function iptalOnayla() {
    if (!iptalHedefi || !hissedar) return;
    iptalEdiliyor = true;
    try {
      const sonuc = await cuzdanApi.tahsilatIptal(hissedar.id, iptalHedefi.id);
      iptalModalAcik = false;
      iptalHedefi = null;
      alert(
        `Tahsilat iptal edildi.\n` +
        `Silinen cüzdan kaydı: ${sonuc.silinen_cuzdan_kayit}\n` +
        `Silinen kasa kaydı: ${sonuc.silinen_kasa_kayit}\n` +
        `Geri açılan borç: ${sonuc.geri_acilan_borc}`
      );
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Tahsilat iptal hatası';
    } finally {
      iptalEdiliyor = false;
    }
  }

  // ─── Yükle ─────────────────────────────────────────────────────────────────

  const hissedarId = $derived(Number($page.params.id));

  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      [hissedar, kasalar, cuzdanHareketleri] = await Promise.all([
        hissedarApi.get(hissedarId),
        kasaApi.getAll(),
        cuzdanApi.getByHissedar(hissedarId)
      ]);
    } catch (e: any) {
      hata = e?.toString() ?? 'Yükleme hatası';
    } finally {
      yukleniyor = false;
    }
  }

  $effect(() => { if (hissedarId) yukle(); });

  // ─── Düzenle ────────────────────────────────────────────────────────────────

  function duzenleAc() {
    if (!hissedar) return;
    fAd = hissedar.ad;
    fSoyad = hissedar.soyad;
    fKasaId = hissedar.kasa_id.toString();
    fAileSiraNo = hissedar.aile_sira_no?.toString() ?? '';
    fTcno = hissedar.tcno ?? '';
    fTel = hissedar.tel ?? '';
    fYakinAdi = hissedar.yakin_adi ?? '';
    fYakinlik = hissedar.yakinlik_derecesi ?? '';
    fAktif = hissedar.aktif;
    duzenleModalAcik = true;
  }

  async function kaydet() {
    if (!fAd.trim() || !fSoyad.trim() || !fKasaId || !hissedar) return;
    kaydediliyor = true;
    try {
      await hissedarApi.update({
        id: hissedar.id,
        ad: fAd.trim(),
        soyad: fSoyad.trim(),
        kasa_id: parseInt(fKasaId),
        aile_sira_no: fAileSiraNo ? parseInt(fAileSiraNo) : undefined,
        tcno: fTcno.trim() || undefined,
        tel: fTel.trim() || undefined,
        yakin_adi: fYakinAdi.trim() || undefined,
        yakinlik_derecesi: fYakinlik.trim() || undefined,
        aktif: fAktif
      });
      duzenleModalAcik = false;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Güncelleme hatası';
    } finally {
      kaydediliyor = false;
    }
  }

  // ─── Sil ────────────────────────────────────────────────────────────────────

  async function sil() {
    if (!hissedar) return;
    kaydediliyor = true;
    try {
      await hissedarApi.delete(hissedar.id);
      goto('/hissedar');
    } catch (e: any) {
      hata = e?.toString() ?? 'Silme hatası';
      kaydediliyor = false;
    }
  }

  // ─── Yardımcılar ────────────────────────────────────────────────────────────

  const kasaSecenekleri = $derived(kasalar.map((k) => ({ value: k.id.toString(), name: `${k.ad} (${k.para_birimi})` })));

  const cuzdanBakiye = $derived(cuzdanHareketleri.length > 0 ? cuzdanHareketleri[0].bakiye : 0);

  function tarihFormat(t: string): string {
    return new Date(t).toLocaleDateString('tr-TR', { day: '2-digit', month: '2-digit', year: 'numeric' });
  }

  function formatTutar(v: number): string {
    return v.toLocaleString('tr-TR', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  // ─── Cüzdan hareket kolonları ──────────────────────────────────────────
  const cuzdanKolonlar: DataTableColumn<CuzdanHareketi>[] = [
    { id: 'tarih', header: 'Tarih', accessor: 'tarih' },
    { id: 'donem', header: 'Dönem', accessor: (h) => h.donem_adi ?? '' },
    { id: 'bilgi', header: 'Bilgi', accessor: 'bilgi' },
    { id: 'borc', header: 'Borç', accessor: 'borc', align: 'right' },
    { id: 'alacak', header: 'Alacak', accessor: 'alacak', align: 'right' },
    { id: 'bakiye', header: 'Bakiye', accessor: 'bakiye', align: 'right', sortable: false, searchable: false },
    { id: 'islem', header: 'İşlem', accessor: () => '', align: 'right', sortable: false, searchable: false }
  ];

  // ─── Para Ekle ──────────────────────────────────────────────────────────────

  function paraEkleAc() {
    fParaTutar = '';
    fParaAciklama = '';
    paraEkleSonuc = null;
    paraEkleModalAcik = true;
  }

  async function paraEkle() {
    const tutar = parseFloat(String(fParaTutar).replace(',', '.'));
    if (!tutar || tutar <= 0 || !hissedar) return;
    paraEkleniyor = true;
    try {
      paraEkleSonuc = await cuzdanApi.paraEkle({
        hissedar_id: hissedar.id,
        tutar,
        aciklama: fParaAciklama.trim() || undefined
      });
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Para ekleme hatası';
      paraEkleModalAcik = false;
    } finally {
      paraEkleniyor = false;
    }
  }

  // ─── PDF ────────────────────────────────────────────────────────────────────

  function pdfIndir() {
    if (!hissedar) return;
    const h = hissedar;
    const toplamBorc = cuzdanHareketleri.reduce((s, c) => s + c.borc, 0);
    const toplamAlacak = cuzdanHareketleri.reduce((s, c) => s + c.alacak, 0);
    const guncelBakiye = cuzdanHareketleri.length > 0 ? cuzdanHareketleri[0].bakiye : 0;

    exportPdf({
      title: `Hissedar: ${h.ad} ${h.soyad}`,
      subtitle: h.aktif ? 'Aktif' : 'Pasif',
      fileName: `hissedar-${h.id}-${h.soyad}-${h.ad}`,
      sections: [
        {
          kind: 'kv',
          heading: 'Genel Bilgiler',
          columns: 2,
          items: [
            { label: 'Ad', value: h.ad },
            { label: 'Soyad', value: h.soyad },
            { label: 'Kasa', value: h.kasa_ad ?? '-' },
            { label: 'Aile Sıra No', value: h.aile_sira_no ?? '-' },
            { label: 'TC No', value: h.tcno ?? '-' },
            { label: 'Telefon', value: h.tel ?? '-' },
            { label: 'Yakın Adı', value: h.yakin_adi ?? '-' },
            { label: 'Yakınlık', value: h.yakinlik_derecesi ?? '-' },
            { label: 'Durum', value: h.aktif ? 'Aktif' : 'Pasif' },
            { label: 'Kayıt Tarihi', value: formatTarih(h.created_at) }
          ]
        },
        {
          kind: 'kv',
          heading: 'Cüzdan Özeti',
          columns: 2,
          items: [
            { label: 'Toplam Borç', value: formatTL(toplamBorc) },
            { label: 'Toplam Alacak', value: formatTL(toplamAlacak) },
            { label: 'Güncel Bakiye', value: formatTL(guncelBakiye) },
            { label: 'Hareket Sayısı', value: cuzdanHareketleri.length }
          ]
        },
        {
          kind: 'table',
          heading: 'Cüzdan Hareketleri',
          columns: ['Tarih', 'Bilgi', 'Dönem', 'Borç', 'Alacak', 'Bakiye'],
          widths: ['auto', '*', 'auto', 'auto', 'auto', 'auto'],
          rows: cuzdanHareketleri.map((c) => [
            formatTarih(c.tarih),
            c.bilgi,
            c.donem_adi ?? '-',
            c.borc > 0 ? formatTL(c.borc) : '',
            c.alacak > 0 ? formatTL(c.alacak) : '',
            formatTL(c.bakiye)
          ])
        }
      ]
    });
  }
</script>

<main class="min-h-screen bg-gray-50 p-6 dark:bg-gray-900">

  <!-- Breadcrumb -->
  <Breadcrumb class="mb-5">
    <BreadcrumbItem href="/hissedar">Hissedarlar</BreadcrumbItem>
    <BreadcrumbItem>{hissedar ? `${hissedar.soyad}, ${hissedar.ad}` : '...'}</BreadcrumbItem>
  </Breadcrumb>

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

  {:else if hissedar}

    <!-- Profil Kartı -->
    <div class="mb-6 rounded-xl border border-gray-200 bg-white shadow-sm dark:border-gray-700 dark:bg-gray-800">
      <div class="p-6">
        <div class="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
          <!-- Sol: Avatar + isim -->
          <div class="flex items-center gap-4">
            <div class="flex h-16 w-16 items-center justify-center rounded-full bg-primary-100 text-2xl font-bold text-primary-700 dark:bg-primary-900 dark:text-primary-300">
              {hissedar.ad[0]}{hissedar.soyad[0]}
            </div>
            <div>
              <Heading tag="h1" class="text-2xl font-bold text-gray-900 dark:text-white">
                {hissedar.ad} {hissedar.soyad}
              </Heading>
              <div class="mt-1 flex items-center gap-2">
                <Badge color={hissedar.aktif ? 'green' : 'red'}>
                  {hissedar.aktif ? 'Aktif' : 'Pasif'}
                </Badge>
                {#if hissedar.aile_sira_no}
                  <span class="text-sm text-gray-500 dark:text-gray-400">
                    Aile Sıra No: <strong>{hissedar.aile_sira_no}</strong>
                  </span>
                {/if}
              </div>
            </div>
          </div>

          <!-- Sağ: Aksiyonlar -->
          <div class="flex gap-2">
            <Button size="sm" color="alternative" class="gap-2" onclick={pdfIndir}>
              <FileLinesSolid class="h-4 w-4" /> PDF
            </Button>
            <Button size="sm" class="gap-2" onclick={duzenleAc}>
              <EditOutline class="h-4 w-4" /> Düzenle
            </Button>
            <Button size="sm" color="red" class="gap-2" onclick={() => (silModalAcik = true)}>
              <TrashBinSolid class="h-4 w-4" /> Sil
            </Button>
          </div>
        </div>
      </div>
    </div>

    <!-- Bilgi Kartları Izgara -->
    <div class="grid grid-cols-1 gap-5 md:grid-cols-2 xl:grid-cols-3">

      <!-- İletişim Bilgileri -->
      <div class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-700 dark:bg-gray-800">
        <div class="mb-4 flex items-center gap-2 text-sm font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
          <PhoneSolid class="h-4 w-4" />
          İletişim
        </div>
        <dl class="space-y-3">
          <div>
            <dt class="text-xs text-gray-500 dark:text-gray-400">Telefon</dt>
            <dd class="font-medium text-gray-900 dark:text-white">
              {#if hissedar.tel}{hissedar.tel}{:else}<span class="italic text-gray-400">—</span>{/if}
            </dd>
          </div>
        </dl>
      </div>

      <!-- Kimlik Bilgileri -->
      <div class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-700 dark:bg-gray-800">
        <div class="mb-4 flex items-center gap-2 text-sm font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
          <ProfileCardSolid class="h-4 w-4" />
          Kimlik
        </div>
        <dl class="space-y-3">
          <div>
            <dt class="text-xs text-gray-500 dark:text-gray-400">TC Kimlik No</dt>
            <dd class="font-mono font-medium text-gray-900 dark:text-white">
              {hissedar.tcno ?? '—'}
            </dd>
          </div>
        </dl>
      </div>

      <!-- Kasa Bilgisi -->
      <div class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-700 dark:bg-gray-800">
        <div class="mb-4 flex items-center gap-2 text-sm font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
          <WalletSolid class="h-4 w-4" />
          Kasa
        </div>
        <dl class="space-y-3">
          <div>
            <dt class="text-xs text-gray-500 dark:text-gray-400">Bağlı Kasa</dt>
            <dd class="font-medium text-gray-900 dark:text-white">
              {#if hissedar.kasa_ad}
                <button
                  class="text-primary-600 hover:underline dark:text-primary-400"
                  onclick={() => goto(`/kasa/${hissedar!.kasa_id}`)}
                >
                  {hissedar.kasa_ad}
                </button>
              {:else}
                —
              {/if}
            </dd>
          </div>
        </dl>
      </div>

      <!-- Yakın Kişi -->
      {#if hissedar.yakin_adi || hissedar.yakinlik_derecesi}
        <div class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-700 dark:bg-gray-800">
          <div class="mb-4 flex items-center gap-2 text-sm font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
            <UsersGroupSolid class="h-4 w-4" />
            Yakın Kişi
          </div>
          <dl class="space-y-3">
            <div>
              <dt class="text-xs text-gray-500 dark:text-gray-400">Adı</dt>
              <dd class="font-medium text-gray-900 dark:text-white">{hissedar.yakin_adi ?? '—'}</dd>
            </div>
            <div>
              <dt class="text-xs text-gray-500 dark:text-gray-400">Yakınlık Derecesi</dt>
              <dd class="font-medium text-gray-900 dark:text-white">{hissedar.yakinlik_derecesi ?? '—'}</dd>
            </div>
          </dl>
        </div>
      {/if}

      <!-- Sistem Bilgileri -->
      <div class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-700 dark:bg-gray-800">
        <div class="mb-4 flex items-center gap-2 text-sm font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">
          <UserCircleSolid class="h-4 w-4" />
          Kayıt Bilgileri
        </div>
        <dl class="space-y-3">
          <div>
            <dt class="text-xs text-gray-500 dark:text-gray-400">Kayıt Tarihi</dt>
            <dd class="text-sm font-medium text-gray-900 dark:text-white">{tarihFormat(hissedar.created_at)}</dd>
          </div>
          <div>
            <dt class="text-xs text-gray-500 dark:text-gray-400">Son Güncelleme</dt>
            <dd class="text-sm font-medium text-gray-900 dark:text-white">{tarihFormat(hissedar.updated_at)}</dd>
          </div>
        </dl>
      </div>
    </div>

    <!-- Cüzdan Bölümü -->
    <div class="mt-8">
      <div class="mb-4 flex items-center justify-between">
        <Heading tag="h2" class="text-xl font-bold text-gray-900 dark:text-white">
          <WalletSolid class="mr-2 inline h-5 w-5" />
          Cüzdan
        </Heading>
        <Button size="sm" color="green" class="gap-2" onclick={paraEkleAc}>
          <PlusOutline class="h-4 w-4" /> Para Ekle
        </Button>
      </div>

      <!-- Bakiye Kartı -->
      <div class="mb-4 rounded-xl border border-gray-200 bg-white p-5 shadow-sm dark:border-gray-700 dark:bg-gray-800">
        <div class="text-sm text-gray-500 dark:text-gray-400">Güncel Bakiye</div>
        <div class="text-2xl font-bold {cuzdanBakiye >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}">
          {formatTutar(cuzdanBakiye)} ₺
        </div>
      </div>

      <!-- Cüzdan Hareketleri Tablosu -->
      <DataTable
        data={cuzdanHareketleri}
        columns={cuzdanKolonlar}
        searchPlaceholder="Dönem veya bilgi ara..."
        exportFileName="cuzdan-hareketleri-{hissedar?.id ?? ''}"
        emptyMessage="Cüzdan hareketi bulunmuyor"
      >
        {#snippet row(hareket, _i, visibleCols)}
          <TableBodyRow>
            {#if visibleCols.has('tarih')}
              <TableBodyCell>{tarihFormat(hareket.tarih)}</TableBodyCell>
            {/if}
            {#if visibleCols.has('donem')}
              <TableBodyCell>{hareket.donem_adi ?? '—'}</TableBodyCell>
            {/if}
            {#if visibleCols.has('bilgi')}
              <TableBodyCell>{hareket.bilgi}</TableBodyCell>
            {/if}
            {#if visibleCols.has('borc')}
              <TableBodyCell class="text-right {hareket.borc > 0 ? 'font-medium text-red-600 dark:text-red-400' : ''}">
                {hareket.borc > 0 ? formatTutar(hareket.borc) : '—'}
              </TableBodyCell>
            {/if}
            {#if visibleCols.has('alacak')}
              <TableBodyCell class="text-right {hareket.alacak > 0 ? 'font-medium text-green-600 dark:text-green-400' : ''}">
                {hareket.alacak > 0 ? formatTutar(hareket.alacak) : '—'}
              </TableBodyCell>
            {/if}
            {#if visibleCols.has('bakiye')}
              <TableBodyCell class="text-right font-semibold {hareket.bakiye >= 0 ? 'text-green-700 dark:text-green-400' : 'text-red-700 dark:text-red-400'}">
                {formatTutar(hareket.bakiye)}
              </TableBodyCell>
            {/if}
            {#if visibleCols.has('islem')}
              <TableBodyCell class="text-right">
                {#if canIptal && hareket.alacak > 0 && hareket.borc === 0 && hareket.operation_id}
                  <Button size="xs" color="red" onclick={() => iptalAc(hareket)}>
                    İptal
                  </Button>
                {/if}
              </TableBodyCell>
            {/if}
          </TableBodyRow>
        {/snippet}
      </DataTable>
    </div>

    <!-- Geri Butonu -->
    <div class="mt-6">
      <button
        class="flex items-center gap-2 text-sm text-gray-500 hover:text-primary-600 dark:text-gray-400 dark:hover:text-primary-400"
        onclick={() => goto('/hissedar')}
      >
        <ArrowLeftOutline class="h-4 w-4" /> Tüm Hissedarlara Dön
      </button>
    </div>

  {:else}
    <p class="text-gray-500">Hissedar bulunamadı.</p>
  {/if}
</main>

<!-- Düzenle Modal -->
<Modal
  bind:open={duzenleModalAcik}
  title="Hissedarı Düzenle"
  size="lg"
  autoclose={false}
>
  <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
    <div>
      <Label for="dAd" class="mb-2">Ad <span class="text-red-500">*</span></Label>
      <Input id="dAd" bind:value={fAd} placeholder="Ad" required />
    </div>
    <div>
      <Label for="dSoyad" class="mb-2">Soyad <span class="text-red-500">*</span></Label>
      <Input id="dSoyad" bind:value={fSoyad} placeholder="Soyad" required />
    </div>
    <div class="sm:col-span-2">
      <Label for="dKasaId" class="mb-2">Kasa <span class="text-red-500">*</span></Label>
      <Select id="dKasaId" bind:value={fKasaId} items={kasaSecenekleri} />
    </div>
    <div class="sm:col-span-2">
      <p class="mb-3 text-xs font-semibold uppercase tracking-wide text-gray-400">Opsiyonel Bilgiler</p>
    </div>
    <div>
      <Label for="dAileSiraNo" class="mb-2">Aile Sıra No</Label>
      <Input id="dAileSiraNo" type="number" min="1" bind:value={fAileSiraNo} placeholder="1" />
    </div>
    <div>
      <Label for="dTcno" class="mb-2">TC Kimlik No</Label>
      <Input id="dTcno" bind:value={fTcno} placeholder="11 haneli" maxlength={11} />
    </div>
    <div>
      <Label for="dTel" class="mb-2">Telefon</Label>
      <Input id="dTel" type="tel" bind:value={fTel} placeholder="05xx xxx xx xx" />
    </div>
    <div>
      <Label for="dYakinAdi" class="mb-2">Yakının Adı</Label>
      <Input id="dYakinAdi" bind:value={fYakinAdi} placeholder="Adı Soyadı" />
    </div>
    <div>
      <Label for="dYakinlik" class="mb-2">Yakınlık Derecesi</Label>
      <Input id="dYakinlik" bind:value={fYakinlik} placeholder="Eş, Anne, Kardeş..." />
    </div>
    <div class="flex items-center gap-3">
      <Toggle bind:checked={fAktif} />
      <span class="text-sm text-gray-700 dark:text-gray-300">
        {fAktif ? 'Aktif' : 'Pasif'}
      </span>
    </div>
  </div>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button
        onclick={kaydet}
        disabled={kaydediliyor || !fAd.trim() || !fSoyad.trim() || !fKasaId}
      >
        {#if kaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Güncelle
      </Button>
      <Button color="alternative" onclick={() => (duzenleModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Sil Onay Modal -->
<Modal bind:open={silModalAcik} title="Hissedar Sil" size="sm" autoclose={false}>
  <p class="text-gray-600 dark:text-gray-400">
    <strong class="text-gray-900 dark:text-white">{hissedar?.ad} {hissedar?.soyad}</strong>
    hissedarını kalıcı olarak silmek istediğinize emin misiniz?
  </p>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button color="red" onclick={sil} disabled={kaydediliyor}>
        {#if kaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Evet, Sil
      </Button>
      <Button color="alternative" onclick={() => (silModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Para Ekle Modal -->
<Modal bind:open={paraEkleModalAcik} title="Cüzdana Para Ekle" size="md" autoclose={false}>
  {#if paraEkleSonuc}
    <div class="space-y-3">
      <div class="rounded-lg bg-green-50 p-4 dark:bg-green-900/20">
        <p class="text-green-700 dark:text-green-400 font-medium">Para başarıyla eklendi!</p>
        <p class="text-sm text-green-600 dark:text-green-300 mt-1">
          Yeni bakiye: <strong>{formatTutar(paraEkleSonuc.yeni_bakiye)} ₺</strong>
        </p>
      </div>
      {#if paraEkleSonuc.tahsil_edilen_borc_sayisi > 0}
        <div class="rounded-lg bg-blue-50 p-4 dark:bg-blue-900/20">
          <p class="text-blue-700 dark:text-blue-400 font-medium">
            Otomatik Tahsilat Yapıldı
          </p>
          <p class="text-sm text-blue-600 dark:text-blue-300 mt-1">
            {paraEkleSonuc.tahsil_edilen_borc_sayisi} borç, toplam
            <strong>{formatTutar(paraEkleSonuc.tahsil_edilen_toplam)} ₺</strong> tahsil edildi.
          </p>
        </div>
      {/if}
    </div>
  {:else}
    <div class="space-y-4">
      <div>
        <Label for="paraTutar" class="mb-2">Tutar (₺) <span class="text-red-500">*</span></Label>
        <Input
          id="paraTutar"
          type="number"
          step="0.01"
          min="0.01"
          bind:value={fParaTutar}
          placeholder="0.00"
          required
        />
      </div>
      <div>
        <Label for="paraAciklama" class="mb-2">Açıklama</Label>
        <Input id="paraAciklama" bind:value={fParaAciklama} placeholder="Opsiyonel açıklama" />
      </div>
    </div>
  {/if}

  {#snippet footer()}
    {#if paraEkleSonuc}
      <Button onclick={() => (paraEkleModalAcik = false)}>Tamam</Button>
    {:else}
      <div class="flex gap-3">
        <Button
          color="green"
          onclick={paraEkle}
          disabled={paraEkleniyor || !fParaTutar || parseFloat(String(fParaTutar).replace(',', '.')) <= 0}
        >
          {#if paraEkleniyor}<Spinner class="me-2" size="4" />{/if}
          Para Ekle
        </Button>
        <Button color="alternative" onclick={() => (paraEkleModalAcik = false)}>İptal</Button>
      </div>
    {/if}
  {/snippet}
</Modal>

<!-- Tahsilat İptal Onay Modal -->
<Modal bind:open={iptalModalAcik} title="Tahsilatı İptal Et" size="md" autoclose={false}>
  {#if iptalHedefi}
    <div class="space-y-3 text-sm text-gray-700 dark:text-gray-200">
      <p class="font-semibold text-red-600 dark:text-red-400">
        Bu tahsilat tamamen geri alınacak!
      </p>
      <div class="rounded-lg bg-gray-50 p-3 dark:bg-gray-700">
        <div><span class="font-medium">Tarih:</span> {tarihFormat(iptalHedefi.tarih)}</div>
        <div><span class="font-medium">Bilgi:</span> {iptalHedefi.bilgi}</div>
        <div><span class="font-medium">Tutar:</span> {formatTutar(iptalHedefi.alacak)} ₺</div>
      </div>
      <ul class="list-inside list-disc space-y-1 text-gray-600 dark:text-gray-300">
        <li>Cüzdandan bu kayıt ve etkilediği borç kapatma kayıtları silinir</li>
        <li>Kasadan ilgili giriş hareketi silinir ve kasa bakiyesi güncellenir</li>
        <li>Kapanan dönem aidat borçları yeniden açılır</li>
      </ul>
      <p class="font-medium">Devam etmek istediğinize emin misiniz?</p>
    </div>
    <div class="flex justify-end gap-2 pt-2">
      <Button color="red" onclick={iptalOnayla} disabled={iptalEdiliyor}>
        {#if iptalEdiliyor}<Spinner size="4" class="me-2" />{/if}
        Evet, İptal Et
      </Button>
      <Button color="alternative" onclick={() => (iptalModalAcik = false)} disabled={iptalEdiliyor}>Vazgeç</Button>
    </div>
  {/if}
</Modal>
