import { describe, it, expect, vi, beforeEach } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { I18nProvider, useI18n } from "./I18nContext";
import { AppSettings } from "../types";

vi.mock("@tauri-apps/api/core");

function buildSettings(
  language: "auto" | "zh-CN" | "en-US",
  resolved_language: "zh-CN" | "en-US",
): AppSettings {
  return {
    auto_update: true,
    save_directory: null,
    launch_at_startup: false,
    theme: "system",
    language,
    resolved_language,
    mkt: "zh-CN",
  };
}

function TestConsumer() {
  const { language, actualLanguage, setLanguage } = useI18n();

  return (
    <div>
      <div data-testid="language">{language}</div>
      <div data-testid="actual-language">{actualLanguage}</div>
      <button onClick={() => void setLanguage("en-US")}>set-en</button>
      <button onClick={() => void setLanguage("auto")}>set-auto</button>
    </div>
  );
}

describe("I18nContext", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should sync language state from backend after setLanguage", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        // 首次加载返回 zh-CN，点击后返回 en-US
        const callCount = vi.mocked(invoke).mock.calls.length;
        if (callCount <= 1) {
          return Promise.resolve(buildSettings("zh-CN", "zh-CN"));
        }
        return Promise.resolve(buildSettings("en-US", "en-US"));
      }
      return Promise.resolve(undefined);
    });

    render(
      <I18nProvider>
        <TestConsumer />
      </I18nProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId("language")).toHaveTextContent("zh-CN");
      expect(screen.getByTestId("actual-language")).toHaveTextContent("zh-CN");
    });

    fireEvent.click(screen.getByText("set-en"));

    await waitFor(() => {
      expect(screen.getByTestId("language")).toHaveTextContent("en-US");
      expect(screen.getByTestId("actual-language")).toHaveTextContent("en-US");
    });
  });

  it("should keep language and actualLanguage consistent when backend sync fails", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        const callCount = vi.mocked(invoke).mock.calls.length;
        if (callCount <= 1) {
          return Promise.resolve(buildSettings("zh-CN", "zh-CN"));
        }
        return Promise.reject(new Error("sync failed"));
      }
      return Promise.resolve(undefined);
    });

    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    render(
      <I18nProvider>
        <TestConsumer />
      </I18nProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId("language")).toHaveTextContent("zh-CN");
      expect(screen.getByTestId("actual-language")).toHaveTextContent("zh-CN");
    });

    fireEvent.click(screen.getByText("set-en"));

    await waitFor(() => {
      expect(screen.getByTestId("language")).toHaveTextContent("en-US");
      expect(screen.getByTestId("actual-language")).toHaveTextContent("en-US");
    });

    consoleErrorSpy.mockRestore();
  });
});
