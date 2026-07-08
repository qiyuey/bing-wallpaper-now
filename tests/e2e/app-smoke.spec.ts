import { expect, test } from "@playwright/test";
import { getTauriCalls, installTauriMock } from "./tauri-mock";
import { expectPageNotBlank } from "./visual";

test.describe("Bing Wallpaper Now web shell", () => {
  test.beforeEach(async ({ page }) => {
    await installTauriMock(page);
  });

  test("renders the startup screen without a blank window", async ({
    page,
  }, testInfo) => {
    const consoleErrors: string[] = [];
    const pageErrors: string[] = [];
    page.on("console", (message) => {
      if (message.type() === "error") {
        consoleErrors.push(message.text());
      }
    });
    page.on("pageerror", (error) => pageErrors.push(error.message));

    await page.goto("/");

    await expect(
      page.getByRole("heading", { name: /Bing Wallpaper\s*Now/i }),
    ).toBeVisible();
    await expect(page.getByRole("button", { name: "Refresh" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Settings" })).toBeVisible();
    await expect(page.getByText("E2E Alpine Lake")).toBeVisible();
    await expectPageNotBlank(page, testInfo);

    const calls = await getTauriCalls(page);
    expect(calls.some((call) => call.cmd === "mark_frontend_ready")).toBe(true);
    expect(calls.some((call) => call.cmd === "report_frontend_error")).toBe(
      false,
    );
    expect(pageErrors).toEqual([]);
    expect(consoleErrors).toEqual([]);
  });

  test("opens settings and renders persisted state", async ({
    page,
  }, testInfo) => {
    await page.goto("/");

    await page.getByRole("button", { name: "Settings" }).click();

    await expect(
      page.getByRole("heading", { name: /^Settings$/ }),
    ).toBeVisible();
    await expect(page.getByLabel("Launch at Startup")).toBeVisible();
    await expect(page.getByText("Wallpaper Market")).toBeVisible();
    await expect(page.getByText("2 wallpapers")).toBeVisible();
    await expectPageNotBlank(page, testInfo);

    const calls = await getTauriCalls(page);
    expect(calls.some((call) => call.cmd === "get_wallpaper_data_stats")).toBe(
      true,
    );
    expect(calls.some((call) => call.cmd === "report_frontend_error")).toBe(
      false,
    );
  });
});
