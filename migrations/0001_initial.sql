-- migrations/0001_initial.sql

CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    relative_path TEXT NOT NULL UNIQUE,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    duration_secs INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE sections (
    id INTEGER PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    media_kind TEXT NOT NULL CHECK (media_kind IN ('video', 'audio')),
    -- 0 = flat list of items, 1 = list of groups of items
    nested INTEGER NOT NULL DEFAULT 0 CHECK (nested IN (0, 1)),
    position INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE collections (
    id INTEGER PRIMARY KEY,
    section_id INTEGER NOT NULL REFERENCES sections(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    poster TEXT,
    description TEXT,
    position INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_collections_section ON collections(section_id);

CREATE TABLE items (
    id INTEGER PRIMARY KEY,
    collection_id INTEGER NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    number INTEGER NOT NULL DEFAULT 0,
    -- NULL for flat sections; a season number for nested video sections
    season_number INTEGER,
    title TEXT,
    poster TEXT,
    description TEXT,
    file_id INTEGER NOT NULL REFERENCES files(id)
);
CREATE INDEX idx_items_collection ON items(collection_id);

INSERT INTO sections (slug, title, media_kind, nested, position) VALUES
    ('movies', 'أفلام', 'video', 0, 0),
    ('series', 'مسلسلات', 'video', 1, 1),
    ('audio',  'صوتيات', 'audio', 1, 2);
