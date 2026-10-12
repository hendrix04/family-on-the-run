#!/bin/sh
set -eu
umask 077

# All durable state, including the publisher identity, lives on one volume.
mkdir -p /data/database
rm -f /tmp/family-ready
if [ ! -f /data/identity.private.pem ]; then
    if [ -f /data/identity.public.pem ] || [ -f /data/configured ]; then
        echo 'Missing signing key in an existing deployment; restore it from backup.' >&2
        exit 1
    fi
    openssl genpkey -algorithm EC -pkeyopt ec_paramgen_curve:prime256v1 -out /data/identity.private.pem
fi
if [ ! -f /data/identity.public.pem ]; then
    openssl pkey -in /data/identity.private.pem -pubout -out /data/identity.public.pem
fi

/opt/spacetime/spacetimedb-standalone start --listen-addr 0.0.0.0:3000 \
    --data-dir /data/database --jwt-priv-key-path /data/identity.private.pem \
    --jwt-pub-key-path /data/identity.public.pem --non-interactive &
server_pid=$!
trap 'kill -TERM "$server_pid" 2>/dev/null || true; wait "$server_pid" 2>/dev/null || true' EXIT
trap 'exit 143' TERM
trap 'exit 130' INT

attempt=0
until curl --fail --silent --output /dev/null http://127.0.0.1:3000/v1/ping; do
    kill -0 "$server_pid"
    attempt=$((attempt + 1))
    if [ "$attempt" -ge 60 ]; then
        echo 'SpacetimeDB did not become ready within 60 seconds.' >&2
        exit 1
    fi
    sleep 1
done

cli() {
    /opt/spacetime/spacetimedb-cli --config-path /data/cli.toml "$@"
}
# Never delete application data to make an incompatible update succeed.
cli publish --server http://127.0.0.1:3000 --bin-path /app/family_on_the_run.wasm \
    --no-config --yes=skip-login --break-clients --delete-data=never family-on-the-run

if [ ! -f /data/configured ]; then
    for code in access-code admin-code; do
        if [ ! -s "/data/$code" ]; then openssl rand -hex 24 > "/data/$code"; fi
    done
    # Suppress CLI errors here so supplied access codes cannot enter deployment logs.
    if ! cli call --server http://127.0.0.1:3000 --no-config family-on-the-run configure_access \
        "$(jq -Rs 'rtrimstr("\n")' /data/access-code)" >/dev/null 2>&1; then
        echo 'Failed to configure household access.' >&2
        exit 1
    fi
    if ! cli call --server http://127.0.0.1:3000 --no-config family-on-the-run configure_admin_access \
        "$(jq -Rs 'rtrimstr("\n")' /data/admin-code)" >/dev/null 2>&1; then
        echo 'Failed to configure admin access.' >&2
        exit 1
    fi
    touch /data/configured
    echo 'Access codes generated. Read /data/access-code and /data/admin-code in the container terminal.'
fi
cli call --server http://127.0.0.1:3000 --no-config family-on-the-run initialize_calendar_schedules
touch /tmp/family-ready
wait "$server_pid"
