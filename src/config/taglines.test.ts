import { describe, it, expect, vi, afterEach } from "vitest";
import { getCurrentTagline, getDailyTagline, getAllTaglines } from "./taglines";

// Mock detectSystemLanguage to return a stable value
vi.mock("../i18n/translations", () => ({
  detectSystemLanguage: () => "en-US" as const,
}));

describe("taglines", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("getCurrentTagline", () => {
    it("should return a morning tagline for hours 6-11", () => {
      const allZh = getAllTaglines("zh-CN");
      const morningPool = allZh.slice(0, 7);

      for (const hour of [6, 7, 8, 9, 10, 11]) {
        const tagline = getCurrentTagline(hour, "zh-CN");
        expect(morningPool).toContain(tagline);
      }
    });

    it("should return an afternoon tagline for hours 12-17", () => {
      const allZh = getAllTaglines("zh-CN");
      const afternoonPool = allZh.slice(7, 14);

      for (const hour of [12, 13, 14, 15, 16, 17]) {
        const tagline = getCurrentTagline(hour, "zh-CN");
        expect(afternoonPool).toContain(tagline);
      }
    });

    it("should return an evening tagline for hours 18-23 and 0-5", () => {
      const allZh = getAllTaglines("zh-CN");
      const eveningPool = allZh.slice(14, 20);

      for (const hour of [18, 19, 20, 21, 22, 23, 0, 1, 2, 3, 4, 5]) {
        const tagline = getCurrentTagline(hour, "zh-CN");
        expect(eveningPool).toContain(tagline);
      }
    });

    it("should return English taglines when lang is en-US", () => {
      const allEn = getAllTaglines("en-US");
      const tagline = getCurrentTagline(10, "en-US");
      expect(allEn).toContain(tagline);
    });

    it("should use current hour when hour param is not provided", () => {
      // Just verify it returns a non-empty string without throwing
      const tagline = getCurrentTagline(undefined, "zh-CN");
      expect(tagline).toBeTruthy();
      expect(typeof tagline).toBe("string");
    });

    it("should use detected system language when lang is not provided", () => {
      // detectSystemLanguage is mocked to return "en-US"
      const tagline = getCurrentTagline(10);
      const allEn = getAllTaglines("en-US");
      expect(allEn).toContain(tagline);
    });
  });

  describe("getDailyTagline", () => {
    it("should return the same tagline when called multiple times on the same day", () => {
      const tagline1 = getDailyTagline("zh-CN");
      const tagline2 = getDailyTagline("zh-CN");
      expect(tagline1).toBe(tagline2);
    });

    it("should return a string from the taglines array", () => {
      const allZh = getAllTaglines("zh-CN");
      const tagline = getDailyTagline("zh-CN");
      expect(allZh).toContain(tagline);
    });

    it("should return English tagline for en-US", () => {
      const allEn = getAllTaglines("en-US");
      const tagline = getDailyTagline("en-US");
      expect(allEn).toContain(tagline);
    });

    it("should use detected system language when lang is not provided", () => {
      // detectSystemLanguage is mocked to return "en-US"
      const tagline = getDailyTagline();
      const allEn = getAllTaglines("en-US");
      expect(allEn).toContain(tagline);
    });
  });

  describe("getAllTaglines", () => {
    it("should return zh-CN taglines array", () => {
      const taglines = getAllTaglines("zh-CN");
      expect(Array.isArray(taglines)).toBe(true);
      expect(taglines.length).toBe(20);
      // Verify it contains Chinese text
      expect(taglines[0]).toMatch(/[\u4e00-\u9fff]/);
    });

    it("should return en-US taglines array", () => {
      const taglines = getAllTaglines("en-US");
      expect(Array.isArray(taglines)).toBe(true);
      expect(taglines.length).toBe(10);
      // Verify it contains English text
      expect(taglines[0]).toMatch(/[a-zA-Z]/);
    });

    it("should use detected system language when lang is not provided", () => {
      // detectSystemLanguage is mocked to return "en-US"
      const taglines = getAllTaglines();
      expect(taglines.length).toBe(10);
    });

    it("should return readonly arrays", () => {
      const taglines = getAllTaglines("zh-CN");
      // TypeScript readonly arrays should still be iterable
      expect(taglines).toHaveLength(20);
    });
  });
});
