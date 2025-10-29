/**
 * Safe event listener utilities for Tauri + React
 * Handles React StrictMode's double-mount behavior and Tauri's event system quirks
 */

/**
 * Wraps a Tauri unlisten function to make it safe to call multiple times
 *
 * This is necessary because:
 * 1. React StrictMode intentionally double-mounts components in dev mode
 * 2. This can cause the same unlisten function to be called twice
 * 3. Tauri's event system throws errors when listeners are in inconsistent state
 *
 * @param unlisten - The original unlisten function from Tauri's listen()
 * @returns A wrapped unlisten function that's safe to call multiple times
 */
export function createSafeUnlisten(unlisten: () => void): () => void {
  let called = false;

  return () => {
    if (called) {
      return; // Already called, silently ignore
    }
    called = true;

    try {
      unlisten();
    } catch (error: unknown) {
      // Catch and ignore Tauri's internal listener errors
      // These occur when the event system is in an inconsistent state
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      if (
        !(
          errorMessage.includes("listeners") &&
          errorMessage.includes("handlerId")
        )
      ) {
        // Re-throw unexpected errors
        throw error;
      }
      // Known Tauri error - silently ignore
    }
  };
}
