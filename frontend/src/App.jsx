import { useState, useEffect, useCallback } from "react";
import { useParams } from "react-router-dom";
import ThinkingCanvas from "./components/ThinkingCanvas";
import ChatBox from "./components/ChatBox";
import HeartbeatControls from "./components/HeartbeatControls";
import * as api from "./api";

export default function App() {
  const { docId } = useParams();
  const [tree, setTree] = useState(null);
  const [messages, setMessages] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [hoverNodeId, setHoverNodeId] = useState(null);
  const [chatLoading, setChatLoading] = useState(false);
  const [heartbeatLoading, setHeartbeatLoading] = useState(false);

  /* Load document */
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    Promise.all([api.getDoc(docId), api.getMessages(docId)])
      .then(([doc, msgs]) => {
        if (cancelled) return;
        setTree(doc.tree);
        setMessages(msgs.messages);
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
      }
      if (res.thinking) {
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
          onHoverNode={setHoverNodeId}
          onMarkSeen={handleMarkSeen}
          hoverNodeId={hoverNodeId}
        />
      </div>
      <div className="sidebar">
        <HeartbeatControls
          onHeartbeat={handleHeartbeat}
          loading={heartbeatLoading}
        />
        <ChatBox
          messages={messages}
          onSend={handleChat}
          loading={chatLoading}
          hoverNodeId={hoverNodeId}
          tree={tree}
        />
      </div>
    </div>
  );
}
