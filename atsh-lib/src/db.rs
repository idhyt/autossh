use parking_lot::Mutex;
use rusqlite::{Connection, Result, params};
use std::path::Path;
use std::sync::OnceLock;
use tracing::{info, warn};

use crate::WORK_DIR_FILE;
use crate::ssh::remote::Remote;
use crate::ssh::secure::{decrypt, encrypt};

static DATABASE: OnceLock<Mutex<Connection>> = OnceLock::new();

pub fn get_connection() -> &'static Mutex<Connection> {
    DATABASE.get_or_init(|| {
        let path = if cfg!(test) {
            WORK_DIR_FILE("test.autossh.db")
        } else {
            WORK_DIR_FILE("autossh.db")
        };

        Mutex::new(db_init(&path).unwrap())
    })
}

fn db_init(p: &Path) -> Result<Connection> {
    if p.is_file() {
        info!(file=?p, "Loading Database exists and using it");
    } else {
        warn!(file=?p, "Loading Database not found, creating a new one");
    }
    let conn = Connection::open(p)?;
    // 创建表（如果不存在）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS records (
            idx INTEGER PRIMARY KEY AUTOINCREMENT,  -- 使用 PRIMARY KEY 自动隐含 UNIQUE
            user TEXT NOT NULL,
            password TEXT NOT NULL,
            ip TEXT NOT NULL,
            port INTEGER NOT NULL,
            authorized BOOLEAN NOT NULL,
            name TEXT,
            note TEXT,
            UNIQUE(idx)  -- 显式声明唯一索引
        )",
        [], // 无参数
    )?;
    // 创建索引
    // conn.execute("CREATE INDEX IF NOT EXISTS idx_remote ON records (idx)", [])?;
    Ok(conn)
}

pub(crate) fn insert(conn: &Connection, remote: &Remote) -> Result<usize> {
    conn.execute(
        "INSERT INTO records (user, password, ip, port, authorized, name, note)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            remote.user,
            encrypt(&remote.password),
            remote.ip,
            remote.port,
            remote.authorized,
            remote.name,
            remote.note,
        ],
    )
}

pub(crate) fn query_index(conn: &Connection, idx: usize) -> Result<Option<Remote>> {
    let mut stmt = conn.prepare(
        "SELECT idx, user, password, ip, port, authorized, name, note 
         FROM records 
         WHERE idx = ?1",
    )?;
    let result = stmt.query_row(params![idx], |row| {
        Ok(Remote {
            index: row.get("idx")?,
            user: row.get("user")?,
            password: decrypt(&row.get::<_, String>("password")?),
            ip: row.get("ip")?,
            port: row.get("port")?,
            authorized: row.get("authorized")?,
            name: row.get("name")?,
            note: row.get("note")?,
        })
    });
    match result {
        Ok(remote) => Ok(Some(remote)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub(crate) fn query_all(conn: &Connection) -> Result<Vec<Remote>> {
    let mut stmt = conn.prepare("SELECT * FROM records")?;
    let records = stmt
        .query_map([], |row| {
            Ok(Remote {
                index: row.get("idx")?,
                user: row.get("user")?,
                password: decrypt(&row.get::<_, String>("password")?),
                authorized: row.get("authorized")?,
                ip: row.get("ip")?,
                port: row.get("port")?,
                name: row.get("name")?,
                note: row.get("note")?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(records)
}

pub(crate) fn delete_index(conn: &Connection, idx: usize) -> Result<usize> {
    conn.execute("DELETE FROM records WHERE idx = ?", params![idx])
}

pub(crate) fn update_authorized(conn: &Connection, idx: usize, authorized: bool) -> Result<usize> {
    conn.execute(
        "UPDATE records SET authorized = ?1 WHERE idx = ?2",
        params![authorized, idx],
    )
}

// test
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_db() {
        let remote = Remote {
            index: 1,
            user: "user".to_string(),
            password: "password".to_string(),
            authorized: true,
            ip: "1.2.3.4".to_string(),
            port: 2222,
            name: Some("name".to_string()),
            note: None,
        };
        // unsafe {std::env::set_var("ASKEY", "test");}
        // let enc_pwd = "HlT+Q0TYYpCrZNKfwM+Kg3VU3rE5hJ9jtohcGG0nU7qq2UOq";
        // test insert
        {
            let conn = get_connection().lock();
            println!("conn: {:#?}", conn);
            let n = insert(&conn, &remote);
            assert!(n.is_ok());
            assert_eq!(n.unwrap(), 1);
            insert(&conn, &remote).unwrap();
        }
        // test query all
        let exist_idx = {
            let conn = get_connection().lock();
            let all = query_all(&conn);
            println!("all: {:#?}", all);
            assert!(all.is_ok());
            let all = all.unwrap();
            assert_eq!(all.len(), 2);
            all.iter().map(|r| r.index).collect::<Vec<_>>()
        };
        // test query by index
        {
            let conn = get_connection().lock();
            let one = query_index(&conn, exist_idx[0]);
            println!("one: {:#?}", one);
            assert!(one.is_ok());
            let one = one.unwrap();
            assert!(one.is_some());
            let one = one.unwrap();
            assert!(one.index == remote.index);
            assert!(one.user == remote.user);
            assert!(one.password == remote.password);
            assert!(one.ip == remote.ip);
            assert!(one.port == remote.port);
            assert!(one.authorized == remote.authorized);
            assert!(one.name == remote.name);
            assert!(one.note == remote.note);
        }

        // delete one
        {
            let conn = get_connection().lock();
            let n = delete_index(&conn, exist_idx[0]);
            println!("delete: {:#?}", n);
            assert!(n.is_ok());
            assert_eq!(n.unwrap(), 1);
        }

        // now we add one again
        {
            let conn = get_connection().lock();
            let n = insert(&conn, &remote);
            assert_eq!(n.unwrap(), 1);
            // wo query all
            let new_index = query_all(&conn)
                .unwrap()
                .iter()
                .map(|r| r.index)
                .collect::<Vec<usize>>();
            println!("exist index: {:#?}", new_index);
            assert_eq!(new_index.len(), 2);
            assert_ne!(new_index, exist_idx);
            assert_eq!(new_index[0], exist_idx[1]);
        }

        // now we delete test data
        {
            let conn = get_connection().lock();
            std::fs::remove_file(Path::new(conn.path().unwrap())).unwrap();
        }
    }
}
