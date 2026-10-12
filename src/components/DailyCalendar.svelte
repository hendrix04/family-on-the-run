<script lang="ts">
  import type { DisplayEvent } from '../lib/calendar-events';
  import { dayBounds, timeLabel } from '../lib/dates';
  import Icon from './Icon.svelte';

  let { date, events, onadd, onselect }: {
    date: Date; events: readonly DisplayEvent[]; onadd: () => void; onselect: (event: DisplayEvent) => void;
  } = $props();
  const bounds = $derived(dayBounds(date));
  const todayEvents = $derived(events.filter(event => Number(event.startAt) < bounds[1] && Number(event.endAt) > bounds[0]).toSorted((a, b) => Number(a.startAt - b.startAt)));
</script>

<section class="dashboard-column agenda-column">
  <header class="column-header">
    <span class="agenda-avatar"><Icon name="calendar" size={23} /></span>
    <div><h2>Daily Calendar</h2><p>{todayEvents.length} {todayEvents.length === 1 ? 'event' : 'events'} planned</p></div>
  </header>
  <div class="column-content agenda-content">
    {#each todayEvents as event (event.id)}
      <button class="agenda-event" style:--event-color={event.color} onclick={() => onselect(event)}>
        <span class="event-time">{event.allDay ? 'ALL DAY' : timeLabel(Number(event.startAt))}</span>
        <strong>{event.title}</strong>
        {#if event.location}<span class="event-location"><Icon name="location" size={14} />{event.location}</span>{/if}
      </button>
    {:else}
      <div class="column-empty"><span class="empty-calendar"><Icon name="calendar" size={28} /></span><h3>A little breathing room.</h3><p>No events on this day.<br />Add one to get started.</p></div>
    {/each}
  </div>
  <footer class="column-footer"><button class="add-task-button" onclick={onadd}><Icon name="plus" size={19} /> Add an event</button></footer>
</section>
