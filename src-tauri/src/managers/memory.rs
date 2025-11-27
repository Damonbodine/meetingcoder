use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Memory {
    pub id: i64,
    pub text: String,
    pub metadata: String,
    pub created_at: i64,
    pub score: f32, // Similarity score
}

pub struct MemoryManager {
    db_path: PathBuf,
    embedding_model: Arc<Mutex<TextEmbedding>>,
}

impl MemoryManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        let app_data_dir = app_handle.path().app_data_dir()?;
        let db_path = app_data_dir.join("memory.db");

        // Initialize embedding model (downloads on first run)
        let model = TextEmbedding::try_new(InitOptions {
            model_name: EmbeddingModel::AllMiniLML6V2,
            show_download_progress: true,
            ..Default::default()
        })?;

        let manager = Self {
            db_path: db_path.clone(),
            embedding_model: Arc::new(Mutex::new(model)),
        };

        manager.init_database()?;

        Ok(manager)
    }

    fn init_database(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        
        // Create table for memories
        // We store the embedding as a BLOB (serialized vector)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                text TEXT NOT NULL,
                metadata TEXT NOT NULL,
                embedding BLOB NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub fn store_memory(&self, text: &str, metadata: &str) -> Result<i64> {
        let embedding = self.generate_embedding(text)?;
        let embedding_bytes = bincode::serialize(&embedding)?;
        let created_at = chrono::Utc::now().timestamp();

        let conn = Connection::open(&self.db_path)?;
        conn.execute(
            "INSERT INTO memories (text, metadata, embedding, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![text, metadata, embedding_bytes, created_at],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn search_memory(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let query_embedding = self.generate_embedding(query)?;
        let conn = Connection::open(&self.db_path)?;

        let mut stmt = conn.prepare("SELECT id, text, metadata, embedding, created_at FROM memories")?;
        let rows = stmt.query_map([], |row| {
            let embedding_bytes: Vec<u8> = row.get(3)?;
            let embedding: Vec<f32> = bincode::deserialize(&embedding_bytes)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Blob, Box::new(e)))?;
            
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                embedding,
                row.get::<_, i64>(4)?,
            ))
        })?;

        let mut memories = Vec::new();
        for row in rows {
            let (id, text, metadata, embedding, created_at) = row?;
            let score = cosine_similarity(&query_embedding, &embedding);
            memories.push(Memory {
                id,
                text,
                metadata,
                created_at,
                score,
            });
        }

        // Sort by score descending
        memories.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        memories.truncate(limit);

        Ok(memories)
    }

    fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let model = self.embedding_model.lock().unwrap();
        let embeddings = model.embed(vec![text], None)?;
        Ok(embeddings[0].clone())
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}
