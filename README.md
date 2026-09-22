## Schema

The schema is derived from the `#[derive(toasty::Model)]` structs in
`src/app/model.rs`. On startup the server calls `push_schema` on a fresh
DB; if the DB already has tables it leaves them alone.

After changing a model, reset the dev DB:

    HOMEX_RESET_SCHEMA=1 ./run

That deletes `Downloads/homex.db` before boot, then recreates it from the
current model.
