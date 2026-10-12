import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { randomUUID } from 'node:crypto';
import { DbConnection } from '../src/module_bindings';
import { checkFeeds } from './calendar-feed-check';

const accessCode = process.env.TEST_HOUSEHOLD_CODE ?? (await readFile('.spacetime/access-code', 'utf8')).trim();
const database = process.env.TEST_SPACETIMEDB_DB_NAME ?? 'family-on-the-run';
const adminCode = process.env.TEST_ADMIN_CODE;
const prefix = `Integration ${randomUUID()}`;
const clients: DbConnection[] = [];

async function connect(token?: string): Promise<{ connection: DbConnection; token: string }> {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error('Connection timed out')), 10_000);
    const connection = DbConnection.builder()
      .withUri('http://127.0.0.1:3000')
      .withDatabaseName(database)
      .withToken(token)
      .onConnect((connection, _identity, savedToken) => {
        connection.subscriptionBuilder()
          .onApplied(() => { clearTimeout(timeout); resolve({ connection, token: savedToken }); })
          .onError(context => { clearTimeout(timeout); reject(context.event); })
          .subscribe([
            'SELECT * FROM my_device',
            'SELECT * FROM household_people',
            'SELECT * FROM household_tasks',
            'SELECT * FROM household_events',
            'SELECT * FROM my_admin',
            'SELECT * FROM admin_google_config',
            'SELECT * FROM admin_google_calendars',
            'SELECT * FROM admin_calendar_feeds',
            'SELECT * FROM household_imported_events',
          ]);
      })
      .onConnectError((_context, error) => { clearTimeout(timeout); reject(error); })
      .build();
    clients.push(connection);
  });
}

async function waitFor(predicate: () => boolean, description: string): Promise<void> {
  const deadline = Date.now() + 10_000;
  while (!predicate()) {
    if (Date.now() > deadline) throw new Error(`Timed out: ${description}`);
    await new Promise(resolve => setTimeout(resolve, 25));
  }
}

try {
  const first = await connect();
  const second = await connect();
  const a = first.connection;
  const b = second.connection;
  assert.equal(a.db.householdPeople.count(), 0n, 'Unpaired clients cannot read people');
  assert.equal(a.db.householdTasks.count(), 0n, 'Unpaired clients cannot read tasks');
  assert.equal(a.db.householdEvents.count(), 0n, 'Unpaired clients cannot read events');
  await assert.rejects(a.reducers.addTask({ personId: 1, taskTitle: prefix }), /Connect this screen/);
  await assert.rejects(a.reducers.joinHousehold({ accessCode: 'incorrect' }), /isn't correct/);
  await assert.rejects(a.reducers.configureAccess({ accessCode: 'x'.repeat(48) }), /Only the database owner/);
  await new Promise<void>((resolve, reject) => {
    const timeout = setTimeout(() => reject(new Error('Private table check timed out')), 10_000);
    a.subscriptionBuilder()
      .onApplied(() => { clearTimeout(timeout); reject(new Error('Private household table was readable')); })
      .onError(() => { clearTimeout(timeout); resolve(); })
      .subscribe('SELECT * FROM household');
  });
  console.log('PASS: unpaired reads, writes, incorrect pairing codes, and owner-only settings are protected.');

  await a.reducers.joinHousehold({ accessCode });
  await b.reducers.joinHousehold({ accessCode });
  await waitFor(() => a.db.householdPeople.count() === 4n && b.db.householdPeople.count() === 4n, 'pair both clients');
  assert.deepEqual([...a.db.householdPeople.iter()].sort((a, b) => a.position - b.position).map(person => person.name), ['Lindsay', 'Matt', 'Carter', 'Sydney']);
  await a.reducers.addTask({ personId: 1, taskTitle: prefix });
  await waitFor(() => [...b.db.householdTasks.iter()].some(task => task.title === prefix), 'task appears on second client');
  const task = [...b.db.householdTasks.iter()].find(task => task.title === prefix)!;
  await b.reducers.updateTask({ taskId: task.id, taskTitle: `${prefix} edited`, completed: true });
  await waitFor(() => [...a.db.householdTasks.iter()].some(row => row.id === task.id && row.completed && row.title.endsWith(' edited')), 'edit and completion reach first client');
  console.log('PASS: task creation, editing, and completion synchronize between two clients.');

  a.disconnect();
  const reconnected = await connect(first.token);
  const c = reconnected.connection;
  assert.equal(c.db.myDevice.count(), 1n, 'Pairing persists for the same identity');
  assert.ok([...c.db.householdTasks.iter()].some(row => row.id === task.id && row.completed), 'Task persists after reconnect');
  await assert.rejects(c.reducers.addTask({ personId: 1, taskTitle: '   ' }), /Enter a title/);
  await assert.rejects(c.reducers.addTask({ personId: 999, taskTitle: prefix }), /Person not found/);

  const start = BigInt(Date.now());
  const event = { eventTitle: prefix, startAt: start, endAt: start + 3_600_000n, allDay: false, color: '#527f70', location: 'Integration test' };
  await assert.rejects(c.reducers.addEvent({ ...event, endAt: start - 1n }), /end time/);
  await c.reducers.addEvent(event);
  await waitFor(() => [...b.db.householdEvents.iter()].some(row => row.title === prefix), 'event reaches second client');
  const savedEvent = [...b.db.householdEvents.iter()].find(row => row.title === prefix)!;
  const stranger = (await connect()).connection;
  assert.equal(stranger.db.householdTasks.count(), 0n, 'Unpaired identity cannot read existing tasks');
  assert.equal(stranger.db.householdEvents.count(), 0n, 'Unpaired identity cannot read existing events');

  await b.reducers.leaveHousehold({});
  await waitFor(() => b.db.myDevice.count() === 0n && b.db.householdPeople.count() === 0n && b.db.householdTasks.count() === 0n && b.db.householdEvents.count() === 0n, 'revoked client loses populated household rows');
  await assert.rejects(b.reducers.addTask({ personId: 1, taskTitle: prefix }), /Connect this screen/);
  console.log('PASS: unpaired clients cannot read existing data; revocation removes populated views and write access.');

  await c.reducers.deleteEvent({ eventId: savedEvent.id });
  await c.reducers.deleteTask({ taskId: task.id });
  await waitFor(() => ![...c.db.householdTasks.iter()].some(row => row.id === task.id) && ![...c.db.householdEvents.iter()].some(row => row.id === savedEvent.id), 'deletions synchronize');
  console.log('PASS: reconnect preserves pairing and data; validation and event creation/deletion work.');
  if (adminCode) {
    await b.reducers.joinHousehold({ accessCode });
    const config = { clientId: '123.apps.googleusercontent.com', clientSecret: 'test-only-secret', redirectUri: 'https://family.example/oauth/google/callback', clearSecret: false };
    await assert.rejects(b.reducers.saveGoogleConfig(config), /Unlock Admin/);
    await assert.rejects(b.reducers.updatePerson({ personId: 1, name: 'Changed', color: '#527f70' }), /Unlock Admin/);
    await assert.rejects(b.reducers.addGoogleCalendar({ calendarId: 'primary', name: 'Family', color: '#527f70' }), /Unlock Admin/);
    await assert.rejects(c.reducers.unlockAdmin({ accessCode }), /isn't correct/);
    await c.reducers.unlockAdmin({ accessCode: adminCode });
    await waitFor(() => c.db.myAdmin.count() === 1n, 'admin unlocked');
    await checkFeeds(c, b, stranger, waitFor, database);
    await assert.rejects(c.reducers.saveGoogleConfig({ ...config, redirectUri: 'http://outside.example/callback' }), /HTTPS/);
    await c.reducers.saveGoogleConfig(config);
    await waitFor(() => c.db.adminGoogleConfig.count() === 1n, 'OAuth config saved');
    const summary = [...c.db.adminGoogleConfig.iter()][0];
    assert.equal(summary.hasClientSecret, true);
    assert.ok(!('clientSecret' in summary), 'Saved secrets are not returned to browsers');
    assert.equal(b.db.adminGoogleConfig.count(), 0n, 'Paired non-admin cannot read OAuth settings');
    await c.reducers.saveGoogleConfig({ ...config, clientSecret: '' });
    assert.equal([...c.db.adminGoogleConfig.iter()][0].hasClientSecret, true, 'Blank field retains secret');
    await c.reducers.saveGoogleConfig({ ...config, clientSecret: '', clientId: '456.apps.googleusercontent.com' });
    await waitFor(() => [...c.db.adminGoogleConfig.iter()][0]?.hasClientSecret === false, 'new client does not reuse old secret');
    await c.reducers.addGoogleCalendar({ calendarId: 'primary', name: 'Family', color: '#527f70' });
    await waitFor(() => c.db.adminGoogleCalendars.count() === 1n, 'source saved');
    await assert.rejects(c.reducers.addGoogleCalendar({ calendarId: 'primary', name: 'Duplicate', color: '#527f70' }), /already configured/);
    assert.equal(b.db.adminGoogleCalendars.count(), 0n);
    const source = [...c.db.adminGoogleCalendars.iter()][0];
    await c.reducers.updatePerson({ personId: 1, name: 'New name', color: '#123456' });
    await waitFor(() => [...b.db.householdPeople.iter()].some(person => person.id === 1 && person.name === 'New name' && person.color === '#123456'), 'member settings update household views');
    await c.reducers.lockAdmin({});
    await waitFor(() => c.db.myAdmin.count() === 0n && c.db.adminGoogleConfig.count() === 0n && c.db.adminGoogleCalendars.count() === 0n, 'locking removes admin views');
    await assert.rejects(c.reducers.clearGoogleConfig({}), /Unlock Admin/);
    await c.reducers.unlockAdmin({ accessCode: adminCode });
    await waitFor(() => c.db.adminGoogleCalendars.count() === 1n, 'saved settings survive relock');
    await c.reducers.removeGoogleCalendar({ sourceId: source.id });
    await c.reducers.clearGoogleConfig({});
    await c.reducers.leaveHousehold({});
    await waitFor(() => c.db.myAdmin.count() === 0n, 'household disconnect revokes admin');
    console.log('PASS: admin authorization, OAuth validation and secret retention, calendar sources, member settings, and locking.');
  }
} finally {
  for (const connection of clients.filter(client => client.isActive)) {
    try {
      for (const task of connection.db.householdTasks.iter()) {
        if (task.title.startsWith(prefix)) await connection.reducers.deleteTask({ taskId: task.id });
      }
      for (const event of connection.db.householdEvents.iter()) {
        if (event.title === prefix) await connection.reducers.deleteEvent({ eventId: event.id });
      }
      await connection.reducers.leaveHousehold({});
    } finally { connection.disconnect(); }
  }
}
