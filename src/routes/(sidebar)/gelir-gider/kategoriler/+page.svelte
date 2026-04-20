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
    Heading,
    P
  } from 'flowbite-svelte';
  import {
    PlusOutline,
    EditOutline,
    TrashBinSolid,
    ArrowLeftOutline,
    TagSolid
  } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import {
    gelirGiderKategoriApi,
    type GelirGiderKategori,
    type CreateKategoriInput,
    type UpdateKategoriInput
  } from '$lib/tauri-api';
  import DataTable from '$lib/components/DataTable.svelte';
  import type { DataTableColumn } from '$lib/components/dataTableUtils';

  // ─── State ──────────────────────────────────────────────────────────────────

  let kategoriler = $state<GelirGiderKategori[]>([]);
  let yukleniyor = $state(true);
  let hata = $state('');

  let modalAcik = $state(false);
  let silModalAcik = $state(false);
  let duzenle = $state<GelirGiderKategori | null>(null);
  let silinecek = $state<GelirGiderKategori | null>(null);
  let kaydediliyor = $state(false);

  // Form
  let fAd = $state('');
  let fTip = $state<'gelir' | 'gider'>('gelir');
  let fAciklama = $state('');

  // ─── Yükle ─────────────────────────────────────────────────────────────────

  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      kategoriler = await gelirGiderKategoriApi.getAll();
    } catch (e: any) {
      hata = e?.toString() ?? 'Yükleme hatası';
    } finally {
      yukleniyor = false;
    }
  }

  $effect(() => { yukle(); });

  // ─── Modal ─────────────────────────────────────────────────────────────────

  function yeniAc() {
    duzenle = null;
    fAd = '';
    fTip = 'gelir';
    fAciklama = '';
    modalAcik = true;
  }

  function duzenleAc(k: GelirGiderKategori) {
    duzenle = k;
    fAd = k.ad;
    fTip = k.tip as 'gelir' | 'gider';
    fAciklama = k.aciklama ?? '';
    modalAcik = true;
  }

  async function kaydet() {
    if (!fAd.trim()) return;
    kaydediliyor = true;
    hata = '';
    try {
      if (duzenle) {
        const input: UpdateKategoriInput = {
          id: duzenle.id,
          ad: fAd.trim(),
          aciklama: fAciklama.trim() || undefined
        };
        await gelirGiderKategoriApi.update(input);
      } else {
        const input: CreateKategoriInput = {
          ad: fAd.trim(),
          tip: fTip,
          aciklama: fAciklama.trim() || undefined
        };
        await gelirGiderKategoriApi.create(input);
      }
      modalAcik = false;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Kayıt hatası';
    } finally {
      kaydediliyor = false;
    }
  }

  // ─── Sil ───────────────────────────────────────────────────────────────────

  function silAc(k: GelirGiderKategori) {
    silinecek = k;
    silModalAcik = true;
  }

  async function sil() {
    if (!silinecek) return;
    kaydediliyor = true;
    hata = '';
    try {
      await gelirGiderKategoriApi.delete(silinecek.id);
      silModalAcik = false;
      silinecek = null;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Silme hatası';
    } finally {
      kaydediliyor = false;
    }
  }

  // ─── Yardımcılar ────────────────────────────────────────────────────────────

  const gelirKategoriler = $derived(kategoriler.filter((k) => k.tip === 'gelir'));
  const giderKategoriler = $derived(kategoriler.filter((k) => k.tip === 'gider'));

  // ─── Kolonlar ────────────────────────────────────────────────────────────────

  const kolonlar: DataTableColumn<GelirGiderKategori>[] = [
    { id: 'ad', header: 'Kategori Adı', accessor: 'ad' },
    { id: 'aciklama', header: 'Açıklama', accessor: (k) => k.aciklama ?? '-' },
    { id: 'durum', header: 'Durum', accessor: (k) => (k.aktif ? 'Aktif' : 'Pasif'), sortable: false, searchable: false },
    { id: 'islemler', header: '', accessor: () => '', sortable: false, searchable: false }
  ];
</script>

<main class="min-h-screen bg-gray-50 p-6 dark:bg-gray-900">

  <!-- Başlık -->
  <div class="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
    <div class="flex items-center gap-3">
      <button
        class="flex items-center gap-1 text-sm text-gray-500 hover:text-primary-600 dark:text-gray-400 dark:hover:text-primary-400"
        onclick={() => goto('/gelir-gider')}
      >
        <ArrowLeftOutline class="h-4 w-4" /> Geri
      </button>
      <div class="flex items-center gap-3">
        <div class="flex h-10 w-10 items-center justify-center rounded-full bg-indigo-100 dark:bg-indigo-900">
          <TagSolid class="h-5 w-5 text-indigo-600 dark:text-indigo-300" />
        </div>
        <div>
          <Heading tag="h1" class="text-2xl font-bold text-gray-900 dark:text-white">
            Gelir / Gider Kategorileri
          </Heading>
          <P class="text-sm text-gray-500 dark:text-gray-400">
            {kategoriler.length} kategori
          </P>
        </div>
      </div>
    </div>
    <Button onclick={yeniAc} class="gap-2">
      <PlusOutline class="h-4 w-4" /> Yeni Kategori
    </Button>
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
  {:else}

    <!-- Gelir Kategorileri -->
    <div class="mb-8">
      <div class="mb-3 flex items-center gap-2">
        <div class="h-1 w-6 rounded-full bg-green-500"></div>
        <Heading tag="h2" class="text-lg font-semibold text-gray-900 dark:text-white">
          Gelir Kategorileri
        </Heading>
        <Badge color="green">{gelirKategoriler.length}</Badge>
      </div>

      {#if gelirKategoriler.length === 0}
        <div class="rounded-lg border border-dashed border-gray-300 p-8 text-center dark:border-gray-600">
          <P class="text-gray-500 dark:text-gray-400">Henüz gelir kategorisi yok</P>
          <Button size="sm" class="mt-3 gap-2" onclick={yeniAc}>
            <PlusOutline class="h-4 w-4" /> Gelir Kategorisi Ekle
          </Button>
        </div>
      {:else}
        <DataTable
          data={gelirKategoriler}
          columns={kolonlar}
          searchPlaceholder="Kategori ara..."
          exportFileName="gelir-kategorileri"
          emptyMessage="Sonuç bulunamadı"
        >
          {#snippet row(k, _i, visibleCols)}
            <TableBodyRow>
              {#if visibleCols.has('ad')}
                <TableBodyCell class="font-medium text-gray-900 dark:text-white">
                  {k.ad}
                </TableBodyCell>
              {/if}
              {#if visibleCols.has('aciklama')}
                <TableBodyCell class="text-sm text-gray-500 dark:text-gray-400">
                  {k.aciklama ?? '—'}
                </TableBodyCell>
              {/if}
              {#if visibleCols.has('durum')}
                <TableBodyCell>
                  <Badge color={k.aktif ? 'green' : 'red'}>{k.aktif ? 'Aktif' : 'Pasif'}</Badge>
                </TableBodyCell>
              {/if}
              {#if visibleCols.has('islemler')}
                <TableBodyCell class="text-right">
                  <div class="flex items-center justify-end gap-2">
                    <button
                      class="rounded p-1.5 text-gray-400 hover:bg-blue-50 hover:text-blue-600 dark:hover:bg-gray-700"
                      onclick={() => duzenleAc(k)}
                      title="Düzenle"
                    >
                      <EditOutline class="h-4 w-4" />
                    </button>
                    <button
                      class="rounded p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-gray-700"
                      onclick={() => silAc(k)}
                      title="Sil"
                    >
                      <TrashBinSolid class="h-4 w-4" />
                    </button>
                  </div>
                </TableBodyCell>
              {/if}
            </TableBodyRow>
          {/snippet}
        </DataTable>
      {/if}
    </div>

    <!-- Gider Kategorileri -->
    <div>
      <div class="mb-3 flex items-center gap-2">
        <div class="h-1 w-6 rounded-full bg-red-500"></div>
        <Heading tag="h2" class="text-lg font-semibold text-gray-900 dark:text-white">
          Gider Kategorileri
        </Heading>
        <Badge color="red">{giderKategoriler.length}</Badge>
      </div>

      {#if giderKategoriler.length === 0}
        <div class="rounded-lg border border-dashed border-gray-300 p-8 text-center dark:border-gray-600">
          <P class="text-gray-500 dark:text-gray-400">Henüz gider kategorisi yok</P>
          <Button size="sm" class="mt-3 gap-2" onclick={() => { fTip = 'gider'; yeniAc(); }}>
            <PlusOutline class="h-4 w-4" /> Gider Kategorisi Ekle
          </Button>
        </div>
      {:else}
        <DataTable
          data={giderKategoriler}
          columns={kolonlar}
          searchPlaceholder="Kategori ara..."
          exportFileName="gider-kategorileri"
          emptyMessage="Sonuç bulunamadı"
        >
          {#snippet row(k, _i, visibleCols)}
            <TableBodyRow>
              {#if visibleCols.has('ad')}
                <TableBodyCell class="font-medium text-gray-900 dark:text-white">
                  {k.ad}
                </TableBodyCell>
              {/if}
              {#if visibleCols.has('aciklama')}
                <TableBodyCell class="text-sm text-gray-500 dark:text-gray-400">
                  {k.aciklama ?? '—'}
                </TableBodyCell>
              {/if}
              {#if visibleCols.has('durum')}
                <TableBodyCell>
                  <Badge color={k.aktif ? 'green' : 'red'}>{k.aktif ? 'Aktif' : 'Pasif'}</Badge>
                </TableBodyCell>
              {/if}
              {#if visibleCols.has('islemler')}
                <TableBodyCell class="text-right">
                  <div class="flex items-center justify-end gap-2">
                    <button
                      class="rounded p-1.5 text-gray-400 hover:bg-blue-50 hover:text-blue-600 dark:hover:bg-gray-700"
                      onclick={() => duzenleAc(k)}
                      title="Düzenle"
                    >
                      <EditOutline class="h-4 w-4" />
                    </button>
                    <button
                      class="rounded p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-gray-700"
                      onclick={() => silAc(k)}
                      title="Sil"
                    >
                      <TrashBinSolid class="h-4 w-4" />
                    </button>
                  </div>
                </TableBodyCell>
              {/if}
            </TableBodyRow>
          {/snippet}
        </DataTable>
      {/if}
    </div>

  {/if}
</main>

<!-- Kategori Ekle/Düzenle Modal -->
<Modal
  bind:open={modalAcik}
  title={duzenle ? 'Kategoriyi Düzenle' : 'Yeni Kategori'}
  size="md"
  autoclose={false}
>
  <div class="flex flex-col gap-4">
    {#if !duzenle}
      <div>
        <Label class="mb-2">Tür *</Label>
        <div class="flex gap-3">
          <label class="flex cursor-pointer items-center gap-2">
            <input type="radio" bind:group={fTip} value="gelir" class="text-green-600" />
            <span class="font-medium text-green-700 dark:text-green-400">Gelir</span>
          </label>
          <label class="flex cursor-pointer items-center gap-2">
            <input type="radio" bind:group={fTip} value="gider" class="text-red-600" />
            <span class="font-medium text-red-700 dark:text-red-400">Gider</span>
          </label>
        </div>
      </div>
    {/if}
    <div>
      <Label for="fAd" class="mb-2">Kategori Adı *</Label>
      <Input id="fAd" bind:value={fAd} placeholder="ör. Kira, Maaş, Elektrik..." required />
    </div>
    <div>
      <Label for="fAciklama" class="mb-2">Açıklama</Label>
      <Textarea id="fAciklama" bind:value={fAciklama} rows={2} placeholder="İsteğe bağlı açıklama..." />
    </div>
  </div>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button onclick={kaydet} disabled={kaydediliyor || !fAd.trim()}>
        {#if kaydediliyor}<Spinner class="me-2" size="4" />{/if}
        {duzenle ? 'Güncelle' : 'Kaydet'}
      </Button>
      <Button color="alternative" onclick={() => (modalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Sil Onay Modal -->
<Modal bind:open={silModalAcik} title="Kategoriyi Sil" size="sm" autoclose={false}>
  <p class="text-gray-600 dark:text-gray-400">
    <strong>{silinecek?.ad}</strong> kategorisini silmek istediğinize emin misiniz?
    Bu kategoriye ait kayıtlar varsa silme işlemi engellenecektir.
  </p>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button color="red" onclick={sil} disabled={kaydediliyor}>
        {#if kaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Sil
      </Button>
      <Button color="alternative" onclick={() => (silModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>
