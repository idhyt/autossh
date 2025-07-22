The more details see [README.md](https://github.com/idhyt/autossh/blob/main/README.md)

## changelog (0.4.0)

- 优化和重构，将存储方式改为数据库
- 使用 `upload/download` 功能替换 `scp` 功能
- 环境变量改为 `ATSH_KEY` ( 暂时兼容 `ASKEY` )
- 编译的可执行文件从 `autossh` 改为 `atsh`

该版本以后所有数据文件都保存在工具的同级目录，方便迁移：

```bash
╰─ tree ~/app/atsh/
~/app/atsh/
├── atsh                             # autossh.exe / atsh.exe in windows
└── .atsh.d                          # atsh data
    ├── atsh.db                      # records database
    ├── atsh_key                     # ssh private key
    ├── atsh_key.pub                 # ssh public key
    ├── config.toml                  # config file with little information
    └── logs                         # log directory
        └── 2025-07-21.json
```

使用 [tool.py](https://github.com/idhyt/autossh/blob/main/tool.py) 将早期版本的数据迁移到数据库中，请执行以下命令：

```bash
# 新工具所在的目录
export ATSH_TOOL_DIR="/path/to/atsh/.atsh.d"
python tool.py toml2db -i ~/.config/autossh/config.toml -o $ATSH_TOOL_DIR
```

## build && install

Install by cargo

```bash
cargo install atsh
```

OR build from source

```bash
git clone --depth=1 https://github.com/idhyt/autossh
cd autossh/cli && cargo build --release
```

## usage

More details see `--help`

### add

```bash
❯ atsh add -u idhyt -p password -i 1.2.3.4 -n ubuntu
+-------+--------+-------+---------+------+
| index | name   | user  | ip      | port |
+=======+========+=======+=========+======+
| 1     | ubuntu | idhyt | 1.2.3.4 | 22   |
+-------+--------+-------+---------+------+
```

### login

```bash
❯ atsh login -i 1
(idhyt@1.2.3.4) Password:
Welcome to Ubuntu 20.04.2 LTS (GNU/Linux 5.4.0-156-generic x86_64)
```

### remove/rm/delete/del

```bash
❯ atsh rm -i 1
+-------+------+------+----+------+
| index | name | user | ip | port |
+-------+------+------+----+------+
```

remove multiple records by `rm -i 1 2 3 ...`

### list/ls/l

```bash
❯ atsh ls
+-------+--------+-------+---------+------+
| index | name   | user  | ip      | port |
+=======+========+=======+=========+======+
| 1     | ubuntu | idhyt | 1.2.3.4 | 22   |
+-------+--------+-------+---------+------+
```

maybe `scp` something, add option parameter `-a/--all` to show password.

```bash
❯ atsh ls --all
+-------+--------+-------+---------+------+----------+
| index | name   | user  | ip      | port | password |
+=======+========+=======+=========+======+==========+
| 1     | ubuntu | idhyt | 1.2.3.4 | 22   | password |
+-------+--------+-------+---------+------+----------+
```

### download / upload

```bash
❯ atsh upload -i 1 -p ./test.txt /tmp/test.txt
❯ atsh download -i 1 -p /tmp/test.txt ./test.txt
```
