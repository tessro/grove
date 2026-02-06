use std::sync::Mutex;

use rusqlite::{params, Connection};

use crate::models::{Document, Message, TreeNode};

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
            );",
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
            "INSERT INTO documents (id, tree) VALUES (?1, ?2)",
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

    pub fn update_tree(&self, id: &str, tree: &TreeNode) -> anyhow::Result<()> {
        let tree_json = serde_json::to_string(tree)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE documents SET tree = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![tree_json, id],
        )?;
        Ok(())
    }

    pub fn add_message(
        &self,
        doc_id: &str,
        role: &str,
        content: &str,
        hover_node_id: Option<&str>,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (doc_id, role, content, hover_node_id) VALUES (?1, ?2, ?3, ?4)",
            params![doc_id, role, content, hover_node_id],
        )?;
        Ok(())
    }

    pub fn get_messages(&self, doc_id: &str, limit: usize) -> anyhow::Result<Vec<Message>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, doc_id, role, content, hover_node_id, created_at
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
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        messages.reverse(); // chronological order
        Ok(messages)
    }
}

fn get_document_inner(conn: &Connection, id: &str) -> anyhow::Result<Document> {
    let mut stmt =
        conn.prepare("SELECT id, tree, created_at, updated_at FROM documents WHERE id = ?1")?;
    let doc = stmt.query_row(params![id], |row| {
        let tree_str: String = row.get(1)?;
        Ok(Document {
            id: row.get(0)?,
            tree: serde_json::from_str(&tree_str).unwrap(),
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
