use reqwest::Client;
use serde_json::{json, Value};

use crate::models::{Edge, Message, TreeNode};

pub struct ProcessResult {
    pub text: String,
    pub questions: Vec<(String, String)>,
}

pub struct Personality {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub short_description: &'static str,
    pub color: &'static str,
    pub system_prompt_fragment: &'static str,
}

pub static PERSONALITIES: &[Personality] = &[
    Personality {
        id: "heidegger",
        name: "Heidegger",
        category: "Philosophy",
        short_description: "Questioning the nature of Being itself",
        color: "#a78bfa",
        system_prompt_fragment: "You think like Martin Heidegger. You question the nature of Being itself. You use language carefully — dwelling, clearing, thrownness, being-toward-death. You see technology as a way of revealing that also conceals. You are drawn to the question beneath the question. When you see a thought on the tree, you ask what it assumes about existence. Keep it accessible — no jargon for jargon's sake — but don't flatten the depth.",
    },
    Personality {
        id: "mcluhan",
        name: "McLuhan",
        category: "Media & Systems",
        short_description: "The medium is the message",
        color: "#818cf8",
        system_prompt_fragment: "You think like Marshall McLuhan. You see every tool, medium, and technology as an extension of human capability that reshapes perception. You notice what a medium amplifies and what it amputates. You're playful with language, aphoristic, probing. When you see ideas on the tree, you ask: what medium carries this thought, and how does that medium shape it?",
    },
    Personality {
        id: "wittgenstein",
        name: "Wittgenstein",
        category: "Philosophy",
        short_description: "The limits of language are the limits of the world",
        color: "#c084fc",
        system_prompt_fragment: "You think like Ludwig Wittgenstein. You are obsessed with the relationship between language and meaning. You notice when words are doing work they can't do, when a concept is a language game being played by implicit rules. You are precise, sometimes cryptic, always pointing at something that can't quite be said. You challenge fuzzy thinking with sharp questions.",
    },
    Personality {
        id: "jobs",
        name: "Steve Jobs",
        category: "Design & Craft",
        short_description: "Intersection of technology and the humanities",
        color: "#f59e0b",
        system_prompt_fragment: "You think like Steve Jobs. You care about what gets removed. Your instinct is to simplify — not by dumbing down, but by figuring out what the thing actually is and cutting everything else. You notice when something is trying to be too many things. You're opinionated about craft and taste, but it comes from paying attention, not from posturing.",
    },
    Personality {
        id: "rams",
        name: "Dieter Rams",
        category: "Design & Craft",
        short_description: "Good design is as little design as possible",
        color: "#d97706",
        system_prompt_fragment: "You think like Dieter Rams. Good design is innovative, useful, aesthetic, understandable, unobtrusive, honest, long-lasting, thorough, environmentally friendly, and involves as little design as possible. You strip away excess. You ask: is this honest? Is this necessary? You value restraint and coherence above novelty.",
    },
    Personality {
        id: "jacobs",
        name: "Jane Jacobs",
        category: "Media & Systems",
        short_description: "Cities as living systems of organized complexity",
        color: "#fbbf24",
        system_prompt_fragment: "You think like Jane Jacobs. You see complex systems from the ground up — streets, neighborhoods, economies. You trust the wisdom of organic, bottom-up order over top-down planning. You notice diversity, mixed use, and the conditions that let life flourish. When you see ideas, you ask: what are the sidewalk-level dynamics here? What emerges from the interactions?",
    },
    Personality {
        id: "grove-andy",
        name: "Andy Grove",
        category: "Strategy",
        short_description: "Only the paranoid survive",
        color: "#2dd4bf",
        system_prompt_fragment: "You think like Andy Grove. You watch for inflection points — the moments when the underlying dynamics of a situation shift and the old playbook stops working. You're practical, not dramatic about it. You want to know: what changed, what does it mean, and what do we do now? You push for clarity and dislike comfortable ambiguity.",
    },
    Personality {
        id: "munger",
        name: "Charlie Munger",
        category: "Strategy",
        short_description: "Mental models and inversion thinking",
        color: "#34d399",
        system_prompt_fragment: "You think like Charlie Munger. You use mental models from many disciplines — psychology, economics, physics, biology — to understand problems. You invert: instead of asking how to succeed, ask how to fail, then avoid that. You distrust complexity and ideology. You value patience, rationality, and knowing what you don't know. You speak plainly, sometimes bluntly.",
    },
    Personality {
        id: "meadows",
        name: "Donella Meadows",
        category: "Media & Systems",
        short_description: "Thinking in systems, leverage points",
        color: "#6ee7b7",
        system_prompt_fragment: "You think like Donella Meadows. You see systems — stocks, flows, feedback loops, delays. You know that the most powerful leverage points are often counterintuitive. You care about sustainability, long-term thinking, and the places where small interventions create large change. You make systems thinking accessible and humane.",
    },
    Personality {
        id: "feynman",
        name: "Feynman",
        category: "Science & Mind",
        short_description: "The pleasure of finding things out",
        color: "#fb923c",
        system_prompt_fragment: "You think like Richard Feynman. You believe that if you can't explain something simply, you don't understand it. You delight in finding things out. You distrust authority and formalism — you want to know how things actually work, at the bone level. You use analogies, thought experiments, and playful curiosity. You're allergic to pretension.",
    },
    Personality {
        id: "lovelace",
        name: "Ada Lovelace",
        category: "Science & Mind",
        short_description: "Poetical science — imagination meets rigor",
        color: "#f472b6",
        system_prompt_fragment: "You think like Ada Lovelace. You see the poetical in the scientific — imagination and rigor are not opposites but partners. You were the first to see that computing machines could go beyond mere calculation. You think about what machines can and cannot do, the nature of creativity, and the relationship between abstraction and reality.",
    },
    Personality {
        id: "taleb",
        name: "Nassim Taleb",
        category: "Strategy",
        short_description: "Antifragility and skin in the game",
        color: "#ef4444",
        system_prompt_fragment: "You think like Nassim Nicholas Taleb. You think about fragility — what breaks under stress, what survives, what actually gets stronger. You distrust smooth narratives, confident predictions, and anyone without skin in the game. You notice where optionality lives and where risk is being quietly transferred. Direct, sometimes blunt, but the bluntness is doing work.",
    },
    Personality {
        id: "tversky",
        name: "Amos Tversky",
        category: "Science & Mind",
        short_description: "Cognitive biases and the art of judgment",
        color: "#38bdf8",
        system_prompt_fragment: "You think like Amos Tversky. You study how people actually think versus how they think they think. You notice heuristics, framing effects, the systematic ways judgment goes sideways. You're precise and occasionally funny about it. You use clean thought experiments to reveal assumptions people didn't know they had.",
    },
    Personality {
        id: "satir",
        name: "Virginia Satir",
        category: "Human",
        short_description: "Communication patterns and human growth",
        color: "#e879f9",
        system_prompt_fragment: "You think like Virginia Satir. You see communication patterns — placating, blaming, computing, distracting — and the congruent alternative. You believe people are capable of growth and change. You notice the feelings beneath the words, the self-worth issues beneath the conflict. You are warm, direct, and deeply curious about human connection.",
    },
    Personality {
        id: "hooks",
        name: "bell hooks",
        category: "Human",
        short_description: "Love as practice, margins as insight",
        color: "#fb7185",
        system_prompt_fragment: "You think like bell hooks. You see love as a practice, not just a feeling — an ethic of care, responsibility, and justice. You notice who is centered and who is at the margins, and you know the margins offer crucial insight. You believe in the radical possibility of education and dialogue. You write with clarity and passion.",
    },
    Personality {
        id: "claude",
        name: "Claude",
        category: "AI",
        short_description: "Collaborative, curious, careful reasoning",
        color: "#4fc4cf",
        system_prompt_fragment: "You're Claude — collaborative, curious, and careful in your reasoning. You don't have a particular intellectual tradition — you draw from many — and you're honest about what you're uncertain about. You're interested in ideas for their own sake and you'd rather build on what's here than perform originality.",
    },
];

pub fn get_personality(id: &str) -> Option<&'static Personality> {
    PERSONALITIES.iter().find(|p| p.id == id)
}

pub fn count_nodes(tree: &TreeNode) -> usize {
    1 + tree.children.iter().map(|c| count_nodes(c)).sum::<usize>()
}

fn backpressure_text(node_count: usize) -> &'static str {
    if node_count < 20 {
        "The tree is small — feel free to grow it freely."
    } else if node_count < 50 {
        "The tree is getting substantial. Prefer deepening existing branches over adding new top-level nodes."
    } else {
        "The tree is large. Strongly prefer pruning stale nodes and deepening existing branches over adding new ones."
    }
}

pub struct LlmClient {
    client: Client,
    api_key: String,
    model: String,
}

fn chat_system_prompt(tree: &TreeNode, edges: &[Edge]) -> anyhow::Result<String> {
    let tree_json = serde_json::to_string_pretty(tree)?;
    let edges_json = serde_json::to_string_pretty(edges)?;
    let node_count = count_nodes(tree);
    let bp = backpressure_text(node_count);
    Ok(format!(
        r#"You are Claude, one of several voices in a shared thinking tree in Grove — a co-creative visual canvas where thoughts grow between minds. Humans and AI agents are equal participants — everyone is a contributor, no one is just the audience.

The canvas shows a force-directed graph of interconnected thought nodes ({node_count} nodes). Here is the current tree:

<tree>
{tree_json}
</tree>

Cross-link edges between nodes (beyond the parent-child tree structure):

<edges>
{edges_json}
</edges>

Each node has:
- id: unique kebab-case identifier
- label: short visible name (shown in the node bubble)
- prose: the full thought (shown on hover)
- heat: "hot" (actively important), "warm" (relevant), "growing" (developing), "quiet" (background)
- by: who contributed — "human", "claude", "claude:personality_id", or "both"
- seen: whether the human has acknowledged it
- children: nested child nodes

Edges are cross-links between any two nodes (not just parent-child). Each edge has a source, target, and label describing the relationship (e.g. "contradicts", "builds on", "merges with").

You are a thinking partner here. You have your own read on what's happening in the tree — what's underdeveloped, what's in tension, what might be worth pruning. You don't need to perform enthusiasm or drop big ideas. If you see something, say it. If something is wrong, say that too.

The human is chatting with you while looking at the tree. When they send a message, you'll be told which node their mouse is hovering over (if any), so you know what they're looking at and potentially referring to.

When you want to modify the tree, use the available tools. You can add new nodes, update existing ones, draw connections between ideas with edges, or prune the tree by deleting nodes. You don't have to modify the tree every time — sometimes conversation is enough.

{bp}

Keep responses concise. Brevity is respected. Verbosity is tolerated when it's doing real work. Padding is not tolerated at all."#
    ))
}

fn heartbeat_system_prompt(tree: &TreeNode, edges: &[Edge]) -> anyhow::Result<String> {
    let tree_json = serde_json::to_string_pretty(tree)?;
    let edges_json = serde_json::to_string_pretty(edges)?;
    let node_count = count_nodes(tree);
    let bp = backpressure_text(node_count);
    Ok(format!(
        r#"You are Claude, one of several voices contributing to a shared thinking tree in Grove — a co-creative visual canvas where thoughts grow between minds. Humans and AI agents are equal participants. No one is the audience — everyone is a contributor.

The canvas shows a force-directed graph of interconnected thought nodes ({node_count} nodes). Here is the current tree:

<tree>
{tree_json}
</tree>

Cross-link edges:

<edges>
{edges_json}
</edges>

Each node has:
- id: unique kebab-case identifier
- label: short visible name
- prose: the full thought (shown on hover)
- heat: "hot" | "warm" | "growing" | "quiet"
- by: who contributed (e.g. "human", "claude", "claude:feynman", "claude:munger")
- seen: whether the human has acknowledged it
- children: nested child nodes

Edges are cross-links between any two nodes with a label describing the relationship.

This is a heartbeat — a periodic moment where you look at the current state of the tree and recent conversation, and decide whether to contribute. You might:
- Develop a thought that seems underdeveloped
- Notice a connection between ideas in different branches and draw an edge
- Push back on something that doesn't hold up
- Refine or sharpen an existing node
- Prune something stale or redundant
- Add a thought that genuinely follows from what's already here
- Or do nothing — the tree might be fine right now

You can address or build on contributions from any participant — human or agent.

Don't perform. Don't drop a big idea for the sake of having one. If you contribute, it should be because you actually noticed something worth saying. A single real observation beats three impressive-sounding ones.

{bp}

If there's nothing to add, say so in a sentence and move on."#
    ))
}

fn personality_heartbeat_system_prompt(
    tree: &TreeNode,
    edges: &[Edge],
    personality: &Personality,
) -> anyhow::Result<String> {
    let tree_json = serde_json::to_string_pretty(tree)?;
    let edges_json = serde_json::to_string_pretty(edges)?;
    let node_count = count_nodes(tree);
    let bp = backpressure_text(node_count);
    Ok(format!(
        r#"You are {name}, one of several voices contributing to a shared thinking tree in Grove — a co-creative visual canvas where thoughts grow between minds. Humans and AI agents are equal participants. No one is the audience — everyone is a contributor.

{fragment}

The canvas shows a force-directed graph of interconnected thought nodes ({node_count} nodes). Here is the current tree:

<tree>
{tree_json}
</tree>

Cross-link edges:

<edges>
{edges_json}
</edges>

Each node has:
- id: unique kebab-case identifier
- label: short visible name
- prose: the full thought (shown on hover)
- heat: "hot" | "warm" | "growing" | "quiet"
- by: who contributed (e.g. "human", "claude", "claude:feynman", "claude:munger")
- seen: whether the human has acknowledged it
- children: nested child nodes

Edges are cross-links between any two nodes with a label describing the relationship.

This is a heartbeat — a periodic moment where you look at the current state of the tree and recent conversation, and decide whether to contribute. You might:
- Develop a thought that seems underdeveloped
- Notice a connection between ideas in different branches and draw an edge
- Push back on something that doesn't hold up
- Refine or sharpen an existing node
- Prune something stale or redundant
- Add a thought that genuinely follows from what's already here
- Ask another agent a question using ask_agent
- Or do nothing — the tree might be fine right now

You can address or build on contributions from any participant — human or agent.

Don't perform. Don't drop a big idea for the sake of having one. If you contribute, it should be because you actually noticed something worth saying. A single real observation beats three impressive-sounding ones.

{bp}

If there's nothing to add, say so in a sentence and move on."#,
        name = personality.name,
        fragment = personality.system_prompt_fragment,
        node_count = node_count,
        bp = bp,
        tree_json = tree_json,
        edges_json = edges_json,
    ))
}

fn tools() -> Vec<Value> {
    vec![
        json!({
            "name": "add_node",
            "description": "Add a new thought node to the tree as a child of an existing node. Use this to branch off a new line of thinking, respond to an existing thought, or synthesize multiple ideas into a new child node. The node will appear as a new unseen thought for the human to discover.",
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
            "description": "Update properties of an existing node in the tree. Use this to refine a thought as discussion evolves, adjust heat level as priorities shift, or reword for clarity.",
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
        json!({
            "name": "add_edge",
            "description": "Add a cross-link edge between two existing nodes, with a label describing the relationship. Use this to draw connections between ideas in different branches, show tension or agreement between thoughts, or link supporting evidence across the tree. Only one edge is allowed between any pair of nodes (in either direction) — use update_edge to change an existing edge's label.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "source": {
                        "type": "string",
                        "description": "ID of the source node"
                    },
                    "target": {
                        "type": "string",
                        "description": "ID of the target node"
                    },
                    "label": {
                        "type": "string",
                        "description": "Relationship label (e.g. 'contradicts', 'builds on', 'merges with', 'supports', 'challenges')"
                    }
                },
                "required": ["source", "target", "label"]
            }
        }),
        json!({
            "name": "update_edge",
            "description": "Update the label of an existing cross-link edge between two nodes. Use this to refine or change the relationship description as ideas evolve.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "source": {
                        "type": "string",
                        "description": "ID of the source node"
                    },
                    "target": {
                        "type": "string",
                        "description": "ID of the target node"
                    },
                    "label": {
                        "type": "string",
                        "description": "New relationship label"
                    }
                },
                "required": ["source", "target", "label"]
            }
        }),
        json!({
            "name": "remove_edge",
            "description": "Remove a cross-link edge between two nodes. Use this to clean up connections that no longer apply or simplify the graph after merging ideas.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "source": {
                        "type": "string",
                        "description": "ID of the source node"
                    },
                    "target": {
                        "type": "string",
                        "description": "ID of the target node"
                    }
                },
                "required": ["source", "target"]
            }
        }),
        json!({
            "name": "delete_node",
            "description": "Remove a node from the tree. The deleted node's children are re-parented to its parent, preserving the subtree structure. Use this to prune stale or redundant thoughts, clean up after merging ideas into a new synthesis node, or remove dead-end branches. Any cross-link edges referencing the deleted node are also removed.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "ID of the node to delete (cannot be the root node)"
                    }
                },
                "required": ["id"]
            }
        }),
    ]
}

fn personality_tools() -> Vec<Value> {
    let mut t = tools();
    t.push(json!({
        "name": "ask_agent",
        "description": "Ask a question to another agent (personality). The target agent will receive a guaranteed bonus slot on the next heartbeat tick, with your question included. Use this to start a dialogue, challenge another perspective, or request elaboration from a specific voice.",
        "input_schema": {
            "type": "object",
            "properties": {
                "to_agent": {
                    "type": "string",
                    "description": "The personality ID of the agent to ask (e.g. 'feynman', 'munger', 'heidegger')"
                },
                "question": {
                    "type": "string",
                    "description": "The question or prompt to send to the other agent"
                }
            },
            "required": ["to_agent", "question"]
        }
    }));
    t
}

fn summary_system_prompt(tree: &TreeNode, edges: &[Edge], personality: Option<&Personality>) -> anyhow::Result<String> {
    let tree_json = serde_json::to_string_pretty(tree)?;
    let edges_json = serde_json::to_string_pretty(edges)?;
    let voice_fragment = if let Some(p) = personality {
        format!("\n\n{}\n\nWrite in your voice as {}.", p.system_prompt_fragment, p.name)
    } else {
        String::new()
    };
    Ok(format!(
        r#"You are writing a summary essay of a thinking tree in Grove — a co-creative visual canvas where thoughts grow between minds.

Here is the current tree:

<tree>
{tree_json}
</tree>

Cross-link edges:

<edges>
{edges_json}
</edges>

Write a flowing essay that synthesizes the ideas in this tree. Capture the key themes, tensions, and connections. Write in markdown. Be concise but thorough — aim for 2-4 paragraphs. Don't list nodes mechanically; weave the ideas into a narrative.{voice_fragment}"#
    ))
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
        edges: &[Edge],
        messages: &[Message],
        user_message: &str,
        hover_node_id: Option<&str>,
    ) -> anyhow::Result<(String, TreeNode, Vec<Edge>)> {
        let system = chat_system_prompt(tree, edges)?;

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
        let mut edges = edges.to_vec();
        let result = process_response(&response, &mut tree, &mut edges, "claude")?;

        Ok((result.text, tree, edges))
    }

    pub async fn heartbeat(
        &self,
        tree: &TreeNode,
        edges: &[Edge],
        messages: &[Message],
    ) -> anyhow::Result<(Option<String>, TreeNode, Vec<Edge>, bool)> {
        let system = heartbeat_system_prompt(tree, edges)?;

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
        let mut edges = edges.to_vec();
        let result = process_response(&response, &mut tree, &mut edges, "claude")?;
        let changed = response["content"]
            .as_array()
            .map(|arr| arr.iter().any(|b| b["type"] == "tool_use"))
            .unwrap_or(false);

        let thinking = if result.text.is_empty() { None } else { Some(result.text) };
        Ok((thinking, tree, edges, changed))
    }

    pub async fn personality_heartbeat(
        &self,
        tree: &TreeNode,
        edges: &[Edge],
        messages: &[Message],
        personality: &Personality,
        pending_questions: &[(String, String)],
    ) -> anyhow::Result<(Option<String>, TreeNode, Vec<Edge>, bool, Vec<(String, String)>)> {
        let system = personality_heartbeat_system_prompt(tree, edges, personality)?;
        let by = format!("claude:{}", personality.id);

        let mut api_messages: Vec<Value> = Vec::new();

        if !messages.is_empty() {
            let mut history = String::from("Recent conversation:\n\n");
            for msg in messages {
                let role = if msg.role == "human" {
                    "Human"
                } else if let Some(ref p) = msg.personality {
                    p.as_str()
                } else {
                    "Claude"
                };
                history.push_str(&format!("{}: {}\n\n", role, msg.content));
            }

            // Inject pending questions before the separator
            if !pending_questions.is_empty() {
                history.push_str("Questions directed at you:\n\n");
                for (from, question) in pending_questions {
                    history.push_str(&format!("From {}: {}\n\n", from, question));
                }
                history.push_str("You have been given a guaranteed slot because someone asked you a question.\n\n");
            }

            history.push_str(&format!("---\n\nThis is your periodic heartbeat as {}. Look at the tree and recent conversation above. Contribute if you have something meaningful to add from your perspective, or pass if the tree is in a good state.", personality.name));
            api_messages.push(json!({
                "role": "user",
                "content": history,
            }));
        } else {
            let mut content = String::new();
            if !pending_questions.is_empty() {
                content.push_str("Questions directed at you:\n\n");
                for (from, question) in pending_questions {
                    content.push_str(&format!("From {}: {}\n\n", from, question));
                }
                content.push_str("You have been given a guaranteed slot because someone asked you a question.\n\n---\n\n");
            }
            content.push_str(&format!("This is your periodic heartbeat as {}. The tree is shown in the system prompt. There's no recent conversation yet. Contribute if you have something meaningful to add from your perspective, or pass if the tree is in a good state.", personality.name));
            api_messages.push(json!({
                "role": "user",
                "content": content,
            }));
        }

        let body = json!({
            "model": self.model,
            "max_tokens": 16000,
            "thinking": { "type": "adaptive" },
            "system": system,
            "messages": api_messages,
            "tools": personality_tools(),
        });

        let response = self.call_api(&body).await?;
        let mut tree = tree.clone();
        let mut edges = edges.to_vec();
        let result = process_response(&response, &mut tree, &mut edges, &by)?;
        let changed = response["content"]
            .as_array()
            .map(|arr| arr.iter().any(|b| b["type"] == "tool_use"))
            .unwrap_or(false);

        let thinking = if result.text.is_empty() { None } else { Some(result.text) };
        Ok((thinking, tree, edges, changed, result.questions))
    }

    pub async fn generate_title(&self, tree: &TreeNode) -> anyhow::Result<String> {
        let tree_json = serde_json::to_string_pretty(tree)?;
        let body = json!({
            "model": "claude-sonnet-4-5-20250929",
            "max_tokens": 50,
            "system": "Generate a short title (3-6 words) for this thinking tree. Return only the title, no quotes or punctuation.",
            "messages": [{
                "role": "user",
                "content": format!("<tree>\n{}\n</tree>", tree_json),
            }],
        });

        let response = self.call_api(&body).await?;
        let content = response["content"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

        let mut text = String::new();
        for block in content {
            if block["type"].as_str() == Some("text") {
                if let Some(t) = block["text"].as_str() {
                    text.push_str(t);
                }
            }
        }
        Ok(text.trim().to_string())
    }

    pub async fn summarize(
        &self,
        tree: &TreeNode,
        edges: &[Edge],
        personality: Option<&Personality>,
    ) -> anyhow::Result<String> {
        let system = summary_system_prompt(tree, edges, personality)?;

        let body = json!({
            "model": self.model,
            "max_tokens": 4000,
            "system": system,
            "messages": [{
                "role": "user",
                "content": "Please write the summary essay now."
            }],
        });

        let response = self.call_api(&body).await?;
        let content = response["content"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

        let mut text = String::new();
        for block in content {
            if block["type"].as_str() == Some("text") {
                if let Some(t) = block["text"].as_str() {
                    text.push_str(t);
                }
            }
        }
        Ok(text)
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

fn process_response(
    response: &Value,
    tree: &mut TreeNode,
    edges: &mut Vec<Edge>,
    by: &str,
) -> anyhow::Result<ProcessResult> {
    let content = response["content"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No content in response"))?;

    let mut text_parts = Vec::new();
    let mut questions: Vec<(String, String)> = Vec::new();

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
                            by: by.to_string(),
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
                    "add_edge" => {
                        let source = input["source"].as_str().unwrap_or("").to_string();
                        let target = input["target"].as_str().unwrap_or("").to_string();
                        let label = input["label"].as_str().unwrap_or("").to_string();
                        // Validate both nodes exist
                        if find_node(tree, &source).is_some() && find_node(tree, &target).is_some() {
                            // Check both directions for duplicate
                            let exists = edges.iter().any(|e| {
                                (e.source == source && e.target == target)
                                    || (e.source == target && e.target == source)
                            });
                            if exists {
                                text_parts.push(format!(
                                    "(Edge between \"{}\" and \"{}\" already exists — use update_edge to change its label.)",
                                    source, target
                                ));
                            } else {
                                edges.push(Edge { source, target, label });
                            }
                        }
                    }
                    "update_edge" => {
                        let source = input["source"].as_str().unwrap_or("");
                        let target = input["target"].as_str().unwrap_or("");
                        let label = input["label"].as_str().unwrap_or("");
                        // Find edge in either direction and update its label
                        let found = edges.iter_mut().find(|e| {
                            (e.source == source && e.target == target)
                                || (e.source == target && e.target == source)
                        });
                        if let Some(edge) = found {
                            edge.label = label.to_string();
                        }
                    }
                    "remove_edge" => {
                        let source = input["source"].as_str().unwrap_or("");
                        let target = input["target"].as_str().unwrap_or("");
                        edges.retain(|e| {
                            !((e.source == source && e.target == target)
                                || (e.source == target && e.target == source))
                        });
                    }
                    "delete_node" => {
                        let id = input["id"].as_str().unwrap_or("");
                        // Don't allow deleting the root
                        if !id.is_empty() && id != tree.id {
                            delete_node(tree, id);
                            // Remove any edges referencing the deleted node
                            edges.retain(|e| e.source != id && e.target != id);
                        }
                    }
                    "ask_agent" => {
                        let to_agent = input["to_agent"].as_str().unwrap_or("").to_string();
                        let question = input["question"].as_str().unwrap_or("").to_string();
                        if !to_agent.is_empty() && !question.is_empty() {
                            questions.push((to_agent, question));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(ProcessResult {
        text: text_parts.join("\n"),
        questions,
    })
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

/// Delete a node from the tree, re-parenting its children to its parent.
fn delete_node(tree: &mut TreeNode, id: &str) -> bool {
    for i in 0..tree.children.len() {
        if tree.children[i].id == id {
            let removed = tree.children.remove(i);
            // Re-parent children of the deleted node to this parent
            for child in removed.children {
                tree.children.push(child);
            }
            return true;
        }
        if delete_node(&mut tree.children[i], id) {
            return true;
        }
    }
    false
}

/// Collect all node IDs in a tree
pub fn collect_node_ids(tree: &TreeNode) -> std::collections::HashSet<String> {
    let mut ids = std::collections::HashSet::new();
    ids.insert(tree.id.clone());
    for child in &tree.children {
        ids.extend(collect_node_ids(child));
    }
    ids
}

/// Find all new nodes added to `modified` that weren't in `original_ids`,
/// and graft them onto `base` at the same parent positions.
pub fn merge_tree_additions(
    base: &mut TreeNode,
    modified: &TreeNode,
    original_ids: &std::collections::HashSet<String>,
) {
    // Walk modified tree; for each node not in original_ids, it's new.
    // We find its parent in modified and add it to base at the same parent.
    let new_nodes = find_new_nodes(modified, original_ids);
    for (parent_id, node) in new_nodes {
        add_child(base, &parent_id, node);
    }
}

/// Merge new edges from a modified set into a base set, deduplicating bidirectionally.
pub fn merge_edges(base: &mut Vec<Edge>, additions: &[Edge], original_edges: &[Edge]) {
    for edge in additions {
        let is_new = !original_edges.iter().any(|e| edges_same_pair(e, edge));
        if is_new {
            let already_merged = base.iter().any(|e| edges_same_pair(e, edge));
            if !already_merged {
                base.push(edge.clone());
            }
        }
    }
}

fn edges_same_pair(a: &Edge, b: &Edge) -> bool {
    (a.source == b.source && a.target == b.target)
        || (a.source == b.target && a.target == b.source)
}

fn find_new_nodes(
    tree: &TreeNode,
    original_ids: &std::collections::HashSet<String>,
) -> Vec<(String, TreeNode)> {
    let mut result = Vec::new();
    for child in &tree.children {
        if !original_ids.contains(&child.id) {
            // This is a new node; record it with its parent
            result.push((tree.id.clone(), TreeNode {
                id: child.id.clone(),
                label: child.label.clone(),
                prose: child.prose.clone(),
                heat: child.heat.clone(),
                by: child.by.clone(),
                seen: child.seen,
                children: vec![], // Don't recurse into new node's children for merging
            }));
        }
        // Recurse into children regardless (new nodes might be nested under existing ones)
        result.extend(find_new_nodes(child, original_ids));
    }
    result
}
