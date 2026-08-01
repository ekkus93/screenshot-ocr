import { useCallback, useEffect, useRef, useState } from "react";
import {
  copyText,
  getDiagnostics,
  getSettings,
  resetSettings,
  startCapture,
  updateSettings,
} from "../lib/tauri";
import type { AppSettings, CaptureStatus, Diagnostics, OcrResult, PublicError } from "../lib/types";

const DEFAULT_SETTINGS: AppSettings = {
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

export function useAppController() {
  const [status, setStatus] = useState<CaptureStatus>("idle");
  const [result, setResult] = useState<OcrResult | null>(null);
  const [editorText, setEditorText] = useState("");
  const [error, setError] = useState<PublicError | null>(null);
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [settingsDirty, setSettingsDirty] = useState(false);
  const [diagnostics, setDiagnostics] = useState<Diagnostics | null>(null);
  const activeRequest = useRef(0);

  const refreshSettings = useCallback(async () => {
    try {
      const loaded = await getSettings();
      setSettings(loaded);
      setSettingsDirty(false);
    } catch (value) {
      setError(normalizeError(value));
    }
  }, []);

  const refreshDiagnostics = useCallback(async () => {
    try {
      setDiagnostics(await getDiagnostics());
    } catch (value) {
      setError(normalizeError(value));
    }
  }, []);

  useEffect(() => {
    void refreshSettings();
    void refreshDiagnostics();
  }, [refreshDiagnostics, refreshSettings]);

  const capture = useCallback(async () => {
    const requestToken = activeRequest.current + 1;
    activeRequest.current = requestToken;
    setStatus("preparing");
    setError(null);
    try {
      setStatus("selecting");
      const nextResult = await startCapture({
        mode: settings.textMode,
        language: settings.language,
        copyPolicy: settings.previewBeforeCopy ? "preview" : "immediate",
        source: "mainWindow",
      });
      if (requestToken !== activeRequest.current) return;
      setResult(nextResult);
      setEditorText(nextResult.text);
      setStatus(nextResult.copied ? "copied" : "reviewing");
    } catch (value) {
      if (requestToken !== activeRequest.current) return;
      const nextError = normalizeError(value);
      if (nextError.code === "capture_cancelled") {
        setStatus("cancelled");
      } else {
        setError(nextError);
        setStatus("error");
      }
    }
  }, [settings]);

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
      setSettings(saved);
      setSettingsDirty(false);
      setError(null);
    } catch (value) {
      setError(normalizeError(value));
    }
  }, [settings]);

  const restoreSettings = useCallback(async () => {
    try {
      const restored = await resetSettings();
      setSettings(restored);
      setSettingsDirty(false);
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
    diagnostics,
    capture,
    copy,
    saveSettings,
    restoreSettings,
    refreshDiagnostics,
    setEditorText,
    clear: () => {
      activeRequest.current += 1;
      setResult(null);
      setEditorText("");
      setError(null);
      setStatus("idle");
    },
    changeSettings: (next: AppSettings) => {
      setSettings(next);
      setSettingsDirty(true);
    },
  };
}
