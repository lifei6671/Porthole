import { needsFirewallNotice } from "../lib/validators";
import type { Rule, RuntimeState, UILogEntry } from "../lib/types";

interface HomePageProps {
  runtime: RuntimeState | null;
  rules: Rule[];
  logs: UILogEntry[];
  processExitReason: string | null;
  onDismissProcessExitReason: () => void;
  onOpenRules: () => void;
  onOpenLogs: () => void;
}

function formatStatusLabel(status: RuntimeState["process_status"] | "unknown") {
  switch (status) {
    case "running":
      return "运行中";
    case "starting":
      return "启动中";
    case "stopping":
      return "停止中";
    case "failed":
      return "异常";
    case "stopped":
      return "已停止";
    default:
      return "未知";
  }
}

export function HomePage({
  runtime,
  rules,
  logs,
  processExitReason,
  onDismissProcessExitReason,
  onOpenRules,
  onOpenLogs,
}: HomePageProps) {
  const exposedRuleCount = rules.filter((rule) => needsFirewallNotice(rule.listen_host)).length;
  const recentLogs = logs.slice(-6).reverse();

  return (
    <div className="content-stack">
      <section className="home-hero-grid">
        <article className="overview-card overview-card-hero">
          <span className="overview-label">gost 运行状态</span>
          <strong className="overview-value">
            {formatStatusLabel(runtime?.process_status ?? "unknown")}
          </strong>
          <p>
            当前活跃规则 {runtime?.active_rule_ids.length ?? 0} 条，适合从这里快速判断转发器
            是否已经成功工作。
          </p>
          <div className="inline-actions">
            <button className="ghost-button" onClick={onOpenRules} type="button">
              查看规则
            </button>
            <button className="ghost-button" onClick={onOpenLogs} type="button">
              查看日志
            </button>
          </div>
        </article>

        <article className="overview-card">
          <span className="overview-label">已保存规则</span>
          <strong className="overview-value">{rules.length}</strong>
          <p>所有 TCP/UDP 转发规则都会集中在 Rules 页面统一管理。</p>
        </article>

        <article className="overview-card">
          <span className="overview-label">对外监听规则</span>
          <strong className="overview-value">{exposedRuleCount}</strong>
          <p>非回环监听规则启动时，会自动同步 Windows 防火墙入站规则。</p>
        </article>
      </section>

      <section className="home-grid">
        <article className="panel">
          <div className="panel-header">
            <div>
              <h3>近期运行状态</h3>
              <p>汇总最近的错误、退出原因和当前进程状态。</p>
            </div>
          </div>

          <div className="info-list">
            <div className="info-row">
              <span>进程状态</span>
              <strong>{formatStatusLabel(runtime?.process_status ?? "unknown")}</strong>
            </div>
            <div className="info-row">
              <span>当前运行规则</span>
              <strong>{runtime?.active_rule_ids.length ?? 0}</strong>
            </div>
            <div className="info-row">
              <span>最近错误</span>
              <strong>{runtime?.last_error?.summary ?? "暂无错误"}</strong>
            </div>
          </div>

          {processExitReason ? (
            <button className="exit-notice" onClick={onDismissProcessExitReason} type="button">
              最近一次 gost 进程退出：{processExitReason}
            </button>
          ) : null}
        </article>

        <article className="panel">
          <div className="panel-header">
            <div>
              <h3>最近日志</h3>
              <p>只展示最近几条关键日志，完整内容请切到 Logs 页面。</p>
            </div>
            <button className="ghost-button" onClick={onOpenLogs} type="button">
              查看全部
            </button>
          </div>

          <div className="recent-log-list">
            {recentLogs.length ? (
              recentLogs.map((entry) => (
                <article className="recent-log-item" key={entry.id}>
                  <div className="recent-log-meta">
                    <span className={`log-tag log-tag-${entry.source}`}>{entry.source}</span>
                    <span className={`log-tag log-tag-${entry.level}`}>{entry.level}</span>
                    <time>{new Date(entry.observedAt).toLocaleTimeString()}</time>
                  </div>
                  <p>{entry.message}</p>
                </article>
              ))
            ) : (
              <p className="placeholder">目前还没有新的运行日志。</p>
            )}
          </div>
        </article>
      </section>
    </div>
  );
}
