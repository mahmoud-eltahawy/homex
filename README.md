## Schema

The schema is defined in `migrations/`. On startup the server runs
`sqlx::migrate!()`, which applies any pending migrations.

After changing a model or a migration, either drop the dev DB
(`rm Downloads/homex.db`) or add a new migration file. Never edit a
migration that has already been applied to a shared DB.

## Compile-time SQL

`sqlx::query_as!` and `sqlx::query!` verify every statement against a live
schema at compile time. Two ways to build:

### With a live schema DB

    $env.DATABASE_URL = "sqlite:///tmp/homex-schema.db"
    cargo leptos watch --split --hot-reload

### With the offline cache (recommended, no DB needed)

    $env.SQLX_OFFLINE = "true"
    cargo leptos watch --split --hot-reload

The offline cache lives in `.sqlx/` and is committed. Regenerate it after
any change to a query or a migration:

    $env.DATABASE_URL = "sqlite:///tmp/homex-schema.db"
    sqlx migrate run
    cargo sqlx prepare -- --features ssr
    git add .sqlx

## Dependencies

HomeX requires `ffmpeg` and `ffprobe` on `PATH` for video poster extraction
and for transcoding non-MP4/WebM/M4V containers into a supported format.
Install via your package manager:

- Debian/Ubuntu: `apt install ffmpeg`
- Fedora: `dnf install ffmpeg`
- macOS: `brew install ffmpeg`

Verify with `ffmpeg -version`. The server will refuse video uploads with a
clear error if either binary is missing.
