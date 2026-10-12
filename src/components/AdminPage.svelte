<script lang="ts">
  import { useReducer, useSpacetimeDB, useTable } from 'spacetimedb/svelte';
  import { reducers, tables } from '../module_bindings';
  import type { Person } from '../module_bindings/types';
  import Icon from './Icon.svelte';
  import GoogleSettings from './GoogleSettings.svelte';
  import MemberSettings from './MemberSettings.svelte';
  import '../admin.css';

  let { people }: { people: readonly Person[] } = $props();
  const connection = useSpacetimeDB();
  const [admins, adminReady] = useTable(tables.myAdmin);
  const unlock = useReducer(reducers.unlockAdmin);
  const lock = useReducer(reducers.lockAdmin);
  let code = $state('');
  let busy = $state(false);
  let error = $state('');
  const unlocked = $derived($admins.length > 0);

  async function unlockScreen(event: SubmitEvent) {
    event.preventDefault();
    if (busy || !$connection.isActive) return;
    busy = true; error = '';
    try { await unlock({ accessCode: code.trim() }); code = ''; }
    catch (failure) { error = failure instanceof Error ? failure.message : String(failure); }
    finally { busy = false; }
  }

  async function lockScreen() {
    try { await lock(); }
    catch (failure) { error = failure instanceof Error ? failure.message : String(failure); }
  }
</script>

{#if error}<p class="admin-error" role="alert">{error}</p>{/if}
{#if !$adminReady}
  <p class="admin-toolbar" role="status">Checking admin access…</p>
{:else if !unlocked}
  <section class="pairing-panel admin-unlock">
    <span class="welcome-icon"><Icon name="settings" size={37} /></span>
    <p class="eyebrow">HOUSEHOLD SETTINGS</p><h2>Unlock Admin.</h2>
    <p>Use your separate admin code to configure integrations and family members.</p>
    <form onsubmit={unlockScreen}>
      <label for="admin-code">Admin access code</label>
      <input id="admin-code" type="password" bind:value={code} autocomplete="off" placeholder="Enter your admin code" required disabled={busy || !$connection.isActive} />
      <button class="primary-button" disabled={busy || !$connection.isActive || !code.trim()}>{busy ? 'Unlocking…' : 'Unlock Admin'}</button>
    </form>
    <p class="pairing-note">The household pairing code does not grant admin access.</p>
  </section>
{:else}
  <div class="admin-toolbar"><p>Manage the settings shared by your household.</p><button class="secondary-button" onclick={lockScreen} disabled={!$connection.isActive}><Icon name="lock" size={18} />Lock Admin</button></div>
  <div class="admin-grid">
    <GoogleSettings connected={$connection.isActive} />
    <section class="settings-card">
      <header><span class="settings-icon"><Icon name="home" /></span><div><h2>Family members</h2><p>Names and colors appear on the Home to-do lists.</p></div></header>
      {#each people as person (person.id)}<MemberSettings {person} connected={$connection.isActive} />{/each}
    </section>
  </div>
{/if}
