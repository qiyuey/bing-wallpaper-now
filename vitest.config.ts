import { defineConfig } from "vitest/config";

/**
 * Vitest configuration for Bing Wallpaper Now
 *
 * Centralizes test + coverage settings so CI and local runs
 * use the same defaults. This mirrors the inline flags previously
 * passed via the coverage script, making maintenance easier.
 *
 * Recommended usage:
 *   npx vitest
 *   npx vitest run
 *   npx vitest run --coverage
 *
 * Coverage thresholds here are initial (soft) targets; CI currently
 * runs in continue-on-error mode for the coverage job, so failing
 * thresholds will not block merges until we raise the gate.
 */
export default defineConfig({
  test: {
    // Use jsdom so React component tests can run without a browser.
    environment: "jsdom",
    // Enable globals (describe, it, expect) without importing from vitest everywhere.
    globals: true,
    // Setup file for test utilities and mocks
    setupFiles: ["./src/test/setup.ts"],

    // File patterns
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
    exclude: [
      "node_modules",
      "dist",
      "coverage",
      "coverage-frontend",
      "src-tauri",
      "**/__fixtures__/**",
      "**/__mocks__/**",
    ],

    // Reporting & coverage
    coverage: {
      enabled: true,
      provider: "v8", // fast, built-in V8 instrumentation
      reportsDirectory: "coverage-frontend",
      reporter: ["text", "lcov", "json"],
      // Initial soft thresholds (match README / quality baseline plan)
      lines: 70,
      functions: 70,
      branches: 60,
      statements: 70,
      // Exclude non-source or generated files from coverage calculations
      exclude: [
        "vite.config.ts",
        "vitest.config.ts",
        "src/main.tsx", // bootstrap/entry (often minimal logic)
        "src/**/*.d.ts",
        "**/*.config.*",
      ],
    },

    // Timeouts / performance tuning (adjust if tests grow)
    testTimeout: 15_000,
    hookTimeout: 15_000,
    clearMocks: true,
    restoreMocks: true,
    unstubGlobals: true,
    unstubEnvs: true,
  },
});
