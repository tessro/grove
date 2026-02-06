use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::db::Db;
use crate::llm::LlmClient;
use crate::models::*;

pub struct AppState {
    pub db: Db,
    pub llm: LlmClient,
}

pub async fn create_doc(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CreateDocResponse>, (StatusCode, String)> {
    let id = generate_short_id();
    state
        .db
        .create_document(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(CreateDocResponse { id }))
}

pub async fn get_doc(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Document>, StatusCode> {
    match state.db.get_document(&id) {
        Ok(Some(doc)) => Ok(Json(doc)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn chat(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (StatusCode, String)> {
    let doc = state
        .db
        .get_document(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    let messages = state
        .db
        .get_messages(&id, 50)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Save user message first
    state
        .db
        .add_message(&id, "human", &req.message, req.hover_node_id.as_deref())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (mut reply, updated_tree) = state
        .llm
        .chat(
            &doc.tree,
            &messages,
            &req.message,
            req.hover_node_id.as_deref(),
        )
        .await
        .map_err(|e| {
            tracing::error!("LLM chat error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("LLM error: {e}"))
        })?;

    // If Claude only used tools and didn't write text, note that the tree changed
    if reply.is_empty() && updated_tree.children.len() != doc.tree.children.len() {
        reply = "(Added new thoughts to the tree.)".to_string();
    }

    // Save assistant reply
    if !reply.is_empty() {
        state
            .db
            .add_message(&id, "assistant", &reply, None)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    // Persist updated tree
    state
        .db
        .update_tree(&id, &updated_tree)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ChatResponse {
        reply,
        tree: updated_tree,
    }))
}

pub async fn heartbeat(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<HeartbeatResponse>, (StatusCode, String)> {
    let doc = state
        .db
        .get_document(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    let messages = state
        .db
        .get_messages(&id, 20)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (thinking, updated_tree, changed) =
        state.llm.heartbeat(&doc.tree, &messages).await.map_err(|e| {
            tracing::error!("Heartbeat LLM error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("LLM error: {e}"))
        })?;

    if changed {
        state
            .db
            .update_tree(&id, &updated_tree)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    // Save heartbeat thinking as a message if present
    if let Some(ref text) = thinking {
        if !text.is_empty() {
            state
                .db
                .add_message(&id, "assistant", text, None)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }
    }

    Ok(Json(HeartbeatResponse {
        thinking,
        tree: updated_tree,
        changed,
    }))
}

pub async fn get_messages(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<MessagesResponse>, StatusCode> {
    let messages = state
        .db
        .get_messages(&id, 100)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(MessagesResponse { messages }))
}

fn generate_short_id() -> String {
    // Generate a short, URL-friendly ID (8 chars from uuid)
    uuid::Uuid::new_v4().to_string()[..8].to_string()
}
