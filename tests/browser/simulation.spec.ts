import { expect, test } from '@playwright/test';

test('simulation dashboard exposes a truthful data boundary', async ({ page }) => {
  const response = await page.goto('/');
  expect(response?.ok()).toBeTruthy();

  await expect(
    page.getByText(/CHẾ ĐỘ MÔ PHỎNG/i)
  ).toBeVisible();

  await expect(
    page.getByText(/synthetic/i).first()
  ).toBeVisible();

  const state = await page.request.get('/api/state');
  expect(state.ok()).toBeTruthy();
  const payload = await state.json();
  expect(payload.dataMode).toBe('SIMULATION');
  expect(payload.evidenceMode).toBe('synthetic_fixture');
  expect(payload.simulatedMutationsEnabled).toBe(false);
});

test('simulation mutating actions are fail-closed by default', async ({ page }) => {
  await page.goto('/');

  const response = await page.request.post('/api/run', {
    data: {},
  });

  expect(response.status()).toBe(503);
  const payload = await response.json();
  expect(payload.error).toBe('simulated_actions_disabled');
  expect(payload.dataMode).toBe('SIMULATION');
});
