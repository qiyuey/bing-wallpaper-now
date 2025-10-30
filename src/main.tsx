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
