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
    Card,
    Heading,
    P
  } from 'flowbite-svelte';
  import {
    PlusOutline,
    EditOutline,
    TrashBinSolid,
    WalletSolid,
    ArrowRightOutline,
    FileLinesSolid
  } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import {
    kasaApi,
    PARA_BIRIMLERI,
    formatBakiye,
    paraSembol,
    type Kasa,
    type CreateKasaInput,
    type ParaBirimi
  } from '$lib/tauri-api';
  import { exportPdf } from '$lib/pdf';
  import Can from '$lib/Can.svelte';
  import { notify } from '$lib/toast';

  // ─── State ──────────────────────────────────────────────────────────────────

  let kasalar = $state<Kasa[]>([]);
  let yukleniyor = $state(true);
  let hata = $state('');

  // Modal state
  let modalAcik = $state(false);
  let silModalAcik = $state(false);
  let duzenleKasa = $state<Kasa | null>(null);
  let silinecekKasa = $state<Kasa | null>(null);
  let kaydediliyor = $state(false);

  // Form
  let formAd = $state('');
  let formParaBirimi = $state<ParaBirimi>('TL');
  let formAciklama = $state('');

  // ─── Yükle ─────────────────────────────────────────────────────────────────

  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      kasalar = await kasaApi.getAll();
    } catch (e: any) {
      hata = e?.toString() ?? 'Bilinmeyen hata';
    } finally {
      yukleniyor = false;
    }
  }

  $effect(() => { yukle(); });

  // ─── Modal Aç/Kapat ─────────────────────────────────────────────────────────

  function yeniKasaAc() {
    duzenleKasa = null;
    formAd = '';
    formParaBirimi = 'TL';
    formAciklama = '';
    modalAcik = true;
  }

  function duzenleAc(kasa: Kasa) {
    duzenleKasa = kasa;
    formAd = kasa.ad;
    formParaBirimi = kasa.para_birimi;
    formAciklama = kasa.aciklama ?? '';
    modalAcik = true;
  }

  function silAc(kasa: Kasa) {
    silinecekKasa = kasa;
    silModalAcik = true;
  }

  // ─── CRUD ───────────────────────────────────────────────────────────────────

  async function kaydet() {
    if (!formAd.trim()) return;
    kaydediliyor = true;
    try {
      if (duzenleKasa) {
        await kasaApi.update({
          id: duzenleKasa.id,
          ad: formAd.trim(),
          para_birimi: formParaBirimi,
          aciklama: formAciklama.trim() || undefined
        });
      } else {
        const input: CreateKasaInput = {
          ad: formAd.trim(),
          para_birimi: formParaBirimi,
          aciklama: formAciklama.trim() || undefined
        };
        await kasaApi.create(input);
      }
      modalAcik = false;
      notify.success(duzenleKasa ? 'Kasa guncellendi' : 'Kasa olusturuldu');
      await yukle();
    } catch (e: any) {
      notify.apiError(e, 'Kayit hatasi');
      hata = e?.message ?? 'Kayit hatasi';
    } finally {
      kaydediliyor = false;
    }
  }

  async function sil() {
    if (!silinecekKasa) return;
    kaydediliyor = true;
    try {
      await kasaApi.delete(silinecekKasa.id);
      silModalAcik = false;
      silinecekKasa = null;
      notify.success('Kasa silindi');
      await yukle();
    } catch (e: any) {
      notify.apiError(e, 'Silme hatasi');
      hata = e?.message ?? 'Silme hatasi';
    } finally {
      kaydediliyor = false;
    }
  }

  // ─── Yardımcılar ────────────────────────────────────────────────────────────

  const paraRenk: Record<ParaBirimi, string> = {
    TL:    'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
    USD:   'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
    EUR:   'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
    ALTIN: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
  };

  const paraBirimiSecenekleri = PARA_BIRIMLERI.map((p) => ({ value: p.value, name: p.label }));

  function pdfIndir() {
    exportPdf({
      title: 'Kasa Listesi',
      subtitle: `Toplam ${kasalar.length} kasa`,
      fileName: `kasalar-${new Date().toISOString().slice(0, 10)}`,
      sections: [
        {
          kind: 'table',
          columns: ['Ad', 'Para Birimi', 'Bakiye', 'Açıklama'],
          widths: ['*', 'auto', 'auto', '*'],
          rows: kasalar.map((k) => [
            k.ad,
            k.para_birimi,
            formatBakiye(k.bakiye, k.para_birimi),
            k.aciklama ?? ''
          ])
        }
      ]
    });
  }
</script>

<main class="min-h-screen bg-gray-50 p-6 dark:bg-gray-900">
  <!-- Başlık -->
  <div class="mb-6 flex items-center justify-between">
    <div>
      <Heading tag="h1" class="text-2xl font-bold text-gray-900 dark:text-white">Kasalar</Heading>
      <P class="mt-1 text-sm text-gray-500 dark:text-gray-400">
        Toplam {kasalar.length} kasa
      </P>
    </div>
    <div class="flex gap-2">
      <Button color="alternative" onclick={pdfIndir} class="gap-2">
        <FileLinesSolid class="h-4 w-4" />
        PDF
      </Button>
      <Can permission="kasa.olustur">
        <Button onclick={yeniKasaAc} class="gap-2">
          <PlusOutline class="h-4 w-4" />
          Yeni Kasa
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

  <!-- Yükleniyor -->
  {#if yukleniyor}
    <div class="flex h-48 items-center justify-center">
      <Spinner size="10" />
    </div>

  <!-- Boş durum -->
  {:else if kasalar.length === 0}
    <div class="flex h-64 flex-col items-center justify-center rounded-xl border-2 border-dashed border-gray-300 dark:border-gray-700">
      <WalletSolid class="mb-3 h-12 w-12 text-gray-400" />
      <p class="text-gray-500 dark:text-gray-400">Henüz kasa eklenmemiş</p>
      <Can permission="kasa.olustur">
        <Button size="sm" class="mt-4 gap-2" onclick={yeniKasaAc}>
          <PlusOutline class="h-4 w-4" /> Kasa Ekle
        </Button>
      </Can>
    </div>

  <!-- Kasa Kartları -->
  {:else}
    <div class="grid grid-cols-1 gap-5 sm:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
      {#each kasalar as kasa (kasa.id)}
        <Card class="relative flex flex-col justify-between p-5 hover:shadow-lg transition-shadow">
          <!-- Badge -->
          <div class="mb-3 flex items-center justify-between">
            <span class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold {paraRenk[kasa.para_birimi]}">
              {paraSembol(kasa.para_birimi)} {kasa.para_birimi}
            </span>
            {#if !kasa.aktif}
              <Badge color="red">Pasif</Badge>
            {/if}
          </div>

          <!-- Kasa Adı -->
          <button
            class="mb-1 text-left text-lg font-bold text-gray-900 hover:text-primary-600 hover:underline dark:text-white dark:hover:text-primary-400"
            onclick={() => goto(`/kasa/${kasa.id}`)}
          >
            {kasa.ad}
          </button>

          <!-- Açıklama -->
          {#if kasa.aciklama}
            <p class="mb-3 text-sm text-gray-500 dark:text-gray-400 line-clamp-2">{kasa.aciklama}</p>
          {/if}

          <!-- Bakiye -->
          <div class="mt-auto rounded-lg bg-gray-100 px-4 py-3 dark:bg-gray-700">
            <p class="text-xs text-gray-500 dark:text-gray-400">Güncel Bakiye</p>
            <p class="text-xl font-bold {kasa.bakiye >= 0 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}">
              {formatBakiye(kasa.bakiye, kasa.para_birimi)}
            </p>
          </div>

          <!-- Aksiyonlar -->
          <div class="mt-4 flex items-center justify-between">
            <div class="flex gap-2">
              <Can permission="kasa.duzenle">
                <button
                  class="rounded p-1.5 text-gray-500 hover:bg-gray-100 hover:text-primary-600 dark:hover:bg-gray-700"
                  onclick={() => duzenleAc(kasa)}
                  title="Düzenle"
                >
                  <EditOutline class="h-4 w-4" />
                </button>
              </Can>
              <Can permission="kasa.sil">
                <button
                  class="rounded p-1.5 text-gray-500 hover:bg-red-50 hover:text-red-600 dark:hover:bg-gray-700"
                  onclick={() => silAc(kasa)}
                  title="Sil"
                >
                  <TrashBinSolid class="h-4 w-4" />
                </button>
              </Can>
            </div>
            <button
              class="flex items-center gap-1 text-sm text-primary-600 hover:underline dark:text-primary-400"
              onclick={() => goto(`/kasa/${kasa.id}`)}
            >
              Detay <ArrowRightOutline class="h-4 w-4" />
            </button>
          </div>
        </Card>
      {/each}
    </div>
  {/if}
</main>

<!-- Kasa Ekle/Düzenle Modal -->
<Modal bind:open={modalAcik} title={duzenleKasa ? 'Kasayı Düzenle' : 'Yeni Kasa Ekle'} size="md" autoclose={false}>

  <div class="flex flex-col gap-4">
    <div>
      <Label for="ad" class="mb-2">Kasa Adı *</Label>
      <Input id="ad" bind:value={formAd} placeholder="örn. Ana Kasa, Döviz Kasası..." required />
    </div>

    <div>
      <Label for="birimi" class="mb-2">Para Birimi *</Label>
      <Select id="birimi" bind:value={formParaBirimi} items={paraBirimiSecenekleri} />
    </div>

    <div>
      <Label for="aciklama" class="mb-2">Açıklama</Label>
      <Textarea id="aciklama" bind:value={formAciklama} rows={3} placeholder="Opsiyonel açıklama..." />
    </div>
  </div>

  {#snippet footer()}
    <div class="flex gap-3">
      <Button onclick={kaydet} disabled={kaydediliyor || !formAd.trim()}>
        {#if kaydediliyor}<Spinner class="me-2" size="4" />{/if}
        {duzenleKasa ? 'Güncelle' : 'Ekle'}
      </Button>
      <Button color="alternative" onclick={() => (modalAcik = false)}>İptal</Button>
    </div>
  {/snippet}
</Modal>

<!-- Sil Onay Modal -->
<Modal bind:open={silModalAcik} title="Kasa Sil" size="sm" autoclose={false}>

  <p class="text-gray-600 dark:text-gray-400">
    <strong class="text-gray-900 dark:text-white">{silinecekKasa?.ad}</strong> kasasını ve tüm hareketlerini
    kalıcı olarak silmek istediğinize emin misiniz?
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
