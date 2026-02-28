import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { listen } from "@tauri-apps/api/event";
import { useTrayEvents } from "./useTrayEvents";
import { EVENTS } from "../config/ui";

vi.mock("@tauri-apps/api/event");

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyEventHandler = (...args: any[]) => void;

describe("useTrayEvents", () => {
  let eventCallbacks: Map<string, AnyEventHandler>;

  beforeEach(() => {
    vi.clearAllMocks();
    eventCallbacks = new Map();

    vi.mocked(listen).mockImplementation(async (event, cb) => {
      eventCallbacks.set(event as string, cb as AnyEventHandler);
      return () => {};
    });
  });

  it("should register listeners for all three tray events", async () => {
    const callbacks = {
      onOpenSettings: vi.fn(),
      onOpenAbout: vi.fn(),
      onOpenFolder: vi.fn(),
    };

    renderHook(() => useTrayEvents(callbacks));

    await waitFor(() => {
      expect(listen).toHaveBeenCalledWith(
        EVENTS.OPEN_SETTINGS,
        expect.any(Function),
      );
      expect(listen).toHaveBeenCalledWith(
        EVENTS.OPEN_ABOUT,
        expect.any(Function),
      );
      expect(listen).toHaveBeenCalledWith(
        EVENTS.OPEN_FOLDER,
        expect.any(Function),
      );
    });
  });

  it("should call onOpenSettings when open-settings event fires", async () => {
    const callbacks = {
      onOpenSettings: vi.fn(),
      onOpenAbout: vi.fn(),
      onOpenFolder: vi.fn(),
    };

    renderHook(() => useTrayEvents(callbacks));

    await waitFor(() => {
      expect(eventCallbacks.has(EVENTS.OPEN_SETTINGS)).toBe(true);
    });

    act(() => {
      eventCallbacks.get(EVENTS.OPEN_SETTINGS)!();
    });

    expect(callbacks.onOpenSettings).toHaveBeenCalledTimes(1);
    expect(callbacks.onOpenAbout).not.toHaveBeenCalled();
    expect(callbacks.onOpenFolder).not.toHaveBeenCalled();
  });

  it("should call onOpenAbout when open-about event fires", async () => {
    const callbacks = {
      onOpenSettings: vi.fn(),
      onOpenAbout: vi.fn(),
      onOpenFolder: vi.fn(),
    };

    renderHook(() => useTrayEvents(callbacks));

    await waitFor(() => {
      expect(eventCallbacks.has(EVENTS.OPEN_ABOUT)).toBe(true);
    });

    act(() => {
      eventCallbacks.get(EVENTS.OPEN_ABOUT)!();
    });

    expect(callbacks.onOpenAbout).toHaveBeenCalledTimes(1);
    expect(callbacks.onOpenSettings).not.toHaveBeenCalled();
    expect(callbacks.onOpenFolder).not.toHaveBeenCalled();
  });

  it("should call onOpenFolder when open-folder event fires", async () => {
    const callbacks = {
      onOpenSettings: vi.fn(),
      onOpenAbout: vi.fn(),
      onOpenFolder: vi.fn(),
    };

    renderHook(() => useTrayEvents(callbacks));

    await waitFor(() => {
      expect(eventCallbacks.has(EVENTS.OPEN_FOLDER)).toBe(true);
    });

    act(() => {
      eventCallbacks.get(EVENTS.OPEN_FOLDER)!();
    });

    expect(callbacks.onOpenFolder).toHaveBeenCalledTimes(1);
    expect(callbacks.onOpenSettings).not.toHaveBeenCalled();
    expect(callbacks.onOpenAbout).not.toHaveBeenCalled();
  });

  it("should use latest callbacks via ref (no re-bind needed)", async () => {
    const initialCallbacks = {
      onOpenSettings: vi.fn(),
      onOpenAbout: vi.fn(),
      onOpenFolder: vi.fn(),
    };

    const { rerender } = renderHook((props) => useTrayEvents(props), {
      initialProps: initialCallbacks,
    });

    await waitFor(() => {
      expect(eventCallbacks.has(EVENTS.OPEN_SETTINGS)).toBe(true);
    });

    const updatedOnOpenSettings = vi.fn();
    rerender({
      ...initialCallbacks,
      onOpenSettings: updatedOnOpenSettings,
    });

    act(() => {
      eventCallbacks.get(EVENTS.OPEN_SETTINGS)!();
    });

    expect(initialCallbacks.onOpenSettings).not.toHaveBeenCalled();
    expect(updatedOnOpenSettings).toHaveBeenCalledTimes(1);
  });

  it("should call unlisteners on unmount", async () => {
    const unlistenFns = [vi.fn(), vi.fn(), vi.fn()];
    let callIndex = 0;

    vi.mocked(listen).mockImplementation(async (event, cb) => {
      eventCallbacks.set(event as string, cb as AnyEventHandler);
      return unlistenFns[callIndex++];
    });

    const callbacks = {
      onOpenSettings: vi.fn(),
      onOpenAbout: vi.fn(),
      onOpenFolder: vi.fn(),
    };

    const { unmount } = renderHook(() => useTrayEvents(callbacks));

    await waitFor(() => {
      expect(eventCallbacks.size).toBe(3);
    });

    unmount();

    for (const fn of unlistenFns) {
      expect(fn).toHaveBeenCalled();
    }
  });
});
