import { defineConfig, devices } from '@playwright/test';

/**
 * E2E test yapilandirmasi.
 * Docker'da calisan frontend (koop-frontend @ :8082) uzerinden test eder.
 * Backend koop-backend @ :3002 uzerinde calismalidir.
 *
 * Farkli bir URL icin: PLAYWRIGHT_BASE_URL=http://localhost:5173 npm run test:e2e
 */
const baseURL = process.env.PLAYWRIGHT_BASE_URL ?? 'http://localhost:8082';

export default defineConfig({
  testDir: 'e2e',
  timeout: 30_000,
  expect: { timeout: 5_000 },
  fullyParallel: false,
  retries: process.env.CI ? 2 : 0,
  reporter: process.env.CI ? 'github' : 'list',
  use: {
    baseURL,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure'
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } }
  ]
});
