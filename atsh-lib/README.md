The autossh/@shell library can be used. See the cli tools [autossh](https://crates.io/crates/autossh)

```rust
use atsh_lib::atsh::{add, download, initialize, list, login, remove, upload};

fn main() {
    // or initialize("/path/to/data/save").expect("initialize failed");
    // default data path is /executable_dir/.atsh.d
    initialize(None).expect("initialize failed");
    // add server info
    add("user", "password", "ip", 22, Some("name"), None).unwrap();
    // login by index
    login(1).unwrap();
    // show all records
    list().unwrap();
    // show all records with password
    list(true).unwrap();
    // do something...
}
```
