import { describe, it, expect } from "vitest";
import {
  BREAKPOINTS,
  CARDS_PER_ROW,
  CARD_DIMENSIONS,
  SPACING,
  calculateRowHeight,
  getCardsPerRow,
} from "./layout";

describe("layout", () => {
  describe("constants", () => {
    it("should have valid breakpoint values in ascending order", () => {
      expect(BREAKPOINTS.NARROW).toBeLessThan(BREAKPOINTS.TABLET);
      expect(BREAKPOINTS.TABLET).toBeLessThan(BREAKPOINTS.DESKTOP_4K);
    });

    it("should have valid cards per row values", () => {
      expect(CARDS_PER_ROW.SINGLE).toBe(1);
      expect(CARDS_PER_ROW.NARROW).toBe(2);
      expect(CARDS_PER_ROW.DESKTOP).toBe(3);
      expect(CARDS_PER_ROW.FOUR_K).toBe(4);
    });

    it("should have positive card dimensions", () => {
      expect(CARD_DIMENSIONS.IMAGE_HEIGHT).toBeGreaterThan(0);
      expect(CARD_DIMENSIONS.PADDING_X).toBeGreaterThan(0);
      expect(CARD_DIMENSIONS.PADDING_Y).toBeGreaterThan(0);
      expect(CARD_DIMENSIONS.TITLE_FONT_SIZE).toBeGreaterThan(0);
      expect(CARD_DIMENSIONS.BUTTON_HEIGHT).toBeGreaterThan(0);
    });

    it("should have positive spacing values", () => {
      expect(SPACING.ROW_GAP_NARROW).toBeGreaterThan(0);
      expect(SPACING.ROW_GAP_DESKTOP).toBeGreaterThan(0);
      expect(SPACING.ROW_MARGIN_BOTTOM).toBeGreaterThan(0);
    });
  });

  describe("getCardsPerRow", () => {
    it("should return SINGLE for width <= NARROW (750)", () => {
      expect(getCardsPerRow(750)).toBe(CARDS_PER_ROW.SINGLE);
      expect(getCardsPerRow(500)).toBe(CARDS_PER_ROW.SINGLE);
      expect(getCardsPerRow(1)).toBe(CARDS_PER_ROW.SINGLE);
    });

    it("should return NARROW for width > NARROW and <= TABLET (751-1024)", () => {
      expect(getCardsPerRow(751)).toBe(CARDS_PER_ROW.NARROW);
      expect(getCardsPerRow(900)).toBe(CARDS_PER_ROW.NARROW);
      expect(getCardsPerRow(1024)).toBe(CARDS_PER_ROW.NARROW);
    });

    it("should return DESKTOP for width > TABLET and < DESKTOP_4K (1025-1399)", () => {
      expect(getCardsPerRow(1025)).toBe(CARDS_PER_ROW.DESKTOP);
      expect(getCardsPerRow(1200)).toBe(CARDS_PER_ROW.DESKTOP);
      expect(getCardsPerRow(1399)).toBe(CARDS_PER_ROW.DESKTOP);
    });

    it("should return FOUR_K for width >= DESKTOP_4K (1400+)", () => {
      expect(getCardsPerRow(1400)).toBe(CARDS_PER_ROW.FOUR_K);
      expect(getCardsPerRow(2000)).toBe(CARDS_PER_ROW.FOUR_K);
      expect(getCardsPerRow(3840)).toBe(CARDS_PER_ROW.FOUR_K);
    });

    it("should handle exact breakpoint boundaries", () => {
      // At NARROW boundary
      expect(getCardsPerRow(BREAKPOINTS.NARROW)).toBe(CARDS_PER_ROW.SINGLE);
      expect(getCardsPerRow(BREAKPOINTS.NARROW + 1)).toBe(CARDS_PER_ROW.NARROW);

      // At TABLET boundary
      expect(getCardsPerRow(BREAKPOINTS.TABLET)).toBe(CARDS_PER_ROW.NARROW);
      expect(getCardsPerRow(BREAKPOINTS.TABLET + 1)).toBe(
        CARDS_PER_ROW.DESKTOP,
      );

      // At DESKTOP_4K boundary
      expect(getCardsPerRow(BREAKPOINTS.DESKTOP_4K - 1)).toBe(
        CARDS_PER_ROW.DESKTOP,
      );
      expect(getCardsPerRow(BREAKPOINTS.DESKTOP_4K)).toBe(CARDS_PER_ROW.FOUR_K);
    });

    it("should handle edge case of zero width", () => {
      expect(getCardsPerRow(0)).toBe(CARDS_PER_ROW.SINGLE);
    });
  });

  describe("calculateRowHeight", () => {
    it("should return a positive number", () => {
      const height = calculateRowHeight();
      expect(height).toBeGreaterThan(0);
    });

    it("should return consistent results", () => {
      expect(calculateRowHeight()).toBe(calculateRowHeight());
    });

    it("should include image height as a component", () => {
      const height = calculateRowHeight();
      expect(height).toBeGreaterThan(CARD_DIMENSIONS.IMAGE_HEIGHT);
    });

    it("should be a reasonable pixel value", () => {
      const height = calculateRowHeight();
      // Row height should be at least image height + some info + buttons
      expect(height).toBeGreaterThan(300);
      // But not unreasonably large
      expect(height).toBeLessThan(600);
    });
  });
});
