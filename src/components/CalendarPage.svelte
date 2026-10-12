<script lang="ts">
  import { onMount } from 'svelte';
  import { Calendar } from 'fullcalendar';
  import themePlugin from 'fullcalendar/themes/monarch';
  import dayGridPlugin from 'fullcalendar/daygrid';
  import timeGridPlugin from 'fullcalendar/timegrid';
  import 'fullcalendar/skeleton.css';
  import 'fullcalendar/themes/monarch/theme.css';
  import type { DisplayEvent } from '../lib/calendar-events';
  import { dateInput } from '../lib/dates';

  let { view, date, events, onselect, ondatechange }: {
    view: 'timeGridWeek' | 'dayGridMonth'; date: Date;
    events: readonly DisplayEvent[]; onselect: (event: DisplayEvent) => void;
    ondatechange: (date: Date) => void;
  } = $props();
  let element: HTMLDivElement;
  let calendar = $state.raw<Calendar>();
  const calendarEvents = $derived(events.map(event => ({
    id: String(event.id), title: event.title,
    start: new Date(Number(event.startAt)), end: new Date(Number(event.endAt)),
    allDay: event.allDay, backgroundColor: event.color, borderColor: event.color,
  })));

  onMount(() => {
    calendar = new Calendar(element, {
      plugins: [themePlugin, dayGridPlugin, timeGridPlugin],
      initialView: view,
      initialDate: date,
      headerToolbar: { left: 'prev,next today', center: 'title', right: '' },
      height: '100%',
      nowIndicator: true,
      scrollTime: '07:00:00',
      slotMinTime: '05:00:00',
      slotMaxTime: '23:00:00',
      events: calendarEvents,
      datesSet: () => {
        const next = calendar?.getDate();
        if (next && dateInput(next) !== dateInput(date)) ondatechange(next);
      },
      eventClick: info => {
        const event = events.find(event => String(event.id) === info.event.id);
        if (event) onselect(event);
      },
    });
    calendar.render();
    return () => calendar?.destroy();
  });

  $effect(() => { calendar?.setOption('events', calendarEvents); });
  $effect(() => { calendar?.changeView(view); });
  $effect(() => { if (calendar && dateInput(calendar.getDate()) !== dateInput(date)) calendar.gotoDate(date); });
</script>

<section class="calendar-panel"><div class="full-calendar" bind:this={element}></div></section>
