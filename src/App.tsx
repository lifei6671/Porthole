import { useState } from "react";

import {
  clearLogs,
  createRule,
  deleteRule,
  startRule,
  stopRule,
  updateRule,
} from "./lib/api";
import { AppToolbar } from "./components/app-toolbar";
import { LogPanel } from "./components/log-panel";
import { RuleDialog } from "./components/rule-dialog";
import { RuleList } from "./components/rule-list";
import { StatusBar } from "./components/status-bar";
import { useRules } from "./hooks/use-rules";
import { useRuntimeEvents } from "./hooks/use-runtime-events";
import type { ProcessLogEntry, Rule, RuleInput, UILogEntry } from "./lib/types";

const maxUILogEntries = 500;

function toUILogEntry(entry: ProcessLogEntry): UILogEntry {
  if (entry.source === "AppInfo" || entry.source === "AppError") {
    return {
      id: `${entry.observed_at}-${entry.source}-${entry.message}`,
      source: "app",
      level: entry.source === "AppError" ? "error" : "info",
      message: entry.message,
      observedAt: entry.observed_at,
    };
  }

  return {
    id: `${entry.observed_at}-${entry.source}-${entry.message}`,
    source: "gost",
    level: entry.source === "Stderr" ? "error" : "info",
    message: entry.message,
    observedAt: entry.observed_at,
  };
}

function appendUILog(
  current: UILogEntry[],
  source: UILogEntry["source"],
  level: UILogEntry["level"],
  message: string,
) {
  const next = current.concat({
    id: `${Date.now()}-${Math.random().toString(16).slice(2)}`,
    source,
    level,
    message,
    observedAt: new Date().toISOString(),
  });
  return next.slice(-maxUILogEntries);
}

export function App() {
  const {
    rules,
    loading: rulesLoading,
    error: rulesError,
    refresh: refreshRules,
  } = useRules();
  const {
    runtime,
    logs,
    processExitReason,
    loading: runtimeLoading,
    error: runtimeError,
    refresh: refreshRuntime,
    startAll,
    stopAll,
    clearLocalLogs,
    clearProcessExitReason,
  } = useRuntimeEvents();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [dialogMode, setDialogMode] = useState<"create" | "edit">("create");
  const [editingRule, setEditingRule] = useState<Rule | null>(null);
  const [appLogs, setAppLogs] = useState<UILogEntry[]>([]);
  const [actionError, setActionError] = useState<string | null>(null);

  const activeRuleCount = runtime?.active_rule_ids.length ?? 0;
  const busy = rulesLoading || runtimeLoading;
  const mergedLogs = appLogs.concat(logs.map(toUILogEntry)).slice(-maxUILogEntries);

  function addAppLog(level: UILogEntry["level"], message: string) {
    setAppLogs((current) => appendUILog(current, "app", level, message));
  }

  async function handleClearLogs() {
    await clearLogs();
    clearLocalLogs();
    setAppLogs([]);
    addAppLog("info", "日志已清空");
  }

  async function handleRefresh() {
    await Promise.all([refreshRules(), refreshRuntime()]);
    addAppLog("info", "已刷新规则和运行状态");
  }

  function handleAddRule() {
    setDialogMode("create");
    setEditingRule(null);
    setDialogOpen(true);
  }

  function handleEditRule(rule: Rule) {
    setDialogMode("edit");
    setEditingRule(rule);
    setDialogOpen(true);
  }

  function closeDialog() {
    setDialogOpen(false);
    setEditingRule(null);
  }

  async function handleSubmitRule(input: RuleInput) {
    setActionError(null);

    try {
      if (dialogMode === "create") {
        await createRule(input);
        addAppLog("info", `已创建规则：${input.name}`);
      } else if (editingRule) {
        await updateRule(editingRule.id, input);
        addAppLog("info", `已更新规则：${input.name}`);
      }

      await Promise.all([refreshRules(), refreshRuntime()]);
      closeDialog();
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setActionError(message);
      addAppLog("error", `保存规则失败：${message}`);
    }
  }

  async function handleStartRule(ruleID: string) {
    try {
      await startRule(ruleID);
      await refreshRuntime();
      addAppLog("info", `已启动规则：${ruleID}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setActionError(message);
      addAppLog("error", `启动规则失败：${message}`);
    }
  }

  async function handleStopRule(ruleID: string) {
    try {
      await stopRule(ruleID);
      await refreshRuntime();
      addAppLog("info", `已停止规则：${ruleID}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setActionError(message);
      addAppLog("error", `停止规则失败：${message}`);
    }
  }

  async function handleDeleteRule(rule: Rule) {
    const confirmed = window.confirm(`确认删除规则“${rule.name}”？`);
    if (!confirmed) {
      return;
    }

    try {
      await deleteRule(rule.id);
      await Promise.all([refreshRules(), refreshRuntime()]);
      addAppLog("info", `已删除规则：${rule.name}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setActionError(message);
      addAppLog("error", `删除规则失败：${message}`);
    }
  }

  async function handleStartAll() {
    try {
      await startAll();
      addAppLog("info", "已执行启动全部");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setActionError(message);
      addAppLog("error", `启动全部失败：${message}`);
    }
  }

  async function handleStopAll() {
    try {
      await stopAll();
      addAppLog("info", "已执行停止全部");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setActionError(message);
      addAppLog("error", `停止全部失败：${message}`);
    }
  }

  return (
    <main className="app-shell">
      <header className="app-header">
        <div>
          <p className="eyebrow">Windows Port Forwarding MVP</p>
          <h1 className="app-title">Porthole</h1>
        </div>
        <div className="header-metrics">
          <article className="metric-card">
            <span className="metric-label">进程状态</span>
            <strong className="metric-value">
              {runtime?.process_status ?? "loading"}
            </strong>
          </article>
          <article className="metric-card">
            <span className="metric-label">当前运行规则</span>
            <strong className="metric-value">{activeRuleCount}</strong>
          </article>
          <article className="metric-card">
            <span className="metric-label">已保存规则</span>
            <strong className="metric-value">{rules.length}</strong>
          </article>
        </div>
      </header>

      <AppToolbar
        busy={busy}
        onAddRule={handleAddRule}
        onRefresh={handleRefresh}
        onStartAll={handleStartAll}
        onStopAll={handleStopAll}
      />

      <section className="workspace" aria-label="Application workspace">
        <RuleList
          error={rulesError}
          loading={rulesLoading}
          onDeleteRule={handleDeleteRule}
          onEditRule={handleEditRule}
          onStartRule={handleStartRule}
          onStopRule={handleStopRule}
          rules={rules}
          runtime={runtime}
        />

        <LogPanel entries={mergedLogs} onClear={handleClearLogs} />
      </section>

      {(runtimeError || actionError) && (
        <p className="error-banner">{actionError ?? runtimeError}</p>
      )}

      {processExitReason ? (
        <button
          className="exit-notice"
          onClick={clearProcessExitReason}
          type="button"
        >
          进程退出：{processExitReason}，点击关闭提示
        </button>
      ) : null}

      <StatusBar
        lastError={runtime?.last_error?.summary ?? actionError ?? runtimeError}
        notice={processExitReason}
        rules={rules}
        runtime={runtime}
      />

      <RuleDialog
        initialRule={editingRule}
        mode={dialogMode}
        onClose={closeDialog}
        onSubmit={handleSubmitRule}
        open={dialogOpen}
      />
    </main>
  );
}
