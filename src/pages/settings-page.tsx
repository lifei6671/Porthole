import { needsFirewallNotice } from "../lib/validators";
import type { Rule, RuntimeState } from "../lib/types";

interface SettingsPageProps {
  rules: Rule[];
  runtime: RuntimeState | null;
}

export function SettingsPage({ rules, runtime }: SettingsPageProps) {
  const exposedRuleCount = rules.filter((rule) => needsFirewallNotice(rule.listen_host)).length;

  return (
    <div className="content-stack">
      <section className="mini-metrics-row">
        <article className="mini-metric-card">
          <span>当前版本</span>
          <strong>0.1.0 MVP</strong>
        </article>
        <article className="mini-metric-card">
          <span>运行状态</span>
          <strong>{runtime?.process_status ?? "unknown"}</strong>
        </article>
        <article className="mini-metric-card">
          <span>外放规则</span>
          <strong>{exposedRuleCount}</strong>
        </article>
      </section>

      <div className="settings-grid">
        <section className="panel">
          <div className="panel-header">
            <div>
              <h3>应用信息</h3>
              <p>当前版本、运行状态以及这期 MVP 的核心能力范围。</p>
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
            <div className="info-row">
              <span>界面结构</span>
              <strong>首页 / 规则 / 日志 / 设置</strong>
            </div>
          </div>
        </section>

        <section className="panel">
          <div className="panel-header">
            <div>
              <h3>Windows 运行策略</h3>
              <p>当前版本已经落地的系统行为，以及需要特别注意的交互说明。</p>
            </div>
          </div>

          <ul className="settings-list">
            <li>非回环监听规则启动时，会自动尝试同步 Windows 防火墙入站规则。</li>
            <li>如果系统需要提升权限，会触发 UAC 确认；取消后不会阻断端口转发本身启动。</li>
            <li>关闭主窗口时会先询问是否隐藏到托盘，隐藏后可通过托盘恢复应用。</li>
            <li>当前检测到 {exposedRuleCount} 条可能需要对外放行的监听规则。</li>
          </ul>
        </section>
      </div>

      <section className="panel panel-tight">
        <div className="panel-header">
          <div>
            <h3>本期范围说明</h3>
            <p>这一页先承载说明型信息，后续再扩展成真正可编辑的系统设置页。</p>
          </div>
        </div>

        <ul className="settings-list">
          <li>当前版本优先保证规则管理、日志查看、Windows 防火墙同步和托盘驻留稳定可用。</li>
          <li>更细粒度的设置项，例如自启动、主题或导出导入，会放到后续版本逐步补齐。</li>
          <li>如果需要排查异常，优先切到 Logs 页面查看 app 与 gost 的混合日志。</li>
        </ul>
      </section>
    </div>
  );
}
