import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface VersionCheckResult {
  current_version: string;
  latest_version: string | null;
  has_update: boolean;
  release_url: string | null;
  platform_available: boolean;
}

export function useVersionCheck() {
  const [checking, setChecking] = useState(false);
  const [result, setResult] = useState<VersionCheckResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const checkForUpdates = useCallback(async () => {
    setChecking(true);
    setError(null);
    try {
      const response = await invoke<VersionCheckResult>("check_for_updates");
      setResult(response);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error("Failed to check for updates:", err);
    } finally {
      setChecking(false);
    }
  }, []);

  return {
    checking,
    result,
    error,
    checkForUpdates,
  };
}
