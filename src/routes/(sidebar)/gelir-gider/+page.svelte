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
    TagSolid,
    ArrowUpOutline,
    ArrowDownOutline
  } from 'flowbite-svelte-icons';
  import {
    gelirGiderApi,
    gelirGiderKategoriApi,
    kasaApi,
    formatBakiye,
    type GelirGiderKayit,
    type GelirGiderKategori,
    type CreateKayitInput,
    type Kasa,
    type ParaBirimi
  } from '$lib/tauri-api';
  import DataTable from '$lib/components/DataTable.svelte';
  import type { DataTableColumn } from '$lib/components/dataTableUtils';
  import Can from '$lib/Can.svelte';
  import { notify } from '$lib/toast';

  // ─── State ──────────────────────────────────────────────────────────────────

  let kayitlar = $state<GelirGiderKayit[]>([]);
  let kategoriler = $state<GelirGiderKategori[]>([]);
  let kasalar = $state<Kasa[]>([]);
  let yukleniyor = $state(true);
  let hata = $state('');

  // Filtre
  let filtreKasaId = $state<number | ''>('');
  let filtreKategoriId = $state<number | ''>('');
  let filtreTip = $state<'tumu' | 'gelir' | 'gider'>('tumu');

  // Modal
  let modalAcik = $state(false);
  let silModalAcik = $state(false);
  let silinecekId = $state<number | null>(null);
  let kaydediliyor = $state(false);

  // Form
  let fTip = $state<'gelir' | 'gider'>('gelir');
  let fKasaId = $state<number | ''>('');
  let fKategoriId = $state<number | ''>('');
  let fTarih = $state(bugun());
  let fTutar = $state('');
  let fAciklama = $state('');
  let fParaBirimi = $state<ParaBirimi | ''>('');

  // ─── Yükle ─────────────────────────────────────────────────────────────────

  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      [kayitlar, kategoriler, kasalar] = await Promise.all([
        gelirGiderApi.getAll(),
        gelirGiderKategoriApi.getAll(),
        kasaApi.getAll()
      ]);
    } catch (e: any) {
      hata = e?.toString() ?? 'Yükleme hatası';
    } finally {
      yukleniyor = false;
    }
  }

  $effect(() => { yukle(); });

  // ─── Filtrelenmiş kayıtlar ───────────────────────────────────────────────────

  const filtrelenmis = $derived(
    kayitlar.filter((k) => {
      if (filtreKasaId && k.kasa_id !== Number(filtreKasaId)) return false;
      if (filtreKategoriId && k.kategori_id !== Number(filtreKategoriId)) return false;
      if (filtreTip !== 'tumu' && k.kategori_tip !== filtreTip) return false;
      return true;
    })
  );

  const toplamGelir = $derived(
    filtrelenmis.filter((k) => k.kategori_tip === 'gelir').reduce((s, k) => s + k.tutar, 0)
  );
  const toplamGider = $derived(
    filtrelenmis.filter((k) => k.kategori_tip === 'gider').reduce((s, k) => s + k.tutar, 0)
  );

  // ─── Modal ─────────────────────────────────────────────────────────────────

  function yeniAc(tip: 'gelir' | 'gider' = 'gelir') {
    fTip = tip;
    const ilkKasa = kasalar.find((k) => k.aktif);
    fKasaId = ilkKasa?.id ?? '';
    fKategoriId = '';
    fTarih = bugun();
    fTutar = '';
    fAciklama = '';
    fParaBirimi = ilkKasa?.para_birimi ?? '';
    modalAcik = true;
  }

  async function kaydet() {
    const tutar = parseFloat(String(fTutar).replace(',', '.'));
    if (!fKasaId || !fKategoriId || !fAciklama.trim() || !fParaBirimi || isNaN(tutar) || tutar <= 0) return;
    if (secilenKasa && fParaBirimi !== secilenKasa.para_birimi) {
      notify.error(`Para birimi uyusmazligi: kasa ${secilenKasa.para_birimi}, secim ${fParaBirimi}`);
      return;
    }
    kaydediliyor = true;
    hata = '';
    try {
      const input: CreateKayitInput = {
        kasa_id: Number(fKasaId),
        kategori_id: Number(fKategoriId),
        tarih: fTarih,
        tutar,
        aciklama: fAciklama.trim(),
        para_birimi: fParaBirimi as ParaBirimi
      };
      await gelirGiderApi.create(input);
      modalAcik = false;
      notify.success(fTip === 'gelir' ? 'Gelir kaydedildi' : 'Gider kaydedildi');
      await yukle();
    } catch (e: any) {
      notify.apiError(e, 'Kayit hatasi');
      hata = e?.message ?? 'Kayit hatasi';
    } finally {
      kaydediliyor = false;
    }
  }

  // ─── Sil ───────────────────────────────────────────────────────────────────

  function silAc(id: number) {
    silinecekId = id;
    silModalAcik = true;
  }

  async function sil() {
    if (silinecekId === null) return;
    kaydediliyor = true;
    hata = '';
    try {
      await gelirGiderApi.delete(silinecekId);
      silModalAcik = false;
      silinecekId = null;
      notify.success('Kayit silindi');
      await yukle();
    } catch (e: any) {
      notify.apiError(e, 'Silme hatasi');
      hata = e?.message ?? 'Silme hatasi';
    } finally {
      kaydediliyor = false;
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

  function tutarFormat(tutar: number): string {
    return tutar.toLocaleString('tr-TR', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  const aktifKasalar = $derived(kasalar.filter((k) => k.aktif));

  const secilenKasa = $derived(
    fKasaId ? kasalar.find((k) => k.id === Number(fKasaId)) : undefined
  );

  // Kasa degistikce para birimini otomatik uyumlu hale getir
  $effect(() => {
    if (secilenKasa && fParaBirimi !== secilenKasa.para_birimi) {
      fParaBirimi = secilenKasa.para_birimi;
    }
  });

  const paraBirimleri: ParaBirimi[] = ['TL', 'USD', 'EUR', 'ALTIN'];

  // Para birimine gore toplam gelir/gider ozetleri
  const toplamlarPB = $derived.by(() => {
    const map = new Map<string, { gelir: number; gider: number }>();
    for (const k of filtrelenmis) {
      const pb = k.kasa_para_birimi ?? 'TL';
      const cur = map.get(pb) ?? { gelir: 0, gider: 0 };
      if (k.kategori_tip === 'gelir') cur.gelir += k.tutar;
      else cur.gider += k.tutar;
      map.set(pb, cur);
    }
    return Array.from(map.entries()).map(([pb, v]) => ({ pb, ...v, net: v.gelir - v.gider }));
  });

  const modalKategoriler = $derived(
    kategoriler.filter((k) => k.tip === fTip && k.aktif)
  );

  // ─── Kolonlar ────────────────────────────────────────────────────────────────

  const kolonlar: DataTableColumn<GelirGiderKayit>[] = [
    { id: 'tarih', header: 'Tarih', accessor: 'tarih' },
    { id: 'tip', header: 'Tür', accessor: 'kategori_tip', sortable: false, searchable: false },
    { id: 'kategori', header: 'Kategori', accessor: 'kategori_ad' },
    { id: 'aciklama', header: 'Açıklama', accessor: 'aciklama' },
    { id: 'kasa', header: 'Kasa', accessor: 'kasa_ad' },
    { id: 'tutar', header: 'Tutar', accessor: 'tutar', align: 'right' },
    { id: 'islemler', header: '', accessor: () => '', sortable: false, searchable: false }
  ];
</script>

<main class="min-h-screen bg-gray-50 p-6 dark:bg-gray-900">

  <!-- Başlık -->
  <div class="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
    <div class="flex items-center gap-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-indigo-100 dark:bg-indigo-900">
        <ArrowUpOutline class="h-5 w-5 text-green-600 dark:text-green-400" />
      </div>
      <div>
        <Heading tag="h1" class="text-2xl font-bold text-gray-900 dark:text-white">
          Gelir / Gider
        </Heading>
        <P class="text-sm text-gray-500 dark:text-gray-400">
          {filtrelenmis.length} kayıt
        </P>
      </div>
    </div>
    <div class="flex flex-wrap gap-2">
      <Button color="alternative" href="/gelir-gider/kategoriler" class="gap-2">
        <TagSolid class="h-4 w-4" /> Kategoriler
      </Button>
      <Can permission="gelir_gider.yonet">
        <Button color="green" onclick={() => yeniAc('gelir')} class="gap-2">
          <ArrowUpOutline class="h-4 w-4" /> Gelir Ekle
        </Button>
      </Can>
      <Can permission="gelir_gider.yonet">
        <Button color="red" onclick={() => yeniAc('gider')} class="gap-2">
          <ArrowDownOutline class="h-4 w-4" /> Gider Ekle
        </Button>
      </Can>
    </div>
  </div>

  <!-- Hata -->
  {#if hata}
    <div class="mb-4 rounded-lg bg-red-50 p-4 text-sm text-red-700 dark:bg-red-900/20 dark:text-red-400">
      {hata}
    </div>
  {/if}

  <!-- Özet Kartlar (para birimine göre) -->
  <div class="mb-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
    {#each toplamlarPB as t (t.pb)}
      <div class="rounded-xl border border-gray-200 bg-white p-4 shadow-sm dark:border-gray-700 dark:bg-gray-800">
        <p class="text-sm font-medium text-gray-500 dark:text-gray-400">{t.pb}</p>
        <div class="mt-1 flex items-baseline justify-between gap-2">
          <span class="text-xs text-green-600 dark:text-green-400">Gelir</span>
          <span class="font-semibold text-green-600 dark:text-green-400">{tutarFormat(t.gelir)}</span>
        </div>
        <div class="flex items-baseline justify-between gap-2">
          <span class="text-xs text-red-600 dark:text-red-400">Gider</span>
          <span class="font-semibold text-red-600 dark:text-red-400">{tutarFormat(t.gider)}</span>
        </div>
        <div class="mt-1 flex items-baseline justify-between gap-2 border-t border-gray-200 pt-1 dark:border-gray-700">
          <span class="text-xs text-gray-500 dark:text-gray-400">Net</span>
          <span class="font-bold {t.net >= 0 ? 'text-gray-900 dark:text-white' : 'text-red-600 dark:text-red-400'}">
            {tutarFormat(t.net)} {t.pb}
          </span>
        </div>
      </div>
    {:else}
      <div class="rounded-xl border border-dashed border-gray-200 bg-white p-4 text-sm text-gray-500 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-400">
        Filtreye uyan kayıt yok
      </div>
    {/each}
  </div>

  <!-- Filtreler -->
  <div class="mb-4 flex flex-wrap items-center gap-3">
    <div class="flex gap-2">
      {#each (['tumu', 'gelir', 'gider'] as const) as tip}
        <button
          class="rounded-lg px-3 py-1.5 text-sm font-medium transition-colors
            {filtreTip === tip
              ? tip === 'gelir' ? 'bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300'
              : tip === 'gider' ? 'bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-300'
              : 'bg-primary-100 text-primary-700 dark:bg-primary-900/40 dark:text-primary-300'
              : 'bg-white text-gray-600 hover:bg-gray-100 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700'}"
          onclick={() => (filtreTip = tip)}
        >
          {tip === 'tumu' ? 'Tümü' : tip === 'gelir' ? 'Gelir' : 'Gider'}
        </button>
      {/each}
    </div>

    <select
      bind:value={filtreKasaId}
      class="rounded-lg border border-gray-300 bg-white px-3 py-1.5 text-sm text-gray-700 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-200"
    >
      <option value="">Tüm Kasalar</option>
      {#each kasalar as k}
        <option value={k.id}>{k.ad}</option>
      {/each}
    </select>

    <select
      bind:value={filtreKategoriId}
      class="rounded-lg border border-gray-300 bg-white px-3 py-1.5 text-sm text-gray-700 dark:border-gray-600 dark:bg-gray-800 dark:text-gray-200"
    >
      <option value="">Tüm Kategoriler</option>
      {#each kategoriler as k}
        <option value={k.id}>{k.ad} ({k.tip})</option>
      {/each}
    </select>

    {#if filtreKasaId || filtreKategoriId || filtreTip !== 'tumu'}
      <button
        class="text-sm text-gray-400 hover:text-gray-600 dark:hover:text-gray-200"
        onclick={() => { filtreKasaId = ''; filtreKategoriId = ''; filtreTip = 'tumu'; }}
      >
        Filtreyi temizle ×
      </button>
    {/if}
  </div>

  <!-- Tablo -->
  {#if yukleniyor}
    <div class="flex h-48 items-center justify-center">
      <Spinner size="10" />
    </div>
  {:else}
    <DataTable
      data={filtrelenmis}
      columns={kolonlar}
      searchPlaceholder="Açıklama, kategori ara..."
      exportFileName="gelir-gider"
      emptyMessage="Kayıt bulunamadı"
    >
      {#snippet row(k, _i, visibleCols)}
        <TableBodyRow>
          {#if visibleCols.has('tarih')}
            <TableBodyCell class="text-sm text-gray-500 dark:text-gray-400">
              {tarihFormat(k.tarih)}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('tip')}
            <TableBodyCell>
              <Badge color={k.kategori_tip === 'gelir' ? 'green' : 'red'}>
                {k.kategori_tip === 'gelir' ? 'Gelir' : 'Gider'}
              </Badge>
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('kategori')}
            <TableBodyCell class="text-sm text-gray-700 dark:text-gray-300">
              {k.kategori_ad}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('aciklama')}
            <TableBodyCell class="font-medium text-gray-900 dark:text-white">
              {k.aciklama}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('kasa')}
            <TableBodyCell class="text-sm text-gray-500 dark:text-gray-400">
              {k.kasa_ad}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('tutar')}
            <TableBodyCell class="text-right font-bold {k.kategori_tip === 'gelir' ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}">
              {k.kategori_tip === 'gider' ? '-' : ''}{tutarFormat(k.tutar)} {k.kasa_para_birimi ?? ''}
            </TableBodyCell>
          {/if}
          {#if visibleCols.has('islemler')}
            <TableBodyCell class="text-center">
              <Can permission="gelir_gider.yonet">
                <button
                  class="rounded p-1.5 text-gray-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-gray-700"
                  onclick={() => silAc(k.id)}
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
        <div class="flex flex-col items-center justify-center py-10">
          <P class="mb-4 text-gray-500 dark:text-gray-400">Henüz kayıt yok</P>
          <div class="flex gap-3">
            <Can permission="gelir_gider.yonet">
              <Button size="sm" color="green" class="gap-2" onclick={() => yeniAc('gelir')}>
                <ArrowUpOutline class="h-4 w-4" /> Gelir Ekle
              </Button>
            </Can>
            <Can permission="gelir_gider.yonet">
              <Button size="sm" color="red" class="gap-2" onclick={() => yeniAc('gider')}>
                <ArrowDownOutline class="h-4 w-4" /> Gider Ekle
              </Button>
            </Can>
          </div>
        </div>
      {/snippet}
    </DataTable>
  {/if}
</main>

<!-- Kayıt Ekle Modal -->
<Modal
  bind:open={modalAcik}
  title={fTip === 'gelir' ? 'Gelir Ekle' : 'Gider Ekle'}
  size="md"
  autoclose={false}
>
  <div class="flex flex-col gap-4">
    <!-- Tür seçimi -->
    <div class="flex gap-3">
      <button
        class="flex-1 rounded-lg border-2 py-2 text-sm font-medium transition-colors
          {fTip === 'gelir' ? 'border-green-500 bg-green-50 text-green-700 dark:bg-green-900/20 dark:text-green-300' : 'border-gray-200 text-gray-500 hover:border-gray-300 dark:border-gray-600 dark:text-gray-400'}"
        onclick={() => { fTip = 'gelir'; fKategoriId = ''; }}
      >
        <ArrowUpOutline class="mx-auto mb-1 h-5 w-5" />
        Gelir
      </button>
      <button
        class="flex-1 rounded-lg border-2 py-2 text-sm font-medium transition-colors
          {fTip === 'gider' ? 'border-red-500 bg-red-50 text-red-700 dark:bg-red-900/20 dark:text-red-300' : 'border-gray-200 text-gray-500 hover:border-gray-300 dark:border-gray-600 dark:text-gray-400'}"
        onclick={() => { fTip = 'gider'; fKategoriId = ''; }}
      >
        <ArrowDownOutline class="mx-auto mb-1 h-5 w-5" />
        Gider
      </button>
    </div>

    <div>
      <Label for="fKasaId" class="mb-2">Kasa *</Label>
      <Select id="fKasaId" bind:value={fKasaId} required>
        <option value="">Kasa seçin...</option>
        {#each aktifKasalar as k}
          <option value={k.id}>{k.ad} ({k.para_birimi})</option>
        {/each}
      </Select>
    </div>

    <div>
      <Label for="fParaBirimi" class="mb-2">Para Birimi *</Label>
      <Select id="fParaBirimi" bind:value={fParaBirimi} required>
        <option value="">Para birimi seçin...</option>
        {#each paraBirimleri as pb}
          <option value={pb}>{pb}</option>
        {/each}
      </Select>
      {#if secilenKasa && fParaBirimi && fParaBirimi !== secilenKasa.para_birimi}
        <p class="mt-1 text-xs font-medium text-red-600 dark:text-red-400">
          Uyarı: Seçtiğiniz kasa <b>{secilenKasa.para_birimi}</b> biriminde,
          ama para birimi olarak <b>{fParaBirimi}</b> seçtiniz. Aynı birime sahip bir kasa seçin
          veya tutarı ilgili birime çevirin.
        </p>
      {:else if secilenKasa}
        <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
          Kasa birimi: <b>{secilenKasa.para_birimi}</b>
        </p>
      {/if}
    </div>

    <div>
      <Label for="fKategoriId" class="mb-2">Kategori *</Label>
      <Select id="fKategoriId" bind:value={fKategoriId} required>
        <option value="">Kategori seçin...</option>
        {#each modalKategoriler as k}
          <option value={k.id}>{k.ad}</option>
        {/each}
      </Select>
      {#if modalKategoriler.length === 0}
        <p class="mt-1 text-xs text-amber-600 dark:text-amber-400">
          Bu tür için kategori yok.
          <a href="/gelir-gider/kategoriler" class="underline">Kategori ekle</a>
        </p>
      {/if}
    </div>

    <div>
      <Label for="fTarih" class="mb-2">Tarih *</Label>
      <Input id="fTarih" type="date" bind:value={fTarih} required />
    </div>

    <div>
      <Label for="fTutar" class="mb-2">Tutar{fParaBirimi ? ` (${fParaBirimi})` : ''} *</Label>
      <Input
        id="fTutar"
        type="number"
        step="0.01"
        min="0.01"
        bind:value={fTutar}
        placeholder="0.00"
        required
      />
    </div>

    <div>
      <Label for="fAciklama" class="mb-2">Açıklama *</Label>
      <Textarea
        id="fAciklama"
        bind:value={fAciklama}
        rows={2}
        placeholder="Kısa açıklama girin..."
        required
      />
    </div>
  </div>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button
        color={fTip === 'gelir' ? 'green' : 'red'}
        onclick={kaydet}
        disabled={kaydediliyor || !fKasaId || !fKategoriId || !fAciklama.trim() || !fTutar || !fParaBirimi || (!!secilenKasa && fParaBirimi !== secilenKasa.para_birimi)}
      >
        {#if kaydediliyor}<Spinner class="me-2" size="4" />{/if}
        Kaydet
      </Button>
      <Button color="alternative" onclick={() => (modalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Sil Onay Modal -->
<Modal bind:open={silModalAcik} title="Kaydı Sil" size="sm" autoclose={false}>
  <p class="text-gray-600 dark:text-gray-400">
    Bu gelir/gider kaydını silmek istediğinize emin misiniz?
    İlgili kasa hareketi de silinerek bakiye yeniden hesaplanacaktır.
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
