import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests/browser',
  timeout: 30_000,
  expect: { timeout: 5_000 },
  use: {
    baseURL: 'http://127.0.0.1:3100',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  webServer: {
    command: 'npm run dev',
    url: 'http://127.0.0.1:3100/api/state',
    reuseExistingServer: false,
    timeout: 60_000,
    env: {
      COMPANY_OS_MODE: 'simulation',
      HOST: '127.0.0.1',
      PORT: '3100',
      ALLOW_SIMULATED_ACTIONS: 'false',
    },
  },
});
