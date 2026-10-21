use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::OnceLock;
use tracing::{debug, warn};

use super::secure::{decrypt, encrypt};
use crate::config::CONFIG;
use crate::connection::Remote;
use crate::error::Result;

static DATABASE: OnceLock<Mutex<Connection>> = OnceLock::new();

pub fn get_connection() -> &'static Mutex<Connection> {
    DATABASE.get_or_init(|| Mutex::new(db_init(&CONFIG.work_dir_file("atsh.db")).unwrap()))
}

fn db_init(p: &Path) -> Result<Connection> {
    if p.is_file() {
        debug!(file=?p, "Loading Database exists and using");
    } else {
        warn!(file=?p, "Loading Database not found and new one");
    }
    let conn = Connection::open(p)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS records (
            idx INTEGER PRIMARY KEY AUTOINCREMENT,
            user TEXT NOT NULL,
            password TEXT NOT NULL,
            ip TEXT NOT NULL,
            port INTEGER NOT NULL,
            authorized BOOLEAN NOT NULL,
            name TEXT,
            note TEXT
        )",
        [],
    )?;
    Ok(conn)
}

pub(crate) fn insert(conn: &Connection, remote: &Remote) -> Result<usize> {
    let enc = encrypt(&remote.password)?;
    let n = conn.execute(
        "INSERT INTO records (user, password, ip, port, authorized, name, note)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            remote.user,
            enc,
            remote.ip,
            remote.port,
            remote.authorized,
            remote.name,
            remote.note,
        ],
    )?;
    Ok(n)
}

pub(crate) fn query_index(conn: &Connection, idx: usize) -> Result<Option<Remote>> {
    let mut stmt = conn.prepare(
        "SELECT idx, user, password, ip, port, authorized, name, note
         FROM records
         WHERE idx = ?1",
    )?;
    let result = stmt.query_row([idx as i32], |row| {
        let enc_password: String = row.get("password")?;
        Ok(Remote {
            index: row.get::<_, i32>("idx")? as usize,
            user: row.get("user")?,
            password: decrypt(&enc_password).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    enc_password.len(),
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
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
        Err(e) => Err(e.into()),
    }
}

pub(crate) fn query_all(conn: &Connection) -> Result<Vec<Remote>> {
    let mut stmt = conn.prepare("SELECT * FROM records")?;
    let records = stmt
        .query_map([], |row| {
            let enc_password: String = row.get("password")?;
            Ok(Remote {
                index: row.get::<_, i32>("idx")? as usize,
                user: row.get("user")?,
                password: decrypt(&enc_password).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        enc_password.len(),
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?,
                ip: row.get("ip")?,
                port: row.get("port")?,
                authorized: row.get("authorized")?,
                name: row.get("name")?,
                note: row.get("note")?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(records)
}

pub(crate) fn delete_index(conn: &Connection, idx: usize) -> Result<usize> {
    let n = conn.execute("DELETE FROM records WHERE idx = ?", [idx as i32])?;
    Ok(n)
}

pub(crate) fn update_authorized(conn: &Connection, idx: usize, authorized: bool) -> Result<usize> {
    let n = conn.execute(
        "UPDATE records SET authorized = ?1 WHERE idx = ?2",
        (authorized, idx as i32),
    )?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db() {
        let db_path = CONFIG.work_dir_file("atsh.db");

        let remote = Remote {
            index: 1,
            user: "user".to_string(),
            password: "password".to_string(),
            ip: "1.2.3.4".to_string(),
            port: 2222,
            authorized: false,
            name: Some("name".to_string()),
            note: None,
        };

        if db_path.is_file() {
            std::fs::remove_file(&db_path).unwrap();
        }

        {
            let conn = get_connection().lock();
            let n = insert(&conn, &remote).unwrap();
            assert_eq!(n, 1);
            insert(&conn, &remote).unwrap();
        }

        {
            {
                let conn = get_connection().lock();
                let n = update_authorized(&conn, 1, true).unwrap();
                assert_eq!(n, 1);
            }
            let conn = get_connection().lock();
            let one = query_index(&conn, 1).unwrap().unwrap();
            assert_eq!(one.authorized, true);
        }

        let exist_idx = {
            let conn = get_connection().lock();
            let all = query_all(&conn).unwrap();
            assert_eq!(all.len(), 2);
            all.iter().map(|r| r.index).collect::<Vec<_>>()
        };

        {
            let conn = get_connection().lock();
            let one = query_index(&conn, exist_idx[0]).unwrap().unwrap();
            assert_eq!(one.index, remote.index);
            assert_eq!(one.user, remote.user);
            assert_eq!(one.password, remote.password);
            assert_eq!(one.ip, remote.ip);
            assert_eq!(one.port, remote.port);
            assert_eq!(one.name, remote.name);
            assert_eq!(one.note, remote.note);
        }

        {
            let conn = get_connection().lock();
            let n = delete_index(&conn, exist_idx[0]).unwrap();
            assert_eq!(n, 1);
        }

        {
            let conn = get_connection().lock();
            let n = insert(&conn, &remote).unwrap();
            assert_eq!(n, 1);
            let new_index = query_all(&conn)
                .unwrap()
                .iter()
                .map(|r| r.index)
                .collect::<Vec<usize>>();
            assert_eq!(new_index.len(), 2);
            assert_ne!(new_index, exist_idx);
            assert_eq!(new_index[0], exist_idx[1]);
        }
    }
}