<script lang="ts">
	import { goto } from '$app/navigation';
	import { invokeApi, setToken } from '$lib/api-client';
	import { setUser, isLoggedIn } from '$lib/auth';
	import { loadMyPermissions } from '$lib/permissions';
	import { onMount } from 'svelte';

	let email = $state('');
	let sifre = $state('');
	let hata = $state('');
	let yukleniyor = $state(false);

	onMount(() => {
		if (isLoggedIn()) {
			goto('/');
		}
	});

	async function girisYap() {
		hata = '';
		yukleniyor = true;
		try {
			const sonuc = await invokeApi<{ token: string; kullanici: any }>('giris', { email, sifre });
			setToken(sonuc.token);
			setUser(sonuc.kullanici);
			await loadMyPermissions();
			goto('/');
		} catch (e) {
			hata = e instanceof Error ? e.message : 'Giriş başarısız';
		} finally {
			yukleniyor = false;
		}
	}
</script>

<main class="flex min-h-screen items-center justify-center bg-gray-50 dark:bg-gray-900">
	<div class="w-full max-w-md px-6">
		<!-- Logo / Başlık -->
		<div class="mb-8 text-center">
			<div class="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-blue-600">
				<svg class="h-8 w-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
						d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z"
					/>
				</svg>
			</div>
			<h1 class="text-2xl font-bold text-gray-900 dark:text-white">Kooperatif Yönetimi</h1>
			<p class="mt-1 text-sm text-gray-500 dark:text-gray-400">Hesabınıza giriş yapın</p>
		</div>

		<!-- Kart -->
		<div class="rounded-xl bg-white p-8 shadow-lg dark:bg-gray-800">
			<form onsubmit={(e) => { e.preventDefault(); girisYap(); }}>
				<!-- E-posta -->
				<div class="mb-5">
					<label for="email" class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
						E-posta adresi
					</label>
					<input
						id="email"
						type="email"
						autocomplete="email"
						bind:value={email}
						required
						class="w-full rounded-lg border border-gray-300 bg-white px-4 py-2.5 text-sm text-gray-900
							   focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500
							   dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
						placeholder="admin@koop.local"
					/>
				</div>

				<!-- Şifre -->
				<div class="mb-6">
					<label for="sifre" class="mb-2 block text-sm font-medium text-gray-700 dark:text-gray-300">
						Şifre
					</label>
					<input
						id="sifre"
						type="password"
						autocomplete="current-password"
						bind:value={sifre}
						required
						class="w-full rounded-lg border border-gray-300 bg-white px-4 py-2.5 text-sm text-gray-900
							   focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500
							   dark:border-gray-600 dark:bg-gray-700 dark:text-white dark:placeholder-gray-400"
						placeholder="••••••••"
					/>
				</div>

				<!-- Hata mesajı -->
				{#if hata}
					<div class="mb-4 rounded-lg bg-red-50 px-4 py-3 text-sm text-red-700 dark:bg-red-900/30 dark:text-red-400">
						{hata}
					</div>
				{/if}

				<!-- Giriş Butonu -->
				<button
					type="submit"
					disabled={yukleniyor}
					class="w-full rounded-lg bg-blue-600 px-5 py-3 text-sm font-semibold text-white
						   hover:bg-blue-700 focus:outline-none focus:ring-4 focus:ring-blue-300
						   disabled:opacity-60 dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800"
				>
					{#if yukleniyor}
						<span class="flex items-center justify-center gap-2">
							<svg class="h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
							</svg>
							Giriş yapılıyor...
						</span>
					{:else}
						Giriş Yap
					{/if}
				</button>
			</form>
		</div>
	</div>
</main>
