---
name: deploy
description: Build, tag, release, and deploy dotacord to the Hetzner VPS.
---

# Deploy

Builds dotacord locally, creates a GitHub release, and deploys the binary to the Hetzner VPS.

## Process

1. Ask the user for a **version tag** (e.g. `v1.0.0`). Suggest the next version based on `gh release list --limit 1` if available.
2. Optionally ask for release notes (one-liner is fine, default to empty).
3. Build:
   ```
   cargo build --release
   ```
4. Create the GitHub release with the binary attached:
   ```
   gh release create <tag> target/release/dotacord --title "<tag>" --notes "<notes>"
   ```
5. Deploy to VPS — download the release binary and restart:
   ```
   ssh root@178.104.97.192 'cd /opt/dotacord && cp dotacord dotacord.bak && curl -sL "https://github.com/Encryptoid/dotacord/releases/download/<tag>/dotacord" -o dotacord && chmod +x dotacord && systemctl restart dotacord.service && systemctl is-active dotacord.service'
   ```
6. Report success or failure to the user.

## Rollback

If the user asks to roll back, list available releases with `gh release list` and repeat step 5 with the chosen tag.

## VPS Context

- **SSH**: `ssh root@178.104.97.192` (local ed25519 key)
- **Binary**: `/opt/dotacord/dotacord`
- **Service**: `dotacord.service`
