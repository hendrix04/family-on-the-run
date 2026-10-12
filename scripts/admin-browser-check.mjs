import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
import { chromium } from 'playwright';

const browser = await chromium.launch({ executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH });
const page = await browser.newPage({ viewport: { width: 2560, height: 1440 }, hasTouch: true });
page.setDefaultTimeout(10_000);
const errors = [];
page.on('pageerror', error => errors.push(error.message));

try {
  await page.goto(process.env.TEST_APP_URL);
  await page.getByLabel('Household access code').fill(process.env.TEST_HOUSEHOLD_CODE);
  await page.getByRole('button', { name: 'Connect this screen', exact: true }).click();
  await page.getByRole('heading', { name: 'Matt To Do' }).waitFor();
  await page.getByLabel('Choose day').fill(process.env.TEST_IMPORTED_DATE);
  await page.getByRole('button').filter({ hasText: process.env.TEST_IMPORTED_TITLE }).click();
  await page.getByText('From Imported calendar. Edit this event in Google Calendar.').waitFor();
  assert.equal(await page.getByRole('button', { name: 'Delete event', exact: true }).count(), 0);
  await page.getByRole('button', { name: 'Done', exact: true }).click();
  for (const tab of ['Weekly Calendar', 'Monthly Calendar']) {
    await page.getByRole('button', { name: tab, exact: true }).click();
    await page.getByText(process.env.TEST_IMPORTED_TITLE, { exact: true }).first().waitFor();
  }
  await page.getByRole('button', { name: 'Admin', exact: true }).click();
  await page.getByLabel('Admin access code').fill('incorrect');
  await page.getByRole('button', { name: 'Unlock Admin', exact: true }).click();
  await page.getByRole('alert').filter({ hasText: "That admin access code isn't correct" }).waitFor();
  await page.getByLabel('Admin access code').fill(process.env.TEST_ADMIN_CODE);
  await page.getByRole('button', { name: 'Unlock Admin', exact: true }).click();
  await page.getByRole('heading', { name: 'Google Calendar', exact: true }).waitFor();
  await page.getByLabel('Calendar name', { exact: true }).fill('Family calendar');
  await page.getByLabel('Secret address in iCal format').fill('https://calendar.google.com/calendar/ical/test/private-browser-test/basic.ics');
  await page.getByRole('button', { name: 'Add calendar feed', exact: true }).click();
  await page.getByRole('button', { name: 'Remove calendar Family calendar', exact: true }).waitFor();
  assert.equal(await page.getByLabel('Secret address in iCal format').inputValue(), '');
  await page.locator('.feed-source').filter({ hasText: 'Family calendar' }).getByRole('button', { name: 'Refresh now', exact: true }).click();
  await page.getByRole('status').filter({ hasText: 'Refresh requested.' }).waitFor();
  await page.reload();
  await page.getByRole('button', { name: 'Admin', exact: true }).click();
  await page.getByRole('button', { name: 'Remove calendar Family calendar', exact: true }).waitFor();
  assert.equal(await page.getByLabel('Secret address in iCal format').inputValue(), '');

  const member = page.getByRole('form').filter({ has: page.getByLabel('Name for member 1', { exact: true }) });
  await page.getByLabel('Name for member 1', { exact: true }).fill('Updated member');
  await page.getByLabel('Color for member 1', { exact: true }).fill('#345678');
  await member.getByRole('button', { name: 'Save member', exact: true }).click();
  await member.getByRole('status').waitFor();
  await page.getByRole('button', { name: 'Home', exact: true }).click();
  await page.getByRole('heading', { name: 'Updated member To Do' }).waitFor();
  await page.getByRole('button', { name: 'Admin', exact: true }).click();
  await mkdir('.spacetime/screenshots', { recursive: true });
  await page.getByRole('heading', { name: 'Google Calendar', exact: true }).waitFor();
  await page.screenshot({ path: '.spacetime/screenshots/admin-2560.png', fullPage: true });
  await page.setViewportSize({ width: 390, height: 844 });
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Admin mobile layout has no horizontal overflow');
  await page.screenshot({ path: '.spacetime/screenshots/admin-mobile.png', fullPage: true });
  await page.getByRole('button', { name: 'Remove calendar Family calendar', exact: true }).click();
  await page.getByRole('button', { name: 'Remove calendar Family calendar', exact: true }).waitFor({ state: 'detached' });
  await page.getByRole('button', { name: 'Lock Admin', exact: true }).click();
  await page.getByLabel('Admin access code').waitFor();
  assert.deepEqual(errors, []);
  console.log('PASS: admin unlock, private ICS feed forms, secret masking, saved settings after reload, manual refresh, imported events in daily/weekly/monthly views, read-only event details, source removal, member editing, locking, and mobile layout.');
} finally { await browser.close(); }
