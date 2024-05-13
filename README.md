## usage

```bash
❯ ./autossh --help
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
❯ ./autossh add -u idhyt -p "[p4ssw0rd}" -i 1.2.3.4 -n Ubuntu
+-------+--------+-------+---------+------+
| index | name   | user  | ip      | port |
+-------+--------+-------+---------+------+
| 1     | Ubuntu | idhyt | 1.2.3.4 | 22   |
+-------+--------+-------+---------+------+
```

## remove/rm/delete/del

```bash
❯ ./autossh remove -i 1
+-------+------+------+----+------+
| index | name | user | ip | port |
+-------+------+------+----+------+
```

## list/ls/l

```bash
❯ ./autossh list
+-------+--------+-------+---------+------+
| index | name   | user  | ip      | port |
+-------+--------+-------+---------+------+
| 1     | Ubuntu | idhyt | 1.2.3.4 | 22   |
+-------+--------+-------+---------+------+
```

## login

```bash
❯ ./autossh login -i 1
(idhyt@1.2.3.4) Password:
Welcome to Ubuntu 20.04.2 LTS (GNU/Linux 5.4.0-156-generic x86_64)
```

## security

the record file is location `$HOME/.autossh.toml`, you can change and backup it.

🚨🚨🚨 note! the `password` fields is plaintext 🚨🚨🚨

if you wish to encrypt it, import environment variables `ASKEY` before use.

```bash
❯ export ASKEY="protected"
❯ autossh add -u root -p "password" -i 1.2.3.4 -n hello
+-------+---------------------+-------+--------------+------+
| index | name                | user  | ip           | port |
+=======+=====================+=======+==============+======+
| 1     | hello               | root  | 1.2.3.4      | 22   |
+-------+---------------------+-------+--------------+------+
❯ cat ~/.autossh.toml | grep password
password = "dsrO12SGroO+FmD0H0WUB3QvGpSl7TEysrPkDYKEIjUG6uEt"
```

🍻 Thanks [passh](https://github.com/clarkwang/passh)

There are still some issues that need to be resolved for `passh`

```bash
❯ ./autossh login -i 1
!! can't execute: ssh: Bad address (14)
```
ensure the ssh info correct and execute several times.
