import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { Sidebar } from "../components/sidebar";

describe("Sidebar", () => {
  it("renders brand, status summary and navigates between pages", async () => {
    const user = userEvent.setup();
    const onNavigate = vi.fn();

    render(
      <Sidebar
        activeRuleCount={2}
        currentPage="home"
        onNavigate={onNavigate}
        processStatus="running"
        totalRuleCount={5}
      />,
    );

    expect(screen.getByText("Porthole")).toBeInTheDocument();
    expect(screen.getByText("运行中")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /规则/i }));
    await user.click(screen.getByRole("button", { name: /日志/i }));

    expect(onNavigate).toHaveBeenCalledWith("rules");
    expect(onNavigate).toHaveBeenCalledWith("logs");
  });
});
