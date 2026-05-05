import { expect, test } from "@playwright/test";
import { readFile } from "node:fs/promises";
import path from "node:path";

test("이미지를 업로드하고 WebP 결과를 다운로드한다", async ({ page }) => {
  const inputPath = path.join(process.cwd(), "src/app/favicon.ico");

  await page.goto("/");
  await page.locator('input[type="file"]').setInputFiles(inputPath);

  await expect(page.getByText("favicon.ico").first()).toBeVisible();
  await page.getByRole("button", { name: "변환하기" }).click();

  const downloadLink = page.locator('a[download="favicon.webp"]').first();
  await expect(downloadLink).toBeVisible({ timeout: 120_000 });

  const downloadPromise = page.waitForEvent("download");
  await downloadLink.click();
  const download = await downloadPromise;
  const outputPath = await download.path();

  expect(download.suggestedFilename()).toBe("favicon.webp");
  expect(outputPath).not.toBeNull();

  const bytes = await readFile(outputPath as string);
  expect(bytes.subarray(0, 4).toString("ascii")).toBe("RIFF");
  expect(bytes.subarray(8, 12).toString("ascii")).toBe("WEBP");
});
