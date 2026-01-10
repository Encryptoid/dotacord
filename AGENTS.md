# Dotacord Agent Guidelines

## Configuration

Depending on the build configuration(debug/release), `build.rs` copies either `dotacord.debug.toml` or `dotacord.release.toml`
to the target directory as `dotacord.toml`. This file is required at runtime and contains configuration settings for the bot.

