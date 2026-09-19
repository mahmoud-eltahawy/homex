
CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    -- relative to media_root, OR an absolute http(s) URL
    relative_path TEXT NOT NULL UNIQUE,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    duration_secs INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE movies (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    poster TEXT,
    description TEXT
);

CREATE TABLE movie_chapters (
    id INTEGER PRIMARY KEY,
    movie_id INTEGER NOT NULL REFERENCES movies(id) ON DELETE CASCADE,
    number INTEGER NOT NULL,
    title TEXT,
    poster TEXT,
    description TEXT,
    file_id INTEGER NOT NULL REFERENCES files(id),
    UNIQUE(movie_id, number)
);
CREATE INDEX idx_movie_chapters_movie ON movie_chapters(movie_id);

CREATE TABLE series (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    poster TEXT,
    description TEXT
);

CREATE TABLE seasons (
    id INTEGER PRIMARY KEY,
    series_id INTEGER NOT NULL REFERENCES series(id) ON DELETE CASCADE,
    number INTEGER NOT NULL,
    UNIQUE(series_id, number)
);
CREATE INDEX idx_seasons_series ON seasons(series_id);

CREATE TABLE episodes (
    id INTEGER PRIMARY KEY,
    season_id INTEGER NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    number INTEGER NOT NULL,
    title TEXT,
    file_id INTEGER NOT NULL REFERENCES files(id),
    UNIQUE(season_id, number)
);
CREATE INDEX idx_episodes_season ON episodes(season_id);

CREATE TABLE audio_groups (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    poster TEXT,
    description TEXT
);

CREATE TABLE audios (
    id INTEGER PRIMARY KEY,
    group_id INTEGER NOT NULL REFERENCES audio_groups(id) ON DELETE CASCADE,
    number INTEGER NOT NULL,
    title TEXT NOT NULL,
    file_id INTEGER NOT NULL REFERENCES files(id),
    UNIQUE(group_id, number)
);
CREATE INDEX idx_audios_group ON audios(group_id);
