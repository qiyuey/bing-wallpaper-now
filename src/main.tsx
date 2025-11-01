// 扩展 Window 接口以支持 React DevTools 钩子
interface ReactDevToolsHook {
  renderers?: Map<number, unknown>;
  supportsFiber?: boolean;
  onCommitFiberRoot?: ((...args: unknown[]) => void) | null;
  inject?: (renderer: unknown) => number;
  rendererInterfaces?: Map<number, unknown>;
}

declare global {
  interface Window {
    __REACT_DEVTOOLS_GLOBAL_HOOK__?: ReactDevToolsHook;
  }
}

// 在开发环境中，在 React 加载前确保全局钩子是干净的
// 检查并清除可能被 shim 的钩子，让 React Refresh 能够正确初始化
if (import.meta.env.DEV) {
  const hookKey = "__REACT_DEVTOOLS_GLOBAL_HOOK__";
  const existingHook = window[hookKey];

  if (existingHook && typeof existingHook === "object") {
    // 检查钩子是否被 Proxy 包装（通过检查属性描述符）
    let isProxy = false;
    try {
      const descriptor = Object.getOwnPropertyDescriptor(window, hookKey);
      if (descriptor && (descriptor.get || descriptor.set)) {
        isProxy = true;
      }
    } catch {
      // 忽略错误
    }

    // 检查钩子是否缺少关键属性
    const hasCriticalProps =
      existingHook.renderers !== undefined ||
      typeof existingHook.supportsFiber === "function" ||
      typeof existingHook.onCommitFiberRoot === "function" ||
      existingHook.inject !== undefined ||
      existingHook.rendererInterfaces !== undefined;

    // 如果钩子被代理或缺少关键属性，清除它
    if (isProxy || !hasCriticalProps) {
      try {
        // 如果使用了 getter/setter，先重新定义属性
        const descriptor = Object.getOwnPropertyDescriptor(window, hookKey);
        if (descriptor && (descriptor.get || descriptor.set)) {
          Object.defineProperty(window, hookKey, {
            configurable: true,
            enumerable: true,
            writable: true,
            value: undefined,
          });
        }
        delete window[hookKey];
      } catch {
        // 忽略错误，继续执行
      }
    }
  }
}

import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ThemeProvider } from "./contexts/ThemeContext";
import { I18nProvider } from "./i18n/I18nContext";
import "./theme.css";

// Suppress Tauri listener errors caused by React StrictMode double-mounting
// These errors are harmless and only occur in development mode
window.addEventListener("unhandledrejection", (event) => {
  const errorMessage = event.reason?.message || String(event.reason);
  if (
    errorMessage.includes("listeners") &&
    errorMessage.includes("handlerId")
  ) {
    event.preventDefault(); // Prevent error from being displayed
  }
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider>
      <I18nProvider>
        <App />
      </I18nProvider>
    </ThemeProvider>
  </React.StrictMode>,
);
