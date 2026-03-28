CREATE TABLE IF NOT EXISTS heroes (
    hero_id     INTEGER PRIMARY KEY NOT NULL,
    name        TEXT    NOT NULL,
    is_carry    INTEGER NOT NULL DEFAULT 0,
    is_mid      INTEGER NOT NULL DEFAULT 0,
    is_offlane  INTEGER NOT NULL DEFAULT 0,
    is_support  INTEGER NOT NULL DEFAULT 0
);
