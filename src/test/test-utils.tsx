import React, { ReactNode } from "react";
import { render, RenderOptions } from "@testing-library/react";
import { I18nProvider } from "../i18n/I18nContext";

// Test wrapper that provides I18nProvider with mocked settings
function TestWrapper({ children }: { children: ReactNode }) {
  // 在组件内部设置 mock，避免全局污染其他测试
  if (!window.navigator || window.navigator.language !== "zh-CN") {
    Object.defineProperty(window, "navigator", {
      writable: true,
      configurable: true,
      value: {
        ...window.navigator,
        language: "zh-CN",
      },
    });
  }
  return <I18nProvider>{children}</I18nProvider>;
}

// Custom render function that includes I18nProvider
export function renderWithI18n(
  ui: React.ReactElement,
  options?: Omit<RenderOptions, "wrapper">,
) {
  return render(ui, { wrapper: TestWrapper, ...options });
}

// Re-export everything from @testing-library/react
export * from "@testing-library/react";
