# atsh-lib

The core library powering [atsh](https://github.com/idhyt/autossh/tree/main/atsh-cli) — an SSH connection manager with automatic passwordless login.

## Usage

```rust
use atsh_lib::atsh;

#[tokio::main]
async fn main() -> Result<(), atsh_lib::AtshError> {
    // initialize with a custom data directory
    let ctx = atsh::initialize(Some("/path/to/data/save"))?;
    // or use the default directory
    // let ctx = atsh::initialize(Option::<&std::path::Path>::None)?;

    // add a server (password is encrypted with ATSH_KEY)
    let index = atsh::add(
        &ctx,
        "user",
        "password",
        "1.2.3.4",
        22,
        &Some("my-server"),
        &None::<&str>,
    )?;

    // login (authorizes the public key on first use)
    atsh::login(&ctx, index, false).await?;
    // force re-authorization
    atsh::login(&ctx, index, true).await?;

    // file transfer
    atsh::upload(&ctx, index, &["/local/file.txt", "/remote/file.txt"]).await?;
    atsh::download(&ctx, index, &["/remote/file.txt", "/local/file.txt"]).await?;

    // run a command on the remote
    atsh::run_command(&ctx, index, "uptime", (None, None)).await?;

    // remove one or more remotes
    atsh::remove(&ctx, &[index]).await?;

    // pretty-print
    atsh::pprint(&ctx, false)?; // basic info
    atsh::pprint(&ctx, true)?;  // all info (password, authorized, note)

    Ok(())
}
```

## Data Layout

All data is stored under a single directory, configured via `initialize`:

- `ATSH_WORK_DIR` env var
- `$XDG_CONFIG_HOME/atsh.d`
- `$HOME/.config/atsh.d`
- `$HOME/.atsh.d`
- `<executable-dir>/.atsh.d`

```
~/.config/atsh.d/
├── config.toml      # remotes and SSH key paths
├── id_rsa           # SSH private key
├── id_rsa.pub       # SSH public key
└── logs/            # daily-rotated JSON logs
```

Migrating data is as simple as copying the whole directory.

## Requirements

Set the `ATSH_KEY` environment variable before adding servers — it is used to encrypt stored passwords:

```bash
export ATSH_KEY="your-secret-key"
```
