const BASE = "/api";

export async function createDoc() {
  const res = await fetch(`${BASE}/docs`, { method: "POST" });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}

export async function getDoc(id) {
  const res = await fetch(`${BASE}/docs/${id}`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}

export async function getMessages(id) {
  const res = await fetch(`${BASE}/docs/${id}/messages`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}

export async function chat(id, message, hoverNodeId) {
  const res = await fetch(`${BASE}/docs/${id}/chat`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      message,
      hover_node_id: hoverNodeId || null,
    }),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}

export async function markSeen(id, nodeId) {
  await fetch(`${BASE}/docs/${id}/mark-seen`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ node_id: nodeId }),
  });
}

export async function heartbeat(id) {
  const res = await fetch(`${BASE}/docs/${id}/heartbeat`, {
    method: "POST",
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}

export async function getPersonalities(id) {
  const res = await fetch(`${BASE}/docs/${id}/personalities`);
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}

export async function setPersonalities(id, personalityIds) {
  const res = await fetch(`${BASE}/docs/${id}/personalities`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ personality_ids: personalityIds }),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
}

export async function updateDocSettings(id, settings) {
  const res = await fetch(`${BASE}/docs/${id}/settings`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(settings),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
}

export async function getSummary(id, voice, forceRefresh) {
  const res = await fetch(`${BASE}/docs/${id}/summary`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      voice: voice || null,
      force_refresh: forceRefresh || false,
    }),
  });
  if (!res.ok) throw new Error(`HTTP ${res.status}`);
  return res.json();
}
