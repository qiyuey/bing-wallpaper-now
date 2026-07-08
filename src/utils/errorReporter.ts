import { invoke } from "@tauri-apps/api/core";

export type FrontendErrorSource =
  | "bootstrap"
  | "mark_frontend_ready"
  | "react-error-boundary"
  | "unhandledrejection"
  | "window.error";

interface FrontendErrorReport {
  source: FrontendErrorSource;
  error: unknown;
  stack?: string;
  context?: Record<string, unknown>;
}

interface UnhandledRejectionLike {
  reason: unknown;
  preventDefault: () => void;
}

interface WindowErrorLike {
  error?: unknown;
  message: string;
}

interface ErrorHandlerTarget {
  addEventListener(
    type: "unhandledrejection",
    listener: (event: UnhandledRejectionLike) => void,
  ): void;
  addEventListener(
    type: "error",
    listener: (event: WindowErrorLike) => void,
  ): void;
}

export function stringifyUnknownError(error: unknown): string {
  if (error instanceof Error) {
    return error.message || error.name;
  }

  if (typeof error === "string") {
    return error;
  }

  try {
    const json = JSON.stringify(error);
    return json ?? String(error);
  } catch {
    return String(error);
  }
}

export function getErrorStack(error: unknown): string | undefined {
  return error instanceof Error ? error.stack : undefined;
}

export function isIgnoredTauriListenerRejection(reason: unknown): boolean {
  const errorMessage =
    reason instanceof Error ? reason.message : stringifyUnknownError(reason);

  return (
    errorMessage.includes("listeners") && errorMessage.includes("handlerId")
  );
}

export function reportFrontendError({
  source,
  error,
  stack,
  context,
}: FrontendErrorReport) {
  let contextText: string | undefined;
  if (context) {
    try {
      contextText = JSON.stringify(context);
    } catch {
      contextText = String(context);
    }
  }

  void invoke("report_frontend_error", {
    source,
    message: stringifyUnknownError(error),
    stack: stack ?? getErrorStack(error) ?? null,
    context: contextText ?? null,
  }).catch(() => {
    // Error reporting must not create another unhandled rejection.
  });
}

export function markFrontendReady() {
  void invoke("mark_frontend_ready").catch((error) => {
    reportFrontendError({
      source: "mark_frontend_ready",
      error,
    });
  });
}

export function installGlobalErrorHandlers(
  target: ErrorHandlerTarget = window as ErrorHandlerTarget,
) {
  target.addEventListener("unhandledrejection", (event) => {
    if (isIgnoredTauriListenerRejection(event.reason)) {
      event.preventDefault();
      return;
    }

    reportFrontendError({
      source: "unhandledrejection",
      error: event.reason,
    });
  });

  target.addEventListener("error", (event) => {
    reportFrontendError({
      source: "window.error",
      error: event.error || event.message,
      stack: getErrorStack(event.error),
    });
  });
}
