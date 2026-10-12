# Keep the runtime aligned with the Rust module and browser SDK.
FROM clockworklabs/spacetime:v2.8.2 AS spacetime

FROM node:22-bookworm-slim AS frontend
WORKDIR /build
COPY package.json package-lock.json ./
RUN npm ci
COPY index.html svelte.config.js tsconfig.json vite.config.ts ./
COPY src ./src
COPY scripts ./scripts
RUN npm run build

FROM rust:bookworm AS backend
WORKDIR /build
RUN rustup target add wasm32-unknown-unknown
COPY spacetimedb ./spacetimedb
COPY --from=frontend /build/dist ./dist
RUN cargo build --locked --release --target wasm32-unknown-unknown --manifest-path spacetimedb/Cargo.toml

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl openssl jq libstdc++6 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 1000 --create-home spacetime \
    && mkdir /data && chown spacetime:spacetime /data
COPY --from=spacetime /opt/spacetime/spacetimedb-standalone /opt/spacetime/spacetimedb-cli /opt/spacetime/
COPY --from=backend /build/spacetimedb/target/wasm32-unknown-unknown/release/family_on_the_run.wasm /app/family_on_the_run.wasm
COPY --chmod=755 deploy/start.sh /app/start.sh
USER spacetime
WORKDIR /app
EXPOSE 3000
VOLUME ["/data"]
HEALTHCHECK --interval=30s --timeout=5s --start-period=120s --retries=3 \
    CMD test -f /tmp/family-ready && curl --fail --silent --output /dev/null http://127.0.0.1:3000/v1/database/family-on-the-run/route/
ENTRYPOINT ["/app/start.sh"]
