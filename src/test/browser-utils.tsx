import { type ReactElement } from "react";
import { render } from "@testing-library/react";
import { I18nProvider } from "../i18n/I18nContext";
import styles from "./browser-fixture.module.css";

export function renderWithI18n(ui: ReactElement) {
  const container = document.createElement("div");
  container.className = styles.viewport;
  document.body.appendChild(container);
  return render(<I18nProvider>{ui}</I18nProvider>, { container });
}
