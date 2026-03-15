use std::path::Path;

use rusqlite::Connection;

use crate::error::Result;
use crate::types::MemoryEntry;

pub struct Database {
    conn: std::sync::Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        let db = Self {
            conn: std::sync::Mutex::new(conn),
        };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS project_memory (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_path TEXT NOT NULL,
                key TEXT NOT NULL,
                value TEXT NOT NULL,
                category TEXT NOT NULL DEFAULT 'general',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE(project_path, key)
            );

            CREATE TABLE IF NOT EXISTS analysis_cache (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                document_uuid TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                analysis_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(document_uuid, content_hash)
            );

            CREATE TABLE IF NOT EXISTS session_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_path TEXT NOT NULL,
                action TEXT NOT NULL,
                details TEXT,
                timestamp TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    pub fn upsert_memory(
        &self,
        project_path: &str,
        key: &str,
        value: &str,
        category: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO project_memory (project_path, key, value, category, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?5)
             ON CONFLICT(project_path, key) DO UPDATE SET
                value = excluded.value,
                category = excluded.category,
                updated_at = excluded.updated_at",
            rusqlite::params![project_path, key, value, category, now],
        )?;
        Ok(())
    }

    pub fn get_memory(
        &self,
        project_path: &str,
        key: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<MemoryEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut entries = Vec::new();

        match (key, category) {
            (Some(k), _) => {
                let mut stmt = conn.prepare(
                    "SELECT key, value, category, created_at, updated_at
                     FROM project_memory WHERE project_path = ?1 AND key = ?2",
                )?;
                let rows = stmt.query_map(rusqlite::params![project_path, k], |row| {
                    Ok(MemoryEntry {
                        key: row.get(0)?,
                        value: row.get(1)?,
                        category: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                    })
                })?;
                for row in rows {
                    entries.push(row?);
                }
            }
            (None, Some(cat)) => {
                let mut stmt = conn.prepare(
                    "SELECT key, value, category, created_at, updated_at
                     FROM project_memory WHERE project_path = ?1 AND category = ?2
                     ORDER BY key",
                )?;
                let rows = stmt.query_map(rusqlite::params![project_path, cat], |row| {
                    Ok(MemoryEntry {
                        key: row.get(0)?,
                        value: row.get(1)?,
                        category: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                    })
                })?;
                for row in rows {
                    entries.push(row?);
                }
            }
            (None, None) => {
                let mut stmt = conn.prepare(
                    "SELECT key, value, category, created_at, updated_at
                     FROM project_memory WHERE project_path = ?1
                     ORDER BY category, key",
                )?;
                let rows = stmt.query_map(rusqlite::params![project_path], |row| {
                    Ok(MemoryEntry {
                        key: row.get(0)?,
                        value: row.get(1)?,
                        category: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                    })
                })?;
                for row in rows {
                    entries.push(row?);
                }
            }
        }

        Ok(entries)
    }

    #[allow(dead_code)]
    pub fn delete_memory(&self, project_path: &str, key: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM project_memory WHERE project_path = ?1 AND key = ?2",
            rusqlite::params![project_path, key],
        )?;
        Ok(())
    }

    pub fn log_session(
        &self,
        project_path: &str,
        action: &str,
        details: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO session_history (project_path, action, details, timestamp)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![project_path, action, details, now],
        )?;
        Ok(())
    }
}
