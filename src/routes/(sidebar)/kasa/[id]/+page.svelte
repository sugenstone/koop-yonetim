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
  import {
    PlusOutline,
    TrashBinSolid,
    ArrowLeftOutline,
    ArrowRightOutline,
    WalletSolid,
    FileLinesSolid
  } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import {
    kasaApi,
    kasaHareketiApi,
    kasaTransferApi,
    formatBakiye,
    paraSembol,
    type Kasa,
    type KasaHareketi,
    type TransferInput,
    type ParaBirimi
  } from '$lib/tauri-api';
  import DataTable from '$lib/components/DataTable.svelte';
  import type { DataTableColumn } from '$lib/components/dataTableUtils';
  import { exportPdf, formatTarih } from '$lib/pdf';
  import Can from '$lib/Can.svelte';
  import { notify } from '$lib/toast';

  // ─── State ──────────────────────────────────────────────────────────────────

  let kasa = $state<Kasa | null>(null);
  let hareketler = $state<KasaHareketi[]>([]);
  let tumKasalar = $state<Kasa[]>([]);
  let yukleniyor = $state(true);
  let hata = $state('');

  // Hareket silme modal
  let silModalAcik = $state(false);
  let silinecekId = $state<number | null>(null);
  let kaydediliyor = $state(false);

  // Transfer modal
  let trfModalAcik = $state(false);
  let trfHedefKasaId = $state<number | ''>('');
  let trfTarih = $state(bugun());
  let trfMiktar = $state('');
  let trfKur = $state('');
  let trfAciklama = $state('');
  let trfKaydediliyor = $state(false);

  // ─── Yükle ─────────────────────────────────────────────────────────────────

  const kasaId = $derived(Number($page.params.id));

  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      [kasa, hareketler, tumKasalar] = await Promise.all([
        kasaApi.get(kasaId),
        kasaHareketiApi.getAll(kasaId),
        kasaApi.getAll()
      ]);
    } catch (e: any) {
      hata = e?.toString() ?? 'Yükleme hatası';
    } finally {
      yukleniyor = false;
    }
  }

  $effect(() => {
    if (kasaId) yukle();
  });

  // ─── Hareket Sil ────────────────────────────────────────────────────────────

  function silAc(id: number) {
    silinecekId = id;
    silModalAcik = true;
  }

  async function sil() {
    if (silinecekId === null) return;
    kaydediliyor = true;
    try {
      await kasaHareketiApi.delete({ id: silinecekId, kasa_id: kasaId });
      silModalAcik = false;
      silinecekId = null;
      notify.success('Hareket silindi');
      await yukle();
    } catch (e: any) {
      notify.apiError(e, 'Silme hatasi');
      hata = e?.message ?? 'Silme hatasi';
    } finally {
      kaydediliyor = false;
    }
  }

  // ─── Transfer ────────────────────────────────────────────────────────────────

  const hedefKasalar = $derived(
    tumKasalar.filter((k) => k.id !== kasaId && k.aktif)
  );

  const secilenHedef = $derived(
    tumKasalar.find((k) => k.id === Number(trfHedefKasaId)) ?? null
  );

  const farkliBirim = $derived(
    kasa !== null && secilenHedef !== null && kasa.para_birimi !== secilenHedef.para_birimi
  );

  const hesaplananKaynakMiktar = $derived(() => {
    const m = parseFloat(String(trfMiktar).replace(',', '.'));
    const k = parseFloat(String(trfKur).replace(',', '.'));
    if (farkliBirim && !isNaN(m) && !isNaN(k) && k > 0) return m * k;
    return null;
  });

  function trfAc() {
    trfHedefKasaId = '';
    trfTarih = bugun();
    trfMiktar = '';
    trfKur = '';
    trfAciklama = '';
    trfModalAcik = true;
  }

  async function trfKaydet() {
    if (!trfHedefKasaId) return;
    const miktar = parseFloat(String(trfMiktar).replace(',', '.'));
    if (isNaN(miktar) || miktar <= 0) return;

    let kur: number | undefined = undefined;
    if (farkliBirim) {
      const k = parseFloat(String(trfKur).replace(',', '.'));
      if (isNaN(k) || k <= 0) return;
      kur = k;
    }

    trfKaydediliyor = true;
    hata = '';
    try {
      const input: TransferInput = {
        kaynak_kasa_id: kasaId,
        hedef_kasa_id: Number(trfHedefKasaId),
        tarih: trfTarih,
        hedef_miktar: miktar,
        kur,
        aciklama: trfAciklama.trim() || undefined
      };
      await kasaTransferApi.create(input);
      trfModalAcik = false;
      notify.success('Transfer yapildi');
      await yukle();
    } catch (e: any) {
      notify.apiError(e, 'Transfer hatasi');
      hata = e?.message ?? 'Transfer hatasi';
    } finally {
      trfKaydediliyor = false;
    }
  }

  // ─── Yardımcılar ────────────────────────────────────────────────────────────

  function bugun(): string {
    return new Date().toISOString().slice(0, 10);
  }

  function tarihFormat(t: string): string {
    return new Date(t).toLocaleDateString('tr-TR', { day: '2-digit', month: '2-digit', year: 'numeric' });
  }

  function sayiFormat(n: number, birimi: ParaBirimi): string {
    if (n === 0) return '—';
    return formatBakiye(n, birimi);
  }

  const toplamGiren = $derived(hareketler.reduce((s, h) => s + h.giren, 0));
  const toplamCikan = $derived(hareketler.reduce((s, h) => s + h.cikan, 0));

  // ─── Kolonlar ───────────────────────────────────────────────────────────
  const hareketKolonlar: DataTableColumn<KasaHareketi>[] = [
    { id: 'tarih', header: 'Tarih', accessor: 'tarih' },
    { id: 'aciklama', header: 'Açıklama', accessor: 'aciklama' },
    { id: 'giren', header: 'Giren', accessor: 'giren', align: 'right' },
    { id: 'cikan', header: 'Çıkan', accessor: 'cikan', align: 'right' },
    { id: 'bakiye', header: 'Bakiye', accessor: 'bakiye', align: 'right', sortable: false, searchable: false },
    { id: 'islemler', header: '', accessor: () => '', sortable: false, searchable: false }
  ];

  // ─── PDF ────────────────────────────────────────────────────────────────────

  function pdfIndir() {
    if (!kasa) return;
    const k = kasa;
    const toplamGiris = hareketler.reduce((s, h) => s + h.giren, 0);
    const toplamCikis = hareketler.reduce((s, h) => s + h.cikan, 0);

    exportPdf({
      title: `Kasa: ${k.ad}`,
      subtitle: `${k.para_birimi} • Güncel Bakiye: ${formatBakiye(k.bakiye, k.para_birimi)}`,
      fileName: `kasa-${k.id}-${k.ad}`,
      sections: [
        {
          kind: 'kv',
          heading: 'Kasa Bilgileri',
          columns: 2,
          items: [
            { label: 'Ad', value: k.ad },
            { label: 'Para Birimi', value: k.para_birimi },
            { label: 'Güncel Bakiye', value: formatBakiye(k.bakiye, k.para_birimi) },
            { label: 'Hareket Sayısı', value: hareketler.length },
            { label: 'Toplam Giriş', value: formatBakiye(toplamGiris, k.para_birimi) },
            { label: 'Toplam Çıkış', value: formatBakiye(toplamCikis, k.para_birimi) },
            { label: 'Açıklama', value: k.aciklama ?? '-' }
          ]
        },
        {
          kind: 'table',
          heading: 'Kasa Hareketleri',
          columns: ['Tarih', 'Açıklama', 'Giren', 'Çıkan', 'Bakiye'],
          widths: ['auto', '*', 'auto', 'auto', 'auto'],
          rows: hareketler.map((h) => [
            formatTarih(h.tarih),
            h.aciklama,
            h.giren > 0 ? formatBakiye(h.giren, k.para_birimi) : '',
            h.cikan > 0 ? formatBakiye(h.cikan, k.para_birimi) : '',
            formatBakiye(h.bakiye, k.para_birimi)
          ])
        }
      ]
    });
  }
</script>

<main class="min-h-screen bg-gray-50 p-6 dark:bg-gray-900">

  <!-- Breadcrumb -->
  <Breadcrumb class="mb-5">
    <BreadcrumbItem href="/kasa">Kasalar</BreadcrumbItem>
    <BreadcrumbItem>{kasa?.ad ?? '...'}</BreadcrumbItem>
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

  {:else if kasa}

    <!-- Kasa Başlık Kartı -->
    <div class="mb-6 rounded-xl border border-gray-200 bg-white p-6 shadow-sm dark:border-gray-700 dark:bg-gray-800">
      <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div class="flex items-center gap-4">
          <div class="flex h-12 w-12 items-center justify-center rounded-full bg-primary-100 dark:bg-primary-900">
            <WalletSolid class="h-6 w-6 text-primary-600 dark:text-primary-300" />
          </div>
          <div>
            <Heading tag="h1" class="text-2xl font-bold text-gray-900 dark:text-white">
              {kasa.ad}
            </Heading>
            {#if kasa.aciklama}
              <P class="text-sm text-gray-500 dark:text-gray-400">{kasa.aciklama}</P>
            {/if}
          </div>
          <Badge color={kasa.aktif ? 'green' : 'red'} class="ml-2">
            {kasa.aktif ? 'Aktif' : 'Pasif'}
          </Badge>
        </div>

        <!-- Bakiye özeti -->
        <div class="flex flex-wrap gap-4">
          <div class="rounded-lg bg-green-50 px-4 py-3 dark:bg-green-900/20">
            <p class="text-xs text-green-600 dark:text-green-400">Toplam Giren</p>
            <p class="text-lg font-bold text-green-700 dark:text-green-300">
              {formatBakiye(toplamGiren, kasa.para_birimi)}
            </p>
          </div>
          <div class="rounded-lg bg-red-50 px-4 py-3 dark:bg-red-900/20">
            <p class="text-xs text-red-600 dark:text-red-400">Toplam Çıkan</p>
            <p class="text-lg font-bold text-red-700 dark:text-red-300">
              {formatBakiye(toplamCikan, kasa.para_birimi)}
            </p>
          </div>
          <div class="rounded-lg bg-gray-100 px-4 py-3 dark:bg-gray-700">
            <p class="text-xs text-gray-500 dark:text-gray-400">Güncel Bakiye</p>
            <p class="text-lg font-bold {kasa.bakiye >= 0 ? 'text-gray-900 dark:text-white' : 'text-red-600 dark:text-red-400'}">
              {formatBakiye(kasa.bakiye, kasa.para_birimi)}
            </p>
          </div>
        </div>
      </div>
    </div>

    <!-- Araç Çubuğu -->
    <div class="mb-4 flex items-center justify-between">
      <Heading tag="h2" class="text-lg font-semibold text-gray-900 dark:text-white">
        Kasa Hareketleri
        <span class="ml-2 text-sm font-normal text-gray-500">({hareketler.length} kayıt)</span>
      </Heading>
      <div class="flex gap-2">
        <Button size="sm" color="alternative" class="gap-2" onclick={pdfIndir}>
          <FileLinesSolid class="h-4 w-4" /> PDF
        </Button>
        <Can permission="kasa.transfer">
          <Button size="sm" color="blue" class="gap-2" onclick={trfAc}>
            <ArrowRightOutline class="h-4 w-4" /> Transfer
          </Button>
        </Can>
      </div>
    </div>

    <!-- Hareketler Tablosu -->
    {#if kasa}
      <DataTable
        data={hareketler}
        columns={hareketKolonlar}
        searchPlaceholder="Açıklama ara..."
        exportFileName="kasa-{kasa.ad}-hareketler"
        emptyMessage="Henüz hareket kaydı yok"
      >
        {#snippet row(h, _i, visibleCols)}
          <TableBodyRow>
            {#if visibleCols.has('tarih')}
              <TableBodyCell class="text-sm text-gray-500 dark:text-gray-400">
                {tarihFormat(h.tarih)}
              </TableBodyCell>
            {/if}
            {#if visibleCols.has('aciklama')}
              <TableBodyCell class="font-medium text-gray-900 dark:text-white">
                {h.aciklama}
              </TableBodyCell>
            {/if}
            {#if visibleCols.has('giren')}
              <TableBodyCell class="text-right font-medium text-green-600 dark:text-green-400">
                {h.giren > 0 ? sayiFormat(h.giren, kasa.para_birimi) : ''}
              </TableBodyCell>
            {/if}
            {#if visibleCols.has('cikan')}
              <TableBodyCell class="text-right font-medium text-red-600 dark:text-red-400">
                {h.cikan > 0 ? sayiFormat(h.cikan, kasa.para_birimi) : ''}
              </TableBodyCell>
            {/if}
            {#if visibleCols.has('bakiye')}
              <TableBodyCell class="text-right font-bold {h.bakiye >= 0 ? 'text-gray-900 dark:text-white' : 'text-red-600 dark:text-red-400'}">
                {formatBakiye(h.bakiye, kasa.para_birimi)}
              </TableBodyCell>
            {/if}
            {#if visibleCols.has('islemler')}
              <TableBodyCell class="text-center">
                <Can permission="kasa.hareket">
                  <button
                    class="rounded p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-gray-700"
                    onclick={() => silAc(h.id)}
                    title="Sil"
                  >
                    <TrashBinSolid class="h-4 w-4" />
                  </button>
                </Can>
              </TableBodyCell>
            {/if}
          </TableBodyRow>
        {/snippet}
        {#snippet empty()}
          <div class="flex flex-col items-center justify-center py-6">
            <P class="text-gray-500 dark:text-gray-400">Henüz hareket kaydı yok</P>
            <p class="mt-2 text-xs text-gray-400">Hareketler gelir/gider veya transfer ile oluşur.</p>
          </div>
        {/snippet}
      </DataTable>
    {/if}

    <!-- Geri Butonu -->
    <div class="mt-6">
      <button
        class="flex items-center gap-2 text-sm text-gray-500 hover:text-primary-600 dark:text-gray-400 dark:hover:text-primary-400"
        onclick={() => goto('/kasa')}
      >
        <ArrowLeftOutline class="h-4 w-4" /> Tüm Kasalara Dön
      </button>
    </div>

  {:else}
    <p class="text-gray-500">Kasa bulunamadı.</p>
  {/if}
</main>

<!-- Hareket Sil Onay Modal -->
<Modal bind:open={silModalAcik} title="Hareketi Sil" size="sm" autoclose={false}>

  <p class="text-gray-600 dark:text-gray-400">
    Bu hareketi silmek istediğinize emin misiniz? Bakiye yeniden hesaplanacaktır.
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

<!-- Transfer Modal -->
<Modal bind:open={trfModalAcik} title="Kasalar Arası Transfer" size="md" autoclose={false}>

  <div class="flex flex-col gap-4">

    <!-- Kaynak Kasa bilgisi -->
    {#if kasa}
      <div class="rounded-lg border border-gray-200 bg-gray-50 p-3 dark:border-gray-600 dark:bg-gray-700">
        <p class="mb-1 text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400">Kaynak Kasa</p>
        <p class="font-semibold text-gray-900 dark:text-white">{kasa.ad}</p>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          {kasa.para_birimi} · Bakiye: {formatBakiye(kasa.bakiye, kasa.para_birimi)}
        </p>
      </div>
    {/if}

    <!-- Hedef Kasa seçimi -->
    <div>
      <Label for="trfHedef" class="mb-2">Hedef Kasa *</Label>
      <Select
        id="trfHedef"
        bind:value={trfHedefKasaId}
        items={hedefKasalar.map((k) => ({ value: k.id, name: `${k.ad} (${k.para_birimi})` }))}
        placeholder="Kasa seçin..."
      />
    </div>

    <!-- Hedef kasa seçildiyse para birimi bilgisi -->
    {#if secilenHedef}
      <div class="rounded-lg border border-blue-200 bg-blue-50 p-3 dark:border-blue-700 dark:bg-blue-900/20">
        <p class="mb-1 text-xs font-semibold uppercase tracking-wide text-blue-600 dark:text-blue-400">Hedef Kasa</p>
        <p class="font-semibold text-gray-900 dark:text-white">{secilenHedef.ad}</p>
        <p class="text-sm text-blue-600 dark:text-blue-400">
          {secilenHedef.para_birimi} · Bakiye: {formatBakiye(secilenHedef.bakiye, secilenHedef.para_birimi as ParaBirimi)}
        </p>
      </div>
    {/if}

    <!-- Tarih -->
    <div>
      <Label for="trfTarih" class="mb-2">Tarih *</Label>
      <Input id="trfTarih" type="date" bind:value={trfTarih} required />
    </div>

    <!-- Farklı para birimi: kur alanı göster -->
    {#if farkliBirim && kasa && secilenHedef}
      <div class="rounded-lg border border-yellow-200 bg-yellow-50 p-3 dark:border-yellow-700 dark:bg-yellow-900/20">
        <p class="mb-3 text-sm font-medium text-yellow-800 dark:text-yellow-300">
          Döviz Dönüşümü: {kasa.para_birimi} ↔ {secilenHedef.para_birimi}
        </p>
        <div class="flex flex-col gap-3">
          <div>
            <Label for="trfKur" class="mb-2">
              Kur: 1 {secilenHedef.para_birimi} = ? {kasa.para_birimi} *
            </Label>
            <Input
              id="trfKur"
              type="number"
              step="0.0001"
              min="0"
              bind:value={trfKur}
              placeholder="Örn: 3500 (1 gr altın = 3500 TL)"
              required
            />
          </div>
          <div>
            <Label for="trfMiktar" class="mb-2">
              Hedef Miktar ({secilenHedef.para_birimi}) *
            </Label>
            <Input
              id="trfMiktar"
              type="number"
              step="0.0001"
              min="0"
              bind:value={trfMiktar}
              placeholder="Örn: 5 (5 gram altın)"
              required
            />
          </div>
          <!-- Hesaplanan kaynak miktar -->
          {#if hesaplananKaynakMiktar() !== null}
            <div class="rounded-md bg-white p-3 shadow-sm dark:bg-gray-800">
              <p class="text-xs text-gray-500 dark:text-gray-400">Kaynaktan düşülecek</p>
              <p class="text-lg font-bold text-red-600 dark:text-red-400">
                {formatBakiye(hesaplananKaynakMiktar()!, kasa.para_birimi)}
              </p>
            </div>
          {/if}
        </div>
      </div>
    {:else if secilenHedef}
      <!-- Aynı para birimi: tek miktar alanı -->
      <div>
        <Label for="trfMiktar" class="mb-2">
          Tutar ({kasa ? paraSembol(kasa.para_birimi) : ''}) *
        </Label>
        <Input
          id="trfMiktar"
          type="number"
          step="0.01"
          min="0"
          bind:value={trfMiktar}
          placeholder="0.00"
          required
        />
      </div>
    {/if}

    <!-- Açıklama -->
    <div>
      <Label for="trfAciklama" class="mb-2">Açıklama (isteğe bağlı)</Label>
      <Textarea id="trfAciklama" bind:value={trfAciklama} rows={2} placeholder="Transfer açıklaması..." />
    </div>

  </div>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button
        color="blue"
        onclick={trfKaydet}
        disabled={
          trfKaydediliyor ||
          !trfHedefKasaId ||
          !trfMiktar ||
          (farkliBirim && !trfKur)
        }
      >
        {#if trfKaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Transfer Yap
      </Button>
      <Button color="alternative" onclick={() => (trfModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>
