# Family on the Run

A family dashboard for a 2560×1440 touchscreen. Svelte provides the browser UI; one SpacetimeDB server stores the household data, runs calendar imports on its scheduler, and serves the compiled frontend through HTTP handlers.

## Working features

- Home has five columns: Daily Calendar, Lindsay To Do, Matt To Do, Carter To Do, Sydney To Do.
- Left navigation includes Home, Weekly Calendar, Monthly Calendar, Food, House, and Admin.
- Tasks support creation, renaming, completion, reopening, deletion, and showing completed tasks.
- Desktop text fields automatically open a touch keyboard with Shift, symbols, cursor editing, and Enter submission. Tap Done to dismiss it; screens narrower than 900px use the native keyboard.
- Daily, weekly, and monthly calendars combine editable dashboard events with read-only Google feed events.
- Admin configures private Google iCal feeds, requests refreshes, shows refresh status, and edits family member names and colors.
- House has static starter routines; Food remains a placeholder.
- Household pairing and separate Admin access persist per browser identity. Changes synchronize across screens.

## Build and start

Build prerequisites: Node.js 22.12 or newer, Rust with the `wasm32-unknown-unknown` target, and SpacetimeDB CLI **2.8.2**. The Rust crate and browser SDK are pinned to that version. Rust procedures and HTTP handlers require SpacetimeDB's `unstable` feature; keep versions aligned.

```sh
npm ci
rustup target add wasm32-unknown-unknown
npm run build
```

Start the persistent local database server in one terminal:

```sh
npm run backend:start
```

In a second terminal:

```sh
npm run backend:publish
```

Publishing builds and embeds the frontend, publishes the database module, initializes schedules for existing feeds, and regenerates client bindings. It preserves household tables and allows existing clients to reconnect after schema changes. It does not reset the database. Newly created household/Admin codes are stored in `.spacetime/access-code` and `.spacetime/admin-code`.

Open **http://127.0.0.1:3000/v1/database/family-on-the-run/route/**, including the trailing slash. This is served by SpacetimeDB itself. Node, Vite, and a calendar worker are not needed at runtime. The database name is inferred from the app's URL, so the same compiled app works in a differently named database.

Keep the ignored `.spacetime/` directory private: it contains database data, owner CLI credentials, access codes, and test artifacts. Back up its persistent data. Browsers store their identity tokens in local storage; a new origin or cleared storage requires pairing again.

For frontend development, `npm run dev` still starts Vite at port 5173 with a `/v1` proxy to SpacetimeDB. After changing backend definitions, run `npm run backend:generate`, then publish again to include the updated browser bindings and UI.

## Deployment

Build one database-module artifact containing the backend, ICS parser, and static Svelte assets:

```sh
npm run backend:build
```

Artifact: `spacetimedb/target/wasm32-unknown-unknown/release/family_on_the_run.wasm`.

Publish the prebuilt module to your server without installing Node or Rust there:

```sh
spacetime publish --server YOUR_SERVER --bin-path family_on_the_run.wasm YOUR_DATABASE
spacetime call --server YOUR_SERVER YOUR_DATABASE initialize_calendar_schedules
```

For a new database, its owner also needs to call `configure_access` and `configure_admin_access` with separate codes of at least 24 characters. Do not pass real codes in public logs. The local publish script performs this initial setup automatically.

The installed server executable can be run directly (copy the platform's `spacetimedb-standalone` binary to your deployment):

```sh
./spacetimedb-standalone start --listen-addr 127.0.0.1:3000 --data-dir ./data \
  --jwt-priv-key-path ./data/identity.private.pem \
  --jwt-pub-key-path ./data/identity.public.pem --non-interactive
```

Keep the server's JWT signing key pair with its private deployment data. The CLI generates these keys when initializing a local server; pass their paths explicitly when running the standalone executable. Preserve the same keys across restarts so existing identity tokens remain valid.

**Production runs one SpacetimeDB standalone server executable/process**, with its persistent database directory. The Wasm module is loaded into that database once and contains the frontend assets. This is a single server deployment, with one module artifact for installation/updates; it does not fuse the application into the SpacetimeDB executable itself. No separate web server, scheduler service, Node process, Grocy, or Home Assistant is required. Terminating the server stops both the app and its schedules. Use HTTPS/WSS for hosted access.

SpacetimeDB exposes database HTTP handlers under `/v1/database/YOUR_DATABASE/route/`; the dashboard lives at that path rather than the server's `/` root. See [HTTP handler documentation](https://spacetimedb.com/docs/functions/http-handlers/).

## Google Calendar

Unlock Admin with the separate Admin code. In Google Calendar on a computer, open Settings → choose a calendar → Integrate calendar → **Secret address in iCal format**. Add that link, a display name, and a color. Repeat for each calendar. No OAuth app, Google Cloud project, or API key is required. Work/school administrators may restrict access to secret feeds. [Google's instructions](https://support.google.com/calendar/answer/37648)

Adding a feed schedules an immediate import plus a repeating five-minute database job. **Refresh now** schedules an immediate import through the same procedure. Removing a feed removes its schedules and imported events. Schedules persist with the database; no external worker or worker identity is used. Interval ticks missed while the server is offline resume according to SpacetimeDB's scheduler. [Schedule documentation](https://spacetimedb.com/docs/tables/schedule-tables/)

The scheduled procedure reads its feed in a transaction, downloads and parses outside the transaction, and atomically replaces that feed's events and status in a new transaction. Failed downloads/parses preserve the previous events and successful-refresh timestamp. Superseded fetches cannot overwrite a newer refresh. Errors shown in Admin do not contain the secret URL.

Imports cover only events that have not started yet, up to 365 days ahead, with recurrence rules, additional/exception dates, moved/cancelled occurrences, IANA timezones, daylight-saving changes, and multi-day all-day events. Custom non-IANA timezone definitions and `RECURRENCE-ID;RANGE=THISANDFUTURE` are rejected with an error rather than guessed. Floating times use the calendar's `X-WR-TIMEZONE`, or UTC when absent. Google feed changes may be delayed; Refresh now fetches what Google currently serves. Imported events remain read-only; edit them in Google Calendar.

HTTP requests have a 20-second timeout. The importer accepts at most 5 MB of ICS data and 10,000 expanded events, and recurrence expansion uses the library's iteration limits. SpacetimeDB's HTTP API materializes the response before the module checks its size; this is not a streaming download limit. Only HTTPS Google Calendar `basic.ics` source URLs are accepted.

Private feed URLs remain in private backend tables and are not returned to household or Admin browsers after saving. Treat them as passwords. To replace a link, remove its feed and add the replacement. Database owners and people with access to the stored database can read URLs; storage is not application-encrypted. Previously saved OAuth settings remain private for compatibility and are not used by this integration.

## Verification

With the server running and the module published:

```sh
npm run build
cargo test --manifest-path spacetimedb/Cargo.toml
npm run test:integration
npm run test:admin
npm run test:admin:browser
npm run test:browser
```

Rust tests cover ICS recurrence exceptions, moved occurrences, cancellations, daylight-saving behavior, and all-day dates. Integration checks cover permissions, pairing, live tasks, reconnect persistence, events, and access revocation. Admin tests create a separate temporary database, exercise real Google feed downloads through SpacetimeDB's scheduler, verify import/status/manual refresh/removal, and check member settings and OAuth compatibility. Calendar integration tests need internet access to Google's public US holiday feed. They do not change the household's calendar settings.

Browser tests run against the app served by SpacetimeDB, including Admin forms, imported events in all three calendar views, read-only details, touchscreen layout at 2560×1440, and mobile overflow. Screenshots are saved under `.spacetime/screenshots/`. Install Playwright Chromium with `npx playwright install chromium`, or set `PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH` to an existing executable.

## Code layout

- `src/App.svelte` and `src/components/`: household dashboard and Admin UI.
- `src/lib/`: date helpers and calendar display models.
- `src/module_bindings/`: generated SpacetimeDB client; regenerate rather than edit.
- `spacetimedb/src/schema.rs`, `lib.rs`, `admin.rs`: private household data, permissions, and reducers/views.
- `spacetimedb/src/calendar_feeds.rs`: feed settings, imported events, and sanitized views.
- `spacetimedb/src/calendar_sync.rs`: built-in schedules, HTTP fetching, and atomic import transactions.
- `spacetimedb/src/calendar_parser.rs`: ICS parsing and recurrence integration.
- `spacetimedb/src/static_site.rs` and `build.rs`: HTTP serving and embedded frontend assets.
- `spacetimedb/vendor/rrule/`: upstream rrule 0.14.0 with a dependency-feature patch to disable browser-only Wasm imports; see `PATCH.md` and its licenses.
- `scripts/`: build/publish commands and verification.
- `reports/` and `research_notes/`: earlier technology research.

One database currently represents one household. Named user accounts, invitations, and offline editing are not implemented. License: MIT; vendored dependencies retain their own licenses.
