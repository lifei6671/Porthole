import { RuleList } from "../components/rule-list";
import type { Rule, RuntimeState } from "../lib/types";

interface RulesPageProps {
  rules: Rule[];
  runtime: RuntimeState | null;
  loading: boolean;
  error: string | null;
  onStartRule: (ruleID: string) => void;
  onStopRule: (ruleID: string) => void;
  onEditRule: (rule: Rule) => void;
  onDeleteRule: (rule: Rule) => void;
}

export function RulesPage(props: RulesPageProps) {
  const runningCount = props.runtime?.active_rule_ids.length ?? 0;

  return (
    <div className="content-stack">
      <section className="mini-metrics-row">
        <article className="mini-metric-card">
          <span>规则总数</span>
          <strong>{props.rules.length}</strong>
        </article>
        <article className="mini-metric-card">
          <span>运行中</span>
          <strong>{runningCount}</strong>
        </article>
        <article className="mini-metric-card">
          <span>默认启用</span>
          <strong>{props.rules.filter((rule) => rule.enabled).length}</strong>
        </article>
      </section>

      <RuleList {...props} />
    </div>
  );
}
