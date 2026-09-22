## Updating the SQLx offline cache

After changing any `sqlx::query!` / `query_scalar!` in `src/app/server/db.rs`,
run with a live dev DB:

    export DATABASE_URL="sqlite://$(pwd)/Downloads/homex.db"
    cargo sqlx prepare --workspace -- --all-features
    git add .sqlx && git commit -m "refresh sqlx offline cache"

CI builds with `SQLX_OFFLINE=true`, so forgetting this step will fail the
build, not ship a broken release.


## Config

`homex.toml` ships production-shaped defaults (`/srv/homex/media`,
`/var/lib/homex`). Local development uses `homex.dev.toml`, which `run`
selects via `HOMEX_CONFIG`. To use a different file:

    HOMEX_CONFIG=/path/to/your.toml cargo leptos watch --split --hot-reload
