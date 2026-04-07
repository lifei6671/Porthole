import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { AppToolbar } from "../components/app-toolbar";
import { RuleList } from "../components/rule-list";
import type { Rule, RuntimeState } from "../lib/types";

function sampleRule(): Rule {
  return {
    id: "rule-1",
    name: "HTTP Forward",
    enabled: true,
    protocol: "tcp",
    listen_host: "0.0.0.0",
    listen_port: 8080,
    target_host: "127.0.0.1",
    target_port: 80,
    remark: "test",
    created_at: "2026-04-07T12:00:00Z",
    updated_at: "2026-04-07T12:00:01Z",
  };
}

function sampleRuntime(): RuntimeState {
  return {
    process_status: "running",
    active_rule_ids: ["rule-1"],
    last_error: null,
    rule_statuses: [
      {
        rule_id: "rule-1",
        status: "running",
        last_error_summary: null,
      },
    ],
  };
}

describe("RuleList", () => {
  it("renders empty state when there are no rules", () => {
    render(
      <RuleList
        error={null}
        loading={false}
        onDeleteRule={vi.fn()}
        onEditRule={vi.fn()}
        onStartRule={vi.fn()}
        onStopRule={vi.fn()}
        rules={[]}
        runtime={null}
      />,
    );

    expect(screen.getByText("还没有可用规则")).toBeInTheDocument();
    expect(
      screen.getByText("点击“新增规则”开始配置第一条端口转发。"),
    ).toBeInTheDocument();
  });

  it("renders rule fields and runtime status", () => {
    render(
      <RuleList
        error={null}
        loading={false}
        onDeleteRule={vi.fn()}
        onEditRule={vi.fn()}
        onStartRule={vi.fn()}
        onStopRule={vi.fn()}
        rules={[sampleRule()]}
        runtime={sampleRuntime()}
      />,
    );

    expect(screen.getByText("HTTP Forward")).toBeInTheDocument();
    expect(screen.getByText("0.0.0.0:8080")).toBeInTheDocument();
    expect(screen.getByText("127.0.0.1:80")).toBeInTheDocument();
    expect(screen.getByText("running")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "停止" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "编辑" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "删除" })).toBeInTheDocument();
  });
});

describe("AppToolbar", () => {
  it("fires callbacks when toolbar buttons are clicked", async () => {
    const user = userEvent.setup();
    const onAddRule = vi.fn();
    const onStartAll = vi.fn();
    const onStopAll = vi.fn();
    const onRefresh = vi.fn();

    render(
      <AppToolbar
        busy={false}
        onAddRule={onAddRule}
        onRefresh={onRefresh}
        onStartAll={onStartAll}
        onStopAll={onStopAll}
      />,
    );

    await user.click(screen.getByRole("button", { name: "新增规则" }));
    await user.click(screen.getByRole("button", { name: "启动全部" }));
    await user.click(screen.getByRole("button", { name: "停止全部" }));
    await user.click(screen.getByRole("button", { name: "刷新状态" }));

    expect(onAddRule).toHaveBeenCalledTimes(1);
    expect(onStartAll).toHaveBeenCalledTimes(1);
    expect(onStopAll).toHaveBeenCalledTimes(1);
    expect(onRefresh).toHaveBeenCalledTimes(1);
  });
});
