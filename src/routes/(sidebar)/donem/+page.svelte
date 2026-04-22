<script lang="ts">
  import {
    Button,
    Badge,
    Modal,
    Label,
    Input,
    Select,
    Spinner,
    TableBodyRow,
    TableBodyCell,
    Heading,
    P
  } from 'flowbite-svelte';
  import {
    CalendarMonthSolid,
    PlusOutline,
    EditOutline,
    TrashBinSolid,
    ArrowRightOutline,
    CalendarWeekSolid,
    FileLinesSolid
  } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import {
    donemApi,
    donemAdi,
    donemYillari,
    AY_ADLARI,
    type Donem,
    type CreateDonemInput
  } from '$lib/tauri-api';
  import DataTable from '$lib/components/DataTable.svelte';
  import type { DataTableColumn } from '$lib/components/dataTableUtils';
  import { exportPdf, formatTL } from '$lib/pdf';

  // ─── State ──────────────────────────────────────────────────────────────────

  let donemler = $state<Donem[]>([]);
  let yukleniyor = $state(true);
  let hata = $state('');

  // DataTable export (sıralanış + filtrelenmiş + görünür kolonlar)
  let pdfRows = $state<Donem[]>([]);
  let pdfCols = $state<DataTableColumn<Donem>[]>([]);

  let modalAcik = $state(false);
  let silModalAcik = $state(false);
  let duzenle = $state<Donem | null>(null);
  let silinecek = $state<Donem | null>(null);
  let kaydediliyor = $state(false);

  // Form
  const yillar = donemYillari();
  let fAy = $state('1');
  let fYil = $state(new Date().getFullYear().toString());
  let fAidat = $state('');

  // ─── Yükle ─────────────────────────────────────────────────────────────────

  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      donemler = await donemApi.getAll();
    } catch (e: any) {
      hata = e?.toString() ?? 'Yükleme hatası';
    } finally {
      yukleniyor = false;
    }
  }

  $effect(() => { yukle(); });

  // ─── PDF ────────────────────────────────────────────────────────────────────

  function pdfIndir() {
    const gorCols = pdfCols.filter((c) => c.id !== 'islemler');
    exportPdf({
      title: 'Dönem Listesi',
      subtitle: `${pdfRows.length} dönem`,
      fileName: `donemler-${new Date().toISOString().slice(0, 10)}`,
      sections: [
        {
          kind: 'table',
          columns: gorCols.map((c) => c.header),
          rows: pdfRows.map((d) =>
            gorCols.map((c) =>
              typeof c.accessor === 'function'
                ? String(c.accessor(d) ?? '')
                : String((d as Record<string, unknown>)[c.accessor as string] ?? '')
            )
          )
        }
      ]
    });
  }

  // ─── Modal Aç/Kapat ─────────────────────────────────────────────────────────

  function yeniAc() {
    duzenle = null;
    fAy = '1';
    fYil = new Date().getFullYear().toString();
    fAidat = '';
    modalAcik = true;
  }

  function duzenleAc(d: Donem) {
    duzenle = d;
    fAy = d.ay.toString();
    fYil = d.yil.toString();
    fAidat = d.hisse_basi_aidat.toString();
    modalAcik = true;
  }

  function silAc(d: Donem) {
    silinecek = d;
    silModalAcik = true;
  }

  // ─── CRUD ───────────────────────────────────────────────────────────────────

  async function kaydet() {
    if (!fAy || !fYil) return;
    kaydediliyor = true;
    try {
      if (duzenle) {
        await donemApi.update({
          id: duzenle.id,
          ay: parseInt(fAy),
          yil: parseInt(fYil),
          hisse_basi_aidat: parseFloat(fAidat) || 0
        });
      } else {
        await donemApi.create({
          ay: parseInt(fAy),
          yil: parseInt(fYil),
          hisse_basi_aidat: parseFloat(fAidat) || 0
        });
      }
      modalAcik = false;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Kayıt hatası';
    } finally {
      kaydediliyor = false;
    }
  }

  async function sil() {
    if (!silinecek) return;
    kaydediliyor = true;
    try {
      await donemApi.delete(silinecek.id);
      silModalAcik = false;
      silinecek = null;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Silme hatası';
    } finally {
      kaydediliyor = false;
    }
  }

  // ─── Seçenekler ─────────────────────────────────────────────────────────────

  const aySecenekleri = AY_ADLARI.map((ad, i) => ({ value: (i + 1).toString(), name: ad }));
  const yilSecenekleri = yillar.map((y) => ({ value: y.toString(), name: y.toString() }));

  function formatAidat(aidat: number): string {
    return new Intl.NumberFormat('tr-TR', { style: 'currency', currency: 'TRY', minimumFractionDigits: 2 }).format(aidat);
  }

  // ─── Kolonlar ───────────────────────────────────────────────────────────────
  const kolonlar: DataTableColumn<Donem>[] = [
    { id: 'donem', header: 'Dönem', accessor: (d) => donemAdi(d.ay, d.yil) },
    { id: 'yil', header: 'Yıl', accessor: 'yil', align: 'center' },
    { id: 'ay', header: 'Ay', accessor: 'ay', align: 'center', hiddenByDefault: true },
    { id: 'aidat', header: 'Hisse Başı Aidat', accessor: 'hisse_basi_aidat', align: 'right' },
    { id: 'toplanti', header: 'Toplantı', accessor: 'toplanti_sayisi', align: 'center' },
    { id: 'durum', header: 'Durum', accessor: (d) => (d.aktif ? 'Aktif' : 'Pasif'), align: 'center' },
    { id: 'islemler', header: '', accessor: () => '', sortable: false, searchable: false }
  ];
</script>

<main class="min-h-screen bg-gray-50 p-6 dark:bg-gray-900">

  <!-- Başlık -->
  <div class="mb-6 flex items-center justify-between">
    <div>
      <Heading tag="h1" class="text-2xl font-bold text-gray-900 dark:text-white">Dönemler</Heading>
      <P class="mt-1 text-sm text-gray-500 dark:text-gray-400">Toplam {donemler.length} dönem</P>
    </div>
    <div class="flex gap-2">
      <Button color="alternative" onclick={pdfIndir} class="gap-2">
        <FileLinesSolid class="h-4 w-4" />
        PDF
      </Button>
      <Button onclick={yeniAc} class="gap-2">
        <PlusOutline class="h-4 w-4" />
        Yeni Dönem
      </Button>
    </div>
  </div>

  <!-- Hata -->
  {#if hata}
    <div class="mb-4 rounded-lg bg-red-50 p-4 text-sm text-red-700 dark:bg-red-900/20 dark:text-red-400">
      {hata}
    </div>
  {/if}

  <!-- İçerik -->
  {#if yukleniyor}
    <div class="flex h-48 items-center justify-center">
      <Spinner size="10" />
    </div>

  {:else}
    <DataTable
      data={donemler}
      columns={kolonlar}
      searchPlaceholder="Dönem ara..."
      exportFileName="donemler"
      emptyMessage="Henüz dönem eklenmemiş"
      bind:exportRows={pdfRows}
      bind:exportVisibleCols={pdfCols}
    >
      {#snippet row(d, _i, visibleCols)}
        <TableBodyRow class="cursor-pointer" onclick={() => goto(`/donem/${d.id}`)}>
          {#if visibleCols.has('donem')}
            <TableBodyCell>
              <div class="flex items-center gap-2">
                <CalendarMonthSolid class="h-4 w-4 text-primary-500" />
                <span class="font-semibold text-gray-900 dark:text-white">{donemAdi(d.ay, d.yil)}</span>
              </div>
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('yil')}
            <TableBodyCell class="text-center">{d.yil}</TableBodyCell>
          {/if}
          {#if visibleCols.has('ay')}
            <TableBodyCell class="text-center">{d.ay}</TableBodyCell>
          {/if}
          {#if visibleCols.has('aidat')}
            <TableBodyCell class="text-right font-semibold">
              {formatAidat(d.hisse_basi_aidat)}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('toplanti')}
            <TableBodyCell class="text-center">
              <span class="inline-flex items-center gap-1 text-sm text-gray-600 dark:text-gray-400">
                <CalendarWeekSolid class="h-3.5 w-3.5" />
                {d.toplanti_sayisi}
              </span>
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('durum')}
            <TableBodyCell class="text-center">
              <Badge color={d.aktif ? 'green' : 'red'}>{d.aktif ? 'Aktif' : 'Pasif'}</Badge>
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('islemler')}
            <TableBodyCell>
              <div class="flex items-center gap-1" onclick={(e) => e.stopPropagation()} role="presentation">
                <button
                  class="rounded p-1.5 text-gray-500 hover:bg-gray-100 hover:text-primary-600 dark:hover:bg-gray-700"
                  onclick={() => duzenleAc(d)}
                  title="Düzenle"
                >
                  <EditOutline class="h-4 w-4" />
                </button>
                <button
                  class="rounded p-1.5 text-gray-500 hover:bg-red-50 hover:text-red-600 dark:hover:bg-gray-700"
                  onclick={() => silAc(d)}
                  title="Sil"
                >
                  <TrashBinSolid class="h-4 w-4" />
                </button>
                <button
                  class="rounded p-1.5 text-gray-500 hover:bg-gray-100 hover:text-primary-600 dark:hover:bg-gray-700"
                  onclick={() => goto(`/donem/${d.id}`)}
                  title="Detay"
                >
                  <ArrowRightOutline class="h-4 w-4" />
                </button>
              </div>
            </TableBodyCell>
          {/if}
        </TableBodyRow>
      {/snippet}
      {#snippet empty()}
        <div class="flex flex-col items-center justify-center py-6">
          <CalendarMonthSolid class="mb-3 h-12 w-12 text-gray-400" />
          <p class="text-gray-500 dark:text-gray-400">Henüz dönem eklenmemiş</p>
          <Button size="sm" class="mt-4 gap-2" onclick={yeniAc}>
            <PlusOutline class="h-4 w-4" /> Dönem Ekle
          </Button>
        </div>
      {/snippet}
    </DataTable>
  {/if}
</main>

<!-- Ekle/Düzenle Modal -->
<Modal
  bind:open={modalAcik}
  title={duzenle ? 'Dönemi Düzenle' : 'Yeni Dönem Ekle'}
  size="sm"
  autoclose={false}
>
  <div class="space-y-4">
    <div>
      <Label for="fAy" class="mb-2">Ay <span class="text-red-500">*</span></Label>
      <Select id="fAy" bind:value={fAy} items={aySecenekleri} />
    </div>
    <div>
      <Label for="fYil" class="mb-2">Yıl <span class="text-red-500">*</span></Label>
      <Select id="fYil" bind:value={fYil} items={yilSecenekleri} />
    </div>
    <div>
      <Label for="fAidat" class="mb-2">Hisse Başı Aidat (₺)</Label>
      <Input
        id="fAidat"
        type="number"
        min="0"
        step="0.01"
        bind:value={fAidat}
        placeholder="0.00"
      />
    </div>

    <!-- Önizleme -->
    {#if fAy && fYil}
      <div class="rounded-lg bg-primary-50 px-4 py-2 text-center dark:bg-primary-900/20">
        <span class="font-bold text-primary-700 dark:text-primary-400">
          {AY_ADLARI[parseInt(fAy) - 1]} {fYil}
        </span>
      </div>
    {/if}
  </div>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button onclick={kaydet} disabled={kaydediliyor || !fAy || !fYil}>
        {#if kaydediliyor}<Spinner class="me-2" size="4" />{/if}
        {duzenle ? 'Güncelle' : 'Ekle'}
      </Button>
      <Button color="alternative" onclick={() => (modalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Sil Onay Modal -->
<Modal bind:open={silModalAcik} title="Dönem Sil" size="sm" autoclose={false}>
  <p class="text-gray-600 dark:text-gray-400">
    <strong class="text-gray-900 dark:text-white">
      {silinecek ? donemAdi(silinecek.ay, silinecek.yil) : ''}
    </strong>
    dönemini ve tüm toplantı/kararlarını silmek istediğinize emin misiniz?
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
