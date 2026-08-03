import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  cancelCapture,
  copyText,
  getDiagnostics,
  getSettings,
  resetSettings,
  startCapture,
  takePendingAppAction,
  updateSettings,
} from "../lib/tauri";
import type {
  AppSettings,
  CaptureStatus,
  Diagnostics,
  OcrResult,
  PendingAppAction,
  PublicError,
  SettingsRecoveryWarning,
} from "../lib/types";

const APP_ACTION_AVAILABLE_EVENT = "screenshot-ocr://app-action-available";

const DEFAULT_SETTINGS: AppSettings = {
  schemaVersion: 1,
  language: "eng",
  textMode: "terminal",
  previewBeforeCopy: false,
  preserveWhitespace: true,
  notifyAfterCopy: true,
  startAtLogin: false,
  closeToTray: true,
  captureBackend: "auto",
  shortcut: "Super+Shift+O",
};

function normalizeError(value: unknown): PublicError {
  if (typeof value === "object" && value !== null && "code" in value && "message" in value) {
    const candidate = value as Partial<PublicError>;
    return {
      code: candidate.code ?? "internal_error",
      message: candidate.message ?? "The operation failed.",
      guidance: candidate.guidance ?? "Try again or inspect diagnostics.",
      retryable: candidate.retryable ?? true,
    };
  }
  return {
    code: "internal_error",
    message: "The operation failed.",
    guidance: "Try again or inspect diagnostics.",
    retryable: true,
  };
}

function throwIfAborted(signal: AbortSignal): void {
  if (signal.aborted) {
    throw new DOMException("Startup initialization was cancelled.", "AbortError");
  }
}

function isAbortError(value: unknown): boolean {
  return value instanceof DOMException && value.name === "AbortError";
}

export function useAppController() {
  const [status, setStatus] = useState<CaptureStatus>("idle");
  const [result, setResult] = useState<OcrResult | null>(null);
  const [editorText, setEditorText] = useState("");
  const [error, setError] = useState<PublicError | null>(null);
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [settingsDirty, setSettingsDirty] = useState(false);
  const [settingsWarning, setSettingsWarning] = useState<SettingsRecoveryWarning | null>(null);
  const [diagnostics, setDiagnostics] = useState<Diagnostics | null>(null);
  const activeRequest = useRef(0);
  const activeJobId = useRef<string | null>(null);
  const initialized = useRef(false);
  const settingsRef = useRef(DEFAULT_SETTINGS);
  const drainingActions = useRef(false);

  const refreshDiagnostics = useCallback(async () => {
    try {
      setDiagnostics(await getDiagnostics());
    } catch (value) {
      setError(normalizeError(value));
    }
  }, []);

  const performCapture = useCallback(
    async (captureSettings: AppSettings, pendingAction?: PendingAppAction) => {
      const requestToken = activeRequest.current + 1;
      const jobId = pendingAction?.jobId ?? crypto.randomUUID();
      activeRequest.current = requestToken;
      activeJobId.current = jobId;
      setStatus("preparing");
      setError(null);
      try {
        setStatus("selecting");
        const nextResult = await startCapture({
          jobId,
          mode: captureSettings.textMode,
          language: captureSettings.language,
          copyPolicy: captureSettings.previewBeforeCopy ? "preview" : "immediate",
          source: pendingAction?.source ?? "mainWindow",
        });
        if (requestToken !== activeRequest.current) return;
        setResult(nextResult);
        setEditorText(nextResult.text);
        setStatus(nextResult.copied ? "copied" : "reviewing");
      } catch (value) {
        if (requestToken !== activeRequest.current) return;
        const nextError = normalizeError(value);
        if (nextError.code === "capture_cancelled") {
          setError(null);
          setStatus("cancelled");
        } else {
          setError(nextError);
          setStatus("error");
        }
      } finally {
        if (activeJobId.current === jobId) {
          activeJobId.current = null;
        }
      }
    },
    [],
  );

  const drainPendingActions = useCallback(async () => {
    if (!initialized.current || drainingActions.current) return;
    drainingActions.current = true;
    try {
      const pendingAction = await takePendingAppAction();
      if (pendingAction !== null) {
        await performCapture(settingsRef.current, pendingAction);
      }
    } catch (value) {
      setError(normalizeError(value));
    } finally {
      drainingActions.current = false;
    }
  }, [performCapture]);

  const capture = useCallback(async () => {
    await performCapture(settings);
  }, [performCapture, settings]);

  const cancel = useCallback(async () => {
    const jobId = activeJobId.current;
    if (jobId === null) return;
    setStatus("cancelling");
    setError(null);
    try {
      await cancelCapture(jobId);
    } catch (value) {
      if (activeJobId.current === jobId) {
        setError(normalizeError(value));
        setStatus("error");
      }
    }
  }, []);

  useEffect(() => {
    const controller = new AbortController();
    const { signal } = controller;
    let unlisten: UnlistenFn | undefined;

    void (async () => {
      try {
        unlisten = await listen(APP_ACTION_AVAILABLE_EVENT, () => {
          void drainPendingActions();
        });
        throwIfAborted(signal);

        // Independent requests — fetch concurrently so one slow round trip
        // doesn't serialize behind the other and double startup latency.
        const [loadedSettings, loadedDiagnostics] = await Promise.all([
          getSettings(),
          getDiagnostics(),
        ]);
        throwIfAborted(signal);
        settingsRef.current = loadedSettings.settings;
        setSettings(loadedSettings.settings);
        setSettingsWarning(loadedSettings.warning);
        setSettingsDirty(false);
        setDiagnostics(loadedDiagnostics);

        initialized.current = true;
        await drainPendingActions();
      } catch (value) {
        if (!isAbortError(value)) {
          setError(normalizeError(value));
        }
      }
    })();

    return () => {
      controller.abort();
      initialized.current = false;
      unlisten?.();
    };
  }, [drainPendingActions]);

  const copy = useCallback(async () => {
    if (editorText.trim().length === 0) {
      setError({
        code: "ocr_empty_result",
        message: "There is no text to copy.",
        guidance: "Capture a text region or enter text in the preview first.",
        retryable: true,
      });
      return;
    }
    try {
      await copyText(editorText);
      setStatus("copied");
      setError(null);
    } catch (value) {
      setError(normalizeError(value));
      setStatus("error");
    }
  }, [editorText]);

  const saveSettings = useCallback(async () => {
    try {
      const saved = await updateSettings(settings);
      settingsRef.current = saved;
      setSettings(saved);
      setSettingsDirty(false);
      setSettingsWarning(null);
      setError(null);
    } catch (value) {
      setError(normalizeError(value));
    }
  }, [settings]);

  const restoreSettings = useCallback(async () => {
    try {
      const restored = await resetSettings();
      settingsRef.current = restored;
      setSettings(restored);
      setSettingsDirty(false);
      setSettingsWarning(null);
      setError(null);
    } catch (value) {
      setError(normalizeError(value));
    }
  }, []);

  return {
    status,
    result,
    editorText,
    error,
    settings,
    settingsDirty,
    settingsWarning,
    diagnostics,
    capture,
    cancel,
    copy,
    saveSettings,
    restoreSettings,
    refreshDiagnostics,
    setEditorText,
    clear: () => {
      if (activeJobId.current !== null) return;
      activeRequest.current += 1;
      setResult(null);
      setEditorText("");
      setError(null);
      setStatus("idle");
    },
    changeSettings: (next: AppSettings) => {
      settingsRef.current = next;
      setSettings(next);
      setSettingsDirty(true);
    },
  };
}
