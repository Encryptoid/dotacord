# Schedule Interval Database Task

We will need a new table in the database.

```sql
CREATE TABLE schedule_events (
    event_id BIGINT PRIMARY KEY,
    server_id BIGINT NOT NULL,
    event_type TEXT NOT NULL, -- Reload
    event_source TEXT NOT NULL, -- Manual or Schedule
    event_time TIMESTAMP NOT NULL
);
```

So we check for the current event type, what is the last event_time for that server_id and event_type. If the last event_time + interval < now, we perform the task and insert a new row into this table with the current timestamp.

Instead of spawning a timer for each task, we will have a single timer that checks every timer_check_mins (configurable) and iterates through all subscribed servers and checks each task. This change makes more sense for the new setup. 

So the statup steps will be:

1. Start a timer that ticks every timer_check_mins
2. On each tick, perform each check(reload/leaderboard month/week) synchronously

Steps for each task:

1. First send an info heartbeat log that we are checking the task for subscribed servers, so heartbeating will be tied to `timer_check_mins`. This will replace the heartbeat task.
1. If leaderboard, get subscribed servers that have a subscription set
3. A check will be accessing the database to see the last published/reloaded time for the server
4. If the last time + interval < now, perform the task and update the last time in database


## Reload Task

The reload task will check for the last reload event for the server. If the last reload event + auto_reload_interval_minutes < now, we perform the reload task and insert a new row into the schedule_events table.
If there is no existing row, we can assume the server has never been reloaded and perform the reload task immediately.
