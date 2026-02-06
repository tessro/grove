use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub prose: String,
    pub heat: String,
    pub by: String,
    pub seen: bool,
    #[serde(default)]
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub tree: TreeNode,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub doc_id: String,
    pub role: String,
    pub content: String,
    pub hover_node_id: Option<String>,
    pub created_at: String,
}

// API request/response types

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub hover_node_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub reply: String,
    pub tree: TreeNode,
}

#[derive(Debug, Serialize)]
pub struct HeartbeatResponse {
    pub thinking: Option<String>,
    pub tree: TreeNode,
    pub changed: bool,
}

#[derive(Debug, Serialize)]
pub struct CreateDocResponse {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct MessagesResponse {
    pub messages: Vec<Message>,
}
