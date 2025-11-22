PRAGMA foreign_keys = ON;

---
--- PLAYERS
---

DROP TABLE IF EXISTS players;
CREATE TABLE IF NOT EXISTS players
(
    player_id INT PRIMARY KEY NOT NULL
);

---
--- SERVERS
---

DROP TABLE IF EXISTS servers;
CREATE TABLE IF NOT EXISTS servers
(
    server_id     BIGINT PRIMARY KEY NOT NULL,
    server_name   TEXT               NOT NULL,
    channel_id    BIGINT             NULL,
    is_sub_week   INTEGER            NOT NULL DEFAULT 0,
    is_sub_month  INTEGER            NOT NULL DEFAULT 0,
    is_sub_reload INTEGER            NOT NULL DEFAULT 0
);

---
--- PLAYER_SERVERS
---

DROP TABLE IF EXISTS player_servers;
CREATE TABLE IF NOT EXISTS player_servers
(
    player_id    INT    NOT NULL,
    server_id    BIGINT NOT NULL,
    player_name  TEXT   NOT NULL,

    FOREIGN KEY (player_id) REFERENCES players (player_id),
    FOREIGN KEY (server_id) REFERENCES servers (server_id)
);

---
--- PLAYER_MATCHES
---

DROP TABLE IF EXISTS player_matches;
CREATE TABLE IF NOT EXISTS player_matches
(
    match_id    INTEGER NOT NULL,
    player_id   INTEGER NOT NULL,
    hero_id     INTEGER NOT NULL,
    kills       INTEGER NOT NULL,
    deaths      INTEGER NOT NULL,
    assists     INTEGER NOT NULL,
    rank        INTEGER NOT NULL,
    party_size  INTEGER NOT NULL,
    faction     INTEGER NOT NULL,
    is_victory  INTEGER NOT NULL,
    start_time  INTEGER NOT NULL,
    duration    INTEGER NOT NULL,
    game_mode   INTEGER NOT NULL,
    lobby_type  INTEGER NOT NULL,

    FOREIGN KEY (player_id) REFERENCES players (player_id)
);

---
--- SCHEDULE_EVENTS
---

DROP TABLE IF EXISTS schedule_events;
CREATE TABLE IF NOT EXISTS schedule_events
(
    event_id     INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    server_id    BIGINT             NOT NULL,
    event_type   TEXT               NOT NULL, -- LeaderboardWeek, LeaderboardMonth, or Reload
    event_source TEXT               NOT NULL, -- Manual or Schedule
    event_time   INTEGER            NOT NULL,

    FOREIGN KEY (server_id) REFERENCES servers (server_id)
);
