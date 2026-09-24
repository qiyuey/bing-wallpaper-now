import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import { playwright } from "@vitest/browser-playwright";

// Run DOM logic in jsdom and layout-sensitive components in Chromium.
export default defineConfig({
  test: {
    globals: true,
    projects: [
      {
        test: {
          name: "unit",
          environment: "jsdom",
          setupFiles: ["./src/test/setup.ts"],
          include: ["src/**/*.{test,spec}.{ts,tsx}"],
          exclude: ["src/**/*.browser.test.{ts,tsx}"],
        },
      },
      {
        plugins: [react()],
        test: {
          name: "browser",
          setupFiles: ["./src/test/setup.browser.ts"],
          include: ["src/**/*.browser.test.{ts,tsx}"],
          browser: {
            enabled: true,
            provider: playwright({ contextOptions: { locale: "zh-CN" } }),
            headless: true,
            viewport: { width: 1280, height: 800 },
            instances: [{ browser: "chromium" }],
          },
        },
      },
    ],

    // Reporting & coverage
    coverage: {
      enabled: true,
      provider: "v8", // fast, built-in V8 instrumentation
      reportsDirectory: "coverage-frontend",
      reporter: ["text", "lcov", "json"],
      // Enforced across unit and browser projects together.
      thresholds: {
        lines: 70, // 当前 81.86%，保持 70%
        functions: 40, // 当前 47.05%，设为 40%（Settings 组件函数较多）
        branches: 60, // 当前 80.76%，保持 60%
        statements: 70, // 当前 81.86%，保持 70%
      },
      // Exclude non-source or generated files from coverage calculations
      exclude: [
        "vite.config.ts",
        "vitest.config.ts",
        "src/main.tsx", // bootstrap/entry (often minimal logic)
        "src/**/*.d.ts",
        "**/*.config.*",
        "dist/**", // 构建产物
        "**/out/**", // 生成的文件
        "**/*-script.js", // 生成的脚本
        "src/types/**", // 类型定义文件（纯接口，无可执行代码）
        "src/test/**", // Test setup and render helpers, not application code
        "src/vite-env.d.ts", // Vite 类型定义
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
