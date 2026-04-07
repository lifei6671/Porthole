interface AppToolbarProps {
  onAddRule: () => void;
  onStartAll: () => void;
  onStopAll: () => void;
  onRefresh: () => void;
  busy?: boolean;
}

export function AppToolbar({
  onAddRule,
  onStartAll,
  onStopAll,
  onRefresh,
  busy = false,
}: AppToolbarProps) {
  return (
    <section className="toolbar-card" aria-label="Application toolbar">
      <div>
        <p className="eyebrow">Control Surface</p>
        <h2 className="toolbar-title">规则管理</h2>
      </div>
      <div className="toolbar-actions">
        <button className="primary-button" onClick={onAddRule} type="button">
          新增规则
        </button>
        <button className="ghost-button" disabled={busy} onClick={onStartAll} type="button">
          启动全部
        </button>
        <button className="ghost-button" disabled={busy} onClick={onStopAll} type="button">
          停止全部
        </button>
        <button className="ghost-button" disabled={busy} onClick={onRefresh} type="button">
          刷新状态
        </button>
      </div>
    </section>
  );
}
