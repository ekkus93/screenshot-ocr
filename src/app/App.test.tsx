import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, test, vi } from "vitest";
import { App } from "./App";

const mocks = vi.hoisted(() => ({
  listen: vi.fn(),
  getSettings: vi.fn(),
  getDiagnostics: vi.fn(),
  takePendingAppAction: vi.fn(),
  startCapture: vi.fn(),
  cancelCapture: vi.fn(),
  copyText: vi.fn(),
  updateSettings: vi.fn(),
  resetSettings: vi.fn(),
}));

const settings = {
  schemaVersion: 1 as const,
  language: "eng" as const,
  textMode: "terminal" as const,
  previewBeforeCopy: true,
  preserveWhitespace: true,
  notifyAfterCopy: true,
  startAtLogin: false,
  closeToTray: true,
  captureBackend: "auto" as const,
  shortcut: "Super+Shift+O" as const,
};

const diagnostics = {
  appVersion: "0.1.0",
  osRelease: "test",
  desktopEnvironment: "test",
  sessionType: "test",
  portalSummary: "test",
  gnomeScreenshot: "test",
  tesseract: "test",
  installedLanguages: ["eng"],
  clipboardStatus: "available",
  trayStatus: "available",
  shortcutStatus: "registered",
  settingsSchemaVersion: 1,
  lastErrorCode: null,
  cleanupFailureCount: 0,
};

vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("../lib/tauri", () => ({
  getSettings: mocks.getSettings,
  getDiagnostics: mocks.getDiagnostics,
  takePendingAppAction: mocks.takePendingAppAction,
  startCapture: mocks.startCapture,
  cancelCapture: mocks.cancelCapture,
  copyText: mocks.copyText,
  updateSettings: mocks.updateSettings,
  resetSettings: mocks.resetSettings,
}));

beforeEach(() => {
  vi.clearAllMocks();
  mocks.listen.mockResolvedValue(vi.fn());
  mocks.getSettings.mockResolvedValue(settings);
  mocks.getDiagnostics.mockResolvedValue(diagnostics);
  mocks.takePendingAppAction.mockResolvedValue(null);
});

test("renders the primary capture workflow", async () => {
  render(<App />);
  expect(screen.getByRole("button", { name: "Capture text from screen" })).toBeVisible();
  expect(screen.getByLabelText("Recognized text editor")).toBeVisible();
  expect(await screen.findByText("Ready to capture")).toBeVisible();
});

test("shows honest no-history copy", async () => {
  const user = userEvent.setup();
  render(<App />);
  await screen.findByText("Ready to capture");

  await user.click(screen.getByRole("button", { name: /history/i }));

  expect(await screen.findByText("History is off")).toBeVisible();
  expect(screen.getByText(/no hidden history database/i)).toBeVisible();
});

test("routes a reserved shortcut action through the normal capture command", async () => {
  mocks.takePendingAppAction.mockResolvedValueOnce({
    action: "startCapture",
    jobId: "11111111-1111-4111-8111-111111111111",
    source: "shortcut",
  });
  mocks.startCapture.mockResolvedValue({
    jobId: "11111111-1111-4111-8111-111111111111",
    text: "cargo test\n",
    meanConfidence: null,
    backend: "gnome_screenshot",
    engine: "tesseract",
    preprocessingVariant: "original",
    warnings: [],
    copied: false,
    elapsedMs: 10,
  });

  render(<App />);

  await waitFor(() => {
    expect(mocks.startCapture).toHaveBeenCalledWith({
      jobId: "11111111-1111-4111-8111-111111111111",
      mode: "terminal",
      language: "eng",
      copyPolicy: "preview",
      source: "shortcut",
    });
  });
  expect(await screen.findByDisplayValue("cargo test\n")).toBeVisible();
});
