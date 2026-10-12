Vendored rrule 0.14.0 from crates.io (MIT OR Apache-2.0).

Only change: its Chrono dependency disables default features and enables `std` and `clock` explicitly. Chrono's default `wasmbind` feature imports browser APIs that SpacetimeDB's non-browser Wasm runtime rejects. Recurrence implementation is unchanged. We use explicit IANA/UTC zones and pass the database timestamp; local-system-time operations are not used.

Upstream: https://github.com/fmeringdal/rust-rrule
