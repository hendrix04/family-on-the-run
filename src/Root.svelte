<script lang="ts">
  import { createSpacetimeDBProvider } from 'spacetimedb/svelte';
  import { DbConnection } from './module_bindings';
  import App from './App.svelte';
  import OnScreenKeyboard from './components/OnScreenKeyboard.svelte';

  const host = import.meta.env.VITE_SPACETIMEDB_HOST ?? window.location.origin;
  const routeDatabase = window.location.pathname.match(/^\/v1\/database\/([^/]+)\/route\//)?.[1];
  const database = routeDatabase ? decodeURIComponent(routeDatabase) : import.meta.env.VITE_SPACETIMEDB_DB_NAME ?? 'family-on-the-run';
  const tokenKey = `${host}/${database}/auth-token`;

  createSpacetimeDBProvider(
    DbConnection.builder()
      .withUri(host)
      .withDatabaseName(database)
      .withToken(localStorage.getItem(tokenKey) ?? undefined)
      .onConnect((_connection, _identity, token) => localStorage.setItem(tokenKey, token))
  );
</script>

<App />
<OnScreenKeyboard />
