use anyhow::{ensure, Result};
use atlas_core::model::Workspace;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use std::path::Path;

pub struct Store {
    connection: Connection,
}
pub struct SavedRevision {
    pub revision: i64,
    pub saved_at: String,
}
impl Store {
    pub fn revisions(&self) -> Result<Vec<SavedRevision>> {
        let mut statement = self
            .connection
            .prepare("SELECT revision, saved_at FROM history ORDER BY revision DESC")?;
        let records = statement.query_map([], |r| {
            Ok(SavedRevision {
                revision: r.get(0)?,
                saved_at: r.get(1)?,
            })
        })?;
        Ok(records.collect::<std::result::Result<Vec<_>, _>>()?)
    }
    pub fn load_revision(&self, revision: i64) -> Result<Workspace> {
        let body: String = self.connection.query_row(
            "SELECT body FROM history WHERE revision=?1",
            [revision],
            |r| r.get(0),
        )?;
        let mut workspace: Workspace = serde_json::from_str(&body)?;
        workspace.upgrade()?;
        Ok(workspace)
    }
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_connection(Connection::open(path)?)
    }
    pub fn memory() -> Result<Self> {
        Self::from_connection(Connection::open_in_memory()?)
    }
    fn from_connection(connection: Connection) -> Result<Self> {
        connection.busy_timeout(std::time::Duration::from_secs(3))?;
        let version: i64 = connection.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        ensure!(version <= 1, "Database schema is newer than this app");
        connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
            CREATE TABLE IF NOT EXISTS workspace (id INTEGER PRIMARY KEY CHECK(id=1), revision INTEGER NOT NULL, body TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS history (revision INTEGER PRIMARY KEY, saved_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, body TEXT NOT NULL);
            PRAGMA user_version=1;")?;
        Ok(Self { connection })
    }
    pub fn load(&self) -> Result<Option<(Workspace, i64)>> {
        let record: Option<(String, i64)> = self
            .connection
            .query_row("SELECT body, revision FROM workspace WHERE id=1", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .optional()?;
        record
            .map(|(body, revision)| {
                let mut w: Workspace = serde_json::from_str(&body)?;
                w.upgrade()?;
                Ok((w, revision))
            })
            .transpose()
    }
    pub fn save(&mut self, workspace: &Workspace, expected_revision: i64) -> Result<i64> {
        workspace.validate()?;
        let mut upgraded = workspace.clone();
        upgraded.upgrade()?;
        let body = serde_json::to_string(&upgraded)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let revision: i64 = tx
            .query_row("SELECT revision FROM workspace WHERE id=1", [], |r| {
                r.get(0)
            })
            .optional()?
            .unwrap_or(0);
        ensure!(revision == expected_revision, "Saved data changed in another process. Export your edits, then reopen; no data was overwritten.");
        let next = revision + 1;
        tx.execute(
            "INSERT INTO history(revision, body) VALUES(?1,?2)",
            params![next, body],
        )?;
        tx.execute("INSERT INTO workspace(id,revision,body) VALUES(1,?1,?2) ON CONFLICT(id) DO UPDATE SET revision=excluded.revision,body=excluded.body",params![next,body])?;
        tx.execute("DELETE FROM history WHERE revision <= ?1", [next - 50])?;
        tx.commit()?;
        Ok(next)
    }
}
