import { render, screen } from "@testing-library/react";

import { HomePage } from "../pages/home-page";
import type { Rule, RuntimeState, UILogEntry } from "../lib/types";

function sampleRuntime(): RuntimeState {
  return {
    process_status: "running",
    active_rule_ids: ["rule-1"],
    last_error: {
      summary: "bind failed",
      observed_at: "2026-04-07T12:01:00Z",
    },
    rule_statuses: [
      {
        rule_id: "rule-1",
        status: "running",
        last_error_summary: null,
      },
    ],
  };
}

function sampleRules(): Rule[] {
  return [
    {
      id: "rule-1",
      name: "Expose 8080",
      enabled: true,
      protocol: "tcp",
      listen_host: "0.0.0.0",
      listen_port: 8080,
      target_host: "127.0.0.1",
      target_port: 80,
      remark: "",
      created_at: "2026-04-07T12:00:00Z",
      updated_at: "2026-04-07T12:00:00Z",
    },
  ];
}

function sampleLogs(): UILogEntry[] {
  return [
    {
      id: "log-1",
      source: "app",
      level: "info",
      message: "已刷新规则和运行状态",
      observedAt: "2026-04-07T12:00:00Z",
    },
  ];
}

describe("HomePage", () => {
  it("renders overview cards and recent logs", () => {
    render(
      <HomePage
        logs={sampleLogs()}
        onDismissProcessExitReason={vi.fn()}
        onOpenLogs={vi.fn()}
        onOpenRules={vi.fn()}
        processExitReason={null}
        rules={sampleRules()}
        runtime={sampleRuntime()}
      />,
    );

    expect(screen.getByText("gost 运行状态")).toBeInTheDocument();
    expect(screen.getAllByText("运行中").length).toBeGreaterThan(0);
    expect(screen.getAllByText("对外监听规则").length).toBeGreaterThan(0);
    expect(screen.getByText("已刷新规则和运行状态")).toBeInTheDocument();
  });
});
