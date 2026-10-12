<script lang="ts">
  import { untrack } from 'svelte';
  import { useReducer } from 'spacetimedb/svelte';
  import { reducers } from '../module_bindings';
  import type { Person } from '../module_bindings/types';
  let { person, connected }: { person: Person; connected: boolean } = $props();
  const update = useReducer(reducers.updatePerson);
  let name = $state(untrack(() => person.name));
  let color = $state(untrack(() => person.color));
  let busy = $state(false);
  let message = $state('');
  let failed = $state(false);

  async function save(event: SubmitEvent) {
    event.preventDefault();
    if (busy || !connected) return;
    busy = true; message = ''; failed = false;
    try { await update({ personId: person.id, name, color }); message = 'Saved'; }
    catch (error) { failed = true; message = error instanceof Error ? error.message : String(error); }
    finally { busy = false; }
  }
</script>

<form class="member-settings" onsubmit={save} aria-label={`Settings for ${person.name}`}>
  <div class="member-fields">
    <label>Name<input aria-label={`Name for member ${person.id}`} bind:value={name} maxlength="50" required disabled={busy || !connected} /></label>
    <label>Color<input type="color" aria-label={`Color for member ${person.id}`} bind:value={color} disabled={busy || !connected} /></label>
    <button class="secondary-button" disabled={busy || !connected || !name.trim()}>{busy ? 'Saving…' : 'Save member'}</button>
  </div>
  {#if message}<p class:admin-error={failed} class:admin-success={!failed} role={failed ? 'alert' : 'status'}>{message}</p>{/if}
</form>
