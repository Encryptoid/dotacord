---
intercept: true
---

Build and run the dotacord Discord bot.

1. Run `cargo build` and report any compile errors.
2. If the build succeeds, run `cargo run` from the project root.
3. Confirm the bot has started and is connected to Discord.

Note: `dotacord.toml` must exist in the project root and the required env var (set via `api_key_var`) must be present. The database at `data/dotacord.db` and `data/heroes.json` must also exist.
