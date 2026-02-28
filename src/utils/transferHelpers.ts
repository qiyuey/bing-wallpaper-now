export type TransferResult = {
  metadata_new: number;
  metadata_updated: number;
  metadata_skipped: number;
  images_copied: number;
  images_skipped: number;
  images_failed: number;
  mkt_count: number;
};

export type TransferMessage = { type: "success" | "error"; text: string };

export type TransferTranslations = {
  selectDirectory: string;
  success: string;
  alreadyUpToDate: string;
  metadataSkipped: string;
  imagesFailed: string;
  notDirectory: string;
  sameDirectory: string;
  noData: string;
  error: string;
};

export function buildTransferMessage(
  result: TransferResult,
  translations: TransferTranslations,
  warningSeparator: string,
): TransferMessage {
  const totalActivity =
    result.metadata_new +
    result.metadata_updated +
    result.images_copied +
    result.metadata_skipped +
    result.images_failed;

  if (totalActivity === 0) {
    return { type: "success", text: translations.alreadyUpToDate };
  }

  let text = translations.success
    .replace("{new}", String(result.metadata_new))
    .replace("{updated}", String(result.metadata_updated))
    .replace("{images}", String(result.images_copied));

  const warnings: string[] = [];
  if (result.metadata_skipped > 0) {
    warnings.push(
      translations.metadataSkipped.replace(
        "{count}",
        String(result.metadata_skipped),
      ),
    );
  }
  if (result.images_failed > 0) {
    warnings.push(
      translations.imagesFailed.replace(
        "{count}",
        String(result.images_failed),
      ),
    );
  }
  if (warnings.length > 0) {
    text += `\n${warnings.join(warningSeparator)}`;
  }

  return { type: "success", text };
}

const TRANSFER_ERROR_MAP: Record<string, keyof TransferTranslations> = {
  NOT_DIRECTORY: "notDirectory",
  SAME_DIRECTORY: "sameDirectory",
  NO_DATA: "noData",
};

export function buildTransferErrorMessage(
  err: unknown,
  translations: TransferTranslations,
): TransferMessage {
  const msg = String(err);
  const key = TRANSFER_ERROR_MAP[msg];
  if (key) {
    return { type: "error", text: translations[key] };
  }
  return { type: "error", text: `${translations.error}: ${msg}` };
}
