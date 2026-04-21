import { expect, test } from '@playwright/test';
import { girisYapApi } from './helpers';

test.describe('Dashboard', () => {
	test.beforeEach(async ({ page }) => {
		await girisYapApi(page);
	});

	test('dashboard yuklenir ve temel kartlar gorunur', async ({ page }) => {
		await page.goto('/dashboard');
		await expect(page.locator('h1')).toBeVisible({ timeout: 10_000 });
		// Yukleme spinner'i bitmeli: 'Hissedarlar' veya 'Kasalar' metni gorunmeli
		await expect(page.getByText(/Hissedar|Kasa|Hisse/i).first()).toBeVisible({ timeout: 15_000 });
	});

	test('hata durumunda sayfa cokmez', async ({ page }) => {
		const errors: string[] = [];
		page.on('pageerror', (err) => errors.push(err.message));
		await page.goto('/dashboard');
		await page.waitForLoadState('networkidle');
		expect(errors).toEqual([]);
	});
});
