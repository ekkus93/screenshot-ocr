import { useEffect, useState } from "react";
import { CapturePanel } from "../features/capture/CapturePanel";
import { DiagnosticsPanel } from "../features/diagnostics/DiagnosticsPanel";
import { SettingsPanel } from "../features/settings/SettingsPanel";
import { useAppController } from "./useAppController";
import { useTheme } from "./useTheme";
import { useWindowControls } from "./useWindowControls";
import type { CaptureStatus } from "../lib/types";

type Tab = "capture" | "history" | "settings" | "diagnostics";

const TABS: { id: Tab; label: string }[] = [
  { id: "capture", label: "Capture" },
  { id: "history", label: "History" },
  { id: "settings", label: "Settings" },
  { id: "diagnostics", label: "Diagnostics" },
];

const STATUS_CHIP: Record<CaptureStatus, { word: string; tone: string }> = {
  idle: { word: "READY", tone: "text-steel-600 dark:text-steel-400" },
  preparing: { word: "PREPARING", tone: "text-phosphor-700 dark:text-phosphor-500" },
  selecting: { word: "SELECTING", tone: "text-phosphor-700 dark:text-phosphor-500" },
  processing: { word: "RECOGNIZING", tone: "text-phosphor-700 dark:text-phosphor-500" },
  cancelling: { word: "CANCELLING", tone: "text-phosphor-700 dark:text-phosphor-500" },
  reviewing: { word: "REVIEW", tone: "text-signal-700 dark:text-signal-500" },
  copied: { word: "COPIED", tone: "text-signal-700 dark:text-signal-500" },
  cancelled: { word: "CANCELLED", tone: "text-steel-600 dark:text-steel-400" },
  error: { word: "ERROR", tone: "text-alert-700 dark:text-alert-500" },
};

export function App() {
  const [tab, setTab] = useState<Tab>("capture");
  const controller = useAppController();
  const { theme, toggleTheme } = useTheme();
  const { isMaximized, minimize, toggleMaximize, close } = useWindowControls();
  const chip = STATUS_CHIP[controller.status];

  useEffect(() => {
    if (controller.status === "preparing") {
      setTab("capture");
    }
  }, [controller.status]);

  return (
    <main className="flex h-screen flex-col overflow-hidden bg-paper-50 p-5 font-sans text-ink-950 dark:bg-ink-950 dark:text-paper-50">
      <div className="mx-auto flex w-full max-w-3xl flex-1 flex-col overflow-hidden">
        <header
          data-tauri-drag-region
          className="mb-3 flex shrink-0 items-center justify-between gap-4 border-b border-steel-200 pb-3 dark:border-ink-700"
        >
          <span className="font-mono text-sm font-semibold tracking-[0.2em] text-steel-600 dark:text-steel-400">
            SCREENSHOT<span className="text-phosphor-600 dark:text-phosphor-500">·</span>OCR
          </span>
          <div className="flex items-center gap-3">
            <span
              aria-live="polite"
              className={`font-mono text-xs font-semibold tracking-widest ${chip.tone}`}
            >
              [ {chip.word} ]
            </span>
            <button
              type="button"
              onClick={toggleTheme}
              aria-pressed={theme === "dark"}
              className="rounded-md border border-steel-200 px-2 py-1 font-mono text-xs tracking-widest text-steel-600 hover:bg-paper-100 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-phosphor-500 dark:border-ink-700 dark:text-steel-400 dark:hover:bg-ink-800"
            >
              {theme === "dark" ? "Switch to light" : "Switch to dark"}
            </button>
            <div className="flex items-center gap-1 border-l border-steel-200 pl-3 dark:border-ink-700">
              <button
                type="button"
                onClick={minimize}
                aria-label="Minimize window"
                className="flex h-6 w-6 items-center justify-center rounded font-mono text-sm text-steel-500 hover:bg-paper-100 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-phosphor-500 dark:text-steel-400 dark:hover:bg-ink-800"
              >
                &#95;
              </button>
              <button
                type="button"
                onClick={toggleMaximize}
                aria-label={isMaximized ? "Restore window" : "Maximize window"}
                className="flex h-6 w-6 items-center justify-center rounded font-mono text-xs text-steel-500 hover:bg-paper-100 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-phosphor-500 dark:text-steel-400 dark:hover:bg-ink-800"
              >
                {isMaximized ? "❐" : "□"}
              </button>
              <button
                type="button"
                onClick={close}
                aria-label="Close window"
                className="flex h-6 w-6 items-center justify-center rounded font-mono text-sm text-steel-500 hover:bg-alert-500 hover:text-white focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-phosphor-500 dark:text-steel-400"
              >
                &#215;
              </button>
            </div>
          </div>
        </header>

        <nav
          aria-label="Application sections"
          className="mb-3 flex shrink-0 gap-1 border-b border-steel-200 dark:border-ink-700"
        >
          {TABS.map((item) => (
            <button
              key={item.id}
              type="button"
              className={`-mb-px border-b-2 px-3 py-1.5 font-mono text-sm tracking-wide focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-phosphor-500 ${
                tab === item.id
                  ? "border-phosphor-500 text-ink-950 dark:text-paper-50"
                  : "border-transparent text-steel-500 hover:text-ink-950 dark:text-steel-400 dark:hover:text-paper-50"
              }`}
              onClick={() => {
                setTab(item.id);
              }}
            >
              {item.label}
            </button>
          ))}
        </nav>

        <section
          data-testid="tab-panel"
          aria-live="polite"
          className="min-h-0 flex-1 overflow-y-auto"
        >
          {tab === "capture" && <CapturePanel controller={controller} />}
          {tab === "history" && (
            <div className="rounded-lg border border-steel-200 bg-white p-6 dark:border-ink-700 dark:bg-ink-900">
              <h1 className="font-mono text-lg font-semibold">History is off</h1>
              <p className="mt-2 text-steel-600 dark:text-steel-400">
                Screenshot OCR v0.1 does not retain captures or recognized text, and no hidden
                history database is created.
              </p>
            </div>
          )}
          {tab === "settings" && <SettingsPanel controller={controller} />}
          {tab === "diagnostics" && <DiagnosticsPanel controller={controller} />}
        </section>
      </div>
    </main>
  );
}
