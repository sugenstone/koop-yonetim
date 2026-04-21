import { expect, test } from '@playwright/test';
import { girisYap, TEST_USER } from './helpers';

test.describe('Kimlik dogrulama', () => {
	test('giris sayfasi acilir ve baslik gorunur', async ({ page }) => {
		await page.goto('/authentication/sign-in');
		await expect(page.locator('h1')).toContainText('Kooperatif');
		await expect(page.locator('#email')).toBeVisible();
		await expect(page.locator('#sifre')).toBeVisible();
	});

	test('gecersiz sifre ile giris hata mesaji gosterir', async ({ page }) => {
		await page.goto('/authentication/sign-in');
		await page.fill('#email', TEST_USER.email);
		await page.fill('#sifre', 'yanlis_sifre_xxx');
		await page.click('button[type="submit"]');
		// Alert kirmizi renk ile gorunmeli
		await expect(page.getByRole('alert')).toBeVisible({ timeout: 10_000 });
		// Hala login sayfasindayiz
		await expect(page).toHaveURL(/sign-in/);
	});

	test('admin ile basarili giris -> dashboard', async ({ page }) => {
		await girisYap(page);
		await expect(page).toHaveURL(/\/dashboard/);
		await expect(page.locator('h1')).toBeVisible();
	});

	test('token localStorage\'a yazilir', async ({ page }) => {
		await girisYap(page);
		const token = await page.evaluate(() => localStorage.getItem('koop_token'));
		expect(token).toBeTruthy();
		expect(token!.split('.').length).toBe(3); // JWT formati
	});
});
