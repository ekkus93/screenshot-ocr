export type CaptureStatus =
  | "idle"
  | "preparing"
  | "selecting"
  | "processing"
  | "reviewing"
  | "copied"
  | "cancelled"
  | "error";

export type TextMode = "terminal" | "document" | "singleLine";
export type CopyPolicy = "preview" | "immediate";

export interface CaptureRequest {
  mode: TextMode;
  language: "eng";
  copyPolicy: CopyPolicy;
  source: "mainWindow" | "tray" | "shortcut" | "commandLine";
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
  settingsSchemaVersion: number;
  lastErrorCode: string | null;
  cleanupFailureCount: number;
}
