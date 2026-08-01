import { render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";
import { App } from "./App";

vi.mock("../lib/tauri", () => ({
  getSettings: vi.fn().mockResolvedValue({
    schemaVersion: 1,
    language: "eng",
    textMode: "terminal",
    previewBeforeCopy: true,
    preserveWhitespace: true,
    notifyAfterCopy: true,
    startAtLogin: false,
    closeToTray: true,
    captureBackend: "auto",
    shortcut: "Super+Shift+O",
  }),
  getDiagnostics: vi.fn().mockResolvedValue({
    appVersion: "0.1.0",
    osRelease: "test",
    desktopEnvironment: "test",
    sessionType: "test",
    portalSummary: "test",
    gnomeScreenshot: "test",
    tesseract: "test",
    installedLanguages: ["eng"],
    clipboardStatus: "available",
    trayStatus: "unknown",
    settingsSchemaVersion: 1,
    lastErrorCode: null,
    cleanupFailureCount: 0,
  }),
  startCapture: vi.fn(),
  copyText: vi.fn(),
  updateSettings: vi.fn(),
  resetSettings: vi.fn(),
}));

test("renders the primary capture workflow", async () => {
  render(<App />);
  expect(screen.getByRole("button", { name: "Capture text from screen" })).toBeVisible();
  expect(screen.getByLabelText("Recognized text editor")).toBeVisible();
  expect(await screen.findByText("Ready to capture")).toBeVisible();
});

test("shows honest no-history copy", () => {
  render(<App />);
  screen.getByRole("button", { name: "History" }).click();
  expect(screen.getByText("History is off")).toBeVisible();
  expect(screen.getByText(/no hidden history database/i)).toBeVisible();
});
