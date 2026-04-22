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
    PlusOutline,
    EditOutline,
    TrashBinSolid,
    UsersSolid,
    ArrowRightOutline,
    FileLinesSolid,
    CloseOutline
  } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import {
    hissedarApi,
    kasaApi,
    type Hissedar,
    type CreateHissedarInput,
    type Kasa
  } from '$lib/tauri-api';
  import DataTable from '$lib/components/DataTable.svelte';
  import type { DataTableColumn } from '$lib/components/dataTableUtils';
  import { exportPdf } from '$lib/pdf';

  // ─── State ──────────────────────────────────────────────────────────────────

  let hissedarlar = $state<Hissedar[]>([]);
  let kasalar = $state<Kasa[]>([]);
  let yukleniyor = $state(true);
  let hata = $state('');

  // DataTable export (sıralanış + filtrelenmiş + görünür kolonlar)
  let pdfRows = $state<Hissedar[]>([]);
  let pdfCols = $state<DataTableColumn<Hissedar>[]>([]);

  // Inline kasa düzenleme
  let inlineKasaId = $state<number | null>(null);
  let inlineKasaYukleniyor = $state(false);

  // Modal
  let modalAcik = $state(false);
  let silModalAcik = $state(false);
  let duzenle = $state<Hissedar | null>(null);
  let silinecek = $state<Hissedar | null>(null);
  let kaydediliyor = $state(false);

  // Form alanları
  let fAd = $state('');
  let fSoyad = $state('');
  let fKasaId = $state('');
  let fAileSiraNo = $state('');
  let fTcno = $state('');
  let fTel = $state('');
  let fYakinAdi = $state('');
  let fYakinlik = $state('');

  // ─── Yükle ─────────────────────────────────────────────────────────────────

  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      [hissedarlar, kasalar] = await Promise.all([
        hissedarApi.getAll(),
        kasaApi.getAll()
      ]);
    } catch (e: any) {
      hata = e?.toString() ?? 'Yükleme hatası';
    } finally {
      yukleniyor = false;
    }
  }

  $effect(() => { yukle(); });

  // ─── Filtreleme ─────────────────────────────────────────────────────────────
  // (DataTable component'i global arama yönetiyor)

  // ─── Kolonlar ───────────────────────────────────────────────────────────────

  const kolonlar: DataTableColumn<Hissedar>[] = [
    { id: 'id', header: '#', accessor: 'id', align: 'left', hiddenByDefault: true },
    { id: 'ad_soyad', header: 'Ad Soyad', accessor: (h) => `${h.ad} ${h.soyad}` },
    { id: 'kasa', header: 'Kasa', accessor: (h) => h.kasa_ad ?? '' },
    { id: 'aile_sira_no', header: 'Aile S.', accessor: (h) => h.aile_sira_no ?? '', align: 'center' },
    { id: 'tcno', header: 'TC No', accessor: (h) => h.tcno ?? '' },
    { id: 'tel', header: 'Telefon', accessor: (h) => h.tel ?? '' },
    { id: 'yakin', header: 'Yakın', accessor: (h) => (h.yakin_adi && h.yakinlik_derecesi ? `${h.yakinlik_derecesi}: ${h.yakin_adi}` : ''), hiddenByDefault: true },
    { id: 'durum', header: 'Durum', accessor: (h) => (h.aktif ? 'Aktif' : 'Pasif'), align: 'center' },
    { id: 'islemler', header: '', accessor: () => '', sortable: false, searchable: false }
  ];

  // ─── Modal Aç/Kapat ─────────────────────────────────────────────────────────

  function yeniAc() {
    duzenle = null;
    fAd = ''; fSoyad = ''; fKasaId = kasalar[0]?.id?.toString() ?? '';
    fAileSiraNo = ''; fTcno = ''; fTel = ''; fYakinAdi = ''; fYakinlik = '';
    modalAcik = true;
  }

  function duzenleAc(h: Hissedar) {
    duzenle = h;
    fAd = h.ad; fSoyad = h.soyad; fKasaId = h.kasa_id.toString();
    fAileSiraNo = h.aile_sira_no?.toString() ?? '';
    fTcno = h.tcno ?? ''; fTel = h.tel ?? '';
    fYakinAdi = h.yakin_adi ?? ''; fYakinlik = h.yakinlik_derecesi ?? '';
    modalAcik = true;
  }

  function silAc(h: Hissedar) {
    silinecek = h;
    silModalAcik = true;
  }

  // ─── CRUD ───────────────────────────────────────────────────────────────────

  async function kaydet() {
    if (!fAd.trim() || !fSoyad.trim() || !fKasaId) return;
    kaydediliyor = true;
    try {
      const kasaIdNum = parseInt(fKasaId);
      if (duzenle) {
        await hissedarApi.update({
          id: duzenle.id,
          ad: fAd.trim(),
          soyad: fSoyad.trim(),
          kasa_id: kasaIdNum,
          aile_sira_no: fAileSiraNo ? parseInt(fAileSiraNo) : undefined,
          tcno: fTcno.trim() || undefined,
          tel: fTel.trim() || undefined,
          yakin_adi: fYakinAdi.trim() || undefined,
          yakinlik_derecesi: fYakinlik.trim() || undefined
        });
      } else {
        const input: CreateHissedarInput = {
          ad: fAd.trim(),
          soyad: fSoyad.trim(),
          kasa_id: kasaIdNum,
          aile_sira_no: fAileSiraNo ? parseInt(fAileSiraNo) : undefined,
          tcno: fTcno.trim() || undefined,
          tel: fTel.trim() || undefined,
          yakin_adi: fYakinAdi.trim() || undefined,
          yakinlik_derecesi: fYakinlik.trim() || undefined
        };
        await hissedarApi.create(input);
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
      await hissedarApi.delete(silinecek.id);
      silModalAcik = false;
      silinecek = null;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Silme hatası';
    } finally {
      kaydediliyor = false;
    }
  }

  // ─── Inline Kasa Güncelleme ──────────────────────────────────────────────────

  async function kasaInlineGuncelle(h: Hissedar, yeniKasaIdStr: string) {
    const yeniKasaId = parseInt(yeniKasaIdStr);
    if (yeniKasaId === h.kasa_id) { inlineKasaId = null; return; }
    inlineKasaYukleniyor = true;
    try {
      await hissedarApi.update({
        id: h.id,
        ad: h.ad,
        soyad: h.soyad,
        kasa_id: yeniKasaId,
        aile_sira_no: h.aile_sira_no ?? undefined,
        tcno: h.tcno ?? undefined,
        tel: h.tel ?? undefined,
        yakin_adi: h.yakin_adi ?? undefined,
        yakinlik_derecesi: h.yakinlik_derecesi ?? undefined
      });
      const yeniKasaAd = kasalar.find((k) => k.id === yeniKasaId)?.ad ?? '';
      hissedarlar = hissedarlar.map((x) =>
        x.id === h.id ? { ...x, kasa_id: yeniKasaId, kasa_ad: yeniKasaAd } : x
      );
    } catch (e: any) {
      hata = e?.toString() ?? 'Kasa güncelleme hatası';
    } finally {
      inlineKasaYukleniyor = false;
      inlineKasaId = null;
    }
  }

  // ─── Yardımcılar ────────────────────────────────────────────────────────────

  const kasaSecenekleri = $derived(kasalar.map((k) => ({ value: k.id.toString(), name: `${k.ad} (${k.para_birimi})` })));

  // ─── PDF ────────────────────────────────────────────────────────────────────

  function pdfIndir() {
    const gorCols = pdfCols.filter((c) => c.id !== 'islemler');
    exportPdf({
      title: 'Hissedar Listesi',
      subtitle: `${pdfRows.length} kayıt (filtre/sıralama uygulanmış)`,
      fileName: `hissedarlar-${new Date().toISOString().slice(0, 10)}`,
      landscape: true,
      sections: [
        {
          kind: 'table',
          columns: gorCols.map((c) => c.header),
          rows: pdfRows.map((h) =>
            gorCols.map((c) =>
              typeof c.accessor === 'function'
                ? String(c.accessor(h) ?? '')
                : String((h as Record<string, unknown>)[c.accessor as string] ?? '')
            )
          )
        }
      ]
    });
  }
</script>

<main class="min-h-screen bg-gray-50 p-6 dark:bg-gray-900">
  <!-- Başlık -->
  <div class="mb-6 flex items-center justify-between">
    <div>
      <Heading tag="h1" class="text-2xl font-bold text-gray-900 dark:text-white">Hissedarlar</Heading>
      <P class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Toplam {hissedarlar.length} hissedar
      </P>
    </div>
    <div class="flex gap-2">
      <Button color="alternative" onclick={pdfIndir} class="gap-2">
        <FileLinesSolid class="h-4 w-4" />
        PDF
      </Button>
      <Button onclick={yeniAc} class="gap-2">
        <PlusOutline class="h-4 w-4" />
        Yeni Hissedar
      </Button>
    </div>
  </div>

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
      data={hissedarlar}
      columns={kolonlar}
      searchPlaceholder="Ad, soyad, TC, telefon, yakın ara..."
      exportFileName="hissedarlar"
      emptyMessage="Henüz hissedar eklenmemiş"
      bind:exportRows={pdfRows}
      bind:exportVisibleCols={pdfCols}
    >
      {#snippet row(h, _i, visibleCols)}
        <TableBodyRow>
          {#if visibleCols.has('id')}
            <TableBodyCell class="text-xs text-gray-400">{h.id}</TableBodyCell>
          {/if}
          {#if visibleCols.has('ad_soyad')}
            <TableBodyCell>
              <button
                class="font-semibold text-gray-900 hover:text-primary-600 hover:underline dark:text-white dark:hover:text-primary-400"
                onclick={() => goto(`/hissedar/${h.id}`)}
              >
                {h.ad} {h.soyad}
              </button>
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('kasa')}
            <TableBodyCell class="text-sm">
              {#if inlineKasaId === h.id}
                <div class="flex items-center gap-1">
                  <select
                    class="rounded border border-primary-400 bg-white px-2 py-1 text-sm text-gray-800 focus:outline-none dark:border-primary-500 dark:bg-gray-700 dark:text-white"
                    value={h.kasa_id.toString()}
                    onchange={(e) => kasaInlineGuncelle(h, (e.target as HTMLSelectElement).value)}
                    disabled={inlineKasaYukleniyor}
                    autofocus
                  >
                    {#each kasalar as k (k.id)}
                      <option value={k.id.toString()}>{k.ad}</option>
                    {/each}
                  </select>
                  {#if inlineKasaYukleniyor}
                    <Spinner size="4" />
                  {:else}
                    <button
                      class="rounded p-0.5 text-gray-400 hover:text-red-500"
                      onclick={() => (inlineKasaId = null)}
                      title="İptal"
                    >
                      <CloseOutline class="h-3.5 w-3.5" />
                    </button>
                  {/if}
                </div>
              {:else}
                <button
                  class="group flex items-center gap-1.5 rounded px-1 py-0.5 text-gray-600 hover:bg-primary-50 hover:text-primary-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-primary-400"
                  onclick={() => (inlineKasaId = h.id)}
                  title="Kasayı değiştirmek için tıklayın"
                >
                  <span>{h.kasa_ad ?? '-'}</span>
                  <EditOutline class="h-3 w-3 opacity-0 transition-opacity group-hover:opacity-60" />
                </button>
              {/if}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('aile_sira_no')}
            <TableBodyCell class="text-center text-sm">
              {h.aile_sira_no ?? '-'}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('tcno')}
            <TableBodyCell class="font-mono text-sm text-gray-600 dark:text-gray-400">
              {h.tcno ?? '-'}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('tel')}
            <TableBodyCell class="text-sm text-gray-600 dark:text-gray-400">
              {h.tel ?? '-'}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('yakin')}
            <TableBodyCell class="text-sm text-gray-600 dark:text-gray-400">
              {#if h.yakin_adi && h.yakinlik_derecesi}
                <span class="text-xs">{h.yakinlik_derecesi}: {h.yakin_adi}</span>
              {:else}
                -
              {/if}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('durum')}
            <TableBodyCell class="text-center">
              <Badge color={h.aktif ? 'green' : 'red'}>
                {h.aktif ? 'Aktif' : 'Pasif'}
              </Badge>
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('islemler')}
            <TableBodyCell>
              <div class="flex items-center gap-1">
                <button
                  class="rounded p-1.5 text-gray-500 hover:bg-gray-100 hover:text-primary-600 dark:hover:bg-gray-700"
                  onclick={() => duzenleAc(h)}
                  title="Düzenle"
                >
                  <EditOutline class="h-4 w-4" />
                </button>
                <button
                  class="rounded p-1.5 text-gray-500 hover:bg-red-50 hover:text-red-600 dark:hover:bg-gray-700"
                  onclick={() => silAc(h)}
                  title="Sil"
                >
                  <TrashBinSolid class="h-4 w-4" />
                </button>
                <button
                  class="rounded p-1.5 text-gray-500 hover:bg-gray-100 hover:text-primary-600 dark:hover:bg-gray-700"
                  onclick={() => goto(`/hissedar/${h.id}`)}
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
          <UsersSolid class="mb-3 h-12 w-12 text-gray-400" />
          <p class="text-gray-500 dark:text-gray-400">Kayıt bulunamadı</p>
          <Button size="sm" class="mt-4 gap-2" onclick={yeniAc}>
            <PlusOutline class="h-4 w-4" /> Hissedar Ekle
          </Button>
        </div>
      {/snippet}
    </DataTable>
  {/if}
</main>

<!-- Hissedar Ekle/Düzenle Modal -->
<Modal
  bind:open={modalAcik}
  title={duzenle ? 'Hissedarı Düzenle' : 'Yeni Hissedar Ekle'}
  size="lg"
  autoclose={false}
>
  <div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
    <!-- Zorunlu alanlar -->
    <div>
      <Label for="fAd" class="mb-2">Ad <span class="text-red-500">*</span></Label>
      <Input id="fAd" bind:value={fAd} placeholder="Ad" required />
    </div>
    <div>
      <Label for="fSoyad" class="mb-2">Soyad <span class="text-red-500">*</span></Label>
      <Input id="fSoyad" bind:value={fSoyad} placeholder="Soyad" required />
    </div>
    <div class="sm:col-span-2">
      <Label for="fKasaId" class="mb-2">Kasa <span class="text-red-500">*</span></Label>
      <Select id="fKasaId" bind:value={fKasaId} items={kasaSecenekleri} />
    </div>

    <!-- Opsiyonel alanlar -->
    <div class="sm:col-span-2">
      <p class="mb-3 text-xs font-semibold uppercase tracking-wide text-gray-400">
        Opsiyonel Bilgiler
      </p>
    </div>
    <div>
      <Label for="fAileSiraNo" class="mb-2">Aile Sıra No</Label>
      <Input id="fAileSiraNo" type="number" min="1" bind:value={fAileSiraNo} placeholder="örn. 1" />
    </div>
    <div>
      <Label for="fTcno" class="mb-2">TC Kimlik No</Label>
      <Input id="fTcno" bind:value={fTcno} placeholder="11 haneli TC no" maxlength={11} />
    </div>
    <div>
      <Label for="fTel" class="mb-2">Telefon</Label>
      <Input id="fTel" type="tel" bind:value={fTel} placeholder="05xx xxx xx xx" />
    </div>
    <div>
      <Label for="fYakinAdi" class="mb-2">Yakının Adı</Label>
      <Input id="fYakinAdi" bind:value={fYakinAdi} placeholder="Adı Soyadı" />
    </div>
    <div>
      <Label for="fYakinlik" class="mb-2">Yakınlık Derecesi</Label>
      <Input id="fYakinlik" bind:value={fYakinlik} placeholder="örn. Eş, Anne, Kardeş..." />
    </div>
  </div>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button
        onclick={kaydet}
        disabled={kaydediliyor || !fAd.trim() || !fSoyad.trim() || !fKasaId}
      >
        {#if kaydediliyor}<Spinner class="me-2" size="4" />{/if}
        {duzenle ? 'Güncelle' : 'Ekle'}
      </Button>
      <Button color="alternative" onclick={() => (modalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Sil Onay Modal -->
<Modal bind:open={silModalAcik} title="Hissedar Sil" size="sm" autoclose={false}>
  <p class="text-gray-600 dark:text-gray-400">
    <strong class="text-gray-900 dark:text-white">{silinecek?.ad} {silinecek?.soyad}</strong>
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
