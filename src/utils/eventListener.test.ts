import { describe, it, expect, vi } from "vitest";
import { createSafeUnlisten } from "./eventListener";

describe("eventListener", () => {
  describe("createSafeUnlisten", () => {
    it("should call the original unlisten function on first call", () => {
      const mockUnlisten = vi.fn();
      const safeUnlisten = createSafeUnlisten(mockUnlisten);

      safeUnlisten();

      expect(mockUnlisten).toHaveBeenCalledTimes(1);
    });

    it("should not call the original unlisten function on subsequent calls", () => {
      const mockUnlisten = vi.fn();
      const safeUnlisten = createSafeUnlisten(mockUnlisten);

      safeUnlisten();
      safeUnlisten();
      safeUnlisten();

      expect(mockUnlisten).toHaveBeenCalledTimes(1);
    });

    it("should silently ignore Tauri listener errors", () => {
      const tauriError = new Error("listeners map is missing handlerId 123");
      const mockUnlisten = vi.fn(() => {
        throw tauriError;
      });
      const safeUnlisten = createSafeUnlisten(mockUnlisten);

      // Should not throw
      expect(() => safeUnlisten()).not.toThrow();
      expect(mockUnlisten).toHaveBeenCalledTimes(1);
    });

    it("should re-throw unexpected errors", () => {
      const unexpectedError = new Error("Unexpected network error");
      const mockUnlisten = vi.fn(() => {
        throw unexpectedError;
      });
      const safeUnlisten = createSafeUnlisten(mockUnlisten);

      expect(() => safeUnlisten()).toThrow("Unexpected network error");
      expect(mockUnlisten).toHaveBeenCalledTimes(1);
    });

    it("should handle non-Error objects thrown by unlisten", () => {
      const mockUnlisten = vi.fn(() => {
        throw "string error with listeners and handlerId";
      });
      const safeUnlisten = createSafeUnlisten(mockUnlisten);

      // Should not throw because error message contains both keywords
      expect(() => safeUnlisten()).not.toThrow();
      expect(mockUnlisten).toHaveBeenCalledTimes(1);
    });

    it("should re-throw non-Error objects that don't match Tauri error pattern", () => {
      const mockUnlisten = vi.fn(() => {
        throw "some other error";
      });
      const safeUnlisten = createSafeUnlisten(mockUnlisten);

      expect(() => safeUnlisten()).toThrow("some other error");
      expect(mockUnlisten).toHaveBeenCalledTimes(1);
    });

    it("should be safe to call multiple times even after error", () => {
      let callCount = 0;
      const mockUnlisten = vi.fn(() => {
        callCount++;
        if (callCount === 1) {
          throw new Error("listeners error with handlerId");
        }
      });
      const safeUnlisten = createSafeUnlisten(mockUnlisten);

      safeUnlisten(); // First call throws error (silently caught)
      safeUnlisten(); // Second call should do nothing

      expect(mockUnlisten).toHaveBeenCalledTimes(1);
    });
  });
});
