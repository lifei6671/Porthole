import { clearLogs } from "./lib/api";
import { AppToolbar } from "./components/app-toolbar";
import { RuleList } from "./components/rule-list";
import { StatusBar } from "./components/status-bar";
import { useRules } from "./hooks/use-rules";
import { useRuntimeEvents } from "./hooks/use-runtime-events";

export function App() {
  const {
    rules,
    loading: rulesLoading,
    error: rulesError,
    refresh: refreshRules,
  } = useRules();
  const {
    runtime,
    logs,
    processExitReason,
    loading: runtimeLoading,
    error: runtimeError,
    refresh: refreshRuntime,
    startAll,
    stopAll,
    clearLocalLogs,
    clearProcessExitReason,
  } = useRuntimeEvents();

  const activeRuleCount = runtime?.active_rule_ids.length ?? 0;
  const busy = rulesLoading || runtimeLoading;

  async function handleClearLogs() {
    await clearLogs();
    clearLocalLogs();
  }

  async function handleRefresh() {
    await Promise.all([refreshRules(), refreshRuntime()]);
  }

  function handleAddRule() {
    window.alert("规则弹窗将在 Task 7 实现。");
  }

  return (
    <main className="app-shell">
      <header className="app-header">
        <div>
          <p className="eyebrow">Windows Port Forwarding MVP</p>
          <h1 className="app-title">Porthole</h1>
        </div>
        <div className="header-metrics">
          <article className="metric-card">
            <span className="metric-label">进程状态</span>
            <strong className="metric-value">
              {runtime?.process_status ?? "loading"}
            </strong>
          </article>
          <article className="metric-card">
            <span className="metric-label">当前运行规则</span>
            <strong className="metric-value">{activeRuleCount}</strong>
          </article>
          <article className="metric-card">
            <span className="metric-label">已保存规则</span>
            <strong className="metric-value">{rules.length}</strong>
          </article>
        </div>
      </header>

      <AppToolbar
        busy={busy}
        onAddRule={handleAddRule}
        onRefresh={handleRefresh}
        onStartAll={startAll}
        onStopAll={stopAll}
      />

      <section className="workspace" aria-label="Application workspace">
        <RuleList
          error={rulesError}
          loading={rulesLoading}
          rules={rules}
          runtime={runtime}
        />

        <section className="panel">
          <div className="panel-header">
            <div>
              <h2>运行态与日志</h2>
              <p>显示 `get_runtime_status` 快照与运行期事件流。</p>
            </div>
            <button className="ghost-button" onClick={handleClearLogs} type="button">
              清空日志
            </button>
          </div>
          {runtimeLoading ? <p className="placeholder">正在加载运行态...</p> : null}
          {runtimeError ? <p className="error-text">{runtimeError}</p> : null}
          {runtime?.last_error ? (
            <p className="error-text">最近错误：{runtime.last_error.summary}</p>
          ) : null}
          {processExitReason ? (
            <button
              className="exit-notice"
              onClick={clearProcessExitReason}
              type="button"
            >
              进程退出：{processExitReason}，点击关闭提示
            </button>
          ) : null}
          <div className="runtime-grid">
            <article className="runtime-card">
              <span className="metric-label">运行状态</span>
              <strong className="metric-value">
                {runtime?.process_status ?? "unknown"}
              </strong>
            </article>
            <article className="runtime-card">
              <span className="metric-label">日志条数</span>
              <strong className="metric-value">{logs.length}</strong>
            </article>
          </div>
          <div className="log-panel">
            {logs.length ? (
              logs.slice(-12).map((entry) => (
                <article
                  className="log-entry"
                  key={`${entry.observed_at}-${entry.message}`}
                >
                  <span className={`log-source log-source-${entry.source.toLowerCase()}`}>
                    {entry.source}
                  </span>
                  <time>{new Date(entry.observed_at).toLocaleTimeString()}</time>
                  <p>{entry.message}</p>
                </article>
              ))
            ) : (
              <p className="placeholder">暂时没有日志。</p>
            )}
          </div>
        </section>
      </section>

      <StatusBar
        lastError={runtime?.last_error?.summary ?? runtimeError}
        notice={processExitReason}
        runtime={runtime}
      />
    </main>
  );
}
