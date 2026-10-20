# apps/store

The store service: conversations, messages, memories, the profile graph, and pending confirmations, held in Postgres and served over HTTP to the engine. It is the only process that opens a database. It never links the engine crate.

```bash
just postgres                       # Postgres on localhost:5432; needs BULLPEN_POSTGRES_PASSWORD in .env
just store                          # needs BULLPEN_STORE__TOKEN and BULLPEN_STORE__DATABASE_URL in .env
cargo test -p store                 # database tests run when BULLPEN_STORE_TEST_DATABASE_URL is set
```

## What it does

- Applies the migrations under `migrations/` at boot, then listens on `store.bind`.
- Requires `Authorization: Bearer <BULLPEN_STORE__TOKEN>` on every route except `GET /health`, which answers 200 once Postgres does.
- Scopes every route and every query by the tenant and user in the path: `/api/v1/tenants/{tenant}/users/{user}/...`.
- Rejects a limit, ttl, or key outside the bounds in the `[store]` section of `bullpen.toml` with 400. A database failure is a 500 that never carries its cause.

The routes are listed under Stores in [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md).

## Layout

```
src/core/       config, telemetry, types, tests. Imports nothing else in the crate.
src/db/         the pool and migrations; one file of queries per store
src/routes/     health, the bearer check, the scope extractor; one file of handlers per store
migrations/     sqlx migrations, embedded at build time
```
