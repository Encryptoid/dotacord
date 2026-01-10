# Dotacord Agent Guidelines

## Configuration

Depending on the build configuration(debug/release), `build.rs` copies either `dotacord.debug.toml` or `dotacord.release.toml`
to the target directory as `dotacord.toml`. This file is required at runtime and contains configuration settings for the bot.

## Emojis

When implementing visual elements such as emojis in the bot's responses, there is a set of predefined `dota` related emojis. Ensure to use these emojis consistently across the bot's features.
Path: `./src/leaderboard/emoji.rs`
