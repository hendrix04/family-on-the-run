import assert from 'node:assert/strict';
import type { DbConnection } from '../src/module_bindings';

export async function checkFeeds(admin: DbConnection, household: DbConnection, stranger: DbConnection,
  waitFor: (predicate: () => boolean, description: string) => Promise<void>, _database: string) {
  const input = { name: 'ICS test calendar', color: '#123456', url: 'https://calendar.google.com/calendar/ical/en.usa%23holiday%40group.v.calendar.google.com/public/basic.ics' };
  await assert.rejects(household.reducers.addCalendarFeed(input), /Unlock Admin/);
  await assert.rejects(admin.reducers.initializeCalendarSchedules({}), /Only the database owner/);
  await assert.rejects(admin.reducers.addCalendarFeed({ ...input, url: 'http://127.0.0.1/basic.ics' }), /Secret address/);
  const importStartedAt = BigInt(Date.now());
  await admin.reducers.addCalendarFeed(input);
  await waitFor(() => admin.db.adminCalendarFeeds.count() === 1n, 'feed saved');
  let summary = [...admin.db.adminCalendarFeeds.iter()][0];
  assert.ok(!('url' in summary), 'Admin summary does not contain the secret URL');
  assert.equal(household.db.adminCalendarFeeds.count(), 0n);
  await assert.rejects(admin.reducers.addCalendarFeed(input), /already configured/);
  await waitFor(() => [...admin.db.adminCalendarFeeds.iter()][0]?.lastSuccess > 0n, 'built-in scheduler downloads and imports real Google feed');
  assert.ok(household.db.householdImportedEvents.count() > 0n);
  assert.ok([...household.db.householdImportedEvents.iter()].every(event => event.startAt >= importStartedAt), 'Only future event starts are imported');
  assert.equal(stranger.db.householdImportedEvents.count(), 0n);
  summary = [...admin.db.adminCalendarFeeds.iter()][0];
  await admin.reducers.refreshCalendarFeed({ feedId: summary.id });
  await waitFor(() => [...admin.db.adminCalendarFeeds.iter()][0]?.lastSuccess > summary.lastSuccess, 'manual refresh executes through scheduler');
  await admin.reducers.removeCalendarFeed({ feedId: summary.id });
  await waitFor(() => household.db.householdImportedEvents.count() === 0n, 'source removal removes imported events');
  // Invalid Google URL checks automatic failure reporting without exposing URL contents.
  await admin.reducers.addCalendarFeed({ ...input, url: 'https://calendar.google.com/calendar/ical/test/private-test/basic.ics' });
  await waitFor(() => [...admin.db.adminCalendarFeeds.iter()][0]?.error.length > 0, 'download error status');
  const failed = [...admin.db.adminCalendarFeeds.iter()][0];
  assert.equal(failed.lastSuccess, 0n);
  assert.ok(!failed.error.includes('private-test'));
  await admin.reducers.removeCalendarFeed({ feedId: failed.id });
  if (process.env.TEST_BROWSER) {
    await admin.reducers.addCalendarFeed({ ...input, name: 'Imported calendar' });
    await waitFor(() => [...admin.db.adminCalendarFeeds.iter()][0]?.lastSuccess > 0n, 'browser calendar imported');
    const event = [...household.db.householdImportedEvents.iter()].sort((a, b) => Number(a.startAt - b.startAt))[0];
    process.env.TEST_IMPORTED_TITLE = event.title;
    process.env.TEST_IMPORTED_DATE = event.startsOn;
  }
  console.log('PASS: feed permissions, private URLs, built-in scheduled Google downloads, imports, manual refresh, error status, and removal.');
}
