import type { Rule } from "../lib/types";

interface DeleteRuleDialogProps {
  open: boolean;
  rule: Rule | null;
  busy?: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}

export function DeleteRuleDialog({
  open,
  rule,
  busy = false,
  onCancel,
  onConfirm,
}: DeleteRuleDialogProps) {
  if (!open || !rule) {
    return null;
  }

  return (
    <div className="dialog-backdrop" role="presentation">
      <div
        aria-labelledby="delete-rule-dialog-title"
        aria-modal="true"
        className="dialog-card delete-rule-dialog"
        role="dialog"
      >
        <div className="dialog-header">
          <div>
            <p className="eyebrow">Delete Rule</p>
            <h2 id="delete-rule-dialog-title">确认删除这条规则？</h2>
          </div>
        </div>

        <div className="delete-rule-body">
          <p>
            删除后，这条规则会从本地配置中移除；如果它当前正在运行，端口转发也会同步停止。
          </p>
          <div className="delete-rule-summary">
            <strong>{rule.name}</strong>
            <span>{rule.id}</span>
          </div>
        </div>

        <div className="dialog-actions">
          <button className="ghost-button" disabled={busy} onClick={onCancel} type="button">
            取消
          </button>
          <button
            className="ghost-button danger-button"
            disabled={busy}
            onClick={onConfirm}
            type="button"
          >
            {busy ? "删除中..." : "确认删除"}
          </button>
        </div>
      </div>
    </div>
  );
}
