## usage

```bash
❯ autossh --help
ssh manager and auto login tool

Usage: autossh [COMMAND]

Commands:
  list    List the remote server
  add     Add the remote server
  remove  Remove the remote server by index
  login   Login the remote server by index
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## add

```bash
❯ autossh add -u idhyt -p password -i 1.2.3.4 -n ubuntu
+-------+--------+-------+---------+------+
| index | name   | user  | ip      | port |
+=======+========+=======+=========+======+
| 1     | ubuntu | idhyt | 1.2.3.4 | 22   |
+-------+--------+-------+---------+------+
```

add other server info by `-N/--note` option, like `-N "expired at 2022-11-11"`

note! the password need to be escaped if there are special characters in it. you can refer to the following [which-characters-need-to-be-escaped-when-using-bash](https://stackoverflow.com/questions/15783701/which-characters-need-to-be-escaped-when-using-bash)

## remove/rm/delete/del

```bash
❯ autossh rm -i 1
+-------+------+------+----+------+
| index | name | user | ip | port |
+-------+------+------+----+------+
```

remove multiple records by `rm -i 1 2 3 ...`

## list/ls/l

```bash
❯ autossh ls
+-------+--------+-------+---------+------+
| index | name   | user  | ip      | port |
+=======+========+=======+=========+======+
| 1     | ubuntu | idhyt | 1.2.3.4 | 22   |
+-------+--------+-------+---------+------+
```

maybe `scp` something, add option parameter `-a/--all` to show password.

```bash
❯ autossh ls --all
+-------+--------+-------+---------+------+----------+
| index | name   | user  | ip      | port | password |
+=======+========+=======+=========+======+==========+
| 1     | ubuntu | idhyt | 1.2.3.4 | 22   | password |
+-------+--------+-------+---------+------+----------+
```

## login

```bash
❯ autossh login -i 1
(idhyt@1.2.3.4) Password:
Welcome to Ubuntu 20.04.2 LTS (GNU/Linux 5.4.0-156-generic x86_64)
```

in windows, you need download [putty.exe](https://www.chiark.greenend.org.uk/~sgtatham/putty/) and place in the same directory as the tool.

## backup or restore

the record file is location `$HOME/.autossh.toml`, you can change and backup it.

if you don't know the location, you can use the debug mode command to find it.

```bash
❯ RUST_LOG=DEBUG autossh list
[2024-06-19T10:04:22Z DEBUG  autossh::ssh::record] the record data located in `/home/idhyt/.autossh.toml`
```

## 💥 security 💥

the `password` fields is plaintext by default,

if you wish to encrypt it, import environment variables `ASKEY` before use.

`export ASKEY="SecretKey"` in bash or `set ASKEY="SecretKey"` in cmd.

```bash
❯ export ASKEY="protected"
❯ autossh add -u idhyt -p password -i 1.2.3.4 -n ubuntu
> autossh list --all
+-------+--------+-------+---------+------+----------+
| index | name   | user  | ip      | port | password |
+=======+========+=======+=========+======+==========+
| 1     | ubuntu | idhyt | 1.2.3.4 | 22   | password |
+-------+--------+-------+---------+------+----------+
❯ cat ~/.autossh.toml | grep password
password = "IiaMr0ce4iKF5AvXf+rtFQ9mET0Ug4hLOoGeybzyOQx/lUvh"
```

🍻 Thanks [passh](https://github.com/clarkwang/passh)

There are still some issues that need to be resolved for `passh`

```bash
❯ ./autossh login -i 1
!! can't execute: ssh: Bad address (14)
```

ensure the ssh info correct and execute several times.
