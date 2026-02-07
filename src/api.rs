use std::collections::HashMap;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::db::Db;
use crate::llm::{self, count_nodes, LlmClient};
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

    // Auto-generate title if tree has 3+ nodes and no title yet
    let title = maybe_generate_title(&state, &id, &updated_tree).await;

    Ok(Json(ChatResponse {
        reply,
        tree: updated_tree,
        edges: updated_edges,
        title,
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

        let title = maybe_generate_title(&state, &id, &updated_tree).await;

        return Ok(Json(HeartbeatResponse {
            thinking,
            tree: updated_tree,
            edges: updated_edges,
            changed,
            results: vec![],
            title,
        }));
    }

    // Personality heartbeat: roll dice to pick how many speak
    let dice_sides = state
        .db
        .get_dice_sides(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Scope rng so it's dropped before any .await (ThreadRng is !Send)
    let mut selected = {
        let mut rng = rand::thread_rng();
        let roll: u32 = rng.gen_range(1..=dice_sides);
        let count = (roll as usize).min(active_personality_ids.len());
        let mut sel = active_personality_ids.clone();
        sel.shuffle(&mut rng);
        sel.truncate(count);
        sel
    };

    // Merge reserved agents (bonus slots from ask_agent questions)
    let reserved = state
        .db
        .get_reserved_agents(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    for agent in &reserved {
        if !selected.contains(agent) && active_personality_ids.contains(agent) {
            selected.push(agent.clone());
        }
    }

    // Build per-agent pending questions map
    let mut questions_map: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for agent_id in &selected {
        let pending = state
            .db
            .get_pending_questions_for(&id, agent_id)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if !pending.is_empty() {
            questions_map.insert(agent_id.clone(), pending);
        }
    }

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

        let title = maybe_generate_title(&state, &id, &updated_tree).await;

        return Ok(Json(HeartbeatResponse {
            thinking,
            tree: updated_tree,
            edges: updated_edges,
            changed,
            results: vec![],
            title,
        }));
    }

    // Fire parallel personality heartbeats
    let original_ids = llm::collect_node_ids(&doc.tree);
    let original_edges = doc.edges.clone();
    let empty_questions: Vec<(String, String)> = Vec::new();

    let futures: Vec<_> = personalities
        .iter()
        .map(|p| {
            let pq = questions_map.get(p.id).unwrap_or(&empty_questions);
            state.llm.personality_heartbeat(&doc.tree, &doc.edges, &messages, p, pq)
        })
        .collect();

    let outcomes = futures::future::join_all(futures).await;

    // Merge results
    let mut merged_tree = doc.tree.clone();
    let mut merged_edges = doc.edges.clone();
    let mut any_changed = false;
    let mut all_thinking_parts: Vec<String> = Vec::new();
    let mut per_personality_results: Vec<HeartbeatPersonalityResult> = Vec::new();
    let mut outgoing_questions: Vec<(&str, String, String)> = Vec::new();

    for (i, outcome) in outcomes.into_iter().enumerate() {
        let personality = personalities[i];
        match outcome {
            Ok((thinking, result_tree, result_edges, changed, questions)) => {
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

                // Collect outgoing questions from this agent
                for (to_agent, question) in questions {
                    outgoing_questions.push((personality.id, to_agent, question));
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

    // Expire questions that were consumed this tick
    let _ = state.db.expire_agent_questions(&id);

    // Insert outgoing questions for next heartbeat
    if !outgoing_questions.is_empty() {
        let q_refs: Vec<(&str, &str, &str)> = outgoing_questions
            .iter()
            .map(|(from, to, q)| (*from, to.as_str(), q.as_str()))
            .collect();
        let _ = state.db.insert_agent_questions(&id, &q_refs);
    }

    let combined_thinking = if all_thinking_parts.is_empty() {
        None
    } else {
        Some(all_thinking_parts.join("\n\n"))
    };

    let title = maybe_generate_title(&state, &id, &merged_tree).await;

    Ok(Json(HeartbeatResponse {
        thinking: combined_thinking,
        tree: merged_tree,
        edges: merged_edges,
        changed: any_changed,
        results: per_personality_results,
        title,
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

    let repel_force = state
        .db
        .get_repel_force(&id)
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
        repel_force,
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
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    if let Some(sides) = req.dice_sides {
        state
            .db
            .set_dice_sides(&id, sides)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    if let Some(force) = req.repel_force {
        state
            .db
            .set_repel_force(&id, force)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_summary(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<SummaryRequest>,
) -> Result<Json<SummaryResponse>, (StatusCode, String)> {
    let doc = state
        .db
        .get_document(&id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    let voice = req.voice.as_deref().unwrap_or("claude");
    let force_refresh = req.force_refresh.unwrap_or(false);

    // Hash the tree JSON for staleness detection
    let tree_json = serde_json::to_string(&doc.tree)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let tree_hash = format!("{:x}", md5::compute(&tree_json));

    // Check cache
    if !force_refresh {
        if let Some((content, cached_hash)) = state
            .db
            .get_summary(&id, voice)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        {
            return Ok(Json(SummaryResponse {
                content,
                voice: voice.to_string(),
                stale: cached_hash != tree_hash,
            }));
        }
    }

    // Generate summary via LLM
    let personality = if voice != "claude" {
        llm::get_personality(voice)
    } else {
        None
    };

    let content = state
        .llm
        .summarize(&doc.tree, &doc.edges, personality)
        .await
        .map_err(|e| {
            tracing::error!("Summary LLM error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, format!("LLM error: {e}"))
        })?;

    // Cache the result
    state
        .db
        .save_summary(&id, voice, &content, &tree_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(SummaryResponse {
        content,
        voice: voice.to_string(),
        stale: false,
    }))
}

/// If tree has 3+ nodes and no title exists yet, generate one and save it.
/// Returns the title (existing or newly generated) if available.
async fn maybe_generate_title(
    state: &Arc<AppState>,
    doc_id: &str,
    tree: &crate::models::TreeNode,
) -> Option<String> {
    // Check if title already exists
    if let Ok(Some(title)) = state.db.get_title(doc_id) {
        return Some(title);
    }
    // Only generate once tree has 3+ nodes
    if count_nodes(tree) < 3 {
        return None;
    }
    match state.llm.generate_title(tree).await {
        Ok(title) => {
            let _ = state.db.set_title(doc_id, &title);
            Some(title)
        }
        Err(e) => {
            tracing::error!("Title generation error: {}", e);
            None
        }
    }
}

fn generate_short_id() -> String {
    // Generate a short, URL-friendly ID (8 chars from uuid)
    uuid::Uuid::new_v4().to_string()[..8].to_string()
}
