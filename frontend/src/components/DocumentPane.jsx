import Markdown from "react-markdown";

export default function DocumentPane({
  summary,
  summaryLoading,
  summaryStale,
  summaryVoice,
  onVoiceChange,
  onRefresh,
  onClose,
  voiceOptions,
}) {
  return (
    <div className="document-pane">
      <div className="document-pane-header">
        <div className="document-pane-controls">
          <select
            className="document-pane-voice-select"
            value={summaryVoice}
            onChange={(e) => onVoiceChange(e.target.value)}
            disabled={summaryLoading}
          >
            {voiceOptions.map((v) => (
              <option key={v.id} value={v.id}>
                {v.name}
              </option>
            ))}
          </select>
          <button
            className="document-pane-refresh"
            onClick={onRefresh}
            disabled={summaryLoading}
            title="Refresh summary"
          >
            {summaryLoading ? "\u21BB" : summaryStale ? "\u21BB stale" : "\u21BB"}
          </button>
        </div>
        <button className="document-pane-close" onClick={onClose}>
          &times;
        </button>
      </div>
      <div className="document-pane-body">
        {summaryLoading && !summary ? (
          <div className="document-pane-loading">
            <div className="loading-spinner" />
            <span>Generating summary...</span>
          </div>
        ) : summary ? (
          <div className="document-pane-content">
            <Markdown>{summary}</Markdown>
          </div>
        ) : (
          <div className="document-pane-empty">
            Click refresh to generate a summary of the tree.
          </div>
        )}
      </div>
    </div>
  );
}
