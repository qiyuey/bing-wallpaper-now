import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { AppErrorBoundary } from "./AppErrorBoundary";

function ThrowingComponent(): never {
  throw new Error("component exploded");
}

describe("AppErrorBoundary", () => {
  it("renders a visible fallback and reports render errors", () => {
    const onError = vi.fn();
    const consoleError = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    render(
      <AppErrorBoundary onError={onError}>
        <ThrowingComponent />
      </AppErrorBoundary>,
    );

    expect(screen.getByRole("alert")).toBeInTheDocument();
    expect(screen.getByText(/Startup failed/)).toBeInTheDocument();
    expect(screen.getByText("component exploded")).toBeInTheDocument();
    expect(onError).toHaveBeenCalledOnce();

    consoleError.mockRestore();
  });
});
