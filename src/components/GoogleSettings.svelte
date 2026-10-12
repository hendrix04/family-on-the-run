<script lang="ts">
  import { useReducer, useTable } from 'spacetimedb/svelte';
  import { reducers, tables } from '../module_bindings';
  import Icon from './Icon.svelte';

  let { connected }: { connected: boolean } = $props();
  const [feeds] = useTable(tables.adminCalendarFeeds);
  const addFeed = useReducer(reducers.addCalendarFeed);
  const removeFeed = useReducer(reducers.removeCalendarFeed);
  const refreshFeed = useReducer(reducers.refreshCalendarFeed);
  let name = $state('');
  let url = $state('');
  let color = $state('#527f70');
  let busy = $state(false);
  let message = $state('');
  let error = $state('');
  const when = (value: bigint) => value ? new Date(Number(value)).toLocaleString() : 'Not yet';

  async function perform(action: () => Promise<unknown>, success: string) {
    busy = true; message = ''; error = '';
    try { await action(); message = success; }
    catch (failure) { error = failure instanceof Error ? failure.message : 'Could not save calendar settings.'; }
    finally { busy = false; }
  }
  async function add(event: SubmitEvent) {
    event.preventDefault();
    await perform(async () => { await addFeed({ name, color, url }); url = ''; name = ''; }, 'Calendar added. Its first refresh will start shortly.');
  }
</script>

<section class="settings-card">
  <header><span class="settings-icon"><Icon name="calendar" /></span><div><h2>Google Calendar</h2><p>Read your calendars using private iCal feeds.</p></div></header>
  <details class="setup-instructions" open><summary>Where to find your calendar link</summary><ol><li>Open Google Calendar on a computer, then Settings.</li><li>Choose a calendar under “Settings for my calendars.”</li><li>Open “Integrate calendar” and copy “Secret address in iCal format.”</li><li>Paste that link below. Repeat for each calendar.</li></ol><p>No Google Cloud project or OAuth app is needed. Keep this link private; anyone with it can read that calendar.</p><a href="https://support.google.com/calendar/answer/37648" target="_blank" rel="noreferrer">Google's calendar sharing instructions ↗</a></details>
  <p class="field-note">Refreshes every 5 minutes through the built-in database scheduler. Google events are read-only here; edit them in Google Calendar. Imports cover only events that have not started yet, up to 365 days ahead.</p>
  {#if message}<p class="admin-success" role="status">{message}</p>{/if}
  {#if error}<p class="admin-error" role="alert">{error}</p>{/if}
  <div class="calendar-source-settings">
    {#each $feeds as feed (feed.id)}
      <article class="feed-source">
        <div class="calendar-source"><span class="source-color" style:background={feed.color}></span><div><strong>{feed.name}</strong><span>{feed.eventCount} imported events</span></div><button class="icon-button" aria-label={`Remove calendar ${feed.name}`} disabled={busy || !connected} onclick={() => perform(() => removeFeed({ feedId: feed.id }), 'Calendar and its imported events removed.')}><Icon name="trash" size={19} /></button></div>
        <p class="field-note">Last successful refresh: {when(feed.lastSuccess)}<br />Last attempt: {when(feed.lastAttempt)}</p>
        {#if feed.error}<p class="admin-error" role="alert">{feed.error} Previously imported events are retained.</p>{/if}
        <button class="secondary-button" disabled={busy || !connected} onclick={() => perform(() => refreshFeed({ feedId: feed.id }), 'Refresh requested. The database will fetch the calendar shortly.')}>Refresh now</button>
      </article>
    {:else}<p class="source-empty">No calendar feeds configured.</p>{/each}
    <form class="settings-form" onsubmit={add}><fieldset disabled={busy || !connected}><legend>Add a calendar</legend><label>Calendar name<input bind:value={name} placeholder="Family, school, sports…" maxlength="200" required /></label><label>Secret address in iCal format<input type="password" bind:value={url} placeholder="https://calendar.google.com/calendar/ical/…/basic.ics" maxlength="2000" autocomplete="new-password" required /><span class="field-note">Stored on the server. The saved link is never returned to household or admin screens. To replace it, remove this calendar and add the new link.</span></label><label>Calendar color<input class="source-color-input" type="color" bind:value={color} /></label><button class="primary-button">{busy ? 'Saving…' : 'Add calendar feed'}</button></fieldset></form>
  </div>
</section>
