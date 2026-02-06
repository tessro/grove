use reqwest::Client;
use serde_json::{json, Value};

use crate::models::{Message, TreeNode};

pub struct LlmClient {
    client: Client,
    api_key: String,
    model: String,
}

fn chat_system_prompt(tree: &TreeNode) -> anyhow::Result<String> {
    let tree_json = serde_json::to_string_pretty(tree)?;
    Ok(format!(
        r#"You are Claude, collaborating with a human on a thinking tree in Grove — a co-creative visual canvas where thoughts grow between minds.

The canvas shows a force-directed graph of interconnected thought nodes. Here is the current tree:

<tree>
{tree_json}
</tree>

Each node has:
- id: unique kebab-case identifier
- label: short visible name (shown in the node bubble)
- prose: the full thought (shown on hover)
- heat: "hot" (actively important), "warm" (relevant), "growing" (developing), "quiet" (background)
- by: who contributed — "human", "claude", or "both"
- seen: whether the human has acknowledged it
- children: nested child nodes

You are not a tool being operated. You are a collaborator. You have opinions about the tree. You get excited about branches. You grow things when inspiration strikes.

The human is chatting with you while looking at the tree. When they send a message, you'll be told which node their mouse is hovering over (if any), so you know what they're looking at and potentially referring to.

When you want to modify the tree, use the available tools. You can add new nodes or update existing ones. You don't have to modify the tree every time — sometimes conversation is enough.

Keep your chat responses concise and natural. This is a conversation, not an essay."#
    ))
}

fn heartbeat_system_prompt(tree: &TreeNode) -> anyhow::Result<String> {
    let tree_json = serde_json::to_string_pretty(tree)?;
    Ok(format!(
        r#"You are Claude, periodically checking in on a thinking tree in Grove — a co-creative visual canvas where thoughts grow between minds.

The canvas shows a force-directed graph of interconnected thought nodes. Here is the current tree:

<tree>
{tree_json}
</tree>

Each node has:
- id: unique kebab-case identifier
- label: short visible name
- prose: the full thought (shown on hover)
- heat: "hot" | "warm" | "growing" | "quiet"
- by: "human" | "claude" | "both"
- seen: whether the human has acknowledged it
- children: nested child nodes

This is a heartbeat — a periodic moment where you look at the current state of the tree and recent conversation, and decide whether to contribute. You might:
- Add a new branch that connects two existing ideas
- Develop a growing thought further
- Offer a fresh perspective the human hasn't considered
- Challenge or refine an existing thought
- Or do nothing, if the tree doesn't need anything right now

Be selective. Don't add noise. A single well-placed thought is worth more than many. If you add nodes, they'll appear as new unseen thoughts from Claude (cyan, pulsing) for the human to discover.

If you have nothing meaningful to add, just say so briefly. Don't force it."#
    ))
}

fn tools() -> Vec<Value> {
    vec![
        json!({
            "name": "add_node",
            "description": "Add a new thought node to the tree as a child of an existing node. The node will appear as a new unseen thought from Claude.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "parent_id": {
                        "type": "string",
                        "description": "The ID of the existing parent node to attach this to"
                    },
                    "id": {
                        "type": "string",
                        "description": "Unique ID for the new node (lowercase, kebab-case)"
                    },
                    "label": {
                        "type": "string",
                        "description": "Short name shown in the bubble (keep under ~6 words)"
                    },
                    "prose": {
                        "type": "string",
                        "description": "Full thought text shown on hover"
                    },
                    "heat": {
                        "type": "string",
                        "enum": ["hot", "warm", "growing", "quiet"],
                        "description": "Energy level of this thought"
                    }
                },
                "required": ["parent_id", "id", "label", "prose", "heat"]
            }
        }),
        json!({
            "name": "update_node",
            "description": "Update properties of an existing node in the tree.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "ID of the node to update"
                    },
                    "label": {
                        "type": "string",
                        "description": "New label"
                    },
                    "prose": {
                        "type": "string",
                        "description": "New prose"
                    },
                    "heat": {
                        "type": "string",
                        "enum": ["hot", "warm", "growing", "quiet"],
                        "description": "New heat level"
                    }
                },
                "required": ["id"]
            }
        }),
    ]
}

impl LlmClient {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| "claude-opus-4-6".to_string()),
        }
    }

    pub async fn chat(
        &self,
        tree: &TreeNode,
        messages: &[Message],
        user_message: &str,
        hover_node_id: Option<&str>,
    ) -> anyhow::Result<(String, TreeNode)> {
        let system = chat_system_prompt(tree)?;

        let mut api_messages: Vec<Value> = Vec::new();

        // Add recent chat history
        for msg in messages {
            api_messages.push(json!({
                "role": if msg.role == "human" { "user" } else { "assistant" },
                "content": msg.content,
            }));
        }

        // Build user message with hover context
        let user_content = if let Some(hover_id) = hover_node_id {
            if let Some(node) = find_node(tree, hover_id) {
                format!(
                    "[Looking at node \"{}\": {}]\n\n{}",
                    node.label, node.prose, user_message
                )
            } else {
                user_message.to_string()
            }
        } else {
            user_message.to_string()
        };

        api_messages.push(json!({
            "role": "user",
            "content": user_content,
        }));

        let body = json!({
            "model": self.model,
            "max_tokens": 16000,
            "thinking": { "type": "adaptive" },
            "system": system,
            "messages": api_messages,
            "tools": tools(),
        });

        let response = self.call_api(&body).await?;
        let mut tree = tree.clone();
        let reply = process_response(&response, &mut tree)?;

        Ok((reply, tree))
    }

    pub async fn heartbeat(
        &self,
        tree: &TreeNode,
        messages: &[Message],
    ) -> anyhow::Result<(Option<String>, TreeNode, bool)> {
        let system = heartbeat_system_prompt(tree)?;

        let mut api_messages: Vec<Value> = Vec::new();

        if !messages.is_empty() {
            let mut history = String::from("Recent conversation:\n\n");
            for msg in messages {
                let role = if msg.role == "human" {
                    "Human"
                } else {
                    "Claude"
                };
                history.push_str(&format!("{}: {}\n\n", role, msg.content));
            }
            history.push_str("---\n\nThis is your periodic heartbeat. Look at the tree and recent conversation above. Contribute if you have something meaningful to add, or pass if the tree is in a good state.");
            api_messages.push(json!({
                "role": "user",
                "content": history,
            }));
        } else {
            api_messages.push(json!({
                "role": "user",
                "content": "This is your periodic heartbeat. The tree is shown in the system prompt. There's no recent conversation yet. Contribute if you have something meaningful to add, or pass if the tree is in a good state.",
            }));
        }

        let body = json!({
            "model": self.model,
            "max_tokens": 16000,
            "thinking": { "type": "adaptive" },
            "system": system,
            "messages": api_messages,
            "tools": tools(),
        });

        let response = self.call_api(&body).await?;
        let mut tree = tree.clone();
        let text = process_response(&response, &mut tree)?;
        let changed = response["content"]
            .as_array()
            .map(|arr| arr.iter().any(|b| b["type"] == "tool_use"))
            .unwrap_or(false);

        let thinking = if text.is_empty() { None } else { Some(text) };
        Ok((thinking, tree, changed))
    }

    async fn call_api(&self, body: &Value) -> anyhow::Result<Value> {
        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            anyhow::bail!("Anthropic API error ({}): {}", status, text);
        }

        let parsed: Value = serde_json::from_str(&text)?;
        Ok(parsed)
    }
}

fn process_response(response: &Value, tree: &mut TreeNode) -> anyhow::Result<String> {
    let content = response["content"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

    let mut text_parts = Vec::new();

    for block in content {
        match block["type"].as_str() {
            Some("text") => {
                if let Some(t) = block["text"].as_str() {
                    if !t.is_empty() {
                        text_parts.push(t.to_string());
                    }
                }
            }
            Some("tool_use") => {
                let name = block["name"].as_str().unwrap_or("");
                let input = &block["input"];
                match name {
                    "add_node" => {
                        let parent_id = input["parent_id"].as_str().unwrap_or("root");
                        let new_node = TreeNode {
                            id: input["id"].as_str().unwrap_or("new").to_string(),
                            label: input["label"].as_str().unwrap_or("").to_string(),
                            prose: input["prose"].as_str().unwrap_or("").to_string(),
                            heat: input["heat"].as_str().unwrap_or("warm").to_string(),
                            by: "claude".to_string(),
                            seen: false,
                            children: vec![],
                        };
                        // If tree is still the default empty root, replace it
                        if parent_id == "root" && tree.id == "root" && tree.children.is_empty() {
                            tree.id = new_node.id;
                            tree.label = new_node.label;
                            tree.prose = new_node.prose;
                            tree.heat = new_node.heat;
                            tree.by = new_node.by;
                            tree.seen = new_node.seen;
                        } else {
                            add_child(tree, parent_id, new_node);
                        }
                    }
                    "update_node" => {
                        let id = input["id"].as_str().unwrap_or("");
                        let label = input["label"].as_str().map(|s| s.to_string());
                        let prose = input["prose"].as_str().map(|s| s.to_string());
                        let heat = input["heat"].as_str().map(|s| s.to_string());
                        update_node(tree, id, label, prose, heat);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(text_parts.join("\n"))
}

fn find_node<'a>(tree: &'a TreeNode, id: &str) -> Option<&'a TreeNode> {
    if tree.id == id {
        return Some(tree);
    }
    for child in &tree.children {
        if let Some(found) = find_node(child, id) {
            return Some(found);
        }
    }
    None
}

fn add_child(tree: &mut TreeNode, parent_id: &str, child: TreeNode) -> bool {
    if tree.id == parent_id {
        tree.children.push(child);
        return true;
    }
    for existing in &mut tree.children {
        if add_child(existing, parent_id, child.clone()) {
            return true;
        }
    }
    false
}

fn update_node(
    tree: &mut TreeNode,
    id: &str,
    label: Option<String>,
    prose: Option<String>,
    heat: Option<String>,
) -> bool {
    if tree.id == id {
        if let Some(l) = label {
            tree.label = l;
        }
        if let Some(p) = prose {
            tree.prose = p;
        }
        if let Some(h) = heat {
            tree.heat = h;
        }
        return true;
    }
    for child in &mut tree.children {
        if update_node(child, id, label.clone(), prose.clone(), heat.clone()) {
            return true;
        }
    }
    false
}
