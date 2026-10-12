<script lang="ts">
  import { onMount } from 'svelte';
  import { useSpacetimeDB, useTable, useReducer } from 'spacetimedb/svelte';
  import { tables, reducers } from './module_bindings';
  import { importedEvent, type DisplayEvent } from './lib/calendar-events';
  import { dateInput, localDate } from './lib/dates';
  import Icon from './components/Icon.svelte';
  import TaskColumn from './components/TaskColumn.svelte';
  import DailyCalendar from './components/DailyCalendar.svelte';
  import CalendarPage from './components/CalendarPage.svelte';
  import EventDialog from './components/EventDialog.svelte';
  import HousePage from './components/HousePage.svelte';
  import AdminPage from './components/AdminPage.svelte';

  const connection = useSpacetimeDB();
  const [devices, deviceReady] = useTable(tables.myDevice);
  const [people] = useTable(tables.householdPeople);
  const [tasks] = useTable(tables.householdTasks);
  const [events] = useTable(tables.householdEvents);
  const [imports] = useTable(tables.householdImportedEvents);
  const calendarEvents = $derived([...$events, ...$imports.map(importedEvent)]);
  const join = useReducer(reducers.joinHousehold);
  const leave = useReducer(reducers.leaveHousehold);
  const tabs = [
    { name: 'Home', icon: 'home' },
    { name: 'Weekly Calendar', icon: 'calendar' },
    { name: 'Monthly Calendar', icon: 'month' },
    { name: 'Food', icon: 'food' },
    { name: 'House', icon: 'house' },
    { name: 'Admin', icon: 'settings' },
  ];
  let tab = $state('Home');
  let selectedDate = $state(new Date());
  let now = $state(new Date());
  let accessCode = $state('');
  let pairing = $state(false);
  let errorMessage = $state('');
  let eventOpen = $state(false);
  let selectedEvent = $state<DisplayEvent | null>(null);
  const paired = $derived($devices.length > 0);
  const sortedPeople = $derived($people.toSorted((a, b) => a.position - b.position));
  const remaining = $derived($tasks.filter(task => !task.completed).length);
  const greeting = $derived(now.getHours() < 12 ? 'Good morning' : now.getHours() < 17 ? 'Good afternoon' : 'Good evening');

  onMount(() => {
    function updateClock() {
      const next = new Date();
      if (dateInput(now) !== dateInput(next) && dateInput(selectedDate) === dateInput(now)) selectedDate = next;
      now = next;
    }
    const timer = setInterval(updateClock, 30_000);
    document.addEventListener('visibilitychange', updateClock);
    return () => { clearInterval(timer); document.removeEventListener('visibilitychange', updateClock); };
  });

  function showError(error: unknown) {
    errorMessage = error instanceof Error ? error.message : String(error);
  }

  async function pair(event: SubmitEvent) {
    event.preventDefault();
    if (pairing || !$connection.isActive) return;
    pairing = true;
    errorMessage = '';
    try { await join({ accessCode: accessCode.trim() }); accessCode = ''; }
    catch (error) { showError(error); }
    finally { pairing = false; }
  }

  function changeDay(days: number) {
    const date = new Date(selectedDate);
    date.setDate(date.getDate() + days);
    selectedDate = date;
  }

  function openEvent(event: DisplayEvent | null = null) { selectedEvent = event; eventOpen = true; }
</script>

<div class="app-shell">
  <aside class="sidebar">
    <button class="brand" aria-label="Family on the Run home" onclick={() => tab = 'Home'}>
      <span class="brand-mark"><Icon name="home" size={29} /></span><span>family<br /><strong>on the run</strong></span>
    </button>
    <p class="sidebar-label">OUR EVERYDAY</p>
    <nav aria-label="Main navigation">
      {#each tabs as item}<button aria-label={item.name} class:active={tab === item.name} aria-current={tab === item.name ? 'page' : undefined} onclick={() => tab = item.name}><Icon name={item.icon} /><span>{item.name}</span>{#if tab === item.name}<span class="nav-indicator"></span>{/if}</button>{/each}
    </nav>
    <div class="sidebar-footer"><span class="family-initials">{#each sortedPeople as person (person.id)}<span style:background={person.color}>{person.name.slice(0, 1)}</span>{/each}</span><p>One family. One place.</p>{#if paired}<button class="disconnect-button" onclick={async () => { try { await leave(); } catch (error) { showError(error); } }}><Icon name="logout" size={16} />Disconnect this screen</button>{/if}</div>
  </aside>

  <main>
    <header class="app-header">
      <div><p class="eyebrow">{now.toLocaleDateString([], { weekday: 'long', month: 'long', day: 'numeric' }).toUpperCase()}</p><h1>{tab === 'Home' ? `${greeting}, family.` : tab}</h1><p class="header-subtitle">{tab === 'Home' ? 'Your day, all together.' : tab === 'Weekly Calendar' || tab === 'Monthly Calendar' ? 'A little space to see what’s ahead.' : tab === 'Food' ? 'Good things start around the table.' : 'Keep the everyday running smoothly.'}</p></div>
      <div class="header-right"><span class="connection-status" class:online={$connection.isActive}><span></span>{$connection.isActive ? 'Connected' : 'Connecting…'}</span><span class="live-clock">{now.toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' })}</span></div>
    </header>

    {#if errorMessage}<div class="error-banner" role="alert"><span>{errorMessage}</span><button class="icon-button" aria-label="Dismiss error" onclick={() => errorMessage = ''}><Icon name="close" size={18} /></button></div>{/if}

    {#if !paired}
      <section class="pairing-panel">
        <span class="welcome-icon"><Icon name="home" size={37} /></span>
        <p class="eyebrow">WELCOME HOME</p><h2>A place for your family's day.</h2><p>Connect this screen to your household to see your plans and to-do lists.</p>
        <form onsubmit={pair}><label for="household-code">Household access code</label><input id="household-code" type="password" bind:value={accessCode} placeholder="Enter your household code" autocomplete="off" required disabled={!$connection.isActive || pairing} /><button class="primary-button" disabled={!$connection.isActive || pairing || !accessCode.trim()}>{pairing ? 'Connecting…' : 'Connect this screen'}<Icon name="chevronRight" size={18} /></button></form>
        <p class="pairing-note">{!$connection.isActive ? 'Waiting for the household server…' : !$deviceReady ? 'Checking this screen…' : 'You only need to connect once on each screen.'}</p>
      </section>
    {:else if tab === 'Home'}
      <div class="home-toolbar"><div class="date-navigation"><button class="icon-button" aria-label="Previous day" onclick={() => changeDay(-1)}><Icon name="chevronLeft" /></button><label class="date-label">{selectedDate.toLocaleDateString([], { weekday: 'short', month: 'short', day: 'numeric' })}<input type="date" aria-label="Choose day" value={dateInput(selectedDate)} onchange={event => { if (event.currentTarget.value) selectedDate = localDate(event.currentTarget.value); }} /></label><button class="icon-button" aria-label="Next day" onclick={() => changeDay(1)}><Icon name="chevronRight" /></button><button class="today-button" onclick={() => selectedDate = new Date()}>Today</button></div><p><strong>{remaining}</strong> things to tackle together</p></div>
      <div class="home-grid"><DailyCalendar date={selectedDate} events={calendarEvents} onadd={() => openEvent()} onselect={openEvent} />{#each sortedPeople as person (person.id)}<TaskColumn {person} tasks={$tasks} connected={$connection.isActive} onerror={showError} />{/each}</div>
      <p class="calendar-note"><Icon name="calendar" size={16} />Google calendar feeds appear here automatically. Manage them in Admin.</p>
    {:else if tab === 'Weekly Calendar' || tab === 'Monthly Calendar'}
      <div class="calendar-actions"><p><span class="calendar-dot"></span>Family calendar</p><button class="primary-button" disabled={!$connection.isActive} onclick={() => openEvent()}><Icon name="plus" size={19} />Add an event</button></div>
      <CalendarPage view={tab === 'Weekly Calendar' ? 'timeGridWeek' : 'dayGridMonth'} date={selectedDate} events={calendarEvents} onselect={openEvent} ondatechange={date => selectedDate = date} />
      <p class="calendar-note">Google events are read-only. Imported calendars cover only events that have not started yet, up to 365 days ahead.</p>
    {:else if tab === 'House'}
      <HousePage />
    {:else if tab === 'Admin'}
      <AdminPage people={sortedPeople} />
    {:else}
      <section class="food-placeholder"><span class="welcome-icon"><Icon name="food" size={39} /></span><p class="eyebrow">AROUND THE TABLE</p><h2>What's for dinner?</h2><p>Recipes, meal plans, and a shared grocery list will live here.</p><span class="coming-soon">Coming next</span></section>
    {/if}
  </main>
</div>

{#if eventOpen && paired}<EventDialog date={selectedDate} selected={selectedEvent} connected={$connection.isActive} onclose={() => eventOpen = false} onerror={showError} />{/if}
