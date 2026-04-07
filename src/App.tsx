import { useState } from "react";

import {
  clearLogs,
  createRule,
  deleteRule,
  startRule,
  stopRule,
  updateRule,
} from "./lib/api";
import { PageHeader } from "./components/page-header";
import { RuleDialog } from "./components/rule-dialog";
import { Sidebar, type AppPage } from "./components/sidebar";
import { useRules } from "./hooks/use-rules";
import { useRuntimeEvents } from "./hooks/use-runtime-events";
import type { ProcessLogEntry, Rule, RuleInput, UILogEntry } from "./lib/types";
import { HomePage } from "./pages/home-page";
import { LogsPage } from "./pages/logs-page";
import { RulesPage } from "./pages/rules-page";
import { SettingsPage } from "./pages/settings-page";

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
  const [currentPage, setCurrentPage] = useState<AppPage>("home");
  const [actionPending, setActionPending] = useState(false);

  const activeRuleCount = runtime?.active_rule_ids.length ?? 0;
  const busy = rulesLoading || runtimeLoading || actionPending;
  const mergedLogs = appLogs.concat(logs.map(toUILogEntry)).slice(-maxUILogEntries);

  function addAppLog(level: UILogEntry["level"], message: string) {
    setAppLogs((current) => appendUILog(current, "app", level, message));
  }

  async function handleClearLogs() {
    setActionPending(true);
    try {
      await clearLogs();
      clearLocalLogs();
      setAppLogs([]);
      addAppLog("info", "日志已清空");
    } finally {
      setActionPending(false);
    }
  }

  async function handleRefresh() {
    setActionPending(true);
    try {
      await Promise.all([refreshRules(), refreshRuntime()]);
      addAppLog("info", "已刷新规则和运行状态");
    } finally {
      setActionPending(false);
    }
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
    setActionPending(true);

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
    } finally {
      setActionPending(false);
    }
  }

  async function handleStartRule(ruleID: string) {
    setActionPending(true);
    try {
      await startRule(ruleID);
      await refreshRuntime();
      addAppLog("info", `已启动规则：${ruleID}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setActionError(message);
      addAppLog("error", `启动规则失败：${message}`);
    } finally {
      setActionPending(false);
    }
  }

  async function handleStopRule(ruleID: string) {
    setActionPending(true);
    try {
      await stopRule(ruleID);
      await refreshRuntime();
      addAppLog("info", `已停止规则：${ruleID}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setActionError(message);
      addAppLog("error", `停止规则失败：${message}`);
    } finally {
      setActionPending(false);
    }
  }

  async function handleDeleteRule(rule: Rule) {
    const confirmed = window.confirm(`确认删除规则“${rule.name}”？`);
    if (!confirmed) {
      return;
    }

    setActionPending(true);
    try {
      await deleteRule(rule.id);
      await Promise.all([refreshRules(), refreshRuntime()]);
      addAppLog("info", `已删除规则：${rule.name}`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setActionError(message);
      addAppLog("error", `删除规则失败：${message}`);
    } finally {
      setActionPending(false);
    }
  }

  async function handleStartAll() {
    setActionPending(true);
    try {
      await startAll();
      addAppLog("info", "已执行启动全部");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setActionError(message);
      addAppLog("error", `启动全部失败：${message}`);
    } finally {
      setActionPending(false);
    }
  }

  async function handleStopAll() {
    setActionPending(true);
    try {
      await stopAll();
      addAppLog("info", "已执行停止全部");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setActionError(message);
      addAppLog("error", `停止全部失败：${message}`);
    } finally {
      setActionPending(false);
    }
  }

  const pageMeta: Record<AppPage, { title: string; description: string }> = {
    home: {
      title: "总览",
      description: "查看当前运行状态、快捷操作和最近日志。",
    },
    rules: {
      title: "规则",
      description: "管理 TCP / UDP 端口转发规则，并对单条规则执行启停操作。",
    },
    logs: {
      title: "日志",
      description: "集中查看 app 与 gost 的运行日志，用于快速定位故障。",
    },
    settings: {
      title: "设置",
      description: "查看应用说明、运行策略和 Windows 防火墙相关提示。",
    },
  };

  const pageActions =
    currentPage === "home" ? (
      <>
        <button className="ghost-button" disabled={busy} onClick={handleStartAll} type="button">
          启动全部
        </button>
        <button className="ghost-button" disabled={busy} onClick={handleStopAll} type="button">
          停止全部
        </button>
      </>
    ) : currentPage === "rules" ? (
      <>
        <button className="ghost-button" disabled={busy} onClick={handleRefresh} type="button">
          刷新
        </button>
        <button className="primary-button" disabled={busy} onClick={handleAddRule} type="button">
          新增规则
        </button>
      </>
    ) : currentPage === "logs" ? (
      <button className="ghost-button" onClick={handleClearLogs} type="button">
        清空日志
      </button>
    ) : null;

  const pageContent =
    currentPage === "home" ? (
      <HomePage
        logs={mergedLogs}
        onDismissProcessExitReason={clearProcessExitReason}
        onOpenLogs={() => setCurrentPage("logs")}
        onOpenRules={() => setCurrentPage("rules")}
        processExitReason={processExitReason}
        rules={rules}
        runtime={runtime}
      />
    ) : currentPage === "rules" ? (
      <RulesPage
        error={rulesError}
        loading={rulesLoading}
        onDeleteRule={handleDeleteRule}
        onEditRule={handleEditRule}
        onStartRule={handleStartRule}
        onStopRule={handleStopRule}
        rules={rules}
        runtime={runtime}
      />
    ) : currentPage === "logs" ? (
      <LogsPage logs={mergedLogs} onClearLogs={handleClearLogs} />
    ) : (
      <SettingsPage rules={rules} runtime={runtime} />
    );

  return (
    <main className="app-shell">
      <Sidebar
        activeRuleCount={activeRuleCount}
        currentPage={currentPage}
        onNavigate={setCurrentPage}
        processStatus={runtime?.process_status ?? "unknown"}
        totalRuleCount={rules.length}
      />

      <section className="app-main">
        <PageHeader
          actions={pageActions}
          description={pageMeta[currentPage].description}
          title={pageMeta[currentPage].title}
        />

        {(runtimeError || actionError) && (
          <p className="error-banner">{actionError ?? runtimeError}</p>
        )}

        {pageContent}
      </section>

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
