import { useState } from "react";

export default function SettingsPanel({ repelForce, onRepelForceChange }) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="settings-panel">
      <div
        className="settings-panel-header"
        onClick={() => setExpanded((v) => !v)}
      >
        <span>Settings</span>
        <span className="personality-panel-toggle">
          {expanded ? "\u25B4" : "\u25BE"}
        </span>
      </div>

      {expanded && (
        <div className="settings-panel-body">
          <div className="settings-row">
            <span>repelling force: {repelForce}</span>
            <input
              type="range"
              className="heartbeat-slider"
              min={5}
              max={60}
              step={1}
              value={repelForce}
              onChange={(e) => onRepelForceChange(Number(e.target.value))}
            />
          </div>
        </div>
      )}
    </div>
  );
}
