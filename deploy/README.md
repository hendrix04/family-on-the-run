# Deploy on Coolify

The root Dockerfile builds the Svelte frontend and embeds it in the Rust database module. The final image runs SpacetimeDB 2.8.2, with no Node, Rust toolchain, web server, or calendar worker at runtime. A small startup script starts the server, publishes the bundled module, initializes calendar schedules, then waits for the server and forwards shutdown signals.

## Coolify settings

1. Create an application from `hendrix04/family-on-the-run`, branch `main`.
2. Choose **Dockerfile** as the build pack. Use the repository root as Base Directory and the root `Dockerfile` as Dockerfile Location.
3. Set **Ports Exposes** to `3000`. Leave host port mappings empty; Coolify's proxy handles public HTTPS and WebSocket traffic.
4. Add your HTTPS domain, for example `https://family.example.com`, and point its DNS at the Coolify server.
5. Before the first deploy, add a **Volume Mount** with Destination Path **`/data`**. Keep this same volume for subsequent deployments. The container runs as UID/GID `1000`; a manually configured bind mount must be writable by that user.
6. Under container naming, choose **Consistent name (no rolling updates)**, or set a unique Custom Container Name such as `family-on-the-run`. Run only one instance. This database uses one local data directory, so the old server must stop before the replacement starts. Updates have a brief interruption.
7. Leave the image's built-in health check enabled. It checks the published dashboard only after startup initialization completes. Use a Stop Grace Period of at least 30 seconds.
8. Deploy. No runtime environment variables or separate scheduled tasks are required.

Open the dashboard at:

```text
https://family.example.com/v1/database/family-on-the-run/route/
```

Keep the trailing slash. The domain's `/` path is not the dashboard. The frontend uses the same domain for its HTTP and WebSocket connection.

In Coolify's application **Terminal**, read the generated codes:

```sh
cat /data/access-code
cat /data/admin-code
```

Use the first code to pair each browser; use the second to unlock Admin. Then add your private Google iCal feeds. Codes are generated on the first boot and retained across updates. They are not printed in deployment logs. No Google OAuth registration is needed.

## What persists

The `/data` volume contains:

- `database/`: household records, tasks, events, calendar links, and schedules.
- `identity.private.pem` and `identity.public.pem`: server signing keys.
- `cli.toml`: database-owner credentials used to publish updates.
- `access-code`, `admin-code`, and `configured`: initial access codes and setup marker.

Back up the entire volume while the application is stopped for a consistent filesystem copy. Keep backups private and outside the deployment server. Do not delete or replace the volume to deploy an update. The startup script never requests data deletion; incompatible schema updates fail instead.

This creates a separate household from the local development server. Local data and existing private calendar links are deliberately excluded from the image. Reconfigure feeds on the hosted instance, or migrate a stopped local deployment's database, signing keys, publisher credentials, and access files together before its first boot. Do not copy a live database directory.

## Local container check

```sh
docker build -t family-on-the-run .
docker volume create family-data
docker run --name family-dashboard --detach \
  --publish 127.0.0.1:3001:3000 --mount source=family-data,target=/data \
  family-on-the-run
docker logs family-dashboard
docker inspect --format '{{.State.Health.Status}}' family-dashboard
```

Open `http://127.0.0.1:3001/v1/database/family-on-the-run/route/`. To check persistence, stop and remove the container, then repeat `docker run` with the same volume. Do not remove the volume.

Run the automated boot and persistence check with `npm run test:container` after building the image as `family-on-the-run:coolify`. It creates and removes an isolated test container and volume.

## Troubleshooting

- **Build fails:** inspect Deployment Logs. The first build downloads the build images and compiles Rust dependencies; later builds can reuse Docker's cache.
- **Permission denied under `/data`:** check ownership of the mounted storage; the runtime user is UID/GID `1000`.
- **Database already locked:** another server still has the same volume open. Use consistent container naming and one instance.
- **Publish unauthorized after moving data:** the original owner CLI credentials or server signing keys were not preserved.
- **Homepage 404:** use the full `/v1/database/family-on-the-run/route/` URL.
- **Dashboard loads but cannot connect:** ensure the proxy passes `/v1` requests and WebSocket upgrades to the same container.

References: [Coolify Dockerfile deployments](https://coolify.io/docs/applications/builds/dockerfile), [persistent storage](https://coolify.io/docs/core/persistent-storage/storage-mounts/overview), [health checks](https://coolify.io/docs/applications/configuration/health-checks), [rolling update controls](https://coolify.io/docs/applications/deployments/rolling-updates), and [SpacetimeDB signing keys](https://spacetimedb.com/docs/how-to/self-hosted-key-rotation/).
