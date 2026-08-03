import { test, expect, type Page } from "@playwright/test";

// GNOME's text-scaling-factor accessibility setting shrinks the effective
// CSS viewport on real WebKitGTK (it is not a pure font-size zoom). At the
// 1.25x factor observed on the validation machine, the app's 720x580
// window behaves like a viewport this small — well under Tailwind's 640px
// `sm:` breakpoint. This is the exact condition that let a `sm:`-gated
// grid regress to one column and overflow. See docs/validation/ and the
// FIX1 status docs for the incident this guards against.
const NOMINAL_VIEWPORT = { width: 720, height: 580 };
const DPI_SCALED_VIEWPORT = { width: 576, height: 464 };

const SETTINGS = {
  schemaVersion: 1 as const,
  language: "eng" as const,
  textMode: "terminal" as const,
  previewBeforeCopy: false,
  preserveWhitespace: true,
  notifyAfterCopy: true,
  startAtLogin: false,
  closeToTray: true,
  captureBackend: "auto" as const,
  shortcut: "Super+Shift+O" as const,
};

const DIAGNOSTICS = {
  appVersion: "0.1.0",
  osRelease: "SYNTHETIC_OCR_FIXTURE_os_release_ubuntu_24_04_1_lts",
  desktopEnvironment: "SYNTHETIC_OCR_FIXTURE_desktop_gnome_46_wayland",
  sessionType: "SYNTHETIC_OCR_FIXTURE_session_wayland",
  portalSummary: "SYNTHETIC_OCR_FIXTURE_portal_org_freedesktop_portal_desktop_1_18",
  gnomeScreenshot: "SYNTHETIC_OCR_FIXTURE_gnome_screenshot_46_0_not_found",
  tesseract: "SYNTHETIC_OCR_FIXTURE_tesseract_5_3_4_eng_installed",
  installedLanguages: ["eng"],
  clipboardStatus: "available",
  trayStatus: "available",
  shortcutStatus: "registered",
  settingsSchemaVersion: 1,
  lastErrorCode: "synthetic_capture_backend_unavailable",
  cleanupFailureCount: 0,
};

const OCR_RESULT = {
  jobId: "11111111-1111-4111-8111-111111111111",
  text: Array.from(
    { length: 30 },
    (_, i) => `SYNTHETIC_OCR_FIXTURE_line_${String(i)}_of_recognized_terminal_output`,
  ).join("\n"),
  meanConfidence: 92.4,
  backend: "gnome_screenshot",
  engine: "tesseract",
  preprocessingVariant: "original",
  warnings: [
    {
      code: "clipboard_write_failed",
      message: "SYNTHETIC_OCR_FIXTURE: The recognized text could not be copied.",
    },
  ],
  copied: false,
  elapsedMs: 42,
};

/**
 * Stubs the Tauri IPC bridge (`window.__TAURI_INTERNALS__`) so the real
 * frontend renders with realistic fixture data in a plain browser — no
 * Tauri runtime is available under Playwright/webkit-less Chromium.
 */
async function installTauriMock(page: Page) {
  await page.addInitScript(
    ({ settings, diagnostics, ocrResult }) => {
      let nextCallbackId = 1;
      const win = window as unknown as {
        __TAURI_INTERNALS__: {
          invoke: (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
          transformCallback: (callback?: (...args: unknown[]) => void, once?: boolean) => number;
          unregisterCallback: (id: number) => void;
          convertFileSrc: (path: string) => string;
          metadata: {
            currentWindow: { label: string };
            currentWebview: { windowLabel: string; label: string };
          };
        };
      };
      win.__TAURI_INTERNALS__ = {
        invoke: (cmd) => {
          switch (cmd) {
            case "get_settings":
              return Promise.resolve({ settings, warning: null });
            case "get_diagnostics":
              return Promise.resolve(diagnostics);
            case "take_pending_app_action":
              return Promise.resolve(null);
            case "start_capture":
              return Promise.resolve(ocrResult);
            case "plugin:window|is_maximized":
              return Promise.resolve(false);
            case "plugin:event|listen":
              return Promise.resolve(nextCallbackId++);
            case "plugin:event|unlisten":
              return Promise.resolve(null);
            default:
              return Promise.resolve(null);
          }
        },
        transformCallback: () => nextCallbackId++,
        unregisterCallback: () => undefined,
        convertFileSrc: (path) => path,
        metadata: {
          currentWindow: { label: "main" },
          currentWebview: { windowLabel: "main", label: "main" },
        },
      };
    },
    { settings: SETTINGS, diagnostics: DIAGNOSTICS, ocrResult: OCR_RESULT },
  );
}

/** Asserts the tab content region never grows its own scrollbar. */
async function expectTabPanelFitsWithoutScroll(page: Page) {
  const panel = page.getByTestId("tab-panel");
  const { scrollHeight, clientHeight } = await panel.evaluate((el) => ({
    scrollHeight: el.scrollHeight,
    clientHeight: el.clientHeight,
  }));
  expect(scrollHeight).toBeLessThanOrEqual(clientHeight + 1);

  const documentOverflows = await page.evaluate(
    () => document.documentElement.scrollHeight > window.innerHeight + 1,
  );
  expect(documentOverflows).toBe(false);
}

test.beforeEach(async ({ page }) => {
  await installTauriMock(page);
});

for (const [label, viewport] of Object.entries({
  "nominal window size": NOMINAL_VIEWPORT,
  "GNOME 1.25x text-scaling effective viewport": DPI_SCALED_VIEWPORT,
})) {
  test.describe(`at ${label} (${String(viewport.width)}x${String(viewport.height)})`, () => {
    test.use({ viewport });

    test("Capture tab fits without scrolling", async ({ page }) => {
      await page.goto("/");
      await expect(page.getByText("Ready to capture")).toBeVisible();
      await expectTabPanelFitsWithoutScroll(page);
    });

    test("Capture tab fits with a reviewing result and warning", async ({ page }) => {
      await page.goto("/");
      await expect(page.getByText("Ready to capture")).toBeVisible();
      await page.getByRole("button", { name: "Capture text from screen" }).click();
      await expect(page.getByText("Review recognized text")).toBeVisible();
      await expectTabPanelFitsWithoutScroll(page);
    });

    test("Settings tab (General) fits without scrolling", async ({ page }) => {
      await page.goto("/");
      await expect(page.getByText("Ready to capture")).toBeVisible();
      await page.getByRole("button", { name: "Settings" }).click();
      await expect(page.getByText("OCR language")).toBeVisible();
      await expectTabPanelFitsWithoutScroll(page);
    });

    test("Settings tab (Reserved) fits without scrolling", async ({ page }) => {
      await page.goto("/");
      await expect(page.getByText("Ready to capture")).toBeVisible();
      await page.getByRole("button", { name: "Settings" }).click();
      await page.getByRole("button", { name: "Reserved" }).click();
      await expect(page.getByText("Notify after copy")).toBeVisible();
      await expectTabPanelFitsWithoutScroll(page);
    });

    test("Diagnostics tab fits without scrolling", async ({ page }) => {
      await page.goto("/");
      await expect(page.getByText("Ready to capture")).toBeVisible();
      await page.getByRole("button", { name: "Diagnostics" }).click();
      await expect(page.getByText("CLEANUP FAILURE COUNT")).toBeVisible();
      await expectTabPanelFitsWithoutScroll(page);
    });
  });
}
