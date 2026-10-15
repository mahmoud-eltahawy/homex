CREATE TABLE files (
    id              BLOB PRIMARY KEY NOT NULL,
    relative_path   TEXT NOT NULL UNIQUE,
    size_bytes      INTEGER NOT NULL,
    duration_secs   INTEGER NOT NULL
);

CREATE TABLE sections (
    id              BLOB PRIMARY KEY NOT NULL,
    slug            TEXT NOT NULL UNIQUE,
    title           TEXT NOT NULL,
    media_kind      TEXT NOT NULL,
    nested          INTEGER NOT NULL,
    position        INTEGER NOT NULL
);

CREATE TABLE video_collections (
    id              BLOB PRIMARY KEY NOT NULL,
    section_id      BLOB NOT NULL,
    section_slug    TEXT NOT NULL,
    title           TEXT NOT NULL,
    poster          TEXT NOT NULL,
    description     TEXT,
    position        INTEGER NOT NULL,
    items_count     INTEGER NOT NULL
);
CREATE INDEX idx_video_collections_section_id   ON video_collections(section_id);
CREATE INDEX idx_video_collections_section_slug ON video_collections(section_slug);

CREATE TABLE audio_collections (
    id              BLOB PRIMARY KEY NOT NULL,
    section_id      BLOB NOT NULL,
    section_slug    TEXT NOT NULL,
    title           TEXT NOT NULL,
    poster          TEXT,
    description     TEXT,
    position        INTEGER NOT NULL,
    items_count     INTEGER NOT NULL
);
CREATE INDEX idx_audio_collections_section_id   ON audio_collections(section_id);
CREATE INDEX idx_audio_collections_section_slug ON audio_collections(section_slug);

CREATE TABLE items (
    id              BLOB PRIMARY KEY NOT NULL,
    collection_id   BLOB NOT NULL,
    number          INTEGER NOT NULL,
    season_number   INTEGER,
    title           TEXT,
    poster          TEXT,
    description     TEXT,
    file_id         BLOB NOT NULL
);
CREATE INDEX idx_items_collection_id ON items(collection_id);
CREATE INDEX idx_items_file_id       ON items(file_id);

CREATE TABLE tags (
    id              BLOB PRIMARY KEY NOT NULL,
    name            TEXT NOT NULL UNIQUE
);

CREATE TABLE section_tags (
    id              BLOB PRIMARY KEY NOT NULL,
    section_id      BLOB NOT NULL,
    tag_id          BLOB NOT NULL
);
CREATE INDEX idx_section_tags_section_id ON section_tags(section_id);
CREATE INDEX idx_section_tags_tag_id     ON section_tags(tag_id);

CREATE TABLE collection_tags (
    id              BLOB PRIMARY KEY NOT NULL,
    collection_id   BLOB NOT NULL,
    tag_id          BLOB NOT NULL
);
CREATE INDEX idx_collection_tags_collection_id ON collection_tags(collection_id);
CREATE INDEX idx_collection_tags_tag_id        ON collection_tags(tag_id);
