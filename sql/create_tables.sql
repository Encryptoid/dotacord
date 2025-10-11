PRAGMA foreign_keys = ON;

---
--- PLAYERS
---

DROP TABLE IF EXISTS PLAYERS;
CREATE TABLE IF NOT EXISTS players
(
    player_id INT PRIMARY KEY NOT NULL
);

---
--- SERVERS
---

DROP TABLE IF EXISTS SERVERS;
CREATE TABLE IF NOT EXISTS servers
(
    server_id   BIGINT PRIMARY KEY NOT NULL,
    server_name TEXT               NOT NULL,
    channel_id  BIGINT             NULL
);

---
--- PLAYER_SERVERS
---

DROP TABLE IF EXISTS player_servers;
CREATE TABLE IF NOT EXISTS player_servers
(
    player_id    INT    NOT NULL,
    server_id    BIGINT NOT NULL,
    user_id      BIGINT NOT NULL,
    discord_name TEXT   NOT NULL,
    player_name  TEXT       NULL,

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



-- delete from player_matches where match_id = 8495446507;
-- select * from servers;
select * from player_servers;


select * from player_matches
order by match_id asc;

INSERT INTO player_matches (
    match_id,
    player_id,
    hero_id,
    kills,
    deaths,
    assists,
    rank,
    party_size,
    faction,
    is_victory,
    start_time,
    duration,
    game_mode,
    lobby_type
)
VALUES (1439386853, 138643094, 80 /* Lone Druid */ , 9, 15, 12, 
0, 0, 0, 1, 1430526282 /* May 02, 2015 */, 5567, 22, 0)