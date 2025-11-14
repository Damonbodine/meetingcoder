use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

#[derive(Clone, Debug)]
pub struct QueueItem {
    pub id: i64,
    pub meeting_id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub file_path: String,
}

pub struct Queue {
    db_path: PathBuf,
}

impl Queue {
    pub fn new(app: &AppHandle) -> Result<Arc<Self>> {
        let db_path: PathBuf = app
            .path()
            .resolve("audio_queue.sqlite", tauri::path::BaseDirectory::AppData)?;
        std::fs::create_dir_all(db_path.parent().unwrap_or_else(|| Path::new(".")))?;
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            CREATE TABLE IF NOT EXISTS queue (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              meeting_id TEXT NOT NULL,
              start_ms INTEGER NOT NULL,
              end_ms INTEGER NOT NULL,
              file_path TEXT NOT NULL,
              status TEXT NOT NULL DEFAULT 'queued',
              attempts INTEGER NOT NULL DEFAULT 0,
              error TEXT,
              created_at INTEGER NOT NULL,
              updated_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_queue_status ON queue(status);
            CREATE INDEX IF NOT EXISTS idx_queue_meeting ON queue(meeting_id);
            "#,
        )?;
        Ok(Arc::new(Self { db_path }))
    }

    fn now_ms() -> i64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    fn open(&self) -> Result<Connection> {
        Ok(Connection::open(&self.db_path)?)
    }

    pub fn enqueue(
        &self,
        meeting_id: &str,
        start_ms: u64,
        end_ms: u64,
        file_path: &str,
    ) -> Result<i64> {
        let conn = self.open()?;
        let now = Self::now_ms();
        conn.execute(
            "INSERT INTO queue (meeting_id, start_ms, end_ms, file_path, status, attempts, created_at, updated_at) VALUES (?, ?, ?, ?, 'queued', 0, ?, ?)",
            params![meeting_id, start_ms as i64, end_ms as i64, file_path, now, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn fetch_next(&self) -> Result<Option<QueueItem>> {
        let conn = self.open()?;
        let tx = conn.unchecked_transaction()?;
        let row: Option<(i64, String, i64, i64, String)> = tx
            .query_row(
                "SELECT id, meeting_id, start_ms, end_ms, file_path FROM queue WHERE status='queued' ORDER BY id LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()?;
        if let Some((id, meeting_id, start_ms, end_ms, file_path)) = row {
            tx.execute(
                "UPDATE queue SET status='processing', updated_at=? WHERE id=?",
                params![Self::now_ms(), id],
            )?;
            tx.commit()?;
            Ok(Some(QueueItem {
                id,
                meeting_id,
                start_ms: start_ms as u64,
                end_ms: end_ms as u64,
                file_path,
            }))
        } else {
            tx.commit()?;
            Ok(None)
        }
    }

    pub fn mark_done(&self, id: i64) -> Result<()> {
        let conn = self.open()?;
        conn.execute(
            "UPDATE queue SET status='done', updated_at=? WHERE id=?",
            params![Self::now_ms(), id],
        )?;
        Ok(())
    }

    pub fn mark_failed(&self, id: i64, error: &str) -> Result<()> {
        let conn = self.open()?;
        conn.execute(
            "UPDATE queue SET status='queued', attempts=attempts+1, error=?, updated_at=? WHERE id=?",
            params![error, Self::now_ms(), id],
        )?;
        Ok(())
    }

    pub fn counts(&self) -> Result<(i64, i64, i64)> {
        let conn = self.open()?;
        let queued: i64 = conn.query_row(
            "SELECT COUNT(*) FROM queue WHERE status='queued'",
            [],
            |r| r.get(0),
        )?;
        let processing: i64 = conn.query_row(
            "SELECT COUNT(*) FROM queue WHERE status='processing'",
            [],
            |r| r.get(0),
        )?;
        let failed: i64 = conn.query_row("SELECT COUNT(*) FROM queue WHERE status!='queued' AND status!='processing' AND status!='done'", [], |r| r.get(0)).unwrap_or(0);
        Ok((queued, processing, failed))
    }

    pub fn backlog_seconds(&self) -> Result<f32> {
        let conn = self.open()?;
        let total_ms: i64 = conn.query_row(
            "SELECT COALESCE(SUM(end_ms - start_ms), 0) FROM queue WHERE status IN ('queued','processing')",
            [],
            |r| r.get(0),
        )?;
        Ok(total_ms as f32 / 1000.0)
    }

    pub fn counts_for_meeting(&self, meeting_id: &str) -> Result<(i64, i64)> {
        let conn = self.open()?;
        let queued: i64 = conn.query_row(
            "SELECT COUNT(*) FROM queue WHERE status='queued' AND meeting_id=?",
            params![meeting_id],
            |r| r.get(0),
        )?;
        let processing: i64 = conn.query_row(
            "SELECT COUNT(*) FROM queue WHERE status='processing' AND meeting_id=?",
            params![meeting_id],
            |r| r.get(0),
        )?;
        Ok((queued, processing))
    }
}
