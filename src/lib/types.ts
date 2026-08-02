export type CaptureStatus =
  | "idle"
  | "preparing"
  | "selecting"
  | "processing"
  | "cancelling"
  | "reviewing"
  | "copied"
  | "cancelled"
  | "error";

export type TextMode = "terminal" | "document" | "singleLine";
export type CopyPolicy = "preview" | "immediate";
export type CaptureSource = "mainWindow" | "tray" | "shortcut" | "commandLine";

export interface CaptureRequest {
  jobId: string;
  mode: TextMode;
  language: "eng";
  copyPolicy: CopyPolicy;
  source: CaptureSource;
}

export interface PendingAppAction {
  action: "startCapture";
  jobId: string;
  source: Exclude<CaptureSource, "mainWindow">;
}

export interface OcrWarning {
  code: string;
  message: string;
}

export interface OcrResult {
  jobId: string;
  text: string;
  meanConfidence: number | null;
  backend: string;
  engine: string;
  preprocessingVariant: string;
  warnings: OcrWarning[];
  copied: boolean;
  elapsedMs: number;
}

export interface PublicError {
  code: string;
  message: string;
  guidance: string;
  retryable: boolean;
}

export interface AppSettings {
  schemaVersion: 1;
  language: "eng";
  textMode: TextMode;
  previewBeforeCopy: boolean;
  preserveWhitespace: boolean;
  notifyAfterCopy: boolean;
  startAtLogin: boolean;
  closeToTray: boolean;
  captureBackend: "auto" | "gnome" | "portal";
  shortcut: "Super+Shift+O";
}

export interface SettingsRecoveryWarning {
  code: string;
  message: string;
  guidance: string;
  recoveredWithDefaults: boolean;
}

export interface SettingsLoadResult {
  settings: AppSettings;
  warning: SettingsRecoveryWarning | null;
}

export interface Diagnostics {
  appVersion: string;
  osRelease: string;
  desktopEnvironment: string;
  sessionType: string;
  portalSummary: string;
  gnomeScreenshot: string;
  tesseract: string;
  installedLanguages: string[];
  clipboardStatus: string;
  trayStatus: string;
  shortcutStatus: string;
  settingsSchemaVersion: number;
  lastErrorCode: string | null;
  cleanupFailureCount: number;
}
