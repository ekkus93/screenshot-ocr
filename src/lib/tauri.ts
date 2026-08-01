import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, CaptureRequest, Diagnostics, OcrResult } from "./types";

export function startCapture(request: CaptureRequest): Promise<OcrResult> {
  return invoke<OcrResult>("start_capture", { request });
}

export async function cancelCapture(jobId: string): Promise<void> {
  await invoke<null>("cancel_capture", { jobId });
}

export function takeStartupCapture(): Promise<boolean> {
  return invoke<boolean>("take_startup_capture");
}

export async function copyText(text: string): Promise<void> {
  await invoke<null>("copy_text", { text });
}

export function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export function updateSettings(settings: AppSettings): Promise<AppSettings> {
  return invoke<AppSettings>("update_settings", { settings });
}

export function resetSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("reset_settings");
}

export function getDiagnostics(): Promise<Diagnostics> {
  return invoke<Diagnostics>("get_diagnostics");
}
