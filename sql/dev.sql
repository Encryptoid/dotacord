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

select * from server_events;