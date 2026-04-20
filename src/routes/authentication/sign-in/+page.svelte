<script lang="ts">
	import { goto } from '$app/navigation';
	import { invokeApi, setToken } from '$lib/api-client';
	import { loadMyPermissions } from '$lib/permissions';
	import { Button, Card, Input, Label, Alert } from 'flowbite-svelte';

	let email = $state('admin@koop.local');
	let sifre = $state('');
	let hata = $state('');
	let yukleniyor = $state(false);

	async function girisYap(e: Event) {
		e.preventDefault();
		hata = '';
		yukleniyor = true;
		try {
			const sonuc = await invokeApi<{ token: string; kullanici: any }>('giris', {
				email,
				sifre
			});
			setToken(sonuc.token);
			if (typeof localStorage !== 'undefined') {
				localStorage.setItem('koop_kullanici', JSON.stringify(sonuc.kullanici));
			}
			await loadMyPermissions();
			goto('/dashboard');
		} catch (err: any) {
			hata = err?.message || 'Giris basarisiz';
		} finally {
			yukleniyor = false;
		}
	}
</script>

<main class="flex min-h-screen items-center justify-center bg-gray-50 px-4 dark:bg-gray-900">
	<Card class="w-full max-w-md p-6">
		<h1 class="mb-6 text-2xl font-bold text-gray-900 dark:text-white">Kooperatif Yonetim - Giris</h1>

		<form class="space-y-4" onsubmit={girisYap}>
			<div>
				<Label for="email" class="mb-2">E-posta</Label>
				<Input id="email" type="email" bind:value={email} required />
			</div>

			<div>
				<Label for="sifre" class="mb-2">Sifre</Label>
				<Input id="sifre" type="password" bind:value={sifre} required />
			</div>

			{#if hata}
				<Alert color="red">{hata}</Alert>
			{/if}

			<Button type="submit" class="w-full" disabled={yukleniyor}>
				{yukleniyor ? 'Giris yapiliyor...' : 'Giris Yap'}
			</Button>

			<p class="text-center text-sm text-gray-600 dark:text-gray-400">
				Hesabiniz yok mu?
				<a href="/authentication/sign-up" class="font-medium text-primary-600 hover:underline dark:text-primary-500">
					Kayit olun
				</a>
			</p>
		</form>
	</Card>
</main>
