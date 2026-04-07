import { render, screen } from "@testing-library/react";

import { SettingsPage } from "../pages/settings-page";
import type { Rule, RuntimeState } from "../lib/types";

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

describe("SettingsPage", () => {
  it("renders summary cards and windows strategy notes", () => {
    render(<SettingsPage rules={sampleRules()} runtime={sampleRuntime()} />);

    expect(screen.getAllByText("当前版本").length).toBeGreaterThan(0);
    expect(screen.getByText("Windows 运行策略")).toBeInTheDocument();
    expect(screen.getByText(/关闭主窗口时会先询问是否隐藏到托盘/)).toBeInTheDocument();
    expect(screen.getByText("本期范围说明")).toBeInTheDocument();
  });
});
