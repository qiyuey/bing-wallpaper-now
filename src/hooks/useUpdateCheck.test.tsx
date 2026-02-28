import { ReactNode } from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useUpdateCheck } from "./useUpdateCheck";
import { EVENTS } from "../config/ui";
import { showSystemNotification } from "../utils/notification";
import { I18nProvider } from "../i18n/I18nContext";

vi.mock("@tauri-apps/api/core");
vi.mock("@tauri-apps/api/event");
vi.mock("../utils/notification");

function wrapper({ children }: { children: ReactNode }) {
  return <I18nProvider>{children}</I18nProvider>;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyEventHandler = (...args: any[]) => void;

describe("useUpdateCheck", () => {
  let eventCallbacks: Map<string, AnyEventHandler>;

  beforeEach(() => {
    vi.clearAllMocks();
    eventCallbacks = new Map();

    vi.mocked(listen).mockImplementation(async (event, cb) => {
      eventCallbacks.set(event as string, cb as AnyEventHandler);
      return () => {};
    });

    vi.mocked(invoke).mockResolvedValue(undefined);
    vi.mocked(showSystemNotification).mockResolvedValue(undefined);
  });

  it("should initialize with null updateInfo", () => {
    const { result } = renderHook(() => useUpdateCheck(), { wrapper });
    expect(result.current.updateInfo).toBeNull();
  });

  it("should register listeners for check-updates-result and check-updates-no-update", async () => {
    renderHook(() => useUpdateCheck(), { wrapper });

    await waitFor(() => {
      expect(listen).toHaveBeenCalledWith(
        EVENTS.CHECK_UPDATES_RESULT,
        expect.any(Function),
      );
      expect(listen).toHaveBeenCalledWith(
        EVENTS.CHECK_UPDATES_NO_UPDATE,
        expect.any(Function),
      );
    });
  });

  it("should set updateInfo when check-updates-result event has valid update", async () => {
    const { result } = renderHook(() => useUpdateCheck(), { wrapper });

    await waitFor(() => {
      expect(eventCallbacks.has(EVENTS.CHECK_UPDATES_RESULT)).toBe(true);
    });

    act(() => {
      eventCallbacks.get(EVENTS.CHECK_UPDATES_RESULT)!({
        payload: {
          has_update: true,
          latest_version: "2.0.0",
          release_url: "https://github.com/releases/2.0.0",
          platform_available: true,
          current_version: "1.0.0",
        },
      });
    });

    expect(result.current.updateInfo).toEqual({
      version: "2.0.0",
      releaseUrl: "https://github.com/releases/2.0.0",
    });
  });

  it("should not set updateInfo when has_update is false", async () => {
    const { result } = renderHook(() => useUpdateCheck(), { wrapper });

    await waitFor(() => {
      expect(eventCallbacks.has(EVENTS.CHECK_UPDATES_RESULT)).toBe(true);
    });

    act(() => {
      eventCallbacks.get(EVENTS.CHECK_UPDATES_RESULT)!({
        payload: {
          has_update: false,
          latest_version: "1.0.0",
          release_url: null,
          platform_available: true,
          current_version: "1.0.0",
        },
      });
    });

    expect(result.current.updateInfo).toBeNull();
  });

  it("should not set updateInfo when platform_available is false", async () => {
    const { result } = renderHook(() => useUpdateCheck(), { wrapper });

    await waitFor(() => {
      expect(eventCallbacks.has(EVENTS.CHECK_UPDATES_RESULT)).toBe(true);
    });

    act(() => {
      eventCallbacks.get(EVENTS.CHECK_UPDATES_RESULT)!({
        payload: {
          has_update: true,
          latest_version: "2.0.0",
          release_url: "https://example.com",
          platform_available: false,
          current_version: "1.0.0",
        },
      });
    });

    expect(result.current.updateInfo).toBeNull();
  });

  it("should show system notification on check-updates-no-update event", async () => {
    renderHook(() => useUpdateCheck(), { wrapper });

    await waitFor(() => {
      expect(eventCallbacks.has(EVENTS.CHECK_UPDATES_NO_UPDATE)).toBe(true);
    });

    act(() => {
      eventCallbacks.get(EVENTS.CHECK_UPDATES_NO_UPDATE)!({ payload: null });
    });

    expect(showSystemNotification).toHaveBeenCalledTimes(1);
  });

  it("should allow clearing updateInfo via setUpdateInfo(null)", async () => {
    const { result } = renderHook(() => useUpdateCheck(), { wrapper });

    await waitFor(() => {
      expect(eventCallbacks.has(EVENTS.CHECK_UPDATES_RESULT)).toBe(true);
    });

    act(() => {
      eventCallbacks.get(EVENTS.CHECK_UPDATES_RESULT)!({
        payload: {
          has_update: true,
          latest_version: "2.0.0",
          release_url: "https://example.com",
          platform_available: true,
          current_version: "1.0.0",
        },
      });
    });

    expect(result.current.updateInfo).not.toBeNull();

    act(() => {
      result.current.setUpdateInfo(null);
    });

    expect(result.current.updateInfo).toBeNull();
  });

  describe("auto-check on startup", () => {
    it("should auto-check after 60s and set updateInfo if update available", async () => {
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === "check_for_updates") {
          return {
            has_update: true,
            latest_version: "3.0.0",
            release_url: "https://example.com/v3",
            platform_available: true,
            current_version: "1.0.0",
          };
        }
        if (cmd === "is_version_ignored") {
          return false;
        }
        return undefined;
      });

      vi.useFakeTimers();
      const { result } = renderHook(() => useUpdateCheck(), { wrapper });

      expect(result.current.updateInfo).toBeNull();

      await act(async () => {
        await vi.advanceTimersByTimeAsync(60000);
      });

      expect(result.current.updateInfo).toEqual({
        version: "3.0.0",
        releaseUrl: "https://example.com/v3",
      });

      vi.useRealTimers();
    });

    it("should not set updateInfo if version is ignored", async () => {
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === "check_for_updates") {
          return {
            has_update: true,
            latest_version: "3.0.0",
            release_url: "https://example.com/v3",
            platform_available: true,
            current_version: "1.0.0",
          };
        }
        if (cmd === "is_version_ignored") {
          return true;
        }
        return undefined;
      });

      vi.useFakeTimers();
      const { result } = renderHook(() => useUpdateCheck(), { wrapper });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(60000);
      });

      expect(invoke).toHaveBeenCalledWith("is_version_ignored", {
        version: "3.0.0",
      });
      expect(result.current.updateInfo).toBeNull();

      vi.useRealTimers();
    });

    it("should not set updateInfo if no update available", async () => {
      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === "check_for_updates") {
          return {
            has_update: false,
            latest_version: "1.0.0",
            release_url: null,
            platform_available: true,
            current_version: "1.0.0",
          };
        }
        return undefined;
      });

      vi.useFakeTimers();
      const { result } = renderHook(() => useUpdateCheck(), { wrapper });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(60000);
      });

      expect(result.current.updateInfo).toBeNull();

      vi.useRealTimers();
    });

    it("should handle auto-check errors gracefully", async () => {
      const consoleSpy = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});

      vi.mocked(invoke).mockImplementation(async (cmd: string) => {
        if (cmd === "check_for_updates") {
          throw new Error("Network error");
        }
        return undefined;
      });

      vi.useFakeTimers();
      const { result } = renderHook(() => useUpdateCheck(), { wrapper });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(60000);
      });

      expect(result.current.updateInfo).toBeNull();
      expect(consoleSpy).toHaveBeenCalledWith(
        "Failed to check for updates:",
        expect.any(Error),
      );

      consoleSpy.mockRestore();
      vi.useRealTimers();
    });
  });
});
