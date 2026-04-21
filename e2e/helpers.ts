import type { Page } from '@playwright/test';

export const TEST_USER = {
	email: process.env.KOOP_TEST_EMAIL ?? 'admin@koop.local',
	sifre: process.env.KOOP_TEST_SIFRE ?? 'Busraasya1334.'
};

/**
 * Sign-in sayfasina gider ve admin olarak giris yapar.
 * Giris sonrasi dashboard'a yonlendirilmesini bekler.
 */
export async function girisYap(page: Page): Promise<void> {
	await page.goto('/authentication/sign-in');
	await page.fill('#email', TEST_USER.email);
	await page.fill('#sifre', TEST_USER.sifre);
	await page.click('button[type="submit"]');
	await page.waitForURL(/\/dashboard/, { timeout: 15_000 });
}

/**
 * Local/session storage'a direkt token enjekte eder. Navigasyon testleri icin
 * UI login'ini her test basinda tekrarlamaktan kacinmak uzere kullanilir.
 */
export async function girisYapApi(page: Page): Promise<string> {
	const base = process.env.KOOP_API_URL ?? 'http://localhost:3002';
	const res = await fetch(`${base}/api/auth/giris`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(TEST_USER)
	});
	if (!res.ok) throw new Error(`Login basarisiz: ${res.status}`);
	const data: { token: string; kullanici: unknown } = await res.json();

	await page.goto('/'); // origin'i ac ki localStorage kullanilabilsin
	await page.evaluate(
		([t, k]) => {
			localStorage.setItem('koop_token', t as string);
			localStorage.setItem('koop_kullanici', JSON.stringify(k));
		},
		[data.token, data.kullanici]
	);
	return data.token;
}
