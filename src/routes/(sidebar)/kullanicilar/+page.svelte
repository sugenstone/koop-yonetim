<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Button,
    Badge,
    Modal,
    Label,
    Input,
    Select,
    Toggle,
    Spinner,
    Heading,
    P,
    Alert
  } from 'flowbite-svelte';
  import {
    PlusOutline,
    EditOutline,
    TrashBinSolid,
    LockSolid,
    UsersSolid,
    ExclamationCircleSolid,
    DatabaseSolid,
    ArrowDownToBracketOutline
  } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import {
    kullaniciApi,
    ROL_ETIKETLERI,
    ROL_ACIKLAMALARI,
    type Kullanici,
    type KullaniciRol
  } from '$lib/tauri-api';
  import { getCurrentUser, invokeApi, getToken } from '$lib/api-client';

  // ─── Yetki Kontrolü ─────────────────────────────────────────────────────────
  const currentUser = getCurrentUser();
  const isAdmin = currentUser?.rol === 'admin';

  // ─── State ──────────────────────────────────────────────────────────────────
  let kullanicilar = $state<Kullanici[]>([]);
  let yukleniyor = $state(true);
  let hata = $state('');

  // Modal: ekle / düzenle
  let formModal = $state(false);
  let duzenle = $state<Kullanici | null>(null);
  let kaydediliyor = $state(false);

  // Form
  let fAd = $state('');
  let fEmail = $state('');
  let fSifre = $state('');
  let fRol = $state<KullaniciRol>('izleyici');
  let fAktif = $state(true);

  // Silme modal
  let silModal = $state(false);
  let silinecek = $state<Kullanici | null>(null);

  // Şifre değiştir modal
  let sifreModal = $state(false);
  let sifreHedef = $state<Kullanici | null>(null);
  let fYeniSifre = $state('');

  // Yedekleme / Sıfırlama
  let yedekModal = $state(false);
  let sifirlaModal = $state(false);
  let adminSifre = $state('');
  let dbIslem = $state(false);
  let dbMesaj = $state('');

  // Bekleyen basvurular
  type Bekleyen = {
    id: number;
    ad: string;
    email: string;
    rol: string;
    onay_durumu: string;
    kayit_tarihi: string;
  };
  let bekleyenler = $state<Bekleyen[]>([]);
  let secilenRoller = $state<Record<number, KullaniciRol>>({});

  const ROLLER: { value: KullaniciRol; name: string }[] = [
    { value: 'admin', name: ROL_ETIKETLERI.admin },
    { value: 'muhasebe', name: ROL_ETIKETLERI.muhasebe },
    { value: 'uye', name: ROL_ETIKETLERI.uye },
    { value: 'izleyici', name: ROL_ETIKETLERI.izleyici }
  ];

  // ─── Yükle ─────────────────────────────────────────────────────────────────
  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      kullanicilar = await kullaniciApi.list();
    } catch (e: any) {
      hata = e.message ?? 'Kullanıcılar yüklenemedi';
    } finally {
      yukleniyor = false;
    }
  }

  onMount(() => {
    if (!isAdmin) {
      goto('/dashboard');
      return;
    }
    yukle();
    yukleBekleyenler();
  });

  async function yukleBekleyenler() {
    try {
      bekleyenler = await invokeApi<Bekleyen[]>('get_bekleyenler');
      // her biri icin default rol izleyici
      for (const b of bekleyenler) {
        if (!secilenRoller[b.id]) secilenRoller[b.id] = 'izleyici';
      }
    } catch (e: any) {
      console.error('Bekleyenler yuklenemedi', e);
    }
  }

  async function onayla(b: Bekleyen) {
    const rol = secilenRoller[b.id] ?? 'izleyici';
    try {
      await invokeApi('onayla_kullanici', { id: b.id, rol });
      await Promise.all([yukle(), yukleBekleyenler()]);
    } catch (e: any) {
      hata = e.message ?? 'Onaylanamadi';
    }
  }

  async function reddet(b: Bekleyen) {
    if (!confirm(`"${b.ad}" kullanicisinin basvurusunu reddetmek istediginize emin misiniz?`)) return;
    try {
      await invokeApi('reddet_kullanici', { id: b.id });
      await yukleBekleyenler();
    } catch (e: any) {
      hata = e.message ?? 'Reddedilemedi';
    }
  }

  // ─── Form İşlemleri ─────────────────────────────────────────────────────────
  function yeniAc() {
    duzenle = null;
    fAd = '';
    fEmail = '';
    fSifre = '';
    fRol = 'izleyici';
    fAktif = true;
    formModal = true;
  }

  function duzenleAc(k: Kullanici) {
    duzenle = k;
    fAd = k.ad;
    fEmail = k.email;
    fSifre = '';
    fRol = k.rol;
    fAktif = k.aktif;
    formModal = true;
  }

  async function kaydet(e: Event) {
    e.preventDefault();
    kaydediliyor = true;
    hata = '';
    try {
      if (duzenle) {
        await kullaniciApi.update({
          id: duzenle.id,
          ad: fAd,
          email: fEmail,
          rol: fRol,
          aktif: fAktif
        });
      } else {
        if (fSifre.length < 6) throw new Error('Şifre en az 6 karakter olmalı');
        await kullaniciApi.create({
          ad: fAd,
          email: fEmail,
          sifre: fSifre,
          rol: fRol
        });
      }
      formModal = false;
      await yukle();
    } catch (e: any) {
      hata = e.message ?? 'Kaydedilemedi';
    } finally {
      kaydediliyor = false;
    }
  }

  // ─── Silme ─────────────────────────────────────────────────────────────────
  function silAc(k: Kullanici) {
    silinecek = k;
    silModal = true;
  }

  async function silOnayla() {
    if (!silinecek) return;
    kaydediliyor = true;
    try {
      await kullaniciApi.delete(silinecek.id);
      silModal = false;
      silinecek = null;
      await yukle();
    } catch (e: any) {
      hata = e.message ?? 'Pasifize edilemedi';
    } finally {
      kaydediliyor = false;
    }
  }

  // ─── Şifre Değiştir ─────────────────────────────────────────────────────────
  function sifreAc(k: Kullanici) {
    sifreHedef = k;
    fYeniSifre = '';
    sifreModal = true;
  }

  async function sifreKaydet(e: Event) {
    e.preventDefault();
    if (!sifreHedef) return;
    if (fYeniSifre.length < 6) {
      hata = 'Şifre en az 6 karakter olmalı';
      return;
    }
    kaydediliyor = true;
    hata = '';
    try {
      await kullaniciApi.changePassword(sifreHedef.id, { yeni_sifre: fYeniSifre });
      sifreModal = false;
      sifreHedef = null;
    } catch (e: any) {
      hata = e.message ?? 'Şifre değiştirilemedi';
    } finally {
      kaydediliyor = false;
    }
  }

  // ─── Veritabanı Yedekle ─────────────────────────────────────────────────────
  async function veritabaniYedekle() {
    dbIslem = true;
    dbMesaj = '';
    try {
      const apiBase = import.meta.env.VITE_API_URL || 'http://localhost:3000';
      const token = getToken();
      const res = await fetch(`${apiBase}/api/admin/yedekle`, {
        method: 'POST',
        headers: {
          Authorization: token ? `Bearer ${token}` : ''
        }
      });
      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        throw new Error(data.hata ?? `HTTP ${res.status}`);
      }
      const blob = await res.blob();
      const disposition = res.headers.get('content-disposition') ?? '';
      const match = disposition.match(/filename="([^"]+)"/);
      const filename = match ? match[1] : 'koop_yedek.sql';
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      a.click();
      URL.revokeObjectURL(url);
      dbMesaj = `Yedek indirildi: ${filename}`;
    } catch (e: any) {
      dbMesaj = e.message ?? 'Yedekleme başarısız';
    } finally {
      dbIslem = false;
    }
  }

  // ─── Veritabanı Sıfırla ──────────────────────────────────────────────────────
  async function veritabaniSifirla(e: Event) {
    e.preventDefault();
    if (!adminSifre) return;
    dbIslem = true;
    dbMesaj = '';
    try {
      await invokeApi('admin_sifirla', { sifre: adminSifre });
      sifirlaModal = false;
      adminSifre = '';
      dbMesaj = 'Veritabanı sıfırlandı. Tüm veriler silindi.';
      await yukle();
    } catch (e: any) {
      dbMesaj = e.message ?? 'Sıfırlama başarısız';
    } finally {
      dbIslem = false;
    }
  }

  // ─── Badge rengi ────────────────────────────────────────────────────────────
  function rolRengi(rol: string): 'red' | 'yellow' | 'blue' | 'gray' {
    switch (rol) {
      case 'admin':
        return 'red';
      case 'muhasebe':
        return 'yellow';
      case 'uye':
        return 'blue';
      default:
        return 'gray';
    }
  }
</script>

<svelte:head>
  <title>Kullanıcı ve Rol Yönetimi</title>
</svelte:head>

<main class="p-4">
  <div class="mb-6 flex items-center justify-between">
    <div>
      <Heading tag="h3" class="flex items-center gap-2">
        <UsersSolid class="h-6 w-6" />
        Kullanıcı ve Rol Yönetimi
      </Heading>
      <P class="text-sm text-gray-600 dark:text-gray-400">
        Sistem kullanıcılarını yönetin ve rollerini atayın.
      </P>
    </div>
    {#if isAdmin}
      <Button color="primary" onclick={yeniAc}>
        <PlusOutline class="me-2 h-4 w-4" /> Yeni Kullanıcı
      </Button>
    {/if}
  </div>

  <!-- Veritabanı İşlemleri (sadece admin) -->
  {#if isAdmin}
    <div class="mb-6 rounded-lg border border-orange-200 bg-orange-50 p-4 dark:border-orange-800 dark:bg-orange-900/20">
      <div class="mb-3 flex items-center gap-2">
        <DatabaseSolid class="h-5 w-5 text-orange-600 dark:text-orange-400" />
        <span class="font-semibold text-orange-800 dark:text-orange-300">Veritabanı İşlemleri</span>
        <Badge color="yellow">Admin</Badge>
      </div>
      <div class="flex flex-wrap gap-2">
        <Button color="alternative" size="sm" onclick={veritabaniYedekle} disabled={dbIslem}>
          <ArrowDownToBracketOutline class="me-2 h-4 w-4" />
          {dbIslem ? 'İşleniyor...' : 'Veritabanı Yedekle'}
        </Button>
        <Button color="red" size="sm" onclick={() => { sifirlaModal = true; adminSifre = ''; dbMesaj = ''; }}>
          <DatabaseSolid class="me-2 h-4 w-4" />
          Veritabanı Sıfırla
        </Button>
      </div>
      {#if dbMesaj}
        <p class="mt-2 text-sm {dbMesaj.includes('başarısız') || dbMesaj.includes('hatalı') ? 'text-red-600 dark:text-red-400' : 'text-green-700 dark:text-green-400'}">
          {dbMesaj}
        </p>
      {/if}
    </div>
  {/if}

  <!-- Rol Açıklama Kartı -->
  <div class="mb-6 grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-4">
    {#each Object.entries(ROL_ETIKETLERI) as [rol, etiket] (rol)}
      <div class="rounded-lg border border-gray-200 bg-white p-3 dark:border-gray-700 dark:bg-gray-800">
        <div class="mb-1 flex items-center gap-2">
          <Badge color={rolRengi(rol)}>{etiket}</Badge>
        </div>
        <p class="text-xs text-gray-600 dark:text-gray-400">
          {ROL_ACIKLAMALARI[rol as KullaniciRol]}
        </p>
      </div>
    {/each}
  </div>

  {#if hata}
    <Alert color="red" class="mb-4">
      <ExclamationCircleSolid slot="icon" class="h-4 w-4" />
      {hata}
    </Alert>
  {/if}

  <!-- Bekleyen Basvurular -->
  {#if bekleyenler.length > 0}
    <div class="mb-6 overflow-hidden rounded-lg border border-yellow-300 bg-yellow-50 dark:border-yellow-800 dark:bg-yellow-900/20">
      <div class="flex items-center gap-2 bg-yellow-100 px-4 py-2 dark:bg-yellow-900/40">
        <Badge color="yellow">{bekleyenler.length}</Badge>
        <span class="font-semibold text-yellow-900 dark:text-yellow-200">Onay Bekleyen Basvurular</span>
      </div>
      <table class="w-full text-left text-sm">
        <thead class="text-xs uppercase text-gray-700 dark:text-gray-300">
          <tr>
            <th class="px-4 py-2">Ad</th>
            <th class="px-4 py-2">E-posta</th>
            <th class="px-4 py-2">Kayit Tarihi</th>
            <th class="px-4 py-2">Atanacak Rol</th>
            <th class="px-4 py-2 text-right">Islem</th>
          </tr>
        </thead>
        <tbody>
          {#each bekleyenler as b (b.id)}
            <tr class="border-t border-yellow-200 dark:border-yellow-800">
              <td class="px-4 py-2 font-medium">{b.ad}</td>
              <td class="px-4 py-2">{b.email}</td>
              <td class="px-4 py-2 text-gray-600 dark:text-gray-400">
                {new Date(b.kayit_tarihi).toLocaleString('tr-TR')}
              </td>
              <td class="px-4 py-2">
                <Select
                  bind:value={secilenRoller[b.id]}
                  items={ROLLER}
                  size="sm"
                  class="min-w-[120px]"
                />
              </td>
              <td class="px-4 py-2 text-right">
                <div class="inline-flex gap-1">
                  <Button size="xs" color="green" onclick={() => onayla(b)}>Onayla</Button>
                  <Button size="xs" color="red" onclick={() => reddet(b)}>Reddet</Button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  {#if yukleniyor}
    <div class="flex justify-center p-10"><Spinner /></div>
  {:else if kullanicilar.length === 0}
    <div class="rounded-lg bg-white p-10 text-center dark:bg-gray-800">
      <UsersSolid class="mx-auto mb-3 h-12 w-12 text-gray-400" />
      <P class="text-gray-600 dark:text-gray-400">Henüz kullanıcı yok.</P>
    </div>
  {:else}
    <div class="overflow-x-auto rounded-lg bg-white shadow dark:bg-gray-800">
      <table class="w-full text-left text-sm">
        <thead class="bg-gray-50 text-xs uppercase text-gray-700 dark:bg-gray-700 dark:text-gray-300">
          <tr>
            <th class="px-4 py-3">#</th>
            <th class="px-4 py-3">Ad</th>
            <th class="px-4 py-3">E-posta</th>
            <th class="px-4 py-3">Rol</th>
            <th class="px-4 py-3">Durum</th>
            <th class="px-4 py-3">Kayıt</th>
            <th class="px-4 py-3 text-right">İşlemler</th>
          </tr>
        </thead>
        <tbody>
          {#each kullanicilar as k (k.id)}
            <tr class="border-b border-gray-100 dark:border-gray-700">
              <td class="px-4 py-3">{k.id}</td>
              <td class="px-4 py-3 font-medium">{k.ad}</td>
              <td class="px-4 py-3">{k.email}</td>
              <td class="px-4 py-3">
                <Badge color={rolRengi(k.rol)}>{ROL_ETIKETLERI[k.rol] ?? k.rol}</Badge>
              </td>
              <td class="px-4 py-3">
                {#if k.aktif}
                  <Badge color="green">Aktif</Badge>
                {:else}
                  <Badge color="dark">Pasif</Badge>
                {/if}
              </td>
              <td class="px-4 py-3 text-gray-600 dark:text-gray-400">
                {new Date(k.created_at).toLocaleDateString('tr-TR')}
              </td>
              <td class="px-4 py-3 text-right">
                <div class="inline-flex gap-1">
                  <Button size="xs" color="alternative" onclick={() => duzenleAc(k)}>
                    <EditOutline class="h-4 w-4" />
                  </Button>
                  <Button size="xs" color="alternative" onclick={() => sifreAc(k)} title="Şifre değiştir">
                    <LockSolid class="h-4 w-4" />
                  </Button>
                  {#if k.id !== currentUser?.sub}
                    <Button size="xs" color="red" onclick={() => silAc(k)}>
                      <TrashBinSolid class="h-4 w-4" />
                    </Button>
                  {/if}
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  <!-- Ekle/Düzenle Modal -->
  <Modal bind:open={formModal} size="md" autoclose={false}>
    <h3 class="mb-4 text-lg font-semibold">
      {duzenle ? 'Kullanıcı Düzenle' : 'Yeni Kullanıcı'}
    </h3>
    <form onsubmit={kaydet} class="space-y-4">
      <Label>
        Ad Soyad
        <Input bind:value={fAd} required class="mt-1" />
      </Label>
      <Label>
        E-posta
        <Input type="email" bind:value={fEmail} required class="mt-1" />
      </Label>
      {#if !duzenle}
        <Label>
          Şifre (min 6 karakter)
          <Input type="password" bind:value={fSifre} required minlength={6} class="mt-1" />
        </Label>
      {/if}
      <Label>
        Rol
        <Select bind:value={fRol} items={ROLLER} class="mt-1" />
        <p class="mt-1 text-xs text-gray-500">{ROL_ACIKLAMALARI[fRol]}</p>
      </Label>
      {#if duzenle}
        <div class="flex items-center gap-2">
          <Toggle bind:checked={fAktif} />
          <span class="text-sm">Aktif</span>
        </div>
      {/if}

      <div class="flex justify-end gap-2 pt-2">
        <Button color="alternative" onclick={() => (formModal = false)} type="button">İptal</Button>
        <Button color="primary" type="submit" disabled={kaydediliyor}>
          {kaydediliyor ? 'Kaydediliyor...' : 'Kaydet'}
        </Button>
      </div>
    </form>
  </Modal>

  <!-- Sil Modal -->
  <Modal bind:open={silModal} size="sm" autoclose={false}>
    <div class="text-center">
      <ExclamationCircleSolid class="mx-auto mb-4 h-12 w-12 text-red-500" />
      <h3 class="mb-2 text-lg font-semibold">
        "{silinecek?.ad}" pasifize edilsin mi?
      </h3>
      <P class="mb-4 text-sm text-gray-600 dark:text-gray-400">
        Kullanıcı silinmez, sadece pasife alınır. Tekrar aktif edilebilir.
      </P>
      <div class="flex justify-center gap-2">
        <Button color="alternative" onclick={() => (silModal = false)}>Vazgeç</Button>
        <Button color="red" onclick={silOnayla} disabled={kaydediliyor}>
          {kaydediliyor ? 'İşleniyor...' : 'Evet, pasifize et'}
        </Button>
      </div>
    </div>
  </Modal>

  <!-- Şifre Değiştir Modal -->
  <Modal bind:open={sifreModal} size="sm" autoclose={false}>
    <h3 class="mb-4 text-lg font-semibold">
      "{sifreHedef?.ad}" - Şifre Değiştir
    </h3>
    <form onsubmit={sifreKaydet} class="space-y-4">
      <Label>
        Yeni Şifre (min 6 karakter)
        <Input type="password" bind:value={fYeniSifre} required minlength={6} class="mt-1" />
      </Label>
      <div class="flex justify-end gap-2 pt-2">
        <Button color="alternative" onclick={() => (sifreModal = false)} type="button">İptal</Button>
        <Button color="primary" type="submit" disabled={kaydediliyor}>
          {kaydediliyor ? 'Kaydediliyor...' : 'Değiştir'}
        </Button>
      </div>
    </form>
  </Modal>

  <!-- Veritabanı Sıfırla Modal -->
  <Modal bind:open={sifirlaModal} size="sm" autoclose={false}>
    <div class="text-center">
      <ExclamationCircleSolid class="mx-auto mb-4 h-12 w-12 text-red-500" />
      <h3 class="mb-2 text-lg font-bold text-red-600">Veritabanı Sıfırla</h3>
      <P class="mb-4 text-sm text-gray-600 dark:text-gray-400">
        Bu işlem <strong>geri alınamaz!</strong> Tüm hissedarlar, dönemler, kasalar,
        hisseler ve işlemler silinir. Kullanıcı hesapları korunur.
      </P>
    </div>
    <form onsubmit={veritabaniSifirla} class="space-y-4">
      <Label>
        Onaylamak için admin şifrenizi girin:
        <Input
          type="password"
          bind:value={adminSifre}
          required
          placeholder="Şifreniz"
          class="mt-1"
          autofocus
        />
      </Label>
      {#if dbMesaj && (dbMesaj.includes('hatalı') || dbMesaj.includes('başarısız'))}
        <p class="text-sm text-red-600">{dbMesaj}</p>
      {/if}
      <div class="flex justify-end gap-2 pt-2">
        <Button color="alternative" onclick={() => (sifirlaModal = false)} type="button">Vazgeç</Button>
        <Button color="red" type="submit" disabled={dbIslem || !adminSifre}>
          {dbIslem ? 'Sıfırlanıyor...' : 'Evet, Tüm Verileri Sil'}
        </Button>
      </div>
    </form>
  </Modal>
</main>
