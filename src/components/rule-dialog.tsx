import { useEffect, useState, type FormEvent } from "react";

import { validateRuleInput, needsFirewallNotice } from "../lib/validators";
import type { Protocol, Rule, RuleInput } from "../lib/types";

interface RuleDialogProps {
  open: boolean;
  mode: "create" | "edit";
  initialRule?: Rule | null;
  onClose: () => void;
  onSubmit: (input: RuleInput) => Promise<void> | void;
}

function buildInitialInput(rule?: Rule | null): RuleInput {
  if (!rule) {
    return {
      name: "",
      enabled: true,
      protocol: "tcp",
      listen_host: "0.0.0.0",
      listen_port: 8080,
      target_host: "127.0.0.1",
      target_port: 80,
      remark: "",
    };
  }

  return {
    name: rule.name,
    enabled: rule.enabled,
    protocol: rule.protocol,
    listen_host: rule.listen_host,
    listen_port: rule.listen_port,
    target_host: rule.target_host,
    target_port: rule.target_port,
    remark: rule.remark,
  };
}

function formatEndpoint(host: string, port: number) {
  const needsBrackets = host.includes(":") && !host.startsWith("[");
  return needsBrackets ? `[${host}]:${port}` : `${host}:${port}`;
}

export function RuleDialog({
  open,
  mode,
  initialRule,
  onClose,
  onSubmit,
}: RuleDialogProps) {
  const [form, setForm] = useState<RuleInput>(buildInitialInput(initialRule));
  const [fieldErrors, setFieldErrors] = useState<
    Partial<Record<keyof RuleInput, string>>
  >({});
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (open) {
      setForm(buildInitialInput(initialRule));
      setFieldErrors({});
      setSubmitting(false);
    }
  }, [initialRule, open]);

  if (!open) {
    return null;
  }

  function updateField<K extends keyof RuleInput>(key: K, value: RuleInput[K]) {
    const next = { ...form, [key]: value };
    setForm(next);
    setFieldErrors(validateRuleInput(next).issues);
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const validation = validateRuleInput(form);
    setFieldErrors(validation.issues);

    if (!validation.isValid) {
      return;
    }

    setSubmitting(true);
    try {
      await onSubmit({
        ...form,
        name: form.name.trim(),
        listen_host: form.listen_host.trim(),
        target_host: form.target_host.trim(),
        remark: form.remark.trim(),
      });
    } catch {
      // 错误由上层统一展示，这里只负责保持弹窗可继续编辑。
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="dialog-backdrop" role="presentation">
      <div
        aria-labelledby="rule-dialog-title"
        aria-modal="true"
        className="dialog-card"
        role="dialog"
      >
        <div className="dialog-header">
          <div>
            <p className="eyebrow">Rule Editor</p>
            <h2 id="rule-dialog-title">
              {mode === "create" ? "新增规则" : "编辑规则"}
            </h2>
          </div>
          <button className="ghost-button" onClick={onClose} type="button">
            关闭
          </button>
        </div>

        <form className="rule-form" onSubmit={handleSubmit}>
          <label>
            <span>名称</span>
            <input
              name="name"
              onChange={(event) => updateField("name", event.target.value)}
              value={form.name}
            />
            {fieldErrors.name ? <small className="field-error">{fieldErrors.name}</small> : null}
          </label>

          <label>
            <span>协议</span>
            <select
              name="protocol"
              onChange={(event) =>
                updateField("protocol", event.target.value as Protocol)
              }
              value={form.protocol}
            >
              <option value="tcp">TCP</option>
              <option value="udp">UDP</option>
            </select>
          </label>

          <div className="form-grid">
            <label>
              <span>监听地址</span>
              <input
                name="listen_host"
                onChange={(event) => updateField("listen_host", event.target.value)}
                value={form.listen_host}
              />
              {fieldErrors.listen_host ? (
                <small className="field-error">{fieldErrors.listen_host}</small>
              ) : null}
            </label>

            <label>
              <span>监听端口</span>
              <input
                max={65535}
                min={1}
                name="listen_port"
                onChange={(event) =>
                  updateField("listen_port", Number(event.target.value || 0))
                }
                type="number"
                value={form.listen_port}
              />
              {fieldErrors.listen_port ? (
                <small className="field-error">{fieldErrors.listen_port}</small>
              ) : null}
            </label>
          </div>

          <div className="form-grid">
            <label>
              <span>目标地址</span>
              <input
                name="target_host"
                onChange={(event) => updateField("target_host", event.target.value)}
                value={form.target_host}
              />
              {fieldErrors.target_host ? (
                <small className="field-error">{fieldErrors.target_host}</small>
              ) : null}
            </label>

            <label>
              <span>目标端口</span>
              <input
                max={65535}
                min={1}
                name="target_port"
                onChange={(event) =>
                  updateField("target_port", Number(event.target.value || 0))
                }
                type="number"
                value={form.target_port}
              />
              {fieldErrors.target_port ? (
                <small className="field-error">{fieldErrors.target_port}</small>
              ) : null}
            </label>
          </div>

          <label>
            <span>备注</span>
            <textarea
              name="remark"
              onChange={(event) => updateField("remark", event.target.value)}
              rows={3}
              value={form.remark}
            />
          </label>

          <label className="checkbox-field">
            <input
              checked={form.enabled}
              onChange={(event) => updateField("enabled", event.target.checked)}
              type="checkbox"
            />
            <span>默认启用</span>
          </label>

          <div className="preview-card">
            <strong>地址预览</strong>
            <p>监听预览：{formatEndpoint(form.listen_host, form.listen_port)}</p>
            <p>目标预览：{formatEndpoint(form.target_host, form.target_port)}</p>
          </div>

          {needsFirewallNotice(form.listen_host) ? (
            <p className="firewall-notice">
              当前监听地址不是回环地址，启动规则时会自动申请放行 Windows 防火墙。
            </p>
          ) : null}

          <div className="dialog-actions">
            <button className="ghost-button" onClick={onClose} type="button">
              取消
            </button>
            <button className="primary-button" disabled={submitting} type="submit">
              {submitting ? "保存中..." : mode === "create" ? "创建规则" : "保存修改"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
