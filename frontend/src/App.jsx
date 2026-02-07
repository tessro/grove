import { useState, useEffect, useCallback, useMemo } from "react";
import { useParams } from "react-router-dom";
import ThinkingCanvas from "./components/ThinkingCanvas";
import ChatBox from "./components/ChatBox";
import HeartbeatControls from "./components/HeartbeatControls";
import PersonalityPanel from "./components/PersonalityPanel";
import * as api from "./api";

export default function App() {
  const { docId } = useParams();
  const [tree, setTree] = useState(null);
  const [edges, setEdges] = useState([]);
  const [messages, setMessages] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [hoverNodeId, setHoverNodeId] = useState(null);
  const [chatLoading, setChatLoading] = useState(false);
  const [heartbeatLoading, setHeartbeatLoading] = useState(false);
  const [personalities, setPersonalities] = useState([]);
  const [activePersonalities, setActivePersonalities] = useState([]);
  const [diceSides, setDiceSides] = useState(3);

  /* Load document + personalities */
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    Promise.all([
      api.getDoc(docId),
      api.getMessages(docId),
      api.getPersonalities(docId),
    ])
      .then(([doc, msgs, pers]) => {
        if (cancelled) return;
        setTree(doc.tree);
        setEdges(doc.edges || []);
        setMessages(msgs.messages);
        setPersonalities(pers.available);
        setActivePersonalities(pers.active);
        setDiceSides(pers.dice_sides);
      })
      .catch((e) => {
        if (cancelled) return;
        setError("Document not found");
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [docId]);

  // Build personality color map for child components (memoized to avoid D3 reflow)
  const personalityColors = useMemo(() => {
    const colors = {};
    for (const p of personalities) {
      colors[p.id] = p.color;
    }
    return colors;
  }, [personalities]);

  /* Chat handler */
  const handleChat = useCallback(
    async (message) => {
      setChatLoading(true);
      // Optimistically add user message
      setMessages((prev) => [
        ...prev,
        { role: "human", content: message, hover_node_id: hoverNodeId },
      ]);
      try {
        const res = await api.chat(docId, message, hoverNodeId);
        setTree(res.tree);
        if (res.edges) setEdges(res.edges);
        if (res.reply) {
          setMessages((prev) => [
            ...prev,
            { role: "assistant", content: res.reply },
          ]);
        }
      } catch (e) {
        console.error("Chat error:", e);
        setMessages((prev) => [
          ...prev,
          {
            role: "assistant",
            content: "(Error reaching Claude. Try again.)",
          },
        ]);
      } finally {
        setChatLoading(false);
      }
    },
    [docId, hoverNodeId],
  );

  /* Mark node as seen in backend */
  const handleMarkSeen = useCallback(
    (nodeId) => {
      api.markSeen(docId, nodeId).catch((e) => {
        console.error("Mark seen error:", e);
      });
    },
    [docId],
  );

  /* Heartbeat handler */
  const handleHeartbeat = useCallback(async () => {
    setHeartbeatLoading(true);
    try {
      const res = await api.heartbeat(docId);
      if (res.changed) {
        setTree(res.tree);
        if (res.edges) setEdges(res.edges);
      }
      // If we got per-personality results, push a message per personality
      if (res.results && res.results.length > 0) {
        for (const r of res.results) {
          if (r.thinking) {
            setMessages((prev) => [
              ...prev,
              {
                role: "assistant",
                content: r.thinking,
                is_heartbeat: true,
                personality: r.personality,
              },
            ]);
          }
        }
      } else if (res.thinking) {
        // Classic heartbeat (no personalities)
        setMessages((prev) => [
          ...prev,
          { role: "assistant", content: res.thinking, is_heartbeat: true },
        ]);
      }
      return res.changed;
    } catch (e) {
      console.error("Heartbeat error:", e);
      return false;
    } finally {
      setHeartbeatLoading(false);
    }
  }, [docId]);

  /* Personality toggle */
  const handleTogglePersonalities = useCallback(
    (newIds) => {
      setActivePersonalities(newIds);
      api.setPersonalities(docId, newIds).catch((e) => {
        console.error("Set personalities error:", e);
      });
    },
    [docId],
  );

  /* Dice sides change */
  const handleDiceSidesChange = useCallback(
    (sides) => {
      setDiceSides(sides);
      api.updateDocSettings(docId, { dice_sides: sides }).catch((e) => {
        console.error("Update settings error:", e);
      });
    },
    [docId],
  );

  if (loading) {
    return (
      <div className="loading">
        <div className="loading-spinner" />
        <p>Loading grove...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="loading">
        <p style={{ color: "rgb(200, 100, 80)" }}>{error}</p>
        <a
          href="/"
          style={{
            color: "rgb(120, 180, 120)",
            fontFamily: "'IBM Plex Mono', monospace",
            fontSize: "12px",
          }}
        >
          Start a new grove
        </a>
      </div>
    );
  }

  return (
    <div className="app">
      <div className="canvas-area">
        <ThinkingCanvas
          tree={tree}
          edges={edges}
          onHoverNode={setHoverNodeId}
          onMarkSeen={handleMarkSeen}
          hoverNodeId={hoverNodeId}
          activePersonalities={activePersonalities}
          personalityColors={personalityColors}
          personalities={personalities}
        />
      </div>
      <div className="sidebar">
        <HeartbeatControls
          onHeartbeat={handleHeartbeat}
          loading={heartbeatLoading}
        />
        <PersonalityPanel
          personalities={personalities}
          activePersonalities={activePersonalities}
          diceSides={diceSides}
          onTogglePersonality={handleTogglePersonalities}
          onDiceSidesChange={handleDiceSidesChange}
        />
        <ChatBox
          messages={messages}
          onSend={handleChat}
          loading={chatLoading}
          hoverNodeId={hoverNodeId}
          tree={tree}
          personalityColors={personalityColors}
          personalities={personalities}
        />
      </div>
    </div>
  );
}
