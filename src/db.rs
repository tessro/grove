use std::sync::Mutex;

use rusqlite::{params, Connection};

use crate::models::{Document, Edge, Message, TreeNode};

pub struct Db {
    conn: Mutex<Connection>,
}

impl Db {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                tree TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                doc_id TEXT NOT NULL REFERENCES documents(id),
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                hover_node_id TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS doc_personalities (
                doc_id TEXT NOT NULL,
                personality_id TEXT NOT NULL,
                PRIMARY KEY (doc_id, personality_id)
            );
            CREATE TABLE IF NOT EXISTS doc_settings (
                doc_id TEXT PRIMARY KEY,
                heartbeat_dice_sides INTEGER NOT NULL DEFAULT 3
            );",
        )?;
        // Migration: add personality column to messages if missing
        let has_personality: bool = conn
            .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='messages'")?
            .query_row([], |row| {
                let sql: String = row.get(0)?;
                Ok(sql.contains("personality"))
            })
            .unwrap_or(false);
        if !has_personality {
            conn.execute_batch("ALTER TABLE messages ADD COLUMN personality TEXT")?;
        }
        // Migration: add edges column to documents if missing
        let has_edges: bool = conn
            .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='documents'")?
            .query_row([], |row| {
                let sql: String = row.get(0)?;
                Ok(sql.contains("edges"))
            })
            .unwrap_or(false);
        if !has_edges {
            conn.execute_batch("ALTER TABLE documents ADD COLUMN edges TEXT NOT NULL DEFAULT '[]'")?;
        }
        // Migration: add repel_force column to doc_settings if missing
        let has_repel_force: bool = conn
            .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name='doc_settings'")?
            .query_row([], |row| {
                let sql: String = row.get(0)?;
                Ok(sql.contains("repel_force"))
            })
            .unwrap_or(false);
        if !has_repel_force {
            conn.execute_batch(
                "ALTER TABLE doc_settings ADD COLUMN repel_force REAL NOT NULL DEFAULT 20.0",
            )?;
        }
        // Create doc_summaries table
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS doc_summaries (
                doc_id TEXT NOT NULL,
                voice TEXT NOT NULL,
                content TEXT NOT NULL,
                tree_hash TEXT NOT NULL,
                PRIMARY KEY (doc_id, voice)
            )",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn create_document(&self, id: &str) -> anyhow::Result<Document> {
        let tree = default_tree();
        let tree_json = serde_json::to_string(&tree)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO documents (id, tree, edges) VALUES (?1, ?2, '[]')",
            params![id, tree_json],
        )?;
        let doc = get_document_inner(&conn, id)?;
        Ok(doc)
    }

    pub fn get_document(&self, id: &str) -> anyhow::Result<Option<Document>> {
        let conn = self.conn.lock().unwrap();
        match get_document_inner(&conn, id) {
            Ok(doc) => Ok(Some(doc)),
            Err(_) => Ok(None),
        }
    }

    pub fn update_tree(&self, id: &str, tree: &TreeNode, edges: &[Edge]) -> anyhow::Result<()> {
        let tree_json = serde_json::to_string(tree)?;
        let edges_json = serde_json::to_string(edges)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE documents SET tree = ?1, edges = ?2, updated_at = datetime('now') WHERE id = ?3",
            params![tree_json, edges_json, id],
        )?;
        Ok(())
    }

    pub fn add_message(
        &self,
        doc_id: &str,
        role: &str,
        content: &str,
        hover_node_id: Option<&str>,
        personality: Option<&str>,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (doc_id, role, content, hover_node_id, personality) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![doc_id, role, content, hover_node_id, personality],
        )?;
        Ok(())
    }

    pub fn get_messages(&self, doc_id: &str, limit: usize) -> anyhow::Result<Vec<Message>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, doc_id, role, content, hover_node_id, personality, created_at
             FROM messages WHERE doc_id = ?1
             ORDER BY created_at DESC LIMIT ?2",
        )?;
        let mut messages = stmt
            .query_map(params![doc_id, limit as i64], |row| {
                Ok(Message {
                    id: row.get(0)?,
                    doc_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    hover_node_id: row.get(4)?,
                    personality: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        messages.reverse(); // chronological order
        Ok(messages)
    }

    pub fn get_active_personalities(&self, doc_id: &str) -> anyhow::Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT personality_id FROM doc_personalities WHERE doc_id = ?1 ORDER BY personality_id",
        )?;
        let ids = stmt
            .query_map(params![doc_id], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(ids)
    }

    pub fn set_personalities(&self, doc_id: &str, personality_ids: &[String]) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM doc_personalities WHERE doc_id = ?1",
            params![doc_id],
        )?;
        for pid in personality_ids {
            conn.execute(
                "INSERT INTO doc_personalities (doc_id, personality_id) VALUES (?1, ?2)",
                params![doc_id, pid],
            )?;
        }
        Ok(())
    }

    pub fn get_dice_sides(&self, doc_id: &str) -> anyhow::Result<u32> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT heartbeat_dice_sides FROM doc_settings WHERE doc_id = ?1",
            params![doc_id],
            |row| row.get(0),
        );
        match result {
            Ok(sides) => Ok(sides),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(3),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_dice_sides(&self, doc_id: &str, sides: u32) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO doc_settings (doc_id, heartbeat_dice_sides) VALUES (?1, ?2)
             ON CONFLICT(doc_id) DO UPDATE SET heartbeat_dice_sides = ?2",
            params![doc_id, sides],
        )?;
        Ok(())
    }

    pub fn get_repel_force(&self, doc_id: &str) -> anyhow::Result<f64> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT repel_force FROM doc_settings WHERE doc_id = ?1",
            params![doc_id],
            |row| row.get(0),
        );
        match result {
            Ok(force) => Ok(force),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(20.0),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_repel_force(&self, doc_id: &str, force: f64) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO doc_settings (doc_id, heartbeat_dice_sides, repel_force) VALUES (?1, 3, ?2)
             ON CONFLICT(doc_id) DO UPDATE SET repel_force = ?2",
            params![doc_id, force],
        )?;
        Ok(())
    }

    pub fn get_summary(&self, doc_id: &str, voice: &str) -> anyhow::Result<Option<(String, String)>> {
        let conn = self.conn.lock().unwrap();
        let result = conn.query_row(
            "SELECT content, tree_hash FROM doc_summaries WHERE doc_id = ?1 AND voice = ?2",
            params![doc_id, voice],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        );
        match result {
            Ok(pair) => Ok(Some(pair)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn save_summary(&self, doc_id: &str, voice: &str, content: &str, tree_hash: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO doc_summaries (doc_id, voice, content, tree_hash) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(doc_id, voice) DO UPDATE SET content = ?3, tree_hash = ?4",
            params![doc_id, voice, content, tree_hash],
        )?;
        Ok(())
    }
}

fn get_document_inner(conn: &Connection, id: &str) -> anyhow::Result<Document> {
    let mut stmt =
        conn.prepare("SELECT id, tree, created_at, updated_at, edges FROM documents WHERE id = ?1")?;
    let doc = stmt.query_row(params![id], |row| {
        let tree_str: String = row.get(1)?;
        let edges_str: String = row.get::<_, String>(4).unwrap_or_else(|_| "[]".to_string());
        Ok(Document {
            id: row.get(0)?,
            tree: serde_json::from_str(&tree_str).unwrap(),
            edges: serde_json::from_str(&edges_str).unwrap_or_default(),
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })?;
    Ok(doc)
}

fn default_tree() -> TreeNode {
    TreeNode {
        id: "root".to_string(),
        label: "New grove".to_string(),
        prose: "A fresh space for thinking together. Share what's on your mind.".to_string(),
        heat: "warm".to_string(),
        by: "system".to_string(),
        seen: true,
        children: vec![],
    }
}
