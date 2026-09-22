## Updating the SQLx offline cache

After changing any `sqlx::query!` / `query_scalar!` in `src/app/server/db.rs`,
run with a live dev DB:

    export DATABASE_URL="sqlite://$(pwd)/Downloads/homex.db"
    cargo sqlx prepare --workspace -- --all-features
    git add .sqlx && git commit -m "refresh sqlx offline cache"

CI builds with `SQLX_OFFLINE=true`, so forgetting this step will fail the
build, not ship a broken release.
