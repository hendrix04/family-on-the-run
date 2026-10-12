<script lang="ts">
  import { tick } from 'svelte';
  import { useReducer } from 'spacetimedb/svelte';
  import { reducers } from '../module_bindings';
  import type { Person, Task } from '../module_bindings/types';
  import Icon from './Icon.svelte';

  let { person, tasks, connected, onerror }: {
    person: Person; tasks: readonly Task[]; connected: boolean; onerror: (error: unknown) => void;
  } = $props();
  const addTask = useReducer(reducers.addTask);
  const updateTask = useReducer(reducers.updateTask);
  const deleteTask = useReducer(reducers.deleteTask);
  let draft = $state('');
  let adding = $state(false);
  let busy = $state(false);
  let showCompleted = $state(false);
  let editing = $state<bigint | null>(null);
  let editedTitle = $state('');
  let taskInput = $state<HTMLInputElement>();
  const mine = $derived(tasks.filter(task => task.personId === person.id).toSorted((a, b) => a.createdAt < b.createdAt ? -1 : 1));
  const open = $derived(mine.filter(task => !task.completed));
  const completed = $derived(mine.filter(task => task.completed));

  async function add(event: SubmitEvent) {
    event.preventDefault();
    if (!draft.trim() || busy || !connected) return;
    busy = true;
    try {
      await addTask({ personId: person.id, taskTitle: draft });
      draft = '';
      taskInput?.focus();
    } catch (error) { onerror(error); }
    finally { busy = false; }
  }

  async function toggle(task: Task) {
    try { await updateTask({ taskId: task.id, taskTitle: task.title, completed: !task.completed }); }
    catch (error) { onerror(error); }
  }

  async function save(task: Task, event: SubmitEvent) {
    event.preventDefault();
    if (!editedTitle.trim() || busy || !connected) return;
    busy = true;
    try {
      await updateTask({ taskId: task.id, taskTitle: editedTitle, completed: task.completed });
      editing = null;
    } catch (error) { onerror(error); }
    finally { busy = false; }
  }

  async function remove(task: Task) {
    try { await deleteTask({ taskId: task.id }); }
    catch (error) { onerror(error); }
  }

  async function showAdd() {
    adding = true;
    await tick();
    taskInput?.focus();
  }
</script>

<section class="dashboard-column task-column" style:--person-color={person.color}>
  <header class="column-header">
    <span class="person-avatar">{person.name.slice(0, 1)}</span>
    <div><h2>{person.name} To Do</h2><p>{open.length} {open.length === 1 ? 'task' : 'tasks'} to go</p></div>
  </header>
  <div class="column-content">
    {#each (showCompleted ? [...open, ...completed] : open) as task (task.id)}
      <div class:task-completed={task.completed} class="task-row">
        <button class="task-check" class:checked={task.completed} aria-label={`${task.completed ? 'Reopen' : 'Complete'} ${task.title}`} aria-pressed={task.completed} disabled={!connected} onclick={() => toggle(task)}>
          {#if task.completed}<Icon name="check" size={19} />{/if}
        </button>
        {#if editing === task.id}
          <form class="task-edit" onsubmit={event => save(task, event)}>
            <input aria-label="Task title" bind:value={editedTitle} maxlength="200" required disabled={!connected || busy} />
            <button class="icon-button" aria-label="Save task" disabled={!connected || busy}><Icon name="check" /></button>
            <button class="icon-button" type="button" aria-label="Cancel edit" onclick={() => editing = null}><Icon name="close" /></button>
          </form>
        {:else}
          <span class="task-title">{task.title}</span>
          <button class="icon-button subtle" aria-label={`Edit ${task.title}`} disabled={!connected} onclick={() => { editing = task.id; editedTitle = task.title; }}><Icon name="edit" size={17} /></button>
          <button class="icon-button subtle" aria-label={`Delete ${task.title}`} disabled={!connected} onclick={() => remove(task)}><Icon name="trash" size={17} /></button>
        {/if}
      </div>
    {:else}
      <div class="column-empty"><span class="empty-check"><Icon name="check" size={26} /></span><h3>A clear little list.</h3><p>Add something to tackle<br />when you're ready.</p></div>
    {/each}
    {#if completed.length > 0}
      <button class="completed-toggle" onclick={() => showCompleted = !showCompleted}>{showCompleted ? 'Hide' : 'Show'} {completed.length} completed {completed.length === 1 ? 'task' : 'tasks'}</button>
    {/if}
  </div>
  <footer class="column-footer">
    {#if adding}
      <form class="task-add" onsubmit={add}>
        <input bind:this={taskInput} bind:value={draft} placeholder="What needs doing?" aria-label={`New task for ${person.name}`} maxlength="200" disabled={!connected || busy} required />
        <button class="icon-button add-submit" aria-label={`Add task for ${person.name}`} disabled={!connected || busy || !draft.trim()}><Icon name="plus" /></button>
      </form>
    {:else}
      <button class="add-task-button" disabled={!connected} onclick={showAdd}><Icon name="plus" size={19} /> Add a task</button>
    {/if}
  </footer>
</section>
