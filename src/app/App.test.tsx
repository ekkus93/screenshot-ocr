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
  mocks.getSettings.mockResolvedValue({ settings, warning: null });
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
  await waitFor(() => {
    expect(screen.getByLabelText("Recognized text editor")).toHaveValue("cargo test\n");
  });
});

test("immediate-copy clipboard failure keeps recognized text available for retry", async () => {
  const user = userEvent.setup();
  mocks.getSettings.mockResolvedValueOnce({
    settings: { ...settings, previewBeforeCopy: false },
    warning: null,
  });
  mocks.startCapture.mockResolvedValue({
    jobId: "22222222-2222-4222-8222-222222222222",
    text: "cargo test --locked\n",
    meanConfidence: null,
    backend: "gnome_screenshot",
    engine: "tesseract",
    preprocessingVariant: "original",
    warnings: [
      {
        code: "clipboard_write_failed",
        message: "The recognized text could not be copied. Review it here and retry copy.",
      },
    ],
    copied: false,
    elapsedMs: 12,
  });

  render(<App />);
  await screen.findByText("Ready to capture");

  await user.click(screen.getByRole("button", { name: "Capture text from screen" }));

  await waitFor(() => {
    expect(mocks.startCapture).toHaveBeenCalledWith(
      expect.objectContaining({ copyPolicy: "immediate" }),
    );
  });
  expect(await screen.findByText("Review recognized text")).toBeVisible();
  expect(screen.getByLabelText("Recognized text editor")).toHaveValue("cargo test --locked\n");
  expect(screen.getByText(/could not be copied/i)).toBeVisible();
  expect(screen.queryByText("Text copied")).not.toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "Copy text" }));
  expect(mocks.copyText).toHaveBeenCalledWith("cargo test --locked\n");
});

test("shows settings save failures on the settings tab without discarding edits", async () => {
  const user = userEvent.setup();
  mocks.updateSettings.mockRejectedValueOnce({
    code: "settings_write_failed",
    message: "Settings could not be saved.",
    guidance: "Check configuration-directory permissions and try again.",
    retryable: true,
  });
  render(<App />);
  await screen.findByText("Ready to capture");

  await user.click(screen.getByRole("button", { name: /settings/i }));
  await user.selectOptions(screen.getByLabelText("Text mode"), "document");
  await user.click(screen.getByRole("button", { name: "Save settings" }));

  expect(await screen.findByText("Settings could not be saved.")).toBeVisible();
  expect(screen.getByLabelText("Text mode")).toHaveValue("document");
});

test("shows corrupt settings recovery as a safe settings warning", async () => {
  const user = userEvent.setup();
  mocks.getSettings.mockResolvedValueOnce({
    settings,
    warning: {
      code: "settings_invalid_recovered",
      message: "Settings could not be loaded, so safe defaults were used.",
      guidance: "Review the settings and save them to replace the invalid configuration.",
      recoveredWithDefaults: true,
    },
  });
  render(<App />);
  await screen.findByText("Ready to capture");

  await user.click(screen.getByRole("button", { name: /settings/i }));

  expect(
    await screen.findByText("Settings could not be loaded, so safe defaults were used."),
  ).toBeVisible();
  expect(screen.queryByText("SYNTHETIC_SECRET_9f33")).not.toBeInTheDocument();
});

test("reserved settings controls are visible but not active", async () => {
  const user = userEvent.setup();
  render(<App />);
  await screen.findByText("Ready to capture");

  await user.click(screen.getByRole("button", { name: /settings/i }));

  expect(screen.getByLabelText(/Notify after copy/)).toBeDisabled();
  expect(screen.getByLabelText(/Start at login/)).toBeDisabled();
  expect(screen.getByLabelText(/Keep running when window closes/)).toBeDisabled();
  expect(screen.getAllByText(/Not implemented in this pre-release build/).length).toBeGreaterThan(
    0,
  );
});
