<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Button,
    Badge,
    Checkbox,
    Spinner,
    Heading,
    P,
    Alert,
    Tabs,
    TabItem
  } from 'flowbite-svelte';
  import {
    UserSettingsSolid,
    CheckCircleSolid,
    ExclamationCircleSolid,
    LockSolid
  } from 'flowbite-svelte-icons';
  import { goto } from '$app/navigation';
  import { izinApi, ROL_ETIKETLERI, ROL_ACIKLAMALARI, type Izin, type KullaniciRol } from '$lib/tauri-api';
  import { getCurrentUser } from '$lib/api-client';

  const currentUser = getCurrentUser();
  const isAdmin = currentUser?.rol === 'admin';

  const ROLLER: KullaniciRol[] = ['admin', 'muhasebe', 'uye', 'izleyici'];

  let izinler = $state<Izin[]>([]);
  let rolIzinleri = $state<Record<KullaniciRol, Set<number>>>({
    admin: new Set(),
    muhasebe: new Set(),
    uye: new Set(),
    izleyici: new Set()
  });

  let yukleniyor = $state(true);
  let kaydediliyor = $state<KullaniciRol | null>(null);
  let hata = $state('');
  let basari = $state('');

  // ─── Türetilmiş: kategori -> izinler ───────────────────────────────────────
  const kategoriler = $derived.by(() => {
    const gruplar = new Map<string, Izin[]>();
    for (const i of izinler) {
      if (!gruplar.has(i.kategori)) gruplar.set(i.kategori, []);
      gruplar.get(i.kategori)!.push(i);
    }
    return Array.from(gruplar.entries()).sort(([a], [b]) => a.localeCompare(b));
  });

  const KATEGORI_ETIKETLERI: Record<string, string> = {
    kullanici: 'Kullanıcı Yönetimi',
    kasa: 'Kasa',
    hissedar: 'Hissedar',
    donem: 'Dönem & Toplantı',
    hisse: 'Hisse',
    borc: 'Aidat / Borç',
    gelir_gider: 'Gelir / Gider',
    sistem: 'Sistem'
  };

  // ─── Yükle ─────────────────────────────────────────────────────────────────
  async function yukle() {
    yukleniyor = true;
    hata = '';
    try {
      izinler = await izinApi.list();
      const sonuc = await Promise.all(ROLLER.map((r) => izinApi.getByRol(r)));
      for (let i = 0; i < ROLLER.length; i++) {
        rolIzinleri[ROLLER[i]] = new Set(sonuc[i]);
      }
    } catch (e: any) {
      hata = e.message ?? 'Yüklenemedi';
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
  });

  // ─── Toggle / Kaydet ───────────────────────────────────────────────────────
  function toggle(rol: KullaniciRol, izinId: number, checked: boolean) {
    const kume = new Set(rolIzinleri[rol]);
    if (checked) kume.add(izinId);
    else kume.delete(izinId);
    rolIzinleri[rol] = kume;
  }

  function tumunuSec(rol: KullaniciRol, secili: boolean) {
    rolIzinleri[rol] = secili ? new Set(izinler.map((i) => i.id)) : new Set();
  }

  async function kaydet(rol: KullaniciRol) {
    kaydediliyor = rol;
    hata = '';
    basari = '';
    try {
      const ids = Array.from(rolIzinleri[rol]).sort((a, b) => a - b);
      const resp = await izinApi.setByRol(rol, ids);
      basari = resp.mesaj;
      setTimeout(() => (basari = ''), 3000);
    } catch (e: any) {
      hata = e.message ?? 'Kaydedilemedi';
    } finally {
      kaydediliyor = null;
    }
  }

  function rolRengi(rol: string): 'red' | 'yellow' | 'blue' | 'gray' {
    switch (rol) {
      case 'admin': return 'red';
      case 'muhasebe': return 'yellow';
      case 'uye': return 'blue';
      default: return 'gray';
    }
  }
</script>

<svelte:head>
  <title>Rol ve İzin Yönetimi</title>
</svelte:head>

<main class="p-4">
  <div class="mb-6">
    <Heading tag="h3" class="flex items-center gap-2">
      <UserSettingsSolid class="h-6 w-6" />
      Rol ve İzin Yönetimi
    </Heading>
    <P class="text-sm text-gray-600 dark:text-gray-400">
      Her role hangi işlemleri yapabileceğini seçin. Değişiklikler hemen uygulanır.
    </P>
  </div>

  {#if hata}
    <Alert color="red" class="mb-4" dismissable>
      <ExclamationCircleSolid slot="icon" class="h-4 w-4" />
      {hata}
    </Alert>
  {/if}

  {#if basari}
    <Alert color="green" class="mb-4">
      <CheckCircleSolid slot="icon" class="h-4 w-4" />
      {basari}
    </Alert>
  {/if}

  {#if yukleniyor}
    <div class="flex justify-center p-10"><Spinner /></div>
  {:else}
    <Tabs tabStyle="underline" contentClass="p-4 bg-white rounded-lg mt-4 dark:bg-gray-800">
      {#each ROLLER as rol (rol)}
        {@const sayi = rolIzinleri[rol].size}
        {@const toplam = izinler.length}
        {@const kilitli = rol === 'admin'}

        <TabItem open={rol === 'muhasebe'}>
          {#snippet titleSlot()}
            <div class="flex items-center gap-2">
              <Badge color={rolRengi(rol)}>{ROL_ETIKETLERI[rol]}</Badge>
              <span class="text-xs text-gray-500">{sayi}/{toplam}</span>
            </div>
          {/snippet}

          <div class="mb-4 flex items-start justify-between gap-4">
            <div>
              <P class="text-sm text-gray-700 dark:text-gray-300">
                {ROL_ACIKLAMALARI[rol]}
              </P>
              {#if kilitli}
                <div class="mt-2 flex items-center gap-1 text-xs text-amber-600 dark:text-amber-400">
                  <LockSolid class="h-3 w-3" />
                  Admin rolü düzenlenemez (sistem güvenliği için tüm izinlere her zaman sahiptir).
                </div>
              {/if}
            </div>
            {#if !kilitli}
              <div class="flex shrink-0 gap-2">
                <Button size="xs" color="alternative" onclick={() => tumunuSec(rol, true)}>
                  Tümünü Seç
                </Button>
                <Button size="xs" color="alternative" onclick={() => tumunuSec(rol, false)}>
                  Temizle
                </Button>
                <Button
                  size="xs"
                  color="primary"
                  disabled={kaydediliyor !== null}
                  onclick={() => kaydet(rol)}
                >
                  {kaydediliyor === rol ? 'Kaydediliyor...' : 'Kaydet'}
                </Button>
              </div>
            {/if}
          </div>

          <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
            {#each kategoriler as [kat, izinList] (kat)}
              <div class="rounded-lg border border-gray-200 p-3 dark:border-gray-700">
                <h4 class="mb-2 text-sm font-semibold text-gray-900 dark:text-white">
                  {KATEGORI_ETIKETLERI[kat] ?? kat}
                </h4>
                <div class="space-y-2">
                  {#each izinList as izin (izin.id)}
                    <label class="flex items-start gap-2 text-sm {kilitli ? 'opacity-60' : ''}">
                      <Checkbox
                        checked={rolIzinleri[rol].has(izin.id)}
                        disabled={kilitli}
                        onchange={(e) => toggle(rol, izin.id, (e.target as HTMLInputElement).checked)}
                      />
                      <div>
                        <div class="font-mono text-xs text-gray-500">{izin.anahtar}</div>
                        <div class="text-gray-700 dark:text-gray-300">{izin.aciklama}</div>
                      </div>
                    </label>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        </TabItem>
      {/each}
    </Tabs>

    <div class="mt-6 rounded-lg border border-blue-200 bg-blue-50 p-4 dark:border-blue-800 dark:bg-blue-950/30">
      <p class="text-xs text-blue-800 dark:text-blue-200">
        <strong>Not:</strong> Bu ekran roller için tanımlı izinleri yönetir. Gerçek erişim kontrolü sunucu
        tarafında yapılır. UI'daki butonlar da kullanıcının sahip olduğu izinlere göre gösterilir/gizlenir.
      </p>
    </div>
  {/if}
</main>
