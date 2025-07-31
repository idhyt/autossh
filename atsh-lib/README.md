The atsh/@shell library can be used. See the cli tools [atsh-cli](https://github.com/idhyt/autossh/tree/main/atsh-cli)

```rust
use atsh_lib::atsh::{
    initialize,
    add, add_remote,
    get, get_all, try_get,
    download, upload,
    remove, login, pprint,
    Remote, CONFIG,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // or initialize("/path/to/data/save").expect("initialize failed");
    // default data path
    initialize(Option::<&std::path::Path>::None)?;
    // set data to $home/.atsh.d
    initialize(std::env::home_dir().map(|h| h.join(".atsh.d")))?;
    // get ATSH_KEY
    let key = CONFIG.get_enc_key()?;
    // set ATSH_KEY
    CONFIG.set_enc_key(Some("secret"))?;
    // or export ATSH_KEY
    unsafe { std::env::set_var("ASKEY", "secret") };
    // clear ATSH_KEY
    CONFIG.set_enc_key(Option::<&str>::None)?;
    // add server info
    add(
        "user",
        "password",
        "ip",
        22,
        &Some("name"),
        &Option::<&str>::None,
    )?;
    // or add remote
    add_remote(&Remote {
        index: 0, // ignore this value, auto increment when insert to database
        user: "user".to_string(),
        password: "password".to_string(),
        ip: "ip".to_string(),
        port: 22,
        authorized: false,
        name: Some("name".to_string()),
        note: Option::<String>::None,
    })?;
    // get all remotes
    let remotes = get_all()?;
    // login by index
    let remote = remotes.get(0).unwrap();
    login(remote.index, false)?;
    // login with reauth,
    // This enforces re-authentication, which can be useful when data is migrated
    login(remote.index, true)?;
    // get remote by index, return Option<Remote>
    let find = get(remote.index)?;
    // get remote by index, return Remote or Error if not found
    let find = try_get(remote.index)?;
    // remove by index
    remove(&vec![remote.index])?;
    // download
    download(
        remote.index,
        &vec!["/path/to/remote/test.txt", "/path/to/host/test.txt"],
    )?;
    // upload
    upload(
        remote.index,
        &vec!["/path/to/local/test.txt", "/path/to/remote/test.txt"],
    )?;
    // pretty print little info
    pprint(false)?;
    // pretty print with all info
    pprint(true)?;
    // create a new ssh key pair
    CONFIG.create_sshkey(
        Option::<&str>::None,
        Option::<&std::path::Path>::None,
        true)?;
    // do something...
    Ok(())
}
```

通过 `initialize` 可以指定数据存放目录，所有数据保存在同一目录下

默认情况数据存放目录优先级: `$HOME/.atsh.d -> $current_exe_dir/.atsh.d -> Error`

数据迁移时候整个目录拷贝即可

```bash
╰─ tree ~/.atsh.d
~/.atsh.d
├── atsh                             # autossh.exe / atsh.exe in windows
└── .atsh.d                          # atsh data
    ├── atsh.db                      # records database
    ├── id_rsa                       # ssh private key
    ├── id_rsa.pub                   # ssh public key
    ├── config.toml                  # config file with little information
    └── logs                         # log directory
        └── 2025-07-21.json
```
