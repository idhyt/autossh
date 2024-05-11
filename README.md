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

## remove/rm

```bash
❯ ./autossh remove -i 1
    +-------+------+------+----+------+
    | index | name | user | ip | port |
    +-------+------+------+----+------+
```

## list

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

🍻 Thanks [passh](https://github.com/clarkwang/passh)
