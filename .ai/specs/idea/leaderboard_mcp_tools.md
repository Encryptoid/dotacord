Check out this request that was made to Dotacord:

```
@Dotacord what is the average time of the games ive played
```

Dotacord didn't really have the tools exposed to it to be able to answer the question.

The leaderboard functionality did have access to this data though, so we could expose it.

My initial idea is something like:

query_leaderboard(section, duration)

The sections from the leaderboard command could be grouped like so:

Section:
- Winrates - Overall + Ranked Winrate
- DurationSpam - Hero spammer stats + match duration stats
- SingleMatch - Most Kills/Assists/Deaths in a single match

And Duration would match the existing:

Duration:
- Day
- Week
- Month
- All Time

Consider this approach.

Open Questions:

- Should the response be structured json? Or maybe just the markdown directly?
- Is the groping for Section good?
- Are parameters correct? Should there be multiple tools? Is one suffi
