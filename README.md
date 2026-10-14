## Schema

The schema is derived from the `#[derive(toasty::Model)]` structs in
`src/app/model.rs`. Video and audio collections are stored in separate
tables (`video_collections` and `audio_collections`) because they diverge
in how their posters are produced.

On startup the server calls `push_schema` on a fresh DB; if the DB already
has tables it leaves them alone.

After changing a model, reset the dev DB:

    HOMEX_RESET_SCHEMA=1 ./run

That deletes `Downloads/homex.db` before boot, then recreates it from the
current model.

## Dependencies

HomeX requires `ffmpeg` and `ffprobe` on `PATH` for video poster extraction
and for transcoding non-MP4/WebM/M4V containers into a supported format.
Install via your package manager:

- Debian/Ubuntu: `apt install ffmpeg`
- Fedora: `dnf install ffmpeg`
- macOS: `brew install ffmpeg`

Verify with `ffmpeg -version`. The server will refuse video uploads with a
clear error if either binary is missing.
