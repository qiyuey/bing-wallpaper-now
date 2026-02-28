import { ReactNode } from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { check, Update } from "@tauri-apps/plugin-updater";
import { useUpdateCheck } from "./useUpdateCheck";
import { showSystemNotification } from "../utils/notification";
import { I18nProvider } from "../i18n/I18nContext";

vi.mock("@tauri-apps/api/core");
vi.mock("@tauri-apps/api/event");
vi.mock("../utils/notification");
vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(),
  Update: vi.fn(),
}));

function wrapper({ children }: { children: ReactNode }) {
  return <I18nProvider>{children}</I18nProvider>;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyEventHandler = (...args: any[]) => void;

function createMockUpdate(version: string): Update {
  return {
    available: true,
    currentVersion: "1.0.0",
    version,
    body: "Release notes",
    rawJson: {},
    downloadAndInstall: vi.fn().mockResolvedValue(undefined),
    download: vi.fn().mockResolvedValue(undefined),
    install: vi.fn().mockResolvedValue(undefined),
    close: vi.fn().mockResolvedValue(undefined),
  } as unknown as Update;
}

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
    vi.mocked(check).mockResolvedValue(null);
    vi.mocked(showSystemNotification).mockResolvedValue(undefined);
  });

  it("should initialize with null updateInfo", () => {
    const { result } = renderHook(() => useUpdateCheck(), { wrapper });
    expect(result.current.updateInfo).toBeNull();
  });

  it("should register listener for tray-check-updates event", async () => {
    renderHook(() => useUpdateCheck(), { wrapper });

    await waitFor(() => {
      expect(listen).toHaveBeenCalledWith(
        "tray-check-updates",
        expect.any(Function),
      );
    });
  });

  it("should set updateInfo when tray check finds an update", async () => {
    const mockUpdate = createMockUpdate("2.0.0");
    vi.mocked(check).mockResolvedValue(mockUpdate);
    vi.mocked(invoke).mockResolvedValue(false);

    const { result } = renderHook(() => useUpdateCheck(), { wrapper });

    await waitFor(() => {
      expect(eventCallbacks.has("tray-check-updates")).toBe(true);
    });

    await act(async () => {
      await eventCallbacks.get("tray-check-updates")!({
        payload: null,
      });
    });

    expect(result.current.updateInfo).not.toBeNull();
    expect(result.current.updateInfo?.version).toBe("2.0.0");
  });

  it("should show notification when tray check finds no update", async () => {
    vi.mocked(check).mockResolvedValue(null);

    renderHook(() => useUpdateCheck(), { wrapper });

    await waitFor(() => {
      expect(eventCallbacks.has("tray-check-updates")).toBe(true);
    });

    await act(async () => {
      await eventCallbacks.get("tray-check-updates")!({
        payload: null,
      });
    });

    expect(showSystemNotification).toHaveBeenCalledTimes(1);
  });

  it("should show notification when tray check errors", async () => {
    vi.mocked(check).mockRejectedValue(new Error("Network error"));

    renderHook(() => useUpdateCheck(), { wrapper });

    await waitFor(() => {
      expect(eventCallbacks.has("tray-check-updates")).toBe(true);
    });

    await act(async () => {
      await eventCallbacks.get("tray-check-updates")!({
        payload: null,
      });
    });

    expect(showSystemNotification).toHaveBeenCalledTimes(1);
  });

  it("should not set updateInfo when tray check finds ignored version", async () => {
    const mockUpdate = createMockUpdate("2.0.0");
    vi.mocked(check).mockResolvedValue(mockUpdate);
    vi.mocked(invoke).mockResolvedValue(true); // is_version_ignored = true

    const { result } = renderHook(() => useUpdateCheck(), { wrapper });

    await waitFor(() => {
      expect(eventCallbacks.has("tray-check-updates")).toBe(true);
    });

    await act(async () => {
      await eventCallbacks.get("tray-check-updates")!({
        payload: null,
      });
    });

    // For tray (manual) check with showNoUpdate=true, ignored version shows "no update" notification
    expect(result.current.updateInfo).toBeNull();
    expect(showSystemNotification).toHaveBeenCalledTimes(1);
  });

  it("should allow clearing updateInfo via setUpdateInfo(null)", async () => {
    const mockUpdate = createMockUpdate("2.0.0");
    vi.mocked(check).mockResolvedValue(mockUpdate);
    vi.mocked(invoke).mockResolvedValue(false);

    const { result } = renderHook(() => useUpdateCheck(), { wrapper });

    await waitFor(() => {
      expect(eventCallbacks.has("tray-check-updates")).toBe(true);
    });

    await act(async () => {
      await eventCallbacks.get("tray-check-updates")!({
        payload: null,
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
      const mockUpdate = createMockUpdate("3.0.0");
      vi.mocked(check).mockResolvedValue(mockUpdate);
      vi.mocked(invoke).mockResolvedValue(false); // not ignored

      vi.useFakeTimers();
      const { result } = renderHook(() => useUpdateCheck(), { wrapper });

      expect(result.current.updateInfo).toBeNull();

      await act(async () => {
        await vi.advanceTimersByTimeAsync(60000);
      });

      expect(result.current.updateInfo).not.toBeNull();
      expect(result.current.updateInfo?.version).toBe("3.0.0");

      vi.useRealTimers();
    });

    it("should not set updateInfo if version is ignored", async () => {
      const mockUpdate = createMockUpdate("3.0.0");
      vi.mocked(check).mockResolvedValue(mockUpdate);
      vi.mocked(invoke).mockResolvedValue(true); // is_version_ignored = true

      vi.useFakeTimers();
      const { result } = renderHook(() => useUpdateCheck(), { wrapper });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(60000);
      });

      expect(invoke).toHaveBeenCalledWith("is_version_ignored", {
        version: "3.0.0",
      });
      expect(result.current.updateInfo).toBeNull();
      // Silent auto-check should NOT show notification
      expect(showSystemNotification).not.toHaveBeenCalled();

      vi.useRealTimers();
    });

    it("should not set updateInfo if no update available", async () => {
      vi.mocked(check).mockResolvedValue(null);

      vi.useFakeTimers();
      const { result } = renderHook(() => useUpdateCheck(), { wrapper });

      await act(async () => {
        await vi.advanceTimersByTimeAsync(60000);
      });

      expect(result.current.updateInfo).toBeNull();
      // Silent auto-check should NOT show notification
      expect(showSystemNotification).not.toHaveBeenCalled();

      vi.useRealTimers();
    });

    it("should handle auto-check errors gracefully", async () => {
      const consoleSpy = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});

      vi.mocked(check).mockRejectedValue(new Error("Network error"));

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
      // Silent auto-check should NOT show notification
      expect(showSystemNotification).not.toHaveBeenCalled();

      consoleSpy.mockRestore();
      vi.useRealTimers();
    });
  });

  describe("timeout fallback", () => {
    it("should show failure notification when check() hangs and times out", async () => {
      const consoleSpy = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});

      // check() returns a promise that never resolves (simulating DNS hang)
      vi.mocked(check).mockReturnValue(new Promise(() => {}));

      renderHook(() => useUpdateCheck(), { wrapper });

      await waitFor(() => {
        expect(eventCallbacks.has("tray-check-updates")).toBe(true);
      });

      // Switch to fake timers AFTER listener is registered
      vi.useFakeTimers();

      await act(async () => {
        eventCallbacks.get("tray-check-updates")!({ payload: null });
        // Advance past the 15s JS-level timeout
        await vi.advanceTimersByTimeAsync(15_000);
      });

      expect(consoleSpy).toHaveBeenCalledWith(
        "Failed to check for updates:",
        expect.objectContaining({
          message: expect.stringContaining("timed out"),
        }),
      );
      expect(showSystemNotification).toHaveBeenCalledTimes(1);

      consoleSpy.mockRestore();
      vi.useRealTimers();
    });

    it("should not time out when check() resolves quickly", async () => {
      vi.mocked(check).mockResolvedValue(null);

      renderHook(() => useUpdateCheck(), { wrapper });

      await waitFor(() => {
        expect(eventCallbacks.has("tray-check-updates")).toBe(true);
      });

      await act(async () => {
        await eventCallbacks.get("tray-check-updates")!({
          payload: null,
        });
      });

      // Should show "no update" notification, not "failed"
      expect(showSystemNotification).toHaveBeenCalledTimes(1);
    });
  });
});
