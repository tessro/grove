import { useState, useRef, useEffect, useCallback } from "react";

export default function ChatBox({
  messages,
  onSend,
  loading,
  hoverNodeId,
  tree,
}) {
  const [input, setInput] = useState("");
  const messagesEndRef = useRef(null);
  const inputRef = useRef(null);

  /* Auto-scroll to bottom on new messages */
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  /* Find hovered node label */
  const hoverLabel = hoverNodeId ? findNodeLabel(tree, hoverNodeId) : null;

  const handleSend = useCallback(() => {
    const text = input.trim();
    if (!text || loading) return;
    setInput("");
    onSend(text);
  }, [input, loading, onSend]);

  const handleKeyDown = useCallback(
    (e) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend],
  );

  return (
    <div className="chat-container">
      <div className="chat-header">Chat</div>

      <div className="chat-messages">
        {messages.length === 0 && (
          <div
            style={{
              fontFamily: "'IBM Plex Serif', serif",
              fontSize: "13px",
              color: "rgb(70, 68, 62)",
              lineHeight: 1.6,
              padding: "20px 0",
            }}
          >
            Say something to start growing the tree together. Hover over a node
            while typing to reference it.
          </div>
        )}
        {messages.map((msg, i) => (
          <div
            key={i}
            className={`chat-msg ${msg.is_heartbeat ? "heartbeat" : ""}`}
          >
            {msg.hover_node_id && (
              <div className="chat-msg-hover-context">
                looking at: {findNodeLabel(tree, msg.hover_node_id) || msg.hover_node_id}
              </div>
            )}
            <div className={`chat-msg-role ${msg.role}`}>
              {msg.role === "human" ? "you" : "claude"}
              {msg.is_heartbeat ? " (heartbeat)" : ""}
            </div>
            <div className="chat-msg-content">{msg.content}</div>
          </div>
        ))}
        {loading && (
          <div className="chat-msg">
            <div className="chat-msg-role assistant">claude</div>
            <div
              className="chat-msg-content"
              style={{ opacity: 0.5, fontStyle: "italic" }}
            >
              thinking...
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      <div className="chat-input-area">
        {hoverLabel && (
          <div className="chat-hover-indicator">
            <span className="chat-hover-dot" />
            hovering: {hoverLabel}
          </div>
        )}
        <div className="chat-input-row">
          <textarea
            ref={inputRef}
            className="chat-input"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Say something..."
            rows={1}
            disabled={loading}
          />
          <button
            className="chat-send"
            onClick={handleSend}
            disabled={loading || !input.trim()}
          >
            Send
          </button>
        </div>
      </div>
    </div>
  );
}

function findNodeLabel(tree, id) {
  if (!tree) return null;
  if (tree.id === id) return tree.label;
  for (const child of tree.children || []) {
    const found = findNodeLabel(child, id);
    if (found) return found;
  }
  return null;
}
