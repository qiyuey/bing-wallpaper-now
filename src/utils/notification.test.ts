import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { showSystemNotification } from "./notification";
import * as notificationPlugin from "@tauri-apps/plugin-notification";

// Mock the notification plugin
vi.mock("@tauri-apps/plugin-notification", () => ({
  sendNotification: vi.fn(),
}));

describe("notification", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("showSystemNotification", () => {
    it("should send notification with correct title and body", async () => {
      const mockSendNotification = vi.mocked(
        notificationPlugin.sendNotification,
      );
      mockSendNotification.mockResolvedValue(undefined);

      await showSystemNotification("Test Title", "Test Body");

      expect(mockSendNotification).toHaveBeenCalledTimes(1);
      expect(mockSendNotification).toHaveBeenCalledWith({
        title: "Test Title",
        body: "Test Body",
      });
    });

    it("should handle notification errors gracefully", async () => {
      const mockSendNotification = vi.mocked(
        notificationPlugin.sendNotification,
      );
      const consoleErrorSpy = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});
      const error = new Error("Notification failed");
      mockSendNotification.mockRejectedValue(error);

      // Should not throw
      await expect(
        showSystemNotification("Test Title", "Test Body"),
      ).resolves.toBeUndefined();

      expect(mockSendNotification).toHaveBeenCalledTimes(1);
      expect(consoleErrorSpy).toHaveBeenCalledWith(
        "Failed to send notification:",
        error,
      );

      consoleErrorSpy.mockRestore();
    });

    it("should handle different title and body combinations", async () => {
      const mockSendNotification = vi.mocked(
        notificationPlugin.sendNotification,
      );
      mockSendNotification.mockResolvedValue(undefined);

      await showSystemNotification("Title 1", "Body 1");
      await showSystemNotification("Title 2", "Body 2");

      expect(mockSendNotification).toHaveBeenCalledTimes(2);
      expect(mockSendNotification).toHaveBeenNthCalledWith(1, {
        title: "Title 1",
        body: "Body 1",
      });
      expect(mockSendNotification).toHaveBeenNthCalledWith(2, {
        title: "Title 2",
        body: "Body 2",
      });
    });
  });
});
