import { useEffect, useRef, useState, useCallback } from "react";
import * as d3 from "d3";

/* ─── Color systems ─── */

const HEAT_CONFIG = {
  hot: {
    baseR: 100,
    color: "#e8642c",
    fg: "#fff8f0",
    fgMuted: "rgba(255,248,240,0.7)",
    bg: "rgba(232,100,44,0.10)",
    glow: "rgba(232,100,44,0.3)",
    strokeW: 2,
    labelSize: 15,
    proseSize: 11.5,
    proseLines: 4,
    labelWeight: 700,
    charWidth: 22,
  },
  warm: {
    baseR: 78,
    color: "#d6a648",
    fg: "#f0e8da",
    fgMuted: "rgba(240,232,218,0.6)",
    bg: "rgba(214,166,72,0.07)",
    glow: "rgba(214,166,72,0.2)",
    strokeW: 1.5,
    labelSize: 13,
    proseSize: 10.5,
    proseLines: 3,
    labelWeight: 600,
    charWidth: 18,
  },
  growing: {
    baseR: 60,
    color: "#78b478",
    fg: "#dce8dc",
    fgMuted: "rgba(220,232,220,0.55)",
    bg: "rgba(120,180,120,0.06)",
    glow: "rgba(120,180,120,0.15)",
    strokeW: 1.5,
    labelSize: 12,
    proseSize: 9.5,
    proseLines: 2,
    labelWeight: 600,
    charWidth: 16,
  },
  quiet: {
    baseR: 46,
    color: "#706e66",
    fg: "#a09e96",
    fgMuted: "rgba(160,158,152,0.5)",
    bg: "rgba(70,70,68,0.06)",
    glow: "transparent",
    strokeW: 1.5,
    labelSize: 11,
    proseSize: 9,
    proseLines: 1,
    labelWeight: 500,
    charWidth: 13,
  },
};

const UNSEEN = {
  color: "#4fc4cf",
  colorBright: "#62e0ec",
  fg: "#e4fbfd",
  fgMuted: "rgba(228,251,253,0.65)",
  bg: "rgba(79,196,207,0.09)",
  glow: "rgba(79,196,207,0.35)",
  strokeW: 2.5,
  linkColor: "rgba(79,196,207,0.3)",
};

function getNodeVisual(node, isSeen) {
  const base = HEAT_CONFIG[node.heat] || HEAT_CONFIG.quiet;
  if (!isSeen) {
    return {
      ...base,
      color: UNSEEN.color,
      fg: UNSEEN.fg,
      fgMuted: UNSEEN.fgMuted,
      bg: UNSEEN.bg,
      glow: UNSEEN.glow,
      strokeW: UNSEEN.strokeW,
    };
  }
  return base;
}

/* ─── Parse personality from `by` field ─── */
function parsePersonality(by) {
  if (by && by.startsWith("claude:")) {
    return by.slice(7); // e.g. "claude:feynman" -> "feynman"
  }
  return null;
}

/* ─── Flatten tree ─── */
function flatten(node, parentId = null) {
  const nodes = [];
  const links = [];
  const base = HEAT_CONFIG[node.heat] || HEAT_CONFIG.quiet;
  nodes.push({
    ...node,
    radius: base.baseR,
    children: undefined,
    _children: node.children,
  });
  if (parentId) links.push({ source: parentId, target: node.id });
  for (const child of node.children || []) {
    const [cn, cl] = flatten(child, node.id);
    nodes.push(...cn);
    links.push(...cl);
  }
  return [nodes, links];
}

/* ─── Word wrap ─── */
function wrapText(text, maxChars) {
  const words = text.split(" ");
  const lines = [];
  let line = "";
  for (const w of words) {
    if (line.length + w.length + 1 > maxChars) {
      lines.push(line);
      line = w;
    } else {
      line = line ? line + " " + w : w;
    }
  }
  if (line) lines.push(line);
  return lines;
}

/* ─── Count unseen nodes ─── */
function countUnseen(node, seenIds) {
  let c = !node.seen && !seenIds.has(node.id) ? 1 : 0;
  for (const child of node.children || []) c += countUnseen(child, seenIds);
  return c;
}

export default function ThinkingCanvas({
  tree,
  edges: crossEdges,
  onHoverNode,
  onMarkSeen,
  hoverNodeId,
  activePersonalities,
  personalityColors,
  personalities,
}) {
  const svgRef = useRef(null);
  const containerRef = useRef(null);
  const simRef = useRef(null);
  const [dims, setDims] = useState({ w: 800, h: 600 });
  const [tooltip, setTooltip] = useState(null);
  const [seenIds, setSeenIds] = useState(new Set());
  const seenIdsRef = useRef(seenIds);
  const tooltipTimeout = useRef(null);

  const markSeen = useCallback((id) => {
    setSeenIds((prev) => {
      if (prev.has(id)) return prev;
      const next = new Set(prev);
      next.add(id);
      seenIdsRef.current = next;
      return next;
    });
  }, []);

  const isSeen = useCallback(
    (node) => node.seen || seenIds.has(node.id),
    [seenIds],
  );

  /* Measure container */
  useEffect(() => {
    const measure = () => {
      if (containerRef.current)
        setDims({
          w: containerRef.current.clientWidth,
          h: containerRef.current.clientHeight,
        });
    };
    measure();
    window.addEventListener("resize", measure);
    return () => window.removeEventListener("resize", measure);
  }, []);

  /* Render D3 visualization */
  useEffect(() => {
    if (!svgRef.current || dims.w === 0 || !tree) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll("*").remove();

    const [nodes, links] = flatten(tree);
    const { w, h } = dims;

    // Build cross-link edges (filter to edges where both nodes exist)
    const nodeIds = new Set(nodes.map((n) => n.id));
    const crossLinks = (crossEdges || [])
      .filter((e) => nodeIds.has(e.source) && nodeIds.has(e.target))
      .map((e) => ({ source: e.source, target: e.target, label: e.label, type: "cross" }));
    const allLinks = [...links, ...crossLinks];

    const defs = svg.append("defs");

    /* Glow filters */
    ["hot", "warm", "growing"].forEach((key) => {
      const filter = defs
        .append("filter")
        .attr("id", `glow-${key}`)
        .attr("x", "-80%")
        .attr("y", "-80%")
        .attr("width", "260%")
        .attr("height", "260%");
      filter
        .append("feGaussianBlur")
        .attr("in", "SourceGraphic")
        .attr("stdDeviation", key === "hot" ? 12 : 7);
      const merge = filter.append("feMerge");
      merge.append("feMergeNode").attr("in", "blur");
      merge.append("feMergeNode").attr("in", "SourceGraphic");
    });

    const unseenFilter = defs
      .append("filter")
      .attr("id", "glow-unseen")
      .attr("x", "-80%")
      .attr("y", "-80%")
      .attr("width", "260%")
      .attr("height", "260%");
    unseenFilter
      .append("feGaussianBlur")
      .attr("in", "SourceGraphic")
      .attr("stdDeviation", 14);
    const unseenMerge = unseenFilter.append("feMerge");
    unseenMerge.append("feMergeNode").attr("in", "blur");
    unseenMerge.append("feMergeNode").attr("in", "SourceGraphic");

    /* Pulse animation */
    const style = document.createElementNS(
      "http://www.w3.org/2000/svg",
      "style",
    );
    style.textContent = `
      @keyframes pulse-unseen {
        0%, 100% { opacity: 0.35; }
        50% { opacity: 0.55; }
      }
      .unseen-glow { animation: pulse-unseen 3s ease-in-out infinite; }
    `;
    svg.node().appendChild(style);

    const g = svg.append("g");
    const zoom = d3
      .zoom()
      .scaleExtent([0.2, 5])
      .on("zoom", (event) => g.attr("transform", event.transform));
    svg.call(zoom);

    const sim = d3
      .forceSimulation(nodes)
      .force(
        "link",
        d3
          .forceLink(allLinks)
          .id((d) => d.id)
          .distance((d) => {
            const sn = nodes.find(
              (n) => n.id === (d.source.id || d.source),
            );
            const tn = nodes.find(
              (n) => n.id === (d.target.id || d.target),
            );
            return (sn?.radius || 60) + (tn?.radius || 60) + 40;
          })
          .strength(0.7),
      )
      .force(
        "charge",
        d3.forceManyBody().strength((d) => -d.radius * 20),
      )
      .force("center", d3.forceCenter(w / 2, h / 2))
      .force(
        "collision",
        d3.forceCollide().radius((d) => d.radius + 12),
      )
      .force("x", d3.forceX(w / 2).strength(0.03))
      .force("y", d3.forceY(h / 2).strength(0.03));

    simRef.current = sim;

    /* Tree links (dashed) */
    const linkSel = g
      .append("g")
      .selectAll("line")
      .data(links)
      .join("line")
      .attr("stroke-dasharray", "6,5")
      .attr("opacity", 0.6)
      .each(function (d) {
        const target = nodes.find(
          (n) => n.id === (d.target.id || d.target),
        );
        const nodeIsSeen = target && (target.seen || seenIdsRef.current.has(target.id));
        d3.select(this)
          .attr("stroke", nodeIsSeen ? "rgb(100,98,90)" : UNSEEN.linkColor)
          .attr("stroke-width", nodeIsSeen ? 1.5 : 1.8);
      });

    /* Cross-link edges (solid) */
    const crossLinkG = g.append("g");
    const crossLinkSel = crossLinkG
      .selectAll("line")
      .data(crossLinks)
      .join("line")
      .attr("stroke", "rgba(180,160,220,0.5)")
      .attr("stroke-width", 1.5)
      .attr("opacity", 0.7);

    const crossLabelSel = crossLinkG
      .selectAll("text")
      .data(crossLinks)
      .join("text")
      .attr("text-anchor", "middle")
      .attr("fill", "rgba(180,160,220,0.7)")
      .attr("font-family", "'IBM Plex Mono', monospace")
      .attr("font-size", "9px")
      .attr("pointer-events", "none")
      .text((d) => d.label);

    /* Node groups */
    const nodeG = g
      .append("g")
      .selectAll("g")
      .data(nodes)
      .join("g")
      .attr("cursor", "grab")
      .call(
        d3
          .drag()
          .on("start", (event, d) => {
            if (!event.active) sim.alphaTarget(0.3).restart();
            d.fx = d.x;
            d.fy = d.y;
          })
          .on("drag", (event, d) => {
            d.fx = event.x;
            d.fy = event.y;
          })
          .on("end", (event, d) => {
            if (!event.active) sim.alphaTarget(0);
            d.fx = null;
            d.fy = null;
          }),
      );

    /* Render each node */
    nodeG.each(function (d) {
      const el = d3.select(this);
      const nodeIsSeen = d.seen || seenIdsRef.current.has(d.id);
      const v = getNodeVisual(d, nodeIsSeen);
      const personalityId = parsePersonality(d.by);
      const pColor = personalityId && personalityColors ? personalityColors[personalityId] : null;

      /* Glow */
      if (v.glow !== "transparent") {
        el.append("circle")
          .attr("r", d.radius + 10)
          .attr("fill", v.glow)
          .attr(
            "filter",
            nodeIsSeen ? `url(#glow-${d.heat})` : "url(#glow-unseen)",
          )
          .attr("class", nodeIsSeen ? "glow" : "glow unseen-glow");
      }

      /* Main circle */
      el.append("circle")
        .attr("r", d.radius)
        .attr("fill", v.bg)
        .attr("stroke", v.color)
        .attr("stroke-width", v.strokeW)
        .attr("class", "main-circle");

      /* Inner ring for personality nodes */
      if (pColor) {
        el.append("circle")
          .attr("r", d.radius - 4)
          .attr("fill", "none")
          .attr("stroke", pColor)
          .attr("stroke-width", 1.5)
          .attr("opacity", 0.8)
          .attr("class", "personality-ring");
      }

      /* Text */
      const labelLines = wrapText(d.label, v.charWidth);
      const labelLH = v.labelSize * 1.3;
      const proseLH = v.proseSize * 1.45;
      const proseLines = wrapText(d.prose, v.charWidth + 2);
      const visibleProseLines = proseLines.slice(0, v.proseLines);
      const truncated = proseLines.length > v.proseLines;
      const totalLabelH = labelLines.length * labelLH;
      const gap = 6;
      const totalProseH = visibleProseLines.length * proseLH;
      const totalH = totalLabelH + gap + totalProseH;
      const startY = -totalH / 2;

      labelLines.forEach((line, i) => {
        el.append("text")
          .attr("text-anchor", "middle")
          .attr("y", startY + i * labelLH + v.labelSize * 0.35)
          .attr("fill", v.fg)
          .attr("font-family", "'IBM Plex Sans', sans-serif")
          .attr("font-weight", v.labelWeight)
          .attr("font-size", v.labelSize + "px")
          .attr("pointer-events", "none")
          .text(line);
      });

      const proseStartY = startY + totalLabelH + gap;
      visibleProseLines.forEach((line, i) => {
        let displayLine = line;
        if (truncated && i === visibleProseLines.length - 1) {
          displayLine = line.slice(0, -3) + "\u2026";
        }
        el.append("text")
          .attr("text-anchor", "middle")
          .attr("y", proseStartY + i * proseLH + v.proseSize * 0.35)
          .attr("fill", v.fgMuted)
          .attr("font-family", "'IBM Plex Serif', serif")
          .attr("font-size", v.proseSize + "px")
          .attr("pointer-events", "none")
          .text(displayLine);
      });
    });

    /* Hover interactions */
    nodeG.on("mouseenter", (event, d) => {
      clearTimeout(tooltipTimeout.current);

      // Report hover to parent
      onHoverNode?.(d.id);

      if (!(d.seen || seenIdsRef.current.has(d.id))) {
        markSeen(d.id);
        onMarkSeen?.(d.id);
        const el = d3.select(event.currentTarget);
        const normalV = { ...(HEAT_CONFIG[d.heat] || HEAT_CONFIG.quiet) };

        el.select(".main-circle")
          .transition()
          .duration(800)
          .ease(d3.easeCubicOut)
          .attr("stroke", normalV.color)
          .attr("stroke-width", normalV.strokeW + 1.5);

        el.select(".glow")
          .transition()
          .duration(800)
          .attr("fill", normalV.glow || "transparent");

        el.selectAll("text").each(function () {
          const textEl = d3.select(this);
          const currentFill = textEl.attr("fill");
          if (currentFill === UNSEEN.fg) {
            textEl.transition().duration(800).attr("fill", normalV.fg);
          } else if (currentFill === UNSEEN.fgMuted) {
            textEl
              .transition()
              .duration(800)
              .attr("fill", normalV.fgMuted || normalV.fg);
          }
        });

        linkSel.each(function (l) {
          const targetId = l.target.id || l.target;
          if (targetId === d.id) {
            d3.select(this)
              .transition()
              .duration(800)
              .attr("stroke", "rgb(100,98,90)")
              .attr("stroke-width", 1.5);
          }
        });
      } else {
        d3.select(event.currentTarget)
          .select(".main-circle")
          .transition()
          .duration(200)
          .attr(
            "stroke-width",
            (HEAT_CONFIG[d.heat] || HEAT_CONFIG.quiet).strokeW + 1.5,
          );
      }

      const nodeIsSeen = d.seen || seenIdsRef.current.has(d.id);
      const v = nodeIsSeen
        ? HEAT_CONFIG[d.heat] || HEAT_CONFIG.quiet
        : getNodeVisual(d, false);
      setTooltip({ node: d, cfg: v, x: event.clientX, y: event.clientY });
    });

    nodeG.on("mousemove", (event) => {
      setTooltip((prev) =>
        prev ? { ...prev, x: event.clientX, y: event.clientY } : null,
      );
    });

    nodeG.on("mouseleave", (event, d) => {
      onHoverNode?.(null);
      const normalV = HEAT_CONFIG[d.heat] || HEAT_CONFIG.quiet;
      d3.select(event.currentTarget)
        .select(".main-circle")
        .transition()
        .duration(300)
        .attr("stroke-width", normalV.strokeW);
      tooltipTimeout.current = setTimeout(() => setTooltip(null), 100);
    });

    sim.on("tick", () => {
      linkSel
        .attr("x1", (d) => d.source.x)
        .attr("y1", (d) => d.source.y)
        .attr("x2", (d) => d.target.x)
        .attr("y2", (d) => d.target.y);
      crossLinkSel
        .attr("x1", (d) => d.source.x)
        .attr("y1", (d) => d.source.y)
        .attr("x2", (d) => d.target.x)
        .attr("y2", (d) => d.target.y);
      crossLabelSel
        .attr("x", (d) => (d.source.x + d.target.x) / 2)
        .attr("y", (d) => (d.source.y + d.target.y) / 2 - 4);
      nodeG.attr("transform", (d) => `translate(${d.x},${d.y})`);
    });

    /* Initial zoom to fit */
    setTimeout(() => {
      const pad = 100;
      let minX = Infinity,
        minY = Infinity,
        maxX = -Infinity,
        maxY = -Infinity;
      nodes.forEach((n) => {
        minX = Math.min(minX, n.x - n.radius);
        minY = Math.min(minY, n.y - n.radius);
        maxX = Math.max(maxX, n.x + n.radius);
        maxY = Math.max(maxY, n.y + n.radius);
      });
      const bw = maxX - minX + pad * 2;
      const bh = maxY - minY + pad * 2;
      const scale = Math.min(w / bw, h / bh, 1.0);
      const cx = (minX + maxX) / 2;
      const cy = (minY + maxY) / 2;
      svg
        .transition()
        .duration(900)
        .ease(d3.easeCubicOut)
        .call(
          zoom.transform,
          d3.zoomIdentity
            .translate(w / 2 - cx * scale, h / 2 - cy * scale)
            .scale(scale),
        );
    }, 1800);

    return () => sim.stop();
  }, [dims, tree, crossEdges, onHoverNode, onMarkSeen, personalityColors]);

  if (!tree) return null;

  const unseenCount = countUnseen(tree, seenIds);

  // Build tooltip "from" display
  const tooltipByDisplay = tooltip ? getByDisplay(tooltip.node.by, personalityColors, personalities) : null;

  /* Tooltip positioning */
  const tooltipStyle = tooltip
    ? (() => {
        const tw = 360;
        const margin = 16;
        let left = tooltip.x + 16;
        let top = tooltip.y - 20;
        if (left + tw > dims.w - margin) left = tooltip.x - tw - 16;
        if (top < margin) top = margin;
        return { left, top };
      })()
    : null;

  // Active personality info for legend
  const activeVoices = (activePersonalities || [])
    .map((id) => {
      const p = (personalities || []).find((x) => x.id === id);
      return p ? { id: p.id, name: p.name, color: p.color } : null;
    })
    .filter(Boolean);

  return (
    <>
      <div ref={containerRef} style={{ width: "100%", height: "100%" }}>
        {/* Header */}
        <div className="header">
          <div className="header-left">
            <div className="header-title">Grove</div>
            <div className="header-subtitle">
              drag &middot; scroll to zoom &middot; hover to read
            </div>
          </div>
          <div className="legend">
            <div className="legend-row">
              {Object.entries(HEAT_CONFIG).map(([key, cfg]) => (
                <div key={key} className="legend-item">
                  <div
                    className="legend-dot"
                    style={{
                      background: cfg.color,
                      boxShadow:
                        key === "hot" ? `0 0 6px ${cfg.color}` : "none",
                    }}
                  />
                  <span className="legend-label">{key}</span>
                </div>
              ))}
            </div>
            <div className="legend-row">
              <div className="legend-item">
                <div
                  className="legend-dot"
                  style={{
                    background: UNSEEN.colorBright,
                    boxShadow: `0 0 6px ${UNSEEN.colorBright}`,
                  }}
                />
                <span className="legend-label">new from claude</span>
              </div>
            </div>
            {activeVoices.length > 0 && (
              <div className="legend-row legend-voices">
                {activeVoices.map((v) => (
                  <div key={v.id} className="legend-item">
                    <div
                      className="legend-dot"
                      style={{
                        background: v.color,
                        boxShadow: `0 0 6px ${v.color}`,
                      }}
                    />
                    <span className="legend-label">{v.name}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Unseen indicator */}
        {unseenCount > 0 && (
          <div className="unseen-indicator">
            <span className="unseen-dot" />
            {unseenCount} new thought{unseenCount > 1 ? "s" : ""}
            &mdash; hover to acknowledge
          </div>
        )}

        <svg
          ref={svgRef}
          width={dims.w}
          height={dims.h}
          style={{ display: "block" }}
        />
      </div>

      {/* Tooltip */}
      {tooltip && (
        <div
          className="tooltip"
          style={{
            left: tooltipStyle.left + "px",
            top: tooltipStyle.top + "px",
            borderColor: tooltip.cfg.color,
          }}
        >
          <div className="tooltip-header">
            <div
              className="tooltip-dot"
              style={{
                background: tooltip.cfg.color,
                boxShadow: `0 0 8px ${tooltip.cfg.color}`,
              }}
            />
            <span className="tooltip-label" style={{ color: tooltip.cfg.fg }}>
              {tooltip.node.label}
            </span>
          </div>
          <p className="tooltip-prose" style={{ color: tooltip.cfg.fg }}>
            {tooltip.node.prose}
          </p>
          <div
            className="tooltip-by"
            style={tooltipByDisplay.color ? { color: tooltipByDisplay.color, opacity: 0.8 } : {}}
          >
            {tooltipByDisplay.text}
          </div>
        </div>
      )}
    </>
  );
}

function getByDisplay(by, personalityColors, personalities) {
  if (!by) return { text: "", color: null };
  const personalityId = parsePersonality(by);
  if (personalityId) {
    const p = (personalities || []).find((x) => x.id === personalityId);
    const name = p ? p.name : personalityId;
    const color = personalityColors ? personalityColors[personalityId] : null;
    return { text: `from ${name}`, color };
  }
  if (by === "claude") return { text: "from claude", color: null };
  if (by === "both") return { text: "shared thought", color: null };
  return { text: `from ${by}`, color: null };
}
