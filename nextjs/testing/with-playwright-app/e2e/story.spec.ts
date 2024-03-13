import test, { expect } from "@playwright/test";
import { entries } from "../storybook-static/index.json";

const primary = entries["example-button--primary"];
test(`snapshot test ${primary.title}: ${primary.name}`, async ({ page }) => {
  await page.goto(`http://localhost:8080/iframe.html?id=${primary.id}`, {
    waitUntil: "networkidle",
    timeout: 1000 * 10,
  });
  await expect(page).toHaveScreenshot([primary.title, `${primary.id}.png`], {
    animations: 'disabled',
    timeout: 1000 * 10,
    threshold: 0.2
  })
});
