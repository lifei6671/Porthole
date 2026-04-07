import type { Rule, RuntimeState } from "../lib/types";
import { needsFirewallNotice } from "../lib/validators";

interface StatusBarProps {
  runtime: RuntimeState | null;
  rules: Rule[];
  lastError?: string | null;
  notice?: string | null;
}

export function StatusBar({ runtime, rules, lastError, notice }: StatusBarProps) {
  const exposedRules = rules.filter((rule) => needsFirewallNotice(rule.listen_host));

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
        <strong>{notice ?? "日志面板已接入，显示最近事件"}</strong>
      </article>
      <article className="status-item status-item-wide">
        <span className="status-label">防火墙提示</span>
        <strong>
          {exposedRules.length
            ? `检测到 ${exposedRules.length} 条非回环监听规则，启动时会自动申请放行 Windows 防火墙`
            : "当前未检测到需要额外放行的监听地址"}
        </strong>
      </article>
    </section>
  );
}
