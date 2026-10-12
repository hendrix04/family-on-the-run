<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { useReducer } from 'spacetimedb/svelte';
  import { reducers } from '../module_bindings';
  import type { DisplayEvent } from '../lib/calendar-events';
  import { dateInput, localDate, timeLabel } from '../lib/dates';
  import Icon from './Icon.svelte';

  let { date, selected = null, onclose, onerror, connected }: {
    date: Date; selected?: DisplayEvent | null; onclose: () => void; onerror: (error: unknown) => void; connected: boolean;
  } = $props();
  const addEvent = useReducer(reducers.addEvent);
  const deleteEvent = useReducer(reducers.deleteEvent);
  let dialog: HTMLDialogElement;
  let title = $state('');
  let eventDate = $state(untrack(() => dateInput(date)));
  let startTime = $state('09:00');
  let endTime = $state('10:00');
  let allDay = $state(false);
  let location = $state('');
  let color = $state('#527f70');
  let busy = $state(false);
  let errorMessage = $state('');
  onMount(() => { dialog.showModal(); });

  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (busy || !connected) return;
    const start = allDay ? localDate(eventDate) : new Date(`${eventDate}T${startTime}`);
    const end = allDay ? localDate(eventDate) : new Date(`${eventDate}T${endTime}`);
    if (allDay) end.setDate(end.getDate() + 1);
    if (!Number.isFinite(start.getTime()) || end <= start) { errorMessage = 'Choose an end time after the start time.'; return; }
    busy = true;
    errorMessage = '';
    try {
      await addEvent({ eventTitle: title, startAt: BigInt(start.getTime()), endAt: BigInt(end.getTime()), allDay, color, location });
      onclose();
    } catch (error) { errorMessage = error instanceof Error ? error.message : 'Could not save the event.'; }
    finally { busy = false; }
  }

  async function remove() {
    if (!selected || typeof selected.id !== 'bigint' || busy || !connected) return;
    busy = true;
    try { await deleteEvent({ eventId: selected.id }); onclose(); }
    catch (error) { onerror(error); }
    finally { busy = false; }
  }
</script>

<dialog bind:this={dialog} oncancel={onclose} onclose={onclose} class="event-dialog">
  <header class="dialog-header"><div><p class="eyebrow">FAMILY CALENDAR</p><h2>{selected ? selected.title : 'Make a little plan.'}</h2></div><button class="icon-button" aria-label="Close event" onclick={onclose}><Icon name="close" /></button></header>
  {#if selected}
    <div class="event-details">
      <p><Icon name="calendar" />{new Date(Number(selected.startAt)).toLocaleDateString([], { weekday: 'long', month: 'long', day: 'numeric' })}</p>
      <p><Icon name="clock" />{selected.allDay ? 'All day' : `${timeLabel(Number(selected.startAt))} – ${timeLabel(Number(selected.endAt))}`}</p>
      {#if selected.location}<p><Icon name="location" />{selected.location}</p>{/if}
      <p class="muted">{selected.sourceName ? `From ${selected.sourceName}. Edit this event in Google Calendar.` : 'This event is saved in your family dashboard.'}</p>
    </div>
    <footer class="dialog-actions">{#if !selected.sourceName}<button class="danger-button" disabled={!connected || busy} onclick={remove}><Icon name="trash" size={18} /> Delete event</button>{/if}<button class="primary-button" onclick={onclose}>Done</button></footer>
  {:else}
    <form onsubmit={save} class="event-form">
      <label>Event name<input bind:value={title} placeholder="What's happening?" maxlength="200" required /></label>
      <label>Date<input type="date" bind:value={eventDate} required /></label>
      <label class="checkbox-label"><input type="checkbox" bind:checked={allDay} />All-day event</label>
      {#if !allDay}<div class="form-grid"><label>Starts<input type="time" bind:value={startTime} required /></label><label>Ends<input type="time" bind:value={endTime} required /></label></div>{/if}
      <label>Location <span class="muted">(optional)</span><input bind:value={location} placeholder="Where are we headed?" maxlength="200" /></label>
      <fieldset class="color-picker"><legend>Calendar color</legend>{#each ['#527f70', '#bf755e', '#6d8ba1', '#b69a55', '#8a7ca4'] as choice}<button type="button" style:background={choice} class:selected={color === choice} aria-label={`Choose ${choice}`} aria-pressed={color === choice} onclick={() => color = choice}>{#if color === choice}<Icon name="check" size={20} />{/if}</button>{/each}</fieldset>
      {#if errorMessage}<p class="form-error" role="alert">{errorMessage}</p>{/if}
      <footer class="dialog-actions"><button class="secondary-button" type="button" onclick={onclose}>Cancel</button><button class="primary-button" disabled={!connected || busy}>{busy ? 'Saving…' : 'Add event'}</button></footer>
    </form>
  {/if}
</dialog>
