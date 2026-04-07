import { needsFirewallNotice } from "../lib/validators";
import type { Rule, RuntimeState } from "../lib/types";

interface SettingsPageProps {
  rules: Rule[];
  runtime: RuntimeState | null;
}

export function SettingsPage({ rules, runtime }: SettingsPageProps) {
  const exposedRuleCount = rules.filter((rule) => needsFirewallNotice(rule.listen_host)).length;

  return (
    <div className="settings-grid">
      <section className="panel">
        <div className="panel-header">
          <div>
            <h3>应用信息</h3>
            <p>当前版本、运行状态以及这期实现范围的摘要说明。</p>
          </div>
        </div>

        <div className="info-list">
          <div className="info-row">
            <span>当前版本</span>
            <strong>0.1.0 MVP</strong>
          </div>
          <div className="info-row">
            <span>运行状态</span>
            <strong>{runtime?.process_status ?? "unknown"}</strong>
          </div>
          <div className="info-row">
            <span>规则数量</span>
            <strong>{rules.length}</strong>
          </div>
        </div>
      </section>

      <section className="panel">
        <div className="panel-header">
          <div>
            <h3>运行说明</h3>
            <p>这一页先承载和 Windows 运行有关的关键说明，后续再扩成真正可编辑设置。</p>
          </div>
        </div>

        <ul className="settings-list">
          <li>非回环监听规则启动时，会自动尝试同步 Windows 防火墙入站规则。</li>
          <li>如果系统需要提升权限，会触发 UAC 确认；取消后不会阻断端口转发本身启动。</li>
          <li>当前 UI 先提供 Home / Rules / Logs / Settings 四个工作页，后续再补更多系统设置。</li>
          <li>目前检测到 {exposedRuleCount} 条可能需要对外放行的监听规则。</li>
        </ul>
      </section>
    </div>
  );
}
