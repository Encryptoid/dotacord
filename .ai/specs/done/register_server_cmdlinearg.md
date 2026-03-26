# register_server CLI Subcommand

## Overview

Add a `register-server` subcommand to the `dotacord` binary that registers a Discord server in the
database without starting the Discord bot. This replaces the need to use the PowerShell script
`scripts/register-server.ps1` for server registration and makes the operation a first-class part of
the binary.

Usage:

```shell
dotacord register-server <server_id> <server_name>
```

This performs a minimal startup (config load + database init only), inserts the server row, prints a
result, and exits. No Discord token, hero cache, or scheduler is involved.

## Diagram

```text
dotacord register-server <id> <name>
        │
        ▼
  config::load_config()
        │
        ▼
  database_access::init_database()
        │
        ▼
  servers_db::insert_server(server_id, server_name)
        │
        ├─ already exists? → print "Server already registered" → exit 0
        └─ inserted ok?    → print "Server registered"         → exit 0
```

## Example Usage

```rust
// Args in main.rs
#[derive(Parser)]
#[command(name = "dotacord")]
struct Args {
    #[arg(short = 'c', long)]
    clear_commands: bool,
    #[arg(short, long)]
    register: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Register a Discord server in the database
    RegisterServer {
        server_id: u64,
        server_name: String,
    },
}

// Early-exit branch in main(), before Discord setup:
if let Some(Command::RegisterServer { server_id, server_name }) = args.command {
    servers_db::insert_server(server_id as i64, &server_name).await?;
    println!("Server '{}' ({}) registered.", server_name, server_id);
    return Ok(());
}

// In servers_db.rs
pub async fn insert_server(server_id: i64, server_name: &str) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let existing = Server::find_by_id(server_id).one(&txn).await?;
    if existing.is_some() {
        println!("Server {} is already registered.", server_id);
        return Ok(());
    }
    let new_server = server::ActiveModel {
        server_id: Set(server_id),
        server_name: Set(server_name.to_string()),
        channel_id: Set(None),
        is_sub_week: Set(0),
        is_sub_month: Set(0),
        is_sub_reload: Set(0),
        weekly_day: Set(None),
        weekly_hour: Set(None),
        monthly_week: Set(None),
        monthly_weekday: Set(None),
        monthly_hour: Set(None),
    };
    Server::insert(new_server).exec(&txn).await?;
    txn.commit().await?;
    Ok(())
}
```

## Flow

`main()` args parse → `Some(Command::RegisterServer)` branch → `config::load_config()` →
`database_access::init_database()` → `servers_db::insert_server()` → stdout result → `return Ok(())`

The existing normal startup path (Discord client, hero cache, scheduler) is **not reached** when the
subcommand is present.

## Implementation Steps

### 1. Add `insert_server` to `src/database/servers_db.rs`

Add a new `pub async fn insert_server(server_id: i64, server_name: &str) -> Result<(), Error>`
function following the existing pattern in the file:

- Get a transaction via `database_access::get_transaction()`
- Check for an existing row with `Server::find_by_id(server_id).one(&txn)`
- If found, print that the server is already registered and return `Ok(())`
- Otherwise build a `server::ActiveModel` with all non-nullable fields set (nullable fields as
  `Set(None)`, `is_sub_*` as `Set(0)`)
- `Server::insert(new_server).exec(&txn).await?`
- `txn.commit().await?`

Imports needed: `use sea_orm::Set;` (already present via `use sea_orm::*;`),
`use crate::database::entities::server` (already present).

### 2. Add `Command` subcommand enum and update `Args` in `src/main.rs`

Add `use clap::Subcommand;` to imports (alongside the existing `use clap::Parser;`).

Add a new enum after `Args`:

```rust
#[derive(Subcommand)]
enum Command {
    /// Register a Discord server in the database
    RegisterServer {
        server_id: u64,
        server_name: String,
    },
}
```

Add a field to `Args`:

```rust
#[command(subcommand)]
command: Option<Command>,
```

### 3. Add early-exit branch in `main()` in `src/main.rs`

After `database_access::init_database(&cfg.database_path).await?;` and **before**
`discord::commands()` is called, insert:

```rust
if let Some(Command::RegisterServer { server_id, server_name }) = args.command {
    database::servers_db::insert_server(server_id as i64, &server_name).await?;
    return Ok(());
}
```

`insert_server` itself prints the outcome, so no additional `println!` is needed in `main`.

### 4. Add `use crate::database::servers_db` import in `src/main.rs`

The `main.rs` currently only imports `database_access` and `hero_cache` from `crate::database`. Add
`servers_db` to that use statement or add a separate one.

