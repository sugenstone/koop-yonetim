import { expect, test } from '@playwright/test';
import { girisYapApi } from './helpers';

const SAYFALAR: Array<{ yol: string; h1: RegExp }> = [
	{ yol: '/kasa', h1: /Kasa/i },
	{ yol: '/hissedar', h1: /Hissedar/i },
	{ yol: '/hisse', h1: /Hisse/i },
	{ yol: '/donem', h1: /Donem|Dönem/i },
	{ yol: '/gelir-gider', h1: /Gelir|Gider/i },
	{ yol: '/borclar', h1: /Borc|Borç/i }
];

test.describe('Modul navigasyonu', () => {
	test.beforeEach(async ({ page }) => {
		await girisYapApi(page);
	});

	for (const { yol, h1 } of SAYFALAR) {
		test(`${yol} sayfasi yuklenir`, async ({ page }) => {
			await page.goto(yol);
			await expect(page.locator('h1').first()).toBeVisible({ timeout: 10_000 });
			await expect(page.locator('h1').first()).toContainText(h1);
		});
	}

	test('yetkisiz istek login\'e yonlendirir', async ({ page }) => {
		// Token'i temizle
		await page.goto('/');
		await page.evaluate(() => localStorage.clear());
		await page.goto('/dashboard');
		await page.waitForURL(/sign-in/, { timeout: 10_000 });
	});
});
