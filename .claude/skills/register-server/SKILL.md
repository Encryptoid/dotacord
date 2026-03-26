---
name: register-server
description: Register a Discord server in the dotacord database on the Hetzner VPS (178.104.97.192).
---

# Register Server

Registers a Discord server on the production dotacord instance running on the Hetzner VPS.

## Process

1. Ask the user for:
   - **Server ID** (Discord snowflake, 17-20 digits)
   - **Server name**
2. Validate the server ID is numeric and 17-20 characters long. If not, tell the user and stop.
3. Validate the server name is not blank. If blank, tell the user and stop.
4. Run:
   ```
   ssh root@178.104.97.192 '/opt/dotacord/dotacord register-server <server_id> "<server_name>"'
   ```
5. Report the output to the user.

## VPS Context

- **SSH**: `ssh root@178.104.97.192` (local ed25519 key)
- **Binary**: `/opt/dotacord/dotacord`
- **Config**: `/opt/dotacord/dotacord.toml`
- **Database**: `/opt/dotacord/data/dotacord.db`
- **Service**: `dotacord.service` (enabled on boot)
- **Discord token env**: `KEY_DOTACORD` (in systemd unit)
- **Redeploy**: `cargo build --release`, `scp target/release/dotacord root@178.104.97.192:/opt/dotacord/`, `ssh root@178.104.97.192 'systemctl restart dotacord.service'`

## Notes

- The subcommand only needs the config and database — no Discord token or internet required.
- If the server is already registered, the command exits cleanly with a message.
