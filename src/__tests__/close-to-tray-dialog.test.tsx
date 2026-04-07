import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { CloseToTrayDialog } from "../components/close-to-tray-dialog";

describe("CloseToTrayDialog", () => {
  it("renders actions and triggers callbacks", async () => {
    const user = userEvent.setup();
    const onCancel = vi.fn();
    const onHideToTray = vi.fn();
    const onExitApplication = vi.fn();

    render(
      <CloseToTrayDialog
        onCancel={onCancel}
        onExitApplication={onExitApplication}
        onHideToTray={onHideToTray}
        open
      />,
    );

    expect(screen.getByText("关闭窗口时隐藏到托盘？")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "隐藏到托盘" }));
    await user.click(screen.getByRole("button", { name: "退出应用" }));
    await user.click(screen.getByRole("button", { name: "取消" }));

    expect(onHideToTray).toHaveBeenCalledTimes(1);
    expect(onExitApplication).toHaveBeenCalledTimes(1);
    expect(onCancel).toHaveBeenCalledTimes(1);
  });
});
