import type { Rule, RuntimeState } from "../lib/types";

interface RuleListProps {
  rules: Rule[];
  runtime: RuntimeState | null;
  loading: boolean;
  error: string | null;
}

function formatEndpoint(host: string, port: number) {
  return `${host}:${port}`;
}

function resolveRuleStatus(runtime: RuntimeState | null, ruleID: string) {
  return runtime?.rule_statuses.find((item) => item.rule_id === ruleID)?.status ?? "stopped";
}

export function RuleList({ rules, runtime, loading, error }: RuleListProps) {
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
                <span className="rule-name-cell">
                  <strong>{rule.name}</strong>
                  <small>{rule.id}</small>
                </span>
                <span>
                  <span className={`badge badge-${rule.protocol}`}>
                    {rule.protocol.toUpperCase()}
                  </span>
                </span>
                <span className="mono-cell">
                  {formatEndpoint(rule.listen_host, rule.listen_port)}
                </span>
                <span className="mono-cell">
                  {formatEndpoint(rule.target_host, rule.target_port)}
                </span>
                <span>{rule.enabled ? "是" : "否"}</span>
                <span>
                  <span className={`status-pill status-${status}`}>{status}</span>
                </span>
                <span className="rule-actions-placeholder">单条操作在 Task 7 实现</span>
              </article>
            );
          })}
        </div>
      ) : null}
    </section>
  );
}
