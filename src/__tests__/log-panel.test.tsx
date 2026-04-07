import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { LogPanel } from "../components/log-panel";
import { StatusBar } from "../components/status-bar";
import type { Rule, RuntimeState, UILogEntry } from "../lib/types";

function sampleLogs(): UILogEntry[] {
  return [
    {
      id: "app-1",
      source: "app",
      level: "info",
      message: "已刷新规则和运行状态",
      observedAt: "2026-04-07T12:00:00Z",
    },
    {
      id: "gost-1",
      source: "gost",
      level: "error",
      message: "bind failed",
      observedAt: "2026-04-07T12:01:00Z",
    },
  ];
}

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

describe("LogPanel", () => {
  it("renders app and gost logs and clears via callback", async () => {
    const user = userEvent.setup();
    const onClear = vi.fn();

    render(<LogPanel entries={sampleLogs()} onClear={onClear} />);

    expect(screen.getByText("已刷新规则和运行状态")).toBeInTheDocument();
    expect(screen.getByText("bind failed")).toBeInTheDocument();
    expect(screen.getByText("app")).toBeInTheDocument();
    expect(screen.getByText("gost")).toBeInTheDocument();
    expect(screen.getByText("INFO")).toBeInTheDocument();
    expect(screen.getByText("ERROR")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "清空日志" }));
    expect(onClear).toHaveBeenCalledTimes(1);
  });
});

describe("StatusBar", () => {
  it("shows firewall notice for non-loopback listeners", () => {
    render(
      <StatusBar
        lastError="bind failed"
        notice={null}
        rules={sampleRules()}
        runtime={sampleRuntime()}
      />,
    );

    expect(
      screen.getByText(/检测到 1 条非回环监听规则，启动时会自动申请放行 Windows 防火墙/),
    ).toBeInTheDocument();
    expect(screen.getByText("bind failed")).toBeInTheDocument();
  });
});
