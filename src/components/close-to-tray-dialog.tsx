interface CloseToTrayDialogProps {
  open: boolean;
  busy?: boolean;
  onCancel: () => void;
  onHideToTray: () => void;
  onExitApplication: () => void;
}

export function CloseToTrayDialog({
  open,
  busy = false,
  onCancel,
  onHideToTray,
  onExitApplication,
}: CloseToTrayDialogProps) {
  if (!open) {
    return null;
  }

  return (
    <div className="dialog-backdrop" role="presentation">
      <div
        aria-labelledby="close-to-tray-dialog-title"
        aria-modal="true"
        className="dialog-card close-to-tray-dialog"
        role="dialog"
      >
        <div className="dialog-header">
          <div>
            <p className="eyebrow">Tray Prompt</p>
            <h2 id="close-to-tray-dialog-title">关闭窗口时隐藏到托盘？</h2>
          </div>
        </div>

        <div className="close-to-tray-body">
          <p>
            选择“隐藏到托盘”后，Porthole 会继续在后台运行，你可以通过系统托盘图标重新打开主窗口。
          </p>
          <p>如果选择“退出应用”，当前正在运行的端口转发会一并停止。</p>
        </div>

        <div className="dialog-actions">
          <button className="ghost-button" disabled={busy} onClick={onCancel} type="button">
            取消
          </button>
          <button
            className="ghost-button"
            disabled={busy}
            onClick={onHideToTray}
            type="button"
          >
            {busy ? "处理中..." : "隐藏到托盘"}
          </button>
          <button
            className="ghost-button danger-button"
            disabled={busy}
            onClick={onExitApplication}
            type="button"
          >
            退出应用
          </button>
        </div>
      </div>
    </div>
  );
}
