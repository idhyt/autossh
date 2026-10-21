use rusqlite::{params, Connection};
use std::path::Path;
use tracing::{debug, warn};

use crate::connection::Remote;
use crate::error::Result;

/// Owned SQLite database handle. Replaces the global `DATABASE` singleton so
/// the connection lifetime and locking are explicit.
pub struct Db {
    conn: Connection,
}

impl Db {
    /// Open (or create) the database at `path`, initializing the schema.
    pub fn open(path: &Path) -> Result<Self> {
        if path.is_file() {
            debug!(file=?path, "database found, opening");
        } else {
            warn!(file=?path, "database not found, creating new");
        }
        let conn = Connection::open(path)?;
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
        Ok(Self { conn })
    }

    pub fn insert(&self, remote: &Remote, enc_password: &str) -> Result<usize> {
        let n = self.conn.execute(
            "INSERT INTO records (user, password, ip, port, authorized, name, note)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                remote.user,
                enc_password,
                remote.ip,
                remote.port,
                remote.authorized,
                remote.name,
                remote.note,
            ],
        )?;
        Ok(n)
    }

    pub fn query_index(&self, idx: usize, dec: impl Fn(&str) -> Result<String>) -> Result<Option<Remote>> {
        let mut stmt = self.conn.prepare(
            "SELECT idx, user, password, ip, port, authorized, name, note
             FROM records
             WHERE idx = ?1",
        )?;
        let result = stmt.query_row([idx as i32], |row| {
            let enc_password: String = row.get("password")?;
            let password = dec(&enc_password).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    enc_password.len(),
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            Ok(Remote {
                index: row.get::<_, i32>("idx")? as usize,
                user: row.get("user")?,
                password,
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

    pub fn query_all(&self, dec: impl Fn(&str) -> Result<String>) -> Result<Vec<Remote>> {
        let mut stmt = self.conn.prepare("SELECT * FROM records")?;
        let records = stmt
            .query_map([], |row| {
                let enc_password: String = row.get("password")?;
                let password = dec(&enc_password).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        enc_password.len(),
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;
                Ok(Remote {
                    index: row.get::<_, i32>("idx")? as usize,
                    user: row.get("user")?,
                    password,
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

    pub fn delete_index(&self, idx: usize) -> Result<usize> {
        let n = self.conn.execute("DELETE FROM records WHERE idx = ?", [idx as i32])?;
        Ok(n)
    }

    pub fn update_authorized(&self, idx: usize, authorized: bool) -> Result<usize> {
        let n = self.conn.execute(
            "UPDATE records SET authorized = ?1 WHERE idx = ?2",
            (authorized, idx as i32),
        )?;
        Ok(n)
    }
}
