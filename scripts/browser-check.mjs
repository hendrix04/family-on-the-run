import assert from 'node:assert/strict';
import { readFile, mkdir } from 'node:fs/promises';
import { randomUUID } from 'node:crypto';
import { chromium } from 'playwright';

const accessCode = (await readFile('.spacetime/access-code', 'utf8')).trim();
const browser = await chromium.launch({ executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH });
const context = await browser.newContext({ viewport: { width: 2560, height: 1440 }, timezoneId: 'America/New_York', hasTouch: true });
const page = await context.newPage();
page.setDefaultTimeout(10_000);
const errors = [];
page.on('pageerror', error => errors.push(error.message));
const title = `Browser check ${randomUUID()}`;
let taskCreated = false;
let eventCreated = false;

try {
  await page.goto(process.env.TEST_APP_URL ?? 'http://127.0.0.1:3000/v1/database/family-on-the-run/route/');
  await page.getByLabel('Household access code').fill(accessCode);
  await page.getByRole('button', { name: 'Connect this screen', exact: true }).click();
  await page.getByRole('heading', { name: 'Lindsay To Do' }).waitFor();
  assert.deepEqual(await page.locator('.column-header h2').allTextContents(), ['Daily Calendar', 'Lindsay To Do', 'Matt To Do', 'Carter To Do', 'Sydney To Do']);
  const boxes = await page.locator('.dashboard-column').evaluateAll(columns => columns.map(column => {
    const box = column.getBoundingClientRect();
    return { x: box.x, top: box.top, right: box.right, bottom: box.bottom };
  }));
  assert.ok(boxes.every((box, index) => box.top === boxes[0].top && box.right <= 2560 && box.bottom <= 1440 && (!index || box.x > boxes[index - 1].x)), 'Five columns fit the touchscreen viewport');
  console.log('PASS: pairing and five-column layout at 2560×1440.');

  const lindsay = page.locator('.task-column').filter({ has: page.getByRole('heading', { name: 'Lindsay To Do' }) });
  await lindsay.getByRole('button', { name: 'Add a task', exact: true }).click();
  await page.getByRole('textbox', { name: 'New task for Lindsay' }).fill(title);
  await page.getByRole('button', { name: 'Add task for Lindsay', exact: true }).click();
  await lindsay.getByText(title, { exact: true }).waitFor();
  taskCreated = true;
  await page.reload();
  await page.getByRole('button', { name: `Complete ${title}`, exact: true }).click();
  await lindsay.getByRole('button', { name: /Show \d+ completed/ }).click();
  await page.getByRole('button', { name: `Reopen ${title}`, exact: true }).waitFor();
  await page.getByRole('button', { name: `Edit ${title}`, exact: true }).click();
  await page.getByRole('textbox', { name: 'Task title', exact: true }).fill(`${title} edited`);
  await page.getByRole('button', { name: 'Save task', exact: true }).click();
  await lindsay.getByText(`${title} edited`, { exact: true }).waitFor();
  await page.getByRole('button', { name: `Delete ${title} edited`, exact: true }).click();
  await lindsay.getByText(`${title} edited`, { exact: true }).waitFor({ state: 'detached' });
  taskCreated = false;
  console.log('PASS: task create, complete, edit, delete, and reload.');

  await page.getByRole('button', { name: 'Add an event', exact: true }).click();
  await page.getByRole('textbox', { name: 'Event name' }).fill(title);
  await page.getByRole('button', { name: 'Add event', exact: true }).click();
  await page.locator('.agenda-event').filter({ hasText: title }).waitFor();
  eventCreated = true;
  await page.getByRole('button', { name: 'Weekly Calendar', exact: true }).click();
  await page.getByRole('button', { name: 'Previous Week', exact: true }).waitFor();
  await page.locator('.full-calendar').getByText(title, { exact: true }).first().waitFor();
  await page.getByRole('button', { name: 'Next Week', exact: true }).click();
  await page.getByRole('button', { name: 'Add an event', exact: true }).click();
  const nextWeekDate = await page.getByLabel('Date', { exact: true }).inputValue();
  const today = await page.evaluate(() => new Date().toLocaleDateString('en-CA'));
  assert.notEqual(nextWeekDate, today, 'Adding an event follows calendar navigation');
  await page.getByRole('button', { name: 'Cancel', exact: true }).click();
  await page.getByRole('button', { name: 'Previous Week', exact: true }).click();
  await page.getByRole('button', { name: 'Monthly Calendar', exact: true }).click();
  await page.getByRole('button', { name: 'Previous Month', exact: true }).waitFor();
  await page.locator('.full-calendar').getByText(title, { exact: true }).first().click();
  await page.getByRole('heading', { name: title, exact: true }).waitFor();
  await page.getByRole('button', { name: 'Delete event', exact: true }).click();
  await page.locator('.full-calendar').getByText(title, { exact: true }).waitFor({ state: 'detached' });
  eventCreated = false;
  console.log('PASS: daily, weekly, and monthly event views and event deletion.');

  await page.getByRole('button', { name: 'House', exact: true }).click();
  await page.getByRole('heading', { name: 'Every day', exact: true }).waitFor();
  await page.getByRole('button', { name: 'Food', exact: true }).click();
  await page.getByRole('heading', { name: "What's for dinner?" }).waitFor();
  await page.getByRole('button', { name: 'Home', exact: true }).click();
  await mkdir('.spacetime/screenshots', { recursive: true });
  await page.screenshot({ path: '.spacetime/screenshots/home-2560.png', fullPage: true });
  await page.getByRole('button', { name: 'Monthly Calendar', exact: true }).click();
  await page.getByRole('button', { name: 'Previous Month', exact: true }).waitFor();
  await page.screenshot({ path: '.spacetime/screenshots/month-2560.png', fullPage: true });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.getByRole('button', { name: 'Home', exact: true }).click();
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Mobile layout has no horizontal overflow');
  assert.deepEqual(errors, [], 'Browser produced no uncaught errors');
  console.log('PASS: 2560×1440 layout, persistent pairing, task CRUD, event CRUD, all tabs, and mobile overflow.');
} finally {
  await page.setViewportSize({ width: 2560, height: 1440 });
  if (taskCreated || eventCreated) {
    await page.getByRole('button', { name: 'Home', exact: true }).click();
    if (taskCreated) {
      await page.getByRole('button', { name: /Show \d+ completed/ }).first().click().catch(() => {});
      await page.getByRole('button', { name: `Delete ${title}`, exact: true }).click().catch(() => {});
      await page.getByRole('button', { name: `Delete ${title} edited`, exact: true }).click().catch(() => {});
    }
    if (eventCreated) {
      await page.locator('.agenda-event').filter({ hasText: title }).click().catch(() => {});
      await page.getByRole('button', { name: 'Delete event', exact: true }).click().catch(() => {});
    }
  }
  await page.getByRole('button', { name: 'Disconnect this screen' }).click().catch(() => {});
  await browser.close();
}
