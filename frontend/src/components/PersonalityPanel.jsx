import { useState, useCallback } from "react";

export default function PersonalityPanel({
  personalities,
  activePersonalities,
  onTogglePersonality,
}) {
  const [expanded, setExpanded] = useState(false);

  const handleToggle = useCallback(
    (id) => {
      const isActive = activePersonalities.includes(id);
      const next = isActive
        ? activePersonalities.filter((p) => p !== id)
        : [...activePersonalities, id];
      onTogglePersonality(next);
    },
    [activePersonalities, onTogglePersonality],
  );

  // Group by category
  const categories = {};
  for (const p of personalities) {
    if (!categories[p.category]) categories[p.category] = [];
    categories[p.category].push(p);
  }

  const activeCount = activePersonalities.length;

  return (
    <div className="personality-panel">
      <div
        className="personality-panel-header"
        onClick={() => setExpanded((v) => !v)}
      >
        <span className="personality-panel-title">
          Voices
          {activeCount > 0 && (
            <span className="personality-panel-count">{activeCount}</span>
          )}
        </span>
        <span className="personality-panel-toggle">
          {expanded ? "\u25B4" : "\u25BE"}
        </span>
      </div>

      {expanded && (
        <div className="personality-panel-body">
          {Object.entries(categories).map(([cat, items]) => (
            <div key={cat} className="personality-category">
              <div className="personality-category-label">{cat}</div>
              <div className="personality-chips">
                {items.map((p) => {
                  const isActive = activePersonalities.includes(p.id);
                  return (
                    <button
                      key={p.id}
                      className={`personality-chip ${isActive ? "active" : ""}`}
                      style={
                        isActive
                          ? {
                              borderColor: p.color,
                              color: p.color,
                              boxShadow: `0 0 8px ${p.color}40`,
                            }
                          : {}
                      }
                      onClick={() => handleToggle(p.id)}
                      title={p.short_description}
                    >
                      <span
                        className="personality-chip-dot"
                        style={{ background: p.color }}
                      />
                      {p.name}
                    </button>
                  );
                })}
              </div>
            </div>
          ))}

        </div>
      )}
    </div>
  );
}
