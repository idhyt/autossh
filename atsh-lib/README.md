# atsh-lib

The core library powering [atsh](https://github.com/idhyt/autossh/tree/main/atsh-cli) — an SSH connection manager with automatic passwordless login.

## Usage

```rust
use atsh_lib::atsh::{self, Remote};

#[tokio::main]
async fn main() -> Result<(), atsh_lib::AtshError> {
    // initialize with a custom data directory
    let ctx = atsh::initialize(Some("/path/to/data/save"))?;
    // or use the default ($HOME/.atsh.d)
    // let ctx = atsh::initialize(Option::<&std::path::Path>::None)?;

    // add a server (password is encrypted with ATSH_KEY)
    atsh::add(
        &ctx,
        "user",
        "password",
        "1.2.3.4",
        22,
        &Some("my-server"),
        &None::<&str>,
    )?;

    // or add a Remote struct directly
    atsh::add_remote(&ctx, &Remote {
        index: 0, // ignored — auto-incremented on insert
        user: "user".to_string(),
        password: "password".to_string(),
        ip: "1.2.3.4".to_string(),
        port: 22,
        authorized: false,
        name: Some("my-server".to_string()),
        note: None,
    })?;

    // list all remotes
    let remotes = atsh::get_all(&ctx)?;
    let remote = &remotes[0];

    // login (authorizes the public key on first use)
    atsh::login(&ctx, remote.index, false).await?;
    // force re-authorization
    atsh::login(&ctx, remote.index, true).await?;

    // get a remote by index
    let find = atsh::get(&ctx, remote.index)?;          // Option<Remote>
    let find = atsh::try_get(&ctx, remote.index)?;       // Remote (or error)

    // file transfer
    atsh::upload(&ctx, remote.index, &["/local/file.txt", "/remote/file.txt"]).await?;
    atsh::download(&ctx, remote.index, &["/remote/file.txt", "/local/file.txt"]).await?;

    // run a command on the remote
    atsh::run_command(&ctx, remote.index, "uptime", (None, None)).await?;

    // remove one or more remotes
    atsh::remove(&ctx, &[remote.index]).await?;

    // pretty-print
    atsh::pprint(&ctx, false)?; // basic info
    atsh::pprint(&ctx, true)?;  // all info (password, authorized, note)

    Ok(())
}
```

## Data Layout

All data is stored under a single directory, configured via `initialize`. Default resolution order: `ATSH_WORK_DIR` env var → `$HOME/.atsh.d`.

```
~/.atsh.d/
├── atsh.db          # SQLite database (passwords encrypted with ATSH_KEY)
├── id_rsa           # SSH private key
├── id_rsa.pub       # SSH public key
├── config.toml      # SSH key path configuration
└── logs/            # Daily-rotated JSON logs
```

Migrating data is as simple as copying the whole directory.

## Requirements

Set the `ATSH_KEY` environment variable before adding servers — it is used to encrypt stored passwords:

```bash
export ATSH_KEY="your-secret-key"
```
