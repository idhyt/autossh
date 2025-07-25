The atsh/@shell library can be used. See the cli tools [atsh-cli](https://github.com/idhyt/autossh/tree/main/atsh-cli)

```rust
use atsh_lib::atsh::{
    initialize,
    add,
    get,
    try_get,
    login,
    get_all,
    remove,
    pprint,
    download,
    upload,
    get_atshkey,
    set_atshkey,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // or initialize("/path/to/data/save").expect("initialize failed");
    // default data path is /executable_dir/.atsh.d
    initialize(None)?;
    // get ATSH_KEY
    let key = get_atshkey()?;
    // set ATSH_KEY
    set_atshkey(Some("secret"))?;
    // or export ATSH_KEY
    unsafe { std::env::set_var("ASKEY", "secret") };
    // clear ATSH_KEY
    set_atshkey(None)?;
    // add server info
    add("user", "password", "ip", 22, Some("name"), None)?;
    // get all remotes
    let remotes = get_all()?;
    // login by index
    let remote = remotes.get(0).unwrap();
    login(remote.index)?;
    // get remote by index, return Option<Remote>
    let remote = get(remote.index)?;
    // get remote by index, return Remote or Error if not found
    let remote = try_get(remote.index)?;
    // remove by index
    remove(remote.index)?;
    // download
    download(remote.index, vec!["/path/to/remote/test.txt", "/path/to/host/test.txt"])?;
    // upload
    upload(remote.index, vec!["/path/to/local/test.txt", "/path/to/remote/test.txt"])?;
    // pretty print little info
    pprint(false)?;
    // pretty print with all info
    pprint(true)?;
    // do something...
}
```
