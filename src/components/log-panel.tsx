import type { UILogEntry } from "../lib/types";

export function normalizeLogLevel(entry: UILogEntry) {
  return entry.level.toUpperCase();
}

interface LogPanelProps {
  entries: UILogEntry[];
  onClear: () => void;
  maxEntries?: number;
}

export function LogPanel({ entries, maxEntries = 14, onClear }: LogPanelProps) {
  return (
    <section className="panel" aria-label="Log panel">
      <div className="panel-header">
        <div>
          <h2>日志面板</h2>
          <p>展示 app / gost 两类日志，最近事件优先显示。</p>
        </div>
        <button className="ghost-button" onClick={onClear} type="button">
          清空日志
        </button>
      </div>

      <div className="log-panel">
        {entries.length ? (
          entries.slice(-maxEntries).map((entry) => (
            <article className="log-entry" key={entry.id}>
              <span className={`log-tag log-tag-${entry.source}`}>{entry.source}</span>
              <span className={`log-tag log-tag-${entry.level}`}>
                {normalizeLogLevel(entry)}
              </span>
              <time>{new Date(entry.observedAt).toLocaleTimeString()}</time>
              <p>{entry.message}</p>
            </article>
          ))
        ) : (
          <p className="placeholder">暂时没有日志。</p>
        )}
      </div>
    </section>
  );
}
