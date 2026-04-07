import { useMemo, useState } from "react";

import { LogPanel } from "../components/log-panel";
import type { UILogEntry } from "../lib/types";

interface LogsPageProps {
  logs: UILogEntry[];
  onClearLogs: () => void;
}

export function LogsPage({ logs, onClearLogs }: LogsPageProps) {
  const [sourceFilter, setSourceFilter] = useState<"all" | "app" | "gost">("all");
  const [levelFilter, setLevelFilter] = useState<"all" | "info" | "error">("all");

  const filteredLogs = useMemo(() => {
    return logs.filter((entry) => {
      const sourceMatched = sourceFilter === "all" || entry.source === sourceFilter;
      const levelMatched = levelFilter === "all" || entry.level === levelFilter;
      return sourceMatched && levelMatched;
    });
  }, [levelFilter, logs, sourceFilter]);

  return (
    <div className="content-stack">
      <section className="panel panel-tight">
        <div className="panel-header">
          <div>
            <h3>日志筛选</h3>
            <p>支持按来源和级别快速缩小范围，便于定位问题。</p>
          </div>
        </div>

        <div className="filter-row">
          <label className="field">
            <span>来源</span>
            <select
              value={sourceFilter}
              onChange={(event) => setSourceFilter(event.target.value as typeof sourceFilter)}
            >
              <option value="all">全部</option>
              <option value="app">app</option>
              <option value="gost">gost</option>
            </select>
          </label>

          <label className="field">
            <span>级别</span>
            <select
              value={levelFilter}
              onChange={(event) => setLevelFilter(event.target.value as typeof levelFilter)}
            >
              <option value="all">全部</option>
              <option value="info">info</option>
              <option value="error">error</option>
            </select>
          </label>

          <article className="filter-summary">
            <span>筛选结果</span>
            <strong>{filteredLogs.length} 条</strong>
          </article>
        </div>
      </section>

      <LogPanel entries={filteredLogs} maxEntries={filteredLogs.length || undefined} onClear={onClearLogs} />
    </div>
  );
}
