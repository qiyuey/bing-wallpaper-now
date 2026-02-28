import { describe, it, expect } from "vitest";
import {
  buildTransferMessage,
  buildTransferErrorMessage,
  TransferResult,
  TransferTranslations,
} from "./transferHelpers";

const translations: TransferTranslations = {
  selectDirectory: "Select directory",
  success: "New: {new}, Updated: {updated}, Images: {images}",
  alreadyUpToDate: "Already up to date",
  metadataSkipped: "{count} metadata skipped",
  imagesFailed: "{count} images failed",
  notDirectory: "Not a directory",
  sameDirectory: "Same directory",
  noData: "No data",
  error: "Error",
};

describe("transferHelpers", () => {
  describe("buildTransferMessage", () => {
    it("should return alreadyUpToDate when no activity", () => {
      const result: TransferResult = {
        metadata_new: 0,
        metadata_updated: 0,
        metadata_skipped: 0,
        images_copied: 0,
        images_skipped: 0,
        images_failed: 0,
        mkt_count: 0,
      };

      const msg = buildTransferMessage(result, translations, "; ");

      expect(msg.type).toBe("success");
      expect(msg.text).toBe("Already up to date");
    });

    it("should format success message with counts", () => {
      const result: TransferResult = {
        metadata_new: 3,
        metadata_updated: 2,
        metadata_skipped: 0,
        images_copied: 5,
        images_skipped: 1,
        images_failed: 0,
        mkt_count: 1,
      };

      const msg = buildTransferMessage(result, translations, "; ");

      expect(msg.type).toBe("success");
      expect(msg.text).toBe("New: 3, Updated: 2, Images: 5");
    });

    it("should append metadata_skipped warning", () => {
      const result: TransferResult = {
        metadata_new: 1,
        metadata_updated: 0,
        metadata_skipped: 4,
        images_copied: 0,
        images_skipped: 0,
        images_failed: 0,
        mkt_count: 1,
      };

      const msg = buildTransferMessage(result, translations, "; ");

      expect(msg.type).toBe("success");
      expect(msg.text).toContain("4 metadata skipped");
    });

    it("should append images_failed warning", () => {
      const result: TransferResult = {
        metadata_new: 0,
        metadata_updated: 0,
        metadata_skipped: 0,
        images_copied: 0,
        images_skipped: 0,
        images_failed: 2,
        mkt_count: 1,
      };

      const msg = buildTransferMessage(result, translations, "; ");

      expect(msg.type).toBe("success");
      expect(msg.text).toContain("2 images failed");
    });

    it("should join multiple warnings with separator", () => {
      const result: TransferResult = {
        metadata_new: 1,
        metadata_updated: 0,
        metadata_skipped: 3,
        images_copied: 0,
        images_skipped: 0,
        images_failed: 2,
        mkt_count: 1,
      };

      const msg = buildTransferMessage(result, translations, " | ");

      expect(msg.text).toContain("3 metadata skipped | 2 images failed");
    });

    it("should count metadata_skipped and images_failed toward totalActivity", () => {
      const result: TransferResult = {
        metadata_new: 0,
        metadata_updated: 0,
        metadata_skipped: 1,
        images_copied: 0,
        images_skipped: 0,
        images_failed: 0,
        mkt_count: 0,
      };

      const msg = buildTransferMessage(result, translations, "; ");

      expect(msg.type).toBe("success");
      expect(msg.text).not.toBe("Already up to date");
    });
  });

  describe("buildTransferErrorMessage", () => {
    it("should map NOT_DIRECTORY to notDirectory translation", () => {
      const msg = buildTransferErrorMessage("NOT_DIRECTORY", translations);

      expect(msg.type).toBe("error");
      expect(msg.text).toBe("Not a directory");
    });

    it("should map SAME_DIRECTORY to sameDirectory translation", () => {
      const msg = buildTransferErrorMessage("SAME_DIRECTORY", translations);

      expect(msg.type).toBe("error");
      expect(msg.text).toBe("Same directory");
    });

    it("should map NO_DATA to noData translation", () => {
      const msg = buildTransferErrorMessage("NO_DATA", translations);

      expect(msg.type).toBe("error");
      expect(msg.text).toBe("No data");
    });

    it("should format unknown errors with error prefix", () => {
      const msg = buildTransferErrorMessage("Some unknown error", translations);

      expect(msg.type).toBe("error");
      expect(msg.text).toBe("Error: Some unknown error");
    });

    it("should handle Error objects", () => {
      const msg = buildTransferErrorMessage(
        new Error("Network failure"),
        translations,
      );

      expect(msg.type).toBe("error");
      expect(msg.text).toBe("Error: Error: Network failure");
    });
  });
});
