import { useState, useEffect, useRef, useCallback } from "react";

export default function HeartbeatControls({ onHeartbeat, loading }) {
  const [active, setActive] = useState(false);
  const [interval, setInterval_] = useState(30);
  const [beating, setBeating] = useState(false);
  const [countdown, setCountdown] = useState(0);
  const timerRef = useRef(null);
  const countdownRef = useRef(null);

  const doHeartbeat = useCallback(async () => {
    setBeating(true);
    try {
      await onHeartbeat();
    } finally {
      setBeating(false);
    }
  }, [onHeartbeat]);

  /* Heartbeat timer */
  useEffect(() => {
    if (active && !loading) {
      setCountdown(interval);

      countdownRef.current = window.setInterval(() => {
        setCountdown((prev) => {
          if (prev <= 1) return interval;
          return prev - 1;
        });
      }, 1000);

      timerRef.current = window.setInterval(() => {
        doHeartbeat();
      }, interval * 1000);

      return () => {
        clearInterval(timerRef.current);
        clearInterval(countdownRef.current);
      };
    } else {
      clearInterval(timerRef.current);
      clearInterval(countdownRef.current);
      setCountdown(0);
    }
  }, [active, interval, loading, doHeartbeat]);

  const toggle = useCallback(() => {
    setActive((prev) => !prev);
  }, []);

  const pulseClass = beating
    ? "heartbeat-pulse beating"
    : active
      ? "heartbeat-pulse active"
      : "heartbeat-pulse";

  return (
    <div className="heartbeat-bar">
      <div className={pulseClass} />
      <span>heartbeat</span>

      <button
        className={`heartbeat-toggle ${active ? "active" : ""}`}
        onClick={toggle}
      >
        {active ? "on" : "off"}
      </button>

      <div className="heartbeat-interval">
        <input
          type="range"
          className="heartbeat-slider"
          min={15}
          max={120}
          step={5}
          value={interval}
          onChange={(e) => setInterval_(Number(e.target.value))}
        />
        <span>{interval}s</span>
      </div>

      {active && countdown > 0 && (
        <span style={{ opacity: 0.5 }}>
          {countdown}s
        </span>
      )}

      <div className="heartbeat-spacer" />

      {!active && (
        <button
          className="heartbeat-toggle"
          onClick={doHeartbeat}
          disabled={loading}
          style={{ fontSize: "9px" }}
        >
          pulse once
        </button>
      )}
    </div>
  );
}
