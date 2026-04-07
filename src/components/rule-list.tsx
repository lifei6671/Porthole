import type { Rule, RuntimeState } from "../lib/types";

interface RuleListProps {
  rules: Rule[];
  runtime: RuntimeState | null;
  loading: boolean;
  error: string | null;
  onStartRule: (ruleID: string) => void;
  onStopRule: (ruleID: string) => void;
  onEditRule: (rule: Rule) => void;
  onDeleteRule: (rule: Rule) => void;
}

function formatEndpoint(host: string, port: number) {
  return `${host}:${port}`;
}

function resolveRuleStatus(runtime: RuntimeState | null, ruleID: string) {
  return runtime?.rule_statuses.find((item) => item.rule_id === ruleID)?.status ?? "stopped";
}

function formatStatusLabel(status: string) {
  switch (status) {
    case "running":
      return "运行中";
    case "starting":
      return "启动中";
    case "stopping":
      return "停止中";
    case "failed":
      return "异常";
    default:
      return "已停止";
  }
}

export function RuleList({
  rules,
  runtime,
  loading,
  error,
  onStartRule,
  onStopRule,
  onEditRule,
  onDeleteRule,
}: RuleListProps) {
  return (
    <section className="panel" aria-label="Rule list">
      <div className="panel-header">
        <div>
          <h2>规则列表</h2>
          <p>显示名称、协议、监听、目标、默认启用和运行状态。</p>
        </div>
      </div>

      {loading ? <p className="placeholder">正在加载规则...</p> : null}
      {error ? <p className="error-text">{error}</p> : null}
      {!loading && !error && !rules.length ? (
        <div className="empty-state">
          <strong>还没有可用规则</strong>
          <p>点击“新增规则”开始配置第一条端口转发。</p>
        </div>
      ) : null}

      {rules.length ? (
        <div className="rule-table">
          <div className="rule-table-head">
            <span>名称</span>
            <span>协议</span>
            <span>监听</span>
            <span>目标</span>
            <span>默认启用</span>
            <span>运行状态</span>
            <span>操作</span>
          </div>
          {rules.map((rule) => {
            const status = resolveRuleStatus(runtime, rule.id);

            return (
              <article className="rule-table-row" key={rule.id}>
                <span className="rule-name-cell" data-label="名称">
                  <span className="rule-name-text">
                    <strong>{rule.name}</strong>
                    <small>{rule.id}</small>
                  </span>
                  <span className="rule-card-summary" aria-hidden="true">
                    <span className={`badge badge-${rule.protocol}`}>
                      {rule.protocol.toUpperCase()}
                    </span>
                    <span className={`status-pill status-${status}`}>
                      {formatStatusLabel(status)}
                    </span>
                    <span className="meta-pill">
                      {rule.enabled ? "已启用" : "未启用"}
                    </span>
                  </span>
                </span>
                <span className="rule-protocol-cell" data-label="协议">
                  <span className={`badge badge-${rule.protocol}`}>
                    {rule.protocol.toUpperCase()}
                  </span>
                </span>
                <span className="mono-cell" data-label="监听">
                  {formatEndpoint(rule.listen_host, rule.listen_port)}
                </span>
                <span className="mono-cell" data-label="目标">
                  {formatEndpoint(rule.target_host, rule.target_port)}
                </span>
                <span className="rule-enabled-cell" data-label="默认启用">
                  {rule.enabled ? "是" : "否"}
                </span>
                <span className="rule-status-cell" data-label="运行状态">
                  <span className={`status-pill status-${status}`}>{status}</span>
                </span>
                <span className="rule-actions" data-label="操作">
                  {status === "running" || status === "starting" ? (
                    <button
                      className="ghost-button mini-button"
                      onClick={() => onStopRule(rule.id)}
                      type="button"
                    >
                      停止
                    </button>
                  ) : (
                    <button
                      className="ghost-button mini-button"
                      onClick={() => onStartRule(rule.id)}
                      type="button"
                    >
                      启动
                    </button>
                  )}
                  <button
                    className="ghost-button mini-button"
                    onClick={() => onEditRule(rule)}
                    type="button"
                  >
                    编辑
                  </button>
                  <button
                    className="ghost-button mini-button danger-button"
                    onClick={() => onDeleteRule(rule)}
                    type="button"
                  >
                    删除
                  </button>
                </span>
              </article>
            );
          })}
        </div>
      ) : null}
    </section>
  );
}
