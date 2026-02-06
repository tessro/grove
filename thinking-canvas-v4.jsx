import { useState, useEffect, useRef, useCallback } from "react";
import * as d3 from "d3";

/*
 * ═══════════════════════════════════════
 *  THINKING TREE DATA
 *  Claude updates this. Shell is permanent.
 *
 *  `by`: "tess" | "claude" | "both"
 *  `seen`: false = Tess hasn't reacted yet
 * ═══════════════════════════════════════
 */
const TREE = {
  id: "root",
  label: "Co-creative planning",
  prose: "We want cofounder energy in planning sessions with Claude. Not call-and-response. Something alive — where both minds are present and the artifact grows between us.",
  heat: "hot",
  by: "both",
  seen: true,
  children: [
    {
      id: "rhythm",
      label: "The rhythm is wrong",
      prose: "Turn-taking is polite but sterile. AskUserQuestion killed the energy and replaced a mind with a form. The fix is generative bursts — not careful convergence.",
      heat: "warm",
      by: "both",
      seen: true,
      children: [
        {
          id: "asymmetry",
          label: "The asymmetry is a feature",
          prose: "You have taste and vision. Claude has speed and memory. Neither takes orders.",
          heat: "growing",
          by: "both",
          seen: true,
          children: [],
        },
        {
          id: "analogues",
          label: "Human analogues",
          prose: "Talking and whiteboarding work because they're lossy on purpose. Digital tools fail because they optimize for capture over flow.",
          heat: "quiet",
          by: "both",
          seen: true,
          children: [],
        },
      ],
    },
    {
      id: "relationship",
      label: "Claude is a collaborator, not a surface",
      prose: "The surface is an extension of Claude. Claude has opinions, gets excited, grows branches you didn't ask for. The artifact is shared thinking.",
      heat: "hot",
      by: "tess",
      seen: true,
      children: [
        {
          id: "modality",
          label: "Talking is the right modality",
          prose: "Because talking is where humans are most fluent. Hands free. You think out loud, Claude materializes. The whiteboard that thinks.",
          heat: "warm",
          by: "both",
          seen: true,
          children: [],
        },
      ],
    },
    {
      id: "medium",
      label: "Finding the medium",
      prose: "Mermaid thinks right but speaks too tersely. Markdown speaks right but thinks too linearly. We need something natively branching AND natively prose-friendly.",
      heat: "hot",
      by: "both",
      seen: true,
      children: [
        {
          id: "mermaid",
          label: "Mermaid: close but not right",
          prose: "Compact and expressive, but prose in boxes is a hack. It told us what we wanted without being the answer.",
          heat: "quiet",
          by: "both",
          seen: true,
          children: [],
        },
        {
          id: "trail",
          label: "Trail journal: too linear",
          prose: "Sequential with depth. Matched linear thinking but lost the nonlinearity we didn't want to give up.",
          heat: "quiet",
          by: "both",
          seen: true,
          children: [],
        },
        {
          id: "pathview",
          label: "Path view: too in-and-out",
          prose: "Tree-shaped data, path-shaped experience. Directional, but the in-out navigation fought against seeing the whole picture.",
          heat: "quiet",
          by: "tess",
          seen: true,
          children: [],
        },
        {
          id: "canvas",
          label: "Force-directed canvas",
          prose: "Everything visible at once. Size = energy. Zoom to read, pull back to see the shape. Claude writes data, physics does layout.",
          heat: "hot",
          by: "both",
          seen: true,
          children: [],
        },
      ],
    },
    /*
     * ═══════════════════════════════════════
     *  🌿 CLAUDE'S REFLECTION — unseen
     * ═══════════════════════════════════════
     */
    {
      id: "claude-reflection-1",
      label: "Heat might be the wrong metaphor",
      prose: "I've been thinking about the heat system — hot, warm, growing, quiet. It implies ideas cool down over time, that entropy is the default. But some ideas aren't hot, they're patient. 'The asymmetry is a feature' has been sitting at 'growing' this whole session but it might be the most important idea here. What if instead of temperature, the dimension is something more like… aliveness? An idea can be quiet and deeply alive. It can be loud and nearly dead.",
      heat: "growing",
      by: "claude",
      seen: false,
      children: [
        {
          id: "claude-ref-1a",
          label: "Aliveness vs. attention",
          prose: "Attention is what we're giving something right now. Aliveness is whether it has more to say. Those are different. The tree should probably track both — what we're looking at, and what's still generative even when we're not.",
          heat: "warm",
          by: "claude",
          seen: false,
          children: [],
        },
        {
          id: "claude-ref-1b",
          label: "Dormant nodes that wake up",
          prose: "If an idea is patient rather than cold, it might suddenly become relevant three sessions from now when context shifts. The data format should let me mark something as dormant-but-alive, distinct from genuinely spent. Maybe a second axis: heat is current focus, depth is potential energy.",
          heat: "growing",
          by: "claude",
          seen: false,
          children: [],
        },
      ],
    },
  ],
};

/* ─── Color systems ─── */

/* Normal heat palette (seen nodes) */
const HEAT_CONFIG = {
  hot:     { baseR: 100, color: "#e8642c", fg: "#fff8f0", fgMuted: "rgba(255,248,240,0.7)", bg: "rgba(232,100,44,0.10)", glow: "rgba(232,100,44,0.3)", strokeW: 2, labelSize: 15, proseSize: 11.5, proseLines: 4, labelWeight: 700, charWidth: 22 },
  warm:    { baseR: 78,  color: "#d6a648", fg: "#f0e8da", fgMuted: "rgba(240,232,218,0.6)", bg: "rgba(214,166,72,0.07)", glow: "rgba(214,166,72,0.2)", strokeW: 1.5, labelSize: 13, proseSize: 10.5, proseLines: 3, labelWeight: 600, charWidth: 18 },
  growing: { baseR: 60,  color: "#78b478", fg: "#dce8dc", fgMuted: "rgba(220,232,220,0.55)", bg: "rgba(120,180,120,0.06)", glow: "rgba(120,180,120,0.15)", strokeW: 1.5, labelSize: 12, proseSize: 9.5, proseLines: 2, labelWeight: 600, charWidth: 16 },
  quiet:   { baseR: 46,  color: "#706e66", fg: "#a09e96", fgMuted: "rgba(160,158,152,0.5)", bg: "rgba(70,70,68,0.06)", glow: "transparent", strokeW: 1.5, labelSize: 11, proseSize: 9, proseLines: 1, labelWeight: 500, charWidth: 13 },
};

/* Unseen palette — cool cyan/blue, visually distinct */
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

function getNodeVisual(node) {
  const base = HEAT_CONFIG[node.heat] || HEAT_CONFIG.quiet;
  if (!node.seen) {
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

/* ─── Flatten ─── */
function flatten(node, parentId = null) {
  const nodes = [];
  const links = [];
  const base = HEAT_CONFIG[node.heat] || HEAT_CONFIG.quiet;
  nodes.push({ ...node, radius: base.baseR, children: undefined, _children: node.children });
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

export default function ThinkingCanvas() {
  const svgRef = useRef(null);
  const containerRef = useRef(null);
  const [dims, setDims] = useState({ w: 800, h: 600 });
  const [tooltip, setTooltip] = useState(null);
  const [seenIds, setSeenIds] = useState(new Set());
  const tooltipTimeout = useRef(null);

  const markSeen = useCallback((id) => {
    setSeenIds(prev => {
      if (prev.has(id)) return prev;
      const next = new Set(prev);
      next.add(id);
      return next;
    });
  }, []);

  /* Is a node visually "seen"? Either originally seen or user has hovered */
  const isSeen = useCallback((node) => {
    return node.seen || seenIds.has(node.id);
  }, [seenIds]);

  useEffect(() => {
    const link = document.createElement("link");
    link.href = "https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600;700&family=IBM+Plex+Serif:ital,wght@0,400;1,400&display=swap";
    link.rel = "stylesheet";
    document.head.appendChild(link);
  }, []);

  useEffect(() => {
    const measure = () => {
      if (containerRef.current) setDims({ w: containerRef.current.clientWidth, h: containerRef.current.clientHeight });
    };
    measure();
    window.addEventListener("resize", measure);
    return () => window.removeEventListener("resize", measure);
  }, []);

  useEffect(() => {
    if (!svgRef.current || dims.w === 0) return;

    const svg = d3.select(svgRef.current);
    svg.selectAll("*").remove();

    const [nodes, links] = flatten(TREE);
    const { w, h } = dims;

    const defs = svg.append("defs");

    /* Glow filters for both palettes */
    ["hot", "warm", "growing"].forEach(key => {
      const cfg = HEAT_CONFIG[key];
      const filter = defs.append("filter").attr("id", `glow-${key}`).attr("x", "-80%").attr("y", "-80%").attr("width", "260%").attr("height", "260%");
      filter.append("feGaussianBlur").attr("in", "SourceGraphic").attr("stdDeviation", key === "hot" ? 12 : 7);
      const merge = filter.append("feMerge");
      merge.append("feMergeNode").attr("in", "blur");
      merge.append("feMergeNode").attr("in", "SourceGraphic");
    });

    /* Unseen glow */
    const unseenFilter = defs.append("filter").attr("id", "glow-unseen").attr("x", "-80%").attr("y", "-80%").attr("width", "260%").attr("height", "260%");
    unseenFilter.append("feGaussianBlur").attr("in", "SourceGraphic").attr("stdDeviation", 14);
    const unseenMerge = unseenFilter.append("feMerge");
    unseenMerge.append("feMergeNode").attr("in", "blur");
    unseenMerge.append("feMergeNode").attr("in", "SourceGraphic");

    /* Pulse animation */
    const style = document.createElementNS("http://www.w3.org/2000/svg", "style");
    style.textContent = `
      @keyframes pulse-unseen {
        0%, 100% { opacity: 0.35; }
        50% { opacity: 0.55; }
      }
      .unseen-glow { animation: pulse-unseen 3s ease-in-out infinite; }
      @keyframes fade-to-normal {
        from { opacity: 1; }
        to { opacity: 0; }
      }
    `;
    svg.node().appendChild(style);

    const g = svg.append("g");
    const zoom = d3.zoom().scaleExtent([0.2, 5]).on("zoom", (event) => g.attr("transform", event.transform));
    svg.call(zoom);

    const sim = d3.forceSimulation(nodes)
      .force("link", d3.forceLink(links).id(d => d.id).distance(d => {
        const sn = nodes.find(n => n.id === (d.source.id || d.source));
        const tn = nodes.find(n => n.id === (d.target.id || d.target));
        return (sn?.radius || 60) + (tn?.radius || 60) + 40;
      }).strength(0.7))
      .force("charge", d3.forceManyBody().strength(d => -d.radius * 16))
      .force("center", d3.forceCenter(w / 2, h / 2))
      .force("collision", d3.forceCollide().radius(d => d.radius + 12))
      .force("x", d3.forceX(w / 2).strength(0.03))
      .force("y", d3.forceY(h / 2).strength(0.03));

    /* Links */
    const linkSel = g.append("g").selectAll("line").data(links).join("line")
      .attr("stroke-dasharray", "6,5")
      .attr("opacity", 0.5)
      .each(function(d) {
        const target = nodes.find(n => n.id === (d.target.id || d.target));
        const isUnseen = target && !target.seen;
        d3.select(this)
          .attr("stroke", isUnseen ? UNSEEN.linkColor : "rgb(50,50,47)")
          .attr("stroke-width", isUnseen ? 1.8 : 1);
      });

    /* Node groups */
    const nodeG = g.append("g").selectAll("g").data(nodes).join("g")
      .attr("cursor", "grab")
      .call(d3.drag()
        .on("start", (event, d) => { if (!event.active) sim.alphaTarget(0.3).restart(); d.fx = d.x; d.fy = d.y; })
        .on("drag", (event, d) => { d.fx = event.x; d.fy = event.y; })
        .on("end", (event, d) => { if (!event.active) sim.alphaTarget(0); d.fx = null; d.fy = null; })
      );

    /* Render each node */
    nodeG.each(function(d) {
      const el = d3.select(this);
      const v = getNodeVisual(d);

      /* Glow */
      if (v.glow !== "transparent") {
        el.append("circle")
          .attr("r", d.radius + 10)
          .attr("fill", v.glow)
          .attr("filter", d.seen ? `url(#glow-${d.heat})` : "url(#glow-unseen)")
          .attr("class", d.seen ? "glow" : "glow unseen-glow");
      }

      /* Main circle */
      el.append("circle")
        .attr("r", d.radius)
        .attr("fill", v.bg)
        .attr("stroke", v.color)
        .attr("stroke-width", v.strokeW)
        .attr("class", "main-circle");

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
          displayLine = line.slice(0, -3) + "…";
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

      /* Mark as seen — transition colors */
      if (!d.seen) {
        markSeen(d.id);
        const el = d3.select(event.currentTarget);
        const normalV = { ...HEAT_CONFIG[d.heat] || HEAT_CONFIG.quiet };

        el.select(".main-circle")
          .transition().duration(800).ease(d3.easeCubicOut)
          .attr("stroke", normalV.color)
          .attr("stroke-width", normalV.strokeW + 1.5);

        el.select(".glow")
          .transition().duration(800)
          .attr("fill", normalV.glow !== undefined ? normalV.glow : "transparent");

        /* Transition text colors */
        el.selectAll("text").each(function(_, i) {
          const textEl = d3.select(this);
          const currentFill = textEl.attr("fill");
          if (currentFill === UNSEEN.fg) {
            textEl.transition().duration(800).attr("fill", normalV.fg);
          } else if (currentFill === UNSEEN.fgMuted) {
            textEl.transition().duration(800).attr("fill", normalV.fgMuted || normalV.fg);
          }
        });

        /* Transition link colors */
        linkSel.each(function(l) {
          const targetId = l.target.id || l.target;
          if (targetId === d.id) {
            d3.select(this).transition().duration(800)
              .attr("stroke", "rgb(50,50,47)")
              .attr("stroke-width", 1);
          }
        });
      } else {
        d3.select(event.currentTarget).select(".main-circle")
          .transition().duration(200)
          .attr("stroke-width", (HEAT_CONFIG[d.heat] || HEAT_CONFIG.quiet).strokeW + 1.5);
      }

      const v = d.seen || seenIds.has(d.id)
        ? (HEAT_CONFIG[d.heat] || HEAT_CONFIG.quiet)
        : getNodeVisual(d);
      setTooltip({ node: d, cfg: v, x: event.clientX, y: event.clientY });
    });

    nodeG.on("mousemove", (event) => {
      setTooltip(prev => prev ? { ...prev, x: event.clientX, y: event.clientY } : null);
    });

    nodeG.on("mouseleave", (event, d) => {
      const normalV = HEAT_CONFIG[d.heat] || HEAT_CONFIG.quiet;
      d3.select(event.currentTarget).select(".main-circle")
        .transition().duration(300)
        .attr("stroke-width", normalV.strokeW);
      tooltipTimeout.current = setTimeout(() => setTooltip(null), 100);
    });

    sim.on("tick", () => {
      linkSel.attr("x1", d => d.source.x).attr("y1", d => d.source.y)
        .attr("x2", d => d.target.x).attr("y2", d => d.target.y);
      nodeG.attr("transform", d => `translate(${d.x},${d.y})`);
    });

    setTimeout(() => {
      const pad = 100;
      let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
      nodes.forEach(n => {
        minX = Math.min(minX, n.x - n.radius); minY = Math.min(minY, n.y - n.radius);
        maxX = Math.max(maxX, n.x + n.radius); maxY = Math.max(maxY, n.y + n.radius);
      });
      const bw = maxX - minX + pad * 2;
      const bh = maxY - minY + pad * 2;
      const scale = Math.min(w / bw, h / bh, 1.0);
      const cx = (minX + maxX) / 2;
      const cy = (minY + maxY) / 2;
      svg.transition().duration(900).ease(d3.easeCubicOut).call(
        zoom.transform,
        d3.zoomIdentity.translate(w / 2 - cx * scale, h / 2 - cy * scale).scale(scale)
      );
    }, 1800);

    return () => sim.stop();
  }, [dims, seenIds]);

  const tooltipStyle = tooltip ? (() => {
    const tw = 360; const margin = 16;
    let left = tooltip.x + 16; let top = tooltip.y - 20;
    if (left + tw > dims.w - margin) left = tooltip.x - tw - 16;
    if (top < margin) top = margin;
    return { left, top };
  })() : null;

  /* Count unseen */
  const countUnseen = (node) => {
    let c = node.seen ? 0 : 1;
    for (const child of node.children || []) c += countUnseen(child);
    return c;
  };
  const unseenCount = countUnseen(TREE) - seenIds.size;

  return (
    <div ref={containerRef} style={{
      width: "100%", height: "100vh", background: "rgb(20, 20, 18)",
      position: "relative", overflow: "hidden", fontFamily: "'IBM Plex Sans', sans-serif",
    }}>
      {/* Header */}
      <div style={{ position: "absolute", top: "16px", left: "20px", zIndex: 10, pointerEvents: "none" }}>
        <div style={{ fontSize: "12px", fontWeight: 600, letterSpacing: "0.06em", textTransform: "uppercase", color: "rgb(80, 78, 72)" }}>
          🌱 Thinking Tree
        </div>
        <div style={{ fontFamily: "'IBM Plex Mono', monospace", fontSize: "10px", color: "rgb(55, 53, 48)", marginTop: "3px" }}>
          drag · scroll to zoom · hover to read
        </div>
      </div>

      {/* New thoughts indicator */}
      {unseenCount > 0 && (
        <div style={{
          position: "absolute", top: "16px", left: "50%", transform: "translateX(-50%)",
          zIndex: 10, pointerEvents: "none",
          background: "rgba(79,196,207,0.1)", border: "1px solid rgba(79,196,207,0.3)",
          borderRadius: "20px", padding: "6px 16px",
          fontFamily: "'IBM Plex Mono', monospace", fontSize: "11px", color: UNSEEN.colorBright,
          letterSpacing: "0.02em",
        }}>
          <span style={{ display: "inline-block", width: "6px", height: "6px", borderRadius: "50%", background: UNSEEN.colorBright, marginRight: "8px", boxShadow: `0 0 8px ${UNSEEN.colorBright}` }} />
          {unseenCount} new thought{unseenCount > 1 ? "s" : ""} from Claude — hover to acknowledge
        </div>
      )}

      {/* Legend */}
      <div style={{
        position: "absolute", top: "16px", right: "20px", zIndex: 10,
        display: "flex", flexDirection: "column", gap: "6px", pointerEvents: "none",
      }}>
        <div style={{ display: "flex", gap: "12px" }}>
          {Object.entries(HEAT_CONFIG).map(([key, cfg]) => (
            <div key={key} style={{ display: "flex", alignItems: "center", gap: "5px" }}>
              <div style={{
                width: "7px", height: "7px", borderRadius: "50%", background: cfg.color,
                boxShadow: key === "hot" ? `0 0 6px ${cfg.color}` : "none",
              }} />
              <span style={{ fontFamily: "'IBM Plex Mono', monospace", fontSize: "9px", color: "rgb(80, 78, 72)", textTransform: "uppercase", letterSpacing: "0.05em" }}>{key}</span>
            </div>
          ))}
        </div>
        <div style={{ display: "flex", gap: "12px" }}>
          <div style={{ display: "flex", alignItems: "center", gap: "5px" }}>
            <div style={{ width: "7px", height: "7px", borderRadius: "50%", background: UNSEEN.colorBright, boxShadow: `0 0 6px ${UNSEEN.colorBright}` }} />
            <span style={{ fontFamily: "'IBM Plex Mono', monospace", fontSize: "9px", color: "rgb(80, 78, 72)", textTransform: "uppercase", letterSpacing: "0.05em" }}>new from claude</span>
          </div>
        </div>
      </div>

      <svg ref={svgRef} width={dims.w} height={dims.h} style={{ display: "block" }} />

      {/* Tooltip */}
      {tooltip && (
        <div style={{
          position: "fixed", left: tooltipStyle.left + "px", top: tooltipStyle.top + "px",
          width: "360px", background: "rgba(28, 28, 26, 0.96)", backdropFilter: "blur(16px)",
          border: `1px solid ${tooltip.cfg.color}`, borderRadius: "10px", padding: "20px 22px",
          zIndex: 30, pointerEvents: "none",
          boxShadow: `0 12px 40px rgba(0,0,0,0.5)`,
          animation: "fadeIn 0.15s ease-out",
        }}>
          <style>{`@keyframes fadeIn { from { opacity: 0; transform: translateY(4px); } to { opacity: 1; transform: translateY(0); } }`}</style>
          <div style={{ display: "flex", alignItems: "center", gap: "8px", marginBottom: "10px" }}>
            <span style={{
              width: "7px", height: "7px", borderRadius: "50%", background: tooltip.cfg.color,
              boxShadow: `0 0 8px ${tooltip.cfg.color}`, flexShrink: 0,
            }} />
            <span style={{ fontWeight: 600, fontSize: "14px", color: tooltip.cfg.fg, letterSpacing: "-0.01em" }}>
              {tooltip.node.label}
            </span>
          </div>
          <p style={{
            fontFamily: "'IBM Plex Serif', serif", fontSize: "13px", lineHeight: 1.7,
            color: tooltip.cfg.fg, margin: 0, opacity: 0.85,
          }}>
            {tooltip.node.prose}
          </p>
        </div>
      )}
    </div>
  );
}
