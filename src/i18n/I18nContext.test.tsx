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

  it("should throw error when useI18n is used outside I18nProvider", () => {
    // Suppress console.error for this test since React will log the error
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    function BadConsumer() {
      useI18n();
      return null;
    }

    expect(() => {
      render(<BadConsumer />);
    }).toThrow("useI18n must be used within I18nProvider");

    consoleErrorSpy.mockRestore();
  });

  it("should fallback to provided lang when backend returns invalid language", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        const callCount = vi.mocked(invoke).mock.calls.length;
        if (callCount <= 1) {
          return Promise.resolve(buildSettings("zh-CN", "zh-CN"));
        }
        // Return settings with an invalid language value
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          launch_at_startup: false,
          theme: "system",
          language: "fr-FR", // invalid
          resolved_language: "zh-CN",
          mkt: "zh-CN",
        });
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
    });

    // After setLanguage with invalid backend response, should fallback
    fireEvent.click(screen.getByText("set-en"));

    await waitFor(() => {
      // Language should be set to "en-US" (the provided lang) since backend value is invalid
      expect(screen.getByTestId("language")).toHaveTextContent("en-US");
    });
  });

  it("should fallback to resolved preference when backend returns invalid resolved_language", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        const callCount = vi.mocked(invoke).mock.calls.length;
        if (callCount <= 1) {
          return Promise.resolve(buildSettings("zh-CN", "zh-CN"));
        }
        // Return settings with valid language but invalid resolved_language
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          launch_at_startup: false,
          theme: "system",
          language: "en-US",
          resolved_language: "fr-FR", // invalid
          mkt: "zh-CN",
        });
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
    });

    fireEvent.click(screen.getByText("set-en"));

    await waitFor(() => {
      expect(screen.getByTestId("language")).toHaveTextContent("en-US");
      // actual-language should fallback to resolved preference for "en-US"
      expect(screen.getByTestId("actual-language")).toHaveTextContent("en-US");
    });
  });
});
