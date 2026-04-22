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
    TrashBinSolid,
    ArrowRightOutline,
    LayersSolid,
    UsersSolid,
    FileLinesSolid
  } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import {
    hisseApi,
    hisseAtamaApi,
    hissedarApi,
    type Hisse,
    type Hissedar,
    type CreateHisseInput,
    type CreateHisseTopluInput,
    type AtamaInput
  } from '$lib/tauri-api';
  import { hissedarLabel, hissedarLabelFromFields } from '$lib/hissedarFormat';
  import DataTable from '$lib/components/DataTable.svelte';
  import type { DataTableColumn } from '$lib/components/dataTableUtils';
  import { exportPdf } from '$lib/pdf';

  // ─── State ──────────────────────────────────────────────────────────────────

  let hisseler = $state<Hisse[]>([]);
  let yukleniyor = $state(true);
  let hata = $state('');

  // DataTable export (sıralanış + filtrelenmiş + görünür kolonlar)
  let pdfRows = $state<Hisse[]>([]);
  let pdfCols = $state<DataTableColumn<Hisse>[]>([]);

  // Tek hisse modal
  let tekModalAcik = $state(false);
  let tekAciklama = $state('');
  let tekAtamaHissedarId = $state<number | ''>('');
  let tekAtamaTarih = $state(bugunTarih());
  let tekAtamaUcret = $state('0');
  let tekKaydediliyor = $state(false);

  // Toplu oluşturma modal
  let topluModalAcik = $state(false);
  let topluAdet = $state('');
  let topluAciklama = $state('');
  let topluAtamaHissedarId = $state<number | ''>('');
  let topluAtamaTarih = $state(bugunTarih());
  let topluAtamaUcret = $state('0');
  let topluKaydediliyor = $state(false);

  // Silme modal
  let silModalAcik = $state(false);
  let silinecek = $state<Hisse | null>(null);
  let silKaydediliyor = $state(false);

  // Atama modal
  let atamaModalAcik = $state(false);
  let atamaHedefHisse = $state<Hisse | null>(null);
  let hissedarlar = $state<Hissedar[]>([]);
  let atamaHissedarId = $state<number | ''>('');
  let atamaTarih = $state(bugunTarih());
  let atamaUcret = $state('0');
  let atamaAciklama = $state('');
  let atamaKaydediliyor = $state(false);

  function bugunTarih(): string {
    return new Date().toISOString().slice(0, 10);
  }

  // ─── Yükle ─────────────────────────────────────────────────────────────────

  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      [hisseler, hissedarlar] = await Promise.all([
        hisseApi.getAll(),
        hissedarApi.getAll()
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

  const musaitSayisi = $derived(hisseler.filter((h) => h.durum === 'musait').length);
  const atanmisSayisi = $derived(hisseler.filter((h) => h.durum === 'atanmis').length);

  // ─── Kolonlar ───────────────────────────────────────────────────
  const kolonlar: DataTableColumn<Hisse>[] = [
    { id: 'kod', header: 'Hisse Kodu', accessor: 'kod' },
    { id: 'durum', header: 'Durum', accessor: (h) => h.durum === 'musait' ? 'Müsait' : h.durum === 'atanmis' ? 'Atanmış' : 'Satıldı' },
    {
      id: 'hissedar',
      header: 'Hissedar / Açıklama',
      accessor: (h) =>
        h.durum === 'atanmis' && h.hissedar_ad
          ? `${h.hissedar_ad} ${h.hissedar_soyad ?? ''} ${h.aciklama ?? ''}`
          : h.aciklama ?? ''
    },
    {
      id: 'olusturulma',
      header: 'Oluşturulma',
      accessor: (h) => h.created_at
    },
    { id: 'islemler', header: '', accessor: () => '', sortable: false, searchable: false }
  ];

  // ─── Tek Hisse Oluştur ──────────────────────────────────────────────────────

  function tekAc() {
    tekAciklama = '';
    tekAtamaHissedarId = '';
    tekAtamaTarih = bugunTarih();
    tekAtamaUcret = '0';
    tekModalAcik = true;
  }

  async function tekKaydet() {
    tekKaydediliyor = true;
    hata = '';
    try {
      const input: CreateHisseInput = {
        aciklama: tekAciklama.trim() || undefined,
        atama_hissedar_id: tekAtamaHissedarId !== '' ? Number(tekAtamaHissedarId) : undefined,
        atama_tarih: tekAtamaHissedarId !== '' ? tekAtamaTarih : undefined,
        atama_ucret: tekAtamaHissedarId !== '' ? (parseFloat(String(tekAtamaUcret).replace(',', '.')) || 0) : undefined
      };
      await hisseApi.create(input);
      tekModalAcik = false;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Oluşturma hatası';
    } finally {
      tekKaydediliyor = false;
    }
  }

  // ─── Toplu Hisse Oluştur ────────────────────────────────────────────────────

  function topluAc() {
    topluAdet = '';
    topluAciklama = '';
    topluAtamaHissedarId = '';
    topluAtamaTarih = bugunTarih();
    topluAtamaUcret = '0';
    topluModalAcik = true;
  }

  async function topluKaydet() {
    const adet = parseInt(String(topluAdet));
    if (isNaN(adet) || adet <= 0 || adet > 500) return;
    topluKaydediliyor = true;
    hata = '';
    try {
      const input: CreateHisseTopluInput = {
        adet,
        aciklama: topluAciklama.trim() || undefined,
        atama_hissedar_id: topluAtamaHissedarId !== '' ? Number(topluAtamaHissedarId) : undefined,
        atama_tarih: topluAtamaHissedarId !== '' ? topluAtamaTarih : undefined,
        atama_ucret: topluAtamaHissedarId !== '' ? (parseFloat(String(topluAtamaUcret).replace(',', '.')) || 0) : undefined
      };
      await hisseApi.createToplu(input);
      topluModalAcik = false;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Toplu oluşturma hatası';
    } finally {
      topluKaydediliyor = false;
    }
  }

  // ─── Atama ──────────────────────────────────────────────────────────────────

  const aktifHissedarlar = $derived(hissedarlar.filter((h) => h.aktif));

  function atamaAc(hisse: Hisse) {
    atamaHedefHisse = hisse;
    atamaHissedarId = '';
    atamaTarih = bugunTarih();
    atamaUcret = '0';
    atamaAciklama = '';
    atamaModalAcik = true;
  }

  async function atamaKaydet() {
    if (!atamaHedefHisse || !atamaHissedarId) return;
    const ucret = parseFloat(String(atamaUcret).replace(',', '.'));
    if (isNaN(ucret) || ucret < 0) return;
    atamaKaydediliyor = true;
    hata = '';
    try {
      const input: AtamaInput = {
        hisse_id: atamaHedefHisse.id,
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

  // ─── Sil ────────────────────────────────────────────────────────────────────

  function silAc(hisse: Hisse) {
    silinecek = hisse;
    silModalAcik = true;
  }

  async function sil() {
    if (!silinecek) return;
    silKaydediliyor = true;
    hata = '';
    try {
      await hisseApi.delete(silinecek.id);
      silModalAcik = false;
      silinecek = null;
      await yukle();
    } catch (e: any) {
      hata = e?.toString() ?? 'Silme hatası';
    } finally {
      silKaydediliyor = false;
    }
  }

  // ─── PDF ────────────────────────────────────────────────────────────────────

  function pdfIndir() {
    const satildiSayisi = pdfRows.filter((h) => h.durum === 'satildi').length;
    const gorCols = pdfCols.filter((c) => c.id !== 'islemler');
    exportPdf({
      title: 'Hisse Listesi',
      subtitle: `${pdfRows.length} kayıt · Müsait ${pdfRows.filter(h=>h.durum==='musait').length} · Atanmış ${pdfRows.filter(h=>h.durum==='atanmis').length} · Satıldı ${satildiSayisi}`,
      fileName: `hisseler-${new Date().toISOString().slice(0, 10)}`,
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
  <div class="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
    <div class="flex items-center gap-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-indigo-100 dark:bg-indigo-900">
        <LayersSolid class="h-5 w-5 text-indigo-600 dark:text-indigo-300" />
      </div>
      <div>
        <Heading tag="h1" class="text-2xl font-bold text-gray-900 dark:text-white">Hisseler</Heading>
        <p class="text-sm text-gray-500 dark:text-gray-400">
          Toplam: {hisseler.length} · Müsait: {musaitSayisi} · Atanmış: {atanmisSayisi}
        </p>
      </div>
    </div>
    <div class="flex gap-2">
      <Button size="sm" color="alternative" class="gap-2" onclick={pdfIndir}>
        <FileLinesSolid class="h-4 w-4" /> PDF
      </Button>
      <Button size="sm" color="light" class="gap-2" onclick={topluAc}>
        <PlusOutline class="h-4 w-4" /> Toplu Oluştur
      </Button>
      <Button size="sm" color="primary" class="gap-2" onclick={tekAc}>
        <PlusOutline class="h-4 w-4" /> Yeni Hisse
      </Button>
    </div>
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
    <DataTable
      data={hisseler}
      columns={kolonlar}
      searchPlaceholder="Hisse kodu, hissedar veya açıklama ara..."
      exportFileName="hisseler"
      emptyMessage="Henüz hisse oluşturulmadı"
      bind:exportRows={pdfRows}
      bind:exportVisibleCols={pdfCols}
    >
      {#snippet row(hisse, _i, visibleCols)}
        <TableBodyRow
          class="cursor-pointer"
          onclick={() => goto(`/hisse/${hisse.id}`)}
        >
          {#if visibleCols.has('kod')}
            <TableBodyCell>
              <span class="font-mono font-semibold text-indigo-600 dark:text-indigo-400">
                {hisse.kod}
              </span>
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('durum')}
            <TableBodyCell>
              <Badge color={hisse.durum === 'musait' ? 'green' : hisse.durum === 'atanmis' ? 'yellow' : 'dark'}>
                {hisse.durum === 'musait' ? 'Müsait' : hisse.durum === 'atanmis' ? 'Atanmış' : 'Satıldı'}
              </Badge>
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('hissedar')}
            <TableBodyCell class="text-gray-500 dark:text-gray-400">
              {#if hisse.durum === 'atanmis' && hisse.hissedar_ad}
                <div class="flex items-center gap-1.5">
                  <UsersSolid class="h-3.5 w-3.5 text-yellow-500 flex-shrink-0" />
                  <span class="font-medium text-gray-800 dark:text-gray-200">
                    {hissedarLabelFromFields(hisse.hissedar_id, hisse.hissedar_ad, hisse.hissedar_soyad, hissedarlar)}
                  </span>
                </div>
                {#if hisse.aciklama}
                  <p class="mt-0.5 text-xs text-gray-400">{hisse.aciklama}</p>
                {/if}
              {:else if hisse.aciklama}
                {hisse.aciklama}
              {:else}
                <span class="text-gray-300 dark:text-gray-600">—</span>
              {/if}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('olusturulma')}
            <TableBodyCell class="text-sm text-gray-500 dark:text-gray-400">
              {new Date(hisse.created_at).toLocaleDateString('tr-TR')}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('islemler')}
            <TableBodyCell>
              <div class="flex items-center gap-1" onclick={(e) => e.stopPropagation()} role="presentation">
                {#if hisse.durum === 'musait'}
                  <button
                    class="flex items-center gap-1 rounded px-2 py-1 text-xs font-medium text-green-700 bg-green-50 hover:bg-green-100 dark:bg-green-900/30 dark:text-green-400 dark:hover:bg-green-900/50"
                    onclick={() => atamaAc(hisse)}
                    title="Hissedara Ata"
                  >
                    <UsersSolid class="h-3.5 w-3.5" /> Ata
                  </button>
                {/if}
                <button
                  class="rounded p-1.5 text-gray-400 hover:bg-indigo-50 hover:text-indigo-600 dark:hover:bg-gray-700"
                  onclick={() => goto(`/hisse/${hisse.id}`)}
                  title="Detay"
                >
                  <ArrowRightOutline class="h-4 w-4" />
                </button>
                {#if hisse.durum === 'musait'}
                  <button
                    class="rounded p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-gray-700"
                    onclick={() => silAc(hisse)}
                    title="Sil"
                  >
                    <TrashBinSolid class="h-4 w-4" />
                  </button>
                {/if}
              </div>
            </TableBodyCell>
          {/if}
        </TableBodyRow>
      {/snippet}
      {#snippet empty()}
        <div class="flex flex-col items-center justify-center py-6">
          <LayersSolid class="mb-3 h-10 w-10 text-gray-400" />
          <P class="text-gray-500 dark:text-gray-400">Kayıt bulunamadı</P>
          <div class="mt-4 flex gap-2">
            <Button size="sm" color="light" onclick={topluAc}>Toplu Oluştur</Button>
            <Button size="sm" onclick={tekAc}>Yeni Hisse</Button>
          </div>
        </div>
      {/snippet}
    </DataTable>
  {/if}
</main>

<!-- Tek Hisse Modal -->
<Modal bind:open={tekModalAcik} title="Yeni Hisse Oluştur" size="md" autoclose={false}>
  <div class="flex flex-col gap-4">
    <p class="text-sm text-gray-500 dark:text-gray-400">
      Hisse kodu otomatik olarak oluşturulacaktır.
    </p>
    <div>
      <Label for="tekAciklama" class="mb-2">Açıklama (isteğe bağlı)</Label>
      <Textarea id="tekAciklama" bind:value={tekAciklama} rows={2} placeholder="Hisse açıklaması..." />
    </div>
    <!-- İsteğe bağlı atama -->
    <div>
      <Label for="tekAtamaHissedar" class="mb-2">Hissedara Ata (isteğe bağlı)</Label>
      <Select
        id="tekAtamaHissedar"
        bind:value={tekAtamaHissedarId}
        placeholder="— Atamasız oluştur —"
        items={aktifHissedarlar.map((h) => ({ value: h.id, name: hissedarLabel(h) }))}
      />
    </div>
    {#if tekAtamaHissedarId !== ''}
      <div class="grid grid-cols-2 gap-3">
        <div>
          <Label for="tekAtamaTarih" class="mb-2">Atama Tarihi *</Label>
          <Input id="tekAtamaTarih" type="date" bind:value={tekAtamaTarih} required />
        </div>
        <div>
          <Label for="tekAtamaUcret" class="mb-2">Satın Alma Ücreti (₺)</Label>
          <Input id="tekAtamaUcret" type="number" step="0.01" min="0" bind:value={tekAtamaUcret} placeholder="0.00" />
        </div>
      </div>
      {#if parseFloat(String(tekAtamaUcret).replace(',', '.')) > 0}
        <p class="rounded-lg bg-orange-50 px-3 py-2 text-xs text-orange-700 dark:bg-orange-900/20 dark:text-orange-400">
          Bu tutar hissedarın cüzdanına borç olarak işlenecektir.
        </p>
      {/if}
    {/if}
  </div>
  {#snippet footer()}
    <div class="flex gap-3">
      <Button color="primary" onclick={tekKaydet} disabled={tekKaydediliyor}>
        {#if tekKaydediliyor}<Spinner class="me-2" size="4" />{/if}
        {tekAtamaHissedarId !== '' ? 'Oluştur & Ata' : 'Oluştur'}
      </Button>
      <Button color="alternative" onclick={() => (tekModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Toplu Hisse Modal -->
<Modal bind:open={topluModalAcik} title="Toplu Hisse Oluştur" size="md" autoclose={false}>
  <div class="flex flex-col gap-4">
    <p class="text-sm text-gray-500 dark:text-gray-400">
      Belirlediğiniz adet kadar hisse otomatik numaralandırılarak oluşturulur.
    </p>
    <div>
      <Label for="topluAdet" class="mb-2">Hisse Adedi (1–500) *</Label>
      <Input
        id="topluAdet"
        type="number"
        min="1"
        max="500"
        bind:value={topluAdet}
        placeholder="Örn: 10"
        required
      />
    </div>
    <div>
      <Label for="topluAciklama" class="mb-2">Açıklama (isteğe bağlı)</Label>
      <Textarea id="topluAciklama" bind:value={topluAciklama} rows={2} placeholder="Tüm hisselere ortak açıklama..." />
    </div>
    <!-- İsteğe bağlı atama -->
    <div>
      <Label for="topluAtamaHissedar" class="mb-2">Hissedara Ata (isteğe bağlı)</Label>
      <Select
        id="topluAtamaHissedar"
        bind:value={topluAtamaHissedarId}
        placeholder="— Atamasız oluştur —"
        items={aktifHissedarlar.map((h) => ({ value: h.id, name: hissedarLabel(h) }))}
      />
    </div>
    {#if topluAtamaHissedarId !== ''}
      <div class="grid grid-cols-2 gap-3">
        <div>
          <Label for="topluAtamaTarih" class="mb-2">Atama Tarihi *</Label>
          <Input id="topluAtamaTarih" type="date" bind:value={topluAtamaTarih} required />
        </div>
        <div>
          <Label for="topluAtamaUcret" class="mb-2">Hisse Başı Ücret (₺)</Label>
          <Input id="topluAtamaUcret" type="number" step="0.01" min="0" bind:value={topluAtamaUcret} placeholder="0.00" />
        </div>
      </div>
      <p class="rounded-lg bg-blue-50 px-3 py-2 text-xs text-blue-700 dark:bg-blue-900/20 dark:text-blue-400">
        Belirtilen ücret her hisseye ayrı ayrı uygulanır.
        {#if parseFloat(String(topluAtamaUcret).replace(',', '.')) > 0 && parseInt(String(topluAdet)) > 0}
          Toplam: {(parseFloat(String(topluAtamaUcret).replace(',', '.')) * parseInt(String(topluAdet))).toLocaleString('tr-TR', { minimumFractionDigits: 2 })} ₺
        {/if}
      </p>
    {/if}
  </div>
  {#snippet footer()}
    <div class="flex gap-3">
      <Button
        color="primary"
        onclick={topluKaydet}
        disabled={topluKaydediliyor || !topluAdet || parseInt(String(topluAdet)) <= 0}
      >
        {#if topluKaydediliyor}<Spinner class="me-2" size="4" />{/if}
        {topluAtamaHissedarId !== '' ? 'Oluştur & Ata' : 'Oluştur'}
      </Button>
      <Button color="alternative" onclick={() => (topluModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Silme Onay Modal -->
<Modal bind:open={silModalAcik} title="Hisseyi Sil" size="sm" autoclose={false}>
  <p class="text-gray-600 dark:text-gray-400">
    <strong class="font-mono text-indigo-600">{silinecek?.kod}</strong> hissesini silmek istediğinize emin misiniz?
  </p>
  {#snippet footer()}
    <div class="flex gap-3">
      <Button color="red" onclick={sil} disabled={silKaydediliyor}>
        {#if silKaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Evet, Sil
      </Button>
      <Button color="alternative" onclick={() => (silModalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Hissedara Ata Modal -->
<Modal bind:open={atamaModalAcik} title="Hissedara Ata" size="md" autoclose={false}>
  <div class="flex flex-col gap-4">

    {#if atamaHedefHisse}
      <div class="rounded-lg border border-indigo-200 bg-indigo-50 p-3 dark:border-indigo-700 dark:bg-indigo-900/20">
        <p class="text-xs font-semibold uppercase tracking-wide text-indigo-600 dark:text-indigo-400">Hisse</p>
        <p class="font-mono font-bold text-indigo-700 dark:text-indigo-300">{atamaHedefHisse.kod}</p>
        {#if atamaHedefHisse.aciklama}
          <p class="mt-0.5 text-xs text-indigo-500 dark:text-indigo-400">{atamaHedefHisse.aciklama}</p>
        {/if}
      </div>
    {/if}

    <div>
      <Label for="lstAatamaHissedar" class="mb-2">Hissedar *</Label>
      <Select
        id="lstAatamaHissedar"
        bind:value={atamaHissedarId}
        items={aktifHissedarlar.map((h) => ({
          value: h.id,
          name: hissedarLabel(h)
        }))}
        placeholder="Hissedar seçin..."
      />
    </div>

    <div>
      <Label for="lstAtamaTarih" class="mb-2">Tarih *</Label>
      <Input id="lstAtamaTarih" type="date" bind:value={atamaTarih} required />
    </div>

    <div>
      <Label for="lstAatamaUcret" class="mb-2">Ücret (₺) — 0 girilebilir</Label>
      <Input
        id="lstAatamaUcret"
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
      <Label for="lstAatamaAciklama" class="mb-2">Açıklama (isteğe bağlı)</Label>
      <Textarea
        id="lstAatamaAciklama"
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
