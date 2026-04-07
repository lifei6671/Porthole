import type { RuntimeState } from "../lib/types";

interface StatusBarProps {
  runtime: RuntimeState | null;
  lastError?: string | null;
  notice?: string | null;
}

export function StatusBar({ runtime, lastError, notice }: StatusBarProps) {
  return (
    <section className="status-bar" aria-label="Runtime status bar">
      <article className="status-item">
        <span className="status-label">gost 进程</span>
        <strong>{runtime?.process_status ?? "unknown"}</strong>
      </article>
      <article className="status-item">
        <span className="status-label">运行规则</span>
        <strong>{runtime?.active_rule_ids.length ?? 0}</strong>
      </article>
      <article className="status-item status-item-wide">
        <span className="status-label">最近错误</span>
        <strong>{lastError ?? "暂无"}</strong>
      </article>
      <article className="status-item status-item-wide">
        <span className="status-label">日志预留区</span>
        <strong>{notice ?? "Task 7 将接入完整日志面板"}</strong>
      </article>
    </section>
  );
}
