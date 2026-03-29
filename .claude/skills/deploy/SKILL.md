---
name: deploy
description: Build, tag, release, and deploy dotacord to the Hetzner VPS.
disable-model-invocation: true
allowed-tools: Bash(pwsh *)
---

# Deploy

Builds dotacord locally, creates a GitHub release, and deploys the binary to the Hetzner VPS.

## Process

1. Ask the user for a **version tag** (e.g. `v1.0.0`). Suggest the next version based on `gh release list --limit 1` if available.
2. Generate release notes from the commit log between the last tag and HEAD: `git log <last-tag>..HEAD --oneline`. Summarize into a concise bullet list. Show the user for confirmation.
3. Run the deploy script:
   ```
   pwsh ${CLAUDE_SKILL_DIR}/scripts/deploy.ps1 -Tag <tag> -Notes "<notes>"
   ```
4. Report success or failure to the user.

## Rollback

If the user asks to roll back, list available releases with `gh release list` and run:
```
pwsh ${CLAUDE_SKILL_DIR}/scripts/rollback.ps1 -Tag <tag>
```

## VPS Context

- **SSH**: `ssh root@178.104.97.192` (local ed25519 key)
- **Binary**: `/opt/dotacord/dotacord`
- **Service**: `dotacord.service`
