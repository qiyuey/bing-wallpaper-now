import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  installGlobalErrorHandlers,
  isIgnoredTauriListenerRejection,
  markFrontendReady,
  reportFrontendError,
  stringifyUnknownError,
} from "./errorReporter";

vi.mock("@tauri-apps/api/core");

describe("errorReporter", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(invoke).mockResolvedValue(null);
  });

  it("stringifies common error values", () => {
    expect(stringifyUnknownError(new Error("boom"))).toBe("boom");
    expect(stringifyUnknownError("plain")).toBe("plain");
    expect(stringifyUnknownError({ code: 500 })).toBe('{"code":500}');
    expect(stringifyUnknownError(undefined)).toBe("undefined");

    const circular: { self?: unknown } = {};
    circular.self = circular;
    expect(stringifyUnknownError(circular)).toBe("[object Object]");
  });

  it("detects harmless Tauri listener cleanup rejections", () => {
    expect(
      isIgnoredTauriListenerRejection(
        new Error("failed to remove listeners handlerId=1"),
      ),
    ).toBe(true);

    expect(isIgnoredTauriListenerRejection(new Error("real crash"))).toBe(
      false,
    );
  });

  it("reports frontend errors through the backend command", () => {
    const error = new Error("render failed");

    reportFrontendError({
      source: "react-error-boundary",
      error,
      context: { phase: "render" },
    });

    expect(invoke).toHaveBeenCalledWith("report_frontend_error", {
      source: "react-error-boundary",
      message: "render failed",
      stack: error.stack ?? null,
      context: '{"phase":"render"}',
    });
  });

  it("falls back when report context cannot be serialized", () => {
    const circular: { self?: unknown } = {};
    circular.self = circular;

    reportFrontendError({
      source: "bootstrap",
      error: "boot failed",
      stack: "stack",
      context: circular,
    });

    expect(invoke).toHaveBeenCalledWith("report_frontend_error", {
      source: "bootstrap",
      message: "boot failed",
      stack: "stack",
      context: "[object Object]",
    });
  });

  it("marks frontend ready through the backend command", () => {
    markFrontendReady();

    expect(invoke).toHaveBeenCalledWith("mark_frontend_ready");
  });

  it("reports mark ready failures without creating unhandled rejections", async () => {
    vi.mocked(invoke)
      .mockRejectedValueOnce(new Error("ready failed"))
      .mockResolvedValueOnce(null);

    markFrontendReady();
    await Promise.resolve();

    expect(invoke).toHaveBeenNthCalledWith(1, "mark_frontend_ready");
    expect(invoke).toHaveBeenNthCalledWith(2, "report_frontend_error", {
      source: "mark_frontend_ready",
      message: "ready failed",
      stack: expect.any(String),
      context: null,
    });
  });

  it("installs global error handlers", () => {
    const listeners: Record<string, (event: never) => void> = {};
    const target = {
      addEventListener: vi.fn(
        (type: string, listener: (event: never) => void) => {
          listeners[type] = listener;
        },
      ),
    };

    installGlobalErrorHandlers(
      target as unknown as Parameters<typeof installGlobalErrorHandlers>[0],
    );

    expect(target.addEventListener).toHaveBeenCalledTimes(2);

    const ignoredPreventDefault = vi.fn();
    listeners.unhandledrejection({
      reason: new Error("failed to remove listeners handlerId=1"),
      preventDefault: ignoredPreventDefault,
    } as never);

    expect(ignoredPreventDefault).toHaveBeenCalledOnce();
    expect(invoke).not.toHaveBeenCalled();

    listeners.unhandledrejection({
      reason: new Error("async crash"),
      preventDefault: vi.fn(),
    } as never);

    expect(invoke).toHaveBeenCalledWith("report_frontend_error", {
      source: "unhandledrejection",
      message: "async crash",
      stack: expect.any(String),
      context: null,
    });

    vi.clearAllMocks();

    const error = new Error("sync crash");
    listeners.error({
      error,
      message: "ignored fallback",
    } as never);

    expect(invoke).toHaveBeenCalledWith("report_frontend_error", {
      source: "window.error",
      message: "sync crash",
      stack: error.stack ?? null,
      context: null,
    });
  });
});
