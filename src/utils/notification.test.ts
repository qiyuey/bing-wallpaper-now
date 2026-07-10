import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { showSystemNotification } from "./notification";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core");

describe("notification", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("showSystemNotification", () => {
    it("should send notification with correct title and body", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await showSystemNotification("Test Title", "Test Body");

      expect(invoke).toHaveBeenCalledTimes(1);
      expect(invoke).toHaveBeenCalledWith("show_system_notification", {
        title: "Test Title",
        body: "Test Body",
      });
    });

    it("should handle notification errors gracefully", async () => {
      const consoleErrorSpy = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});
      const error = new Error("Notification failed");
      vi.mocked(invoke).mockRejectedValue(error);

      // Should not throw
      await expect(
        showSystemNotification("Test Title", "Test Body"),
      ).resolves.toBeUndefined();

      expect(invoke).toHaveBeenCalledTimes(1);
      expect(consoleErrorSpy).toHaveBeenCalledWith(
        "Failed to send notification:",
        error,
      );

      consoleErrorSpy.mockRestore();
    });

    it("should handle different title and body combinations", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await showSystemNotification("Title 1", "Body 1");
      await showSystemNotification("Title 2", "Body 2");

      expect(invoke).toHaveBeenCalledTimes(2);
      expect(invoke).toHaveBeenNthCalledWith(1, "show_system_notification", {
        title: "Title 1",
        body: "Body 1",
      });
      expect(invoke).toHaveBeenNthCalledWith(2, "show_system_notification", {
        title: "Title 2",
        body: "Body 2",
      });
    });
  });
});
