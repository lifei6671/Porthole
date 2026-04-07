import brandIcon from "../../src-tauri/icons/icon.png";

import type { ProcessStatus } from "../lib/types";

export type AppPage = "home" | "rules" | "logs" | "settings";

interface SidebarProps {
  currentPage: AppPage;
  onNavigate: (page: AppPage) => void;
  processStatus: ProcessStatus | "unknown";
  activeRuleCount: number;
  totalRuleCount: number;
}

const navigationItems: Array<{ id: AppPage; label: string; hint: string }> = [
  { id: "home", label: "首页", hint: "总览与快捷操作" },
  { id: "rules", label: "规则", hint: "规则与端口转发" },
  { id: "logs", label: "日志", hint: "运行日志与排障" },
  { id: "settings", label: "设置", hint: "应用设置与说明" },
];

function formatProcessStatus(status: SidebarProps["processStatus"]) {
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

export function Sidebar({
  currentPage,
  onNavigate,
  processStatus,
  activeRuleCount,
  totalRuleCount,
}: SidebarProps) {
  return (
    <aside className="sidebar-shell" aria-label="Primary navigation">
      <div className="sidebar-brand">
        <img alt="Porthole brand mark" className="sidebar-brand-icon" src={brandIcon} />
        <div>
          <p className="sidebar-brand-kicker">Windows Port Forwarding</p>
          <h1>Porthole</h1>
        </div>
      </div>

      <div className="sidebar-runtime-card">
        <span className={`sidebar-status-pill status-${processStatus}`}>
          {formatProcessStatus(processStatus)}
        </span>
        <dl className="sidebar-runtime-grid">
          <div>
            <dt>运行规则</dt>
            <dd>{activeRuleCount}</dd>
          </div>
          <div>
            <dt>全部规则</dt>
            <dd>{totalRuleCount}</dd>
          </div>
        </dl>
      </div>

      <nav className="sidebar-nav">
        {navigationItems.map((item) => (
          <button
            key={item.id}
            className={`sidebar-nav-item ${currentPage === item.id ? "is-active" : ""}`}
            onClick={() => onNavigate(item.id)}
            type="button"
          >
            <strong>{item.label}</strong>
            <span>{item.hint}</span>
          </button>
        ))}
      </nav>
    </aside>
  );
}
