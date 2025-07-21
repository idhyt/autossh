import click
import sys
import os
from typing import Optional, Dict
from pathlib import Path
import tomli, tomli_w
import sqlite3
import platform
import subprocess


@click.group()
def cli():
    pass


def set_permissions(path: Path, perm):
    pass


def create_config(data: Dict, output: Path):
    if 'sshkey' not in data:
        return
    sshkey = data['sshkey']
    # copy sshkey
    public, private = Path(sshkey['public']), Path(sshkey['private'])
    assert public.is_file()
    assert private.is_file()
    output_public, output_private = output / 'atsh_key.pub', output / 'atsh_key'
    output_public.write_bytes(public.read_bytes())
    output_private.write_bytes(private.read_bytes())
    if platform.system() == "Windows":
        cmds = [
            # public
            f'icacls {output_public} /reset',
            f'icacls {output_public} /grant:r "%USERNAME%:(R,W)"',
            f'icacls {output_public} /grant:r "Everyone:(R)"',
            f'icacls {output_public} /inheritance:r',
            # private
            f'icacls {output_private} /reset',
            f'icacls {output_private} /grant:r "%USERNAME%:(R,W)"',
            f'icacls {output_private} /inheritance:r'
        ]
        for cmd in cmds:
            subprocess.run(cmd, shell=True, check=True)
    else:
        os.chmod(output_public, 0o644)
        os.chmod(output_private, 0o600)

    print(f'[+] success write sshkey to {output}')

    with open(output / 'config.toml', 'wb') as f:
        tomli_w.dump(
            {
                'sshkey': {
                    'public': str(output_public),
                    'private': str(output_private)
                }
            }, f)
    print(f'[+] success write config.toml at {output / "config.toml"}')


def create_database(data: Dict, output: Path):
    if 'remotes' not in data:
        return
    if 'list' not in data['remotes']:
        return
    remotes = data['remotes']['list']
    if len(remotes) == 0:
        print('[!] no remotes')
        return

    def create_table(_conn):
        _cursor = _conn.cursor()
        _cursor.execute('''
            CREATE TABLE IF NOT EXISTS records (
                idx INTEGER PRIMARY KEY AUTOINCREMENT,
                user TEXT NOT NULL,
                password TEXT,
                ip TEXT,
                port INTEGER,
                authorized BOOLEAN,
                name TEXT,
                note TEXT,
                UNIQUE(idx)
            )
            ''')
        # cursor.execute('CREATE INDEX IF NOT EXISTS idx_index ON records(idx)')
        _conn.commit()

    db_path = output / 'atsh.db'
    if db_path.is_file():
        db_path.unlink()
    # create database
    conn = sqlite3.connect(db_path)
    create_table(conn)

    remotes_data = [(r['index'], r['user'], r['password'], r['ip'], r['port'],
                     r['authorized'], r['name'] if 'name' in r else None,
                     r['note'] if 'note' in r else None) for r in remotes]
    cursor = conn.cursor()
    cursor.executemany(
        '''
        INSERT OR REPLACE INTO records 
        (idx, user, password, ip, port, authorized, name, note)
        VALUES(?,?,?,?,?,?,?,?)
    ''', remotes_data)
    conn.commit()

    cursor = conn.cursor()
    cursor.execute("SELECT COUNT(*) FROM records")
    count = cursor.fetchone()[0]
    if count != len(remotes_data):
        print(
            f'[-] write records from toml to database failed, expected {len(remotes_data)}, but written {count}'
        )
        sys.exit(1)

    print(f"[+] success write {count} records to {db_path}")


@cli.command()
@click.option('-i',
              '--input',
              required=True,
              help='The remote records in origin toml config')
@click.option(
    '-o',
    '--output',
    required=False,
    help='The remote records database, default output to /input/dir/atsh/')
def toml2db(*, input: str, output: Optional[str] = None):
    input_path = Path(input)
    if not input_path.is_file():
        print("[-] input not exists")
        sys.exit(1)
    output_path = input_path.with_name('atsh') if not output else Path(output)
    output_path = output_path.absolute()
    print(f"[*] output: {output_path}")

    if not output_path.is_dir():
        output_path.mkdir(parents=True)

    with open(input_path, 'rb') as f:
        config = tomli.load(f)
    create_config(config, output_path)
    create_database(config, output_path)


if __name__ == '__main__':
    cli()
