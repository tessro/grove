use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub label: String,
}

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

impl TreeNode {
    /// Recursively find a node by ID and set `seen = true`. Returns whether it was found.
    pub fn mark_seen(&mut self, node_id: &str) -> bool {
        if self.id == node_id {
            self.seen = true;
            return true;
        }
        for child in &mut self.children {
            if child.mark_seen(node_id) {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub tree: TreeNode,
    #[serde(default)]
    pub edges: Vec<Edge>,
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
    pub personality: Option<String>,
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
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeartbeatPersonalityResult {
    pub personality: String,
    pub thinking: Option<String>,
    pub contributed: bool,
}

#[derive(Debug, Serialize)]
pub struct HeartbeatResponse {
    pub thinking: Option<String>,
    pub tree: TreeNode,
    pub edges: Vec<Edge>,
    pub changed: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub results: Vec<HeartbeatPersonalityResult>,
}

#[derive(Debug, Serialize)]
pub struct CreateDocResponse {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct MessagesResponse {
    pub messages: Vec<Message>,
}

#[derive(Debug, Deserialize)]
pub struct MarkSeenRequest {
    pub node_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SetPersonalitiesRequest {
    pub personality_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub dice_sides: Option<u32>,
    pub repel_force: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct PersonalityInfo {
    pub id: String,
    pub name: String,
    pub category: String,
    pub short_description: String,
    pub color: String,
}

#[derive(Debug, Serialize)]
pub struct PersonalitiesResponse {
    pub available: Vec<PersonalityInfo>,
    pub active: Vec<String>,
    pub dice_sides: u32,
    pub repel_force: f64,
}

#[derive(Debug, Deserialize)]
pub struct SummaryRequest {
    pub voice: Option<String>,
    pub force_refresh: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SummaryResponse {
    pub content: String,
    pub voice: String,
    pub stale: bool,
}
