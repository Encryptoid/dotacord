CREATE TABLE IF NOT EXISTS hero_nicknames (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    hero_id     INTEGER NOT NULL,
    nickname    TEXT    NOT NULL COLLATE NOCASE,
    FOREIGN KEY (hero_id) REFERENCES heroes(hero_id),
    UNIQUE(hero_id, nickname)
);
