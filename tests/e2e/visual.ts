import { expect, Page, TestInfo } from "@playwright/test";
import { PNG } from "pngjs";

export async function expectPageNotBlank(page: Page, testInfo: TestInfo) {
  const screenshot = await page.screenshot({ fullPage: false });
  await testInfo.attach("non-blank-check.png", {
    body: screenshot,
    contentType: "image/png",
  });

  const png = PNG.sync.read(screenshot);
  const sampleStep = Math.max(1, Math.floor((png.width * png.height) / 20_000));
  let sampled = 0;
  let nonWhite = 0;

  for (let pixel = 0; pixel < png.width * png.height; pixel += sampleStep) {
    const offset = pixel * 4;
    const alpha = png.data[offset + 3] ?? 255;
    if (alpha === 0) continue;

    sampled += 1;
    const red = png.data[offset] ?? 255;
    const green = png.data[offset + 1] ?? 255;
    const blue = png.data[offset + 2] ?? 255;

    if (!(red >= 248 && green >= 248 && blue >= 248)) {
      nonWhite += 1;
    }
  }

  expect(sampled, "screenshot should contain sampled pixels").toBeGreaterThan(
    0,
  );
  expect(
    nonWhite / sampled,
    "screenshot should not be blank or all-white",
  ).toBeGreaterThan(0.02);
}
