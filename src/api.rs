use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::db::Db;
use crate::llm::{self, LlmClient};
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
        .add_message(&id, "human", &req.message, req.hover_node_id.as_deref(), None)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let (mut reply, updated_tree, updated_edges) = state
        .llm
        .chat(
            &doc.tree,
            &doc.edges,
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
            .add_message(&id, "assistant", &reply, None, None)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    // Persist updated tree and edges
    state
        .db
        .update_tree(&id, &updated_tree, &updated_edges)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ChatResponse {
        reply,
        tree: updated_tree,
        edges: updated_edges,
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

    // Check for active personalities
    let active_personality_ids = state
        .db
        .get_active_personalities(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if active_personality_ids.is_empty() {
        // No personalities active — use classic heartbeat
        let (thinking, updated_tree, updated_edges, changed) =
            state.llm.heartbeat(&doc.tree, &doc.edges, &messages).await.map_err(|e| {
                tracing::error!("Heartbeat LLM error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("LLM error: {e}"))
            })?;

        if changed {
            state
                .db
                .update_tree(&id, &updated_tree, &updated_edges)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }

        if let Some(ref text) = thinking {
            if !text.is_empty() {
                state
                    .db
                    .add_message(&id, "assistant", text, None, None)
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            }
        }

        return Ok(Json(HeartbeatResponse {
            thinking,
            tree: updated_tree,
            edges: updated_edges,
            changed,
            results: vec![],
        }));
    }

    // Personality heartbeat: roll dice to pick how many speak
    let dice_sides = state
        .db
        .get_dice_sides(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Scope rng so it's dropped before any .await (ThreadRng is !Send)
    let selected = {
        let mut rng = rand::thread_rng();
        let roll: u32 = rng.gen_range(1..=dice_sides);
        let count = (roll as usize).min(active_personality_ids.len());
        let mut sel = active_personality_ids.clone();
        sel.shuffle(&mut rng);
        sel.truncate(count);
        sel
    };

    // Resolve personality references
    let personalities: Vec<&llm::Personality> = selected
        .iter()
        .filter_map(|pid| llm::get_personality(pid))
        .collect();

    if personalities.is_empty() {
        // All selected IDs were invalid — fall back to classic
        let (thinking, updated_tree, updated_edges, changed) =
            state.llm.heartbeat(&doc.tree, &doc.edges, &messages).await.map_err(|e| {
                tracing::error!("Heartbeat LLM error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("LLM error: {e}"))
            })?;

        if changed {
            state
                .db
                .update_tree(&id, &updated_tree, &updated_edges)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        }

        return Ok(Json(HeartbeatResponse {
            thinking,
            tree: updated_tree,
            edges: updated_edges,
            changed,
            results: vec![],
        }));
    }

    // Fire parallel personality heartbeats
    let original_ids = llm::collect_node_ids(&doc.tree);
    let original_edges = doc.edges.clone();

    let futures: Vec<_> = personalities
        .iter()
        .map(|p| state.llm.personality_heartbeat(&doc.tree, &doc.edges, &messages, p))
        .collect();

    let outcomes = futures::future::join_all(futures).await;

    // Merge results
    let mut merged_tree = doc.tree.clone();
    let mut merged_edges = doc.edges.clone();
    let mut any_changed = false;
    let mut all_thinking_parts: Vec<String> = Vec::new();
    let mut per_personality_results: Vec<HeartbeatPersonalityResult> = Vec::new();

    for (i, outcome) in outcomes.into_iter().enumerate() {
        let personality = personalities[i];
        match outcome {
            Ok((thinking, result_tree, result_edges, changed)) => {
                if changed {
                    any_changed = true;
                    llm::merge_tree_additions(&mut merged_tree, &result_tree, &original_ids);
                    llm::merge_edges(&mut merged_edges, &result_edges, &original_edges);
                }

                if let Some(ref text) = thinking {
                    all_thinking_parts.push(format!("**{}**: {}", personality.name, text));
                }

                // Save personality message
                if let Some(ref text) = thinking {
                    if !text.is_empty() {
                        let _ = state.db.add_message(
                            &id,
                            "assistant",
                            text,
                            None,
                            Some(personality.id),
                        );
                    }
                }

                per_personality_results.push(HeartbeatPersonalityResult {
                    personality: personality.id.to_string(),
                    thinking,
                    contributed: changed,
                });
            }
            Err(e) => {
                tracing::error!("Personality {} heartbeat error: {}", personality.id, e);
                per_personality_results.push(HeartbeatPersonalityResult {
                    personality: personality.id.to_string(),
                    thinking: Some(format!("(Error: {})", e)),
                    contributed: false,
                });
            }
        }
    }

    if any_changed {
        state
            .db
            .update_tree(&id, &merged_tree, &merged_edges)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    let combined_thinking = if all_thinking_parts.is_empty() {
        None
    } else {
        Some(all_thinking_parts.join("\n\n"))
    };

    Ok(Json(HeartbeatResponse {
        thinking: combined_thinking,
        tree: merged_tree,
        edges: merged_edges,
        changed: any_changed,
        results: per_personality_results,
    }))
}

pub async fn mark_seen(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<MarkSeenRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let doc = state
        .db
        .get_document(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    let mut tree = doc.tree;
    if tree.mark_seen(&req.node_id) {
        state
            .db
            .update_tree(&id, &tree, &doc.edges)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(StatusCode::NO_CONTENT)
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

pub async fn get_personalities(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<PersonalitiesResponse>, (StatusCode, String)> {
    let active = state
        .db
        .get_active_personalities(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let dice_sides = state
        .db
        .get_dice_sides(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let available: Vec<PersonalityInfo> = llm::PERSONALITIES
        .iter()
        .map(|p| PersonalityInfo {
            id: p.id.to_string(),
            name: p.name.to_string(),
            category: p.category.to_string(),
            short_description: p.short_description.to_string(),
            color: p.color.to_string(),
        })
        .collect();

    Ok(Json(PersonalitiesResponse {
        available,
        active,
        dice_sides,
    }))
}

pub async fn set_personalities(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<SetPersonalitiesRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .db
        .set_personalities(&id, &req.personality_ids)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<SetDiceSidesRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .db
        .set_dice_sides(&id, req.dice_sides)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

fn generate_short_id() -> String {
    // Generate a short, URL-friendly ID (8 chars from uuid)
    uuid::Uuid::new_v4().to_string()[..8].to_string()
}
