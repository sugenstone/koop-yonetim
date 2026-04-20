<script lang="ts">
	import { invokeApi } from '$lib/api-client';
	import { Button, Card, Input, Label, Alert } from 'flowbite-svelte';

	let ad = $state('');
	let email = $state('');
	let sifre = $state('');
	let sifre2 = $state('');
	let hata = $state('');
	let basari = $state('');
	let yukleniyor = $state(false);

	async function kayitOl(e: Event) {
		e.preventDefault();
		hata = '';
		basari = '';
		if (sifre !== sifre2) {
			hata = 'Sifreler eslesmiyor';
			return;
		}
		if (sifre.length < 6) {
			hata = 'Sifre en az 6 karakter olmali';
			return;
		}
		yukleniyor = true;
		try {
			const sonuc = await invokeApi<{ mesaj: string }>('kayit', { ad, email, sifre });
			basari = sonuc.mesaj || 'Kayit basarili. Onay bekleniyor.';
			ad = '';
			email = '';
			sifre = '';
			sifre2 = '';
		} catch (err: any) {
			hata = err?.message || 'Kayit basarisiz';
		} finally {
			yukleniyor = false;
		}
	}
</script>

<main class="flex min-h-screen items-center justify-center bg-gray-50 px-4 dark:bg-gray-900">
	<Card class="w-full max-w-md p-6">
		<h1 class="mb-6 text-2xl font-bold text-gray-900 dark:text-white">
			Kooperatif Yonetim - Kayit
		</h1>

		<form class="space-y-4" onsubmit={kayitOl}>
			<div>
				<Label for="ad" class="mb-2">Ad Soyad</Label>
				<Input id="ad" type="text" bind:value={ad} required minlength={2} />
			</div>

			<div>
				<Label for="email" class="mb-2">E-posta</Label>
				<Input id="email" type="email" bind:value={email} required />
			</div>

			<div>
				<Label for="sifre" class="mb-2">Sifre</Label>
				<Input id="sifre" type="password" bind:value={sifre} required minlength={6} />
			</div>

			<div>
				<Label for="sifre2" class="mb-2">Sifre (tekrar)</Label>
				<Input id="sifre2" type="password" bind:value={sifre2} required minlength={6} />
			</div>

			{#if hata}
				<Alert color="red">{hata}</Alert>
			{/if}
			{#if basari}
				<Alert color="green">{basari}</Alert>
			{/if}

			<Button type="submit" class="w-full" disabled={yukleniyor}>
				{yukleniyor ? 'Kayit yapiliyor...' : 'Kayit Ol'}
			</Button>

			<p class="text-center text-sm text-gray-600 dark:text-gray-400">
				Zaten hesabiniz var mi?
				<a
					href="/authentication/sign-in"
					class="font-medium text-primary-600 hover:underline dark:text-primary-500"
				>
					Giris yapin
				</a>
			</p>
		</form>
	</Card>
</main>
