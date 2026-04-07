import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { DeleteRuleDialog } from "../components/delete-rule-dialog";
import type { Rule } from "../lib/types";

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

describe("DeleteRuleDialog", () => {
  it("renders rule summary and triggers callbacks", async () => {
    const user = userEvent.setup();
    const onCancel = vi.fn();
    const onConfirm = vi.fn();

    render(
      <DeleteRuleDialog
        onCancel={onCancel}
        onConfirm={onConfirm}
        open
        rule={sampleRule()}
      />,
    );

    expect(screen.getByText("确认删除这条规则？")).toBeInTheDocument();
    expect(screen.getByText("HTTP Forward")).toBeInTheDocument();
    expect(screen.getByText("rule-1")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "确认删除" }));
    await user.click(screen.getByRole("button", { name: "取消" }));

    expect(onConfirm).toHaveBeenCalledTimes(1);
    expect(onCancel).toHaveBeenCalledTimes(1);
  });
});
