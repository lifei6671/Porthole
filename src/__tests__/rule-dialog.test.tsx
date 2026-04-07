import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { RuleDialog } from "../components/rule-dialog";
import type { RuleInput } from "../lib/types";

describe("RuleDialog", () => {
  it("shows validation errors for invalid input", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn();

    render(
      <RuleDialog
        mode="create"
        onClose={vi.fn()}
        onSubmit={onSubmit}
        open
      />,
    );

    await user.clear(screen.getByRole("textbox", { name: "名称" }));
    await user.clear(screen.getByRole("textbox", { name: "监听地址" }));
    await user.clear(screen.getByRole("spinbutton", { name: "监听端口" }));
    await user.click(screen.getByRole("button", { name: "创建规则" }));

    expect(screen.getByText("名称不能为空")).toBeInTheDocument();
    expect(screen.getByText("监听地址不能为空")).toBeInTheDocument();
    expect(screen.getByText("监听端口必须在 1-65535 之间")).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it("submits valid input and updates previews", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn<Promise<void>, [RuleInput]>().mockResolvedValue();

    render(
      <RuleDialog
        mode="create"
        onClose={vi.fn()}
        onSubmit={onSubmit}
        open
      />,
    );

    await user.clear(screen.getByRole("textbox", { name: "名称" }));
    await user.type(screen.getByRole("textbox", { name: "名称" }), "IPv6 Rule");
    await user.clear(screen.getByLabelText(/监听地址/));
    await user.type(screen.getByLabelText(/监听地址/), "::");
    await user.clear(screen.getByLabelText(/监听端口/));
    await user.type(screen.getByLabelText(/监听端口/), "5353");
    await user.clear(screen.getByLabelText(/目标地址/));
    await user.type(screen.getByLabelText(/目标地址/), "::1");

    expect(screen.getByText("监听预览：[::]:5353")).toBeInTheDocument();
    expect(screen.getByText("目标预览：[::1]:80")).toBeInTheDocument();
    expect(
      screen.getByText("当前监听地址不是回环地址，启动规则时会自动申请放行 Windows 防火墙。"),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "创建规则" }));

    expect(onSubmit).toHaveBeenCalledTimes(1);
    expect(onSubmit.mock.calls[0][0].name).toBe("IPv6 Rule");
    expect(onSubmit.mock.calls[0][0].listen_host).toBe("::");
  });
});
