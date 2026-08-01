import { useState } from "react";
import { CapturePanel } from "../features/capture/CapturePanel";
import { DiagnosticsPanel } from "../features/diagnostics/DiagnosticsPanel";
import { SettingsPanel } from "../features/settings/SettingsPanel";
import { useAppController } from "./useAppController";

type Tab = "capture" | "history" | "settings" | "diagnostics";

export function App() {
  const [tab, setTab] = useState<Tab>("capture");
  const controller = useAppController();

  return (
    <main className="min-h-screen bg-slate-100 p-4 text-slate-950 dark:bg-slate-950 dark:text-slate-100 sm:p-6">
      <div className="mx-auto grid max-w-5xl gap-5 md:grid-cols-[12rem_1fr]">
        <nav
          aria-label="Application sections"
          className="rounded-2xl border border-slate-200 bg-white p-2 dark:border-slate-800 dark:bg-slate-900"
        >
          <div className="grid grid-cols-2 gap-2 md:grid-cols-1">
            {(["capture", "history", "settings", "diagnostics"] as const).map((item) => (
              <button
                key={item}
                type="button"
                className={`rounded-xl px-3 py-2 text-left capitalize focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-500 ${
                  tab === item
                    ? "bg-indigo-100 font-medium text-indigo-900 dark:bg-indigo-950 dark:text-indigo-100"
                    : "hover:bg-slate-100 dark:hover:bg-slate-800"
                }`}
                onClick={() => {
                  setTab(item);
                }}
              >
                {item}
              </button>
            ))}
          </div>
          <p className="mt-4 hidden px-3 text-sm text-slate-500 md:block">Local processing only</p>
        </nav>

        <section className="min-w-0" aria-live="polite">
          {tab === "capture" && <CapturePanel controller={controller} />}
          {tab === "history" && (
            <div className="rounded-2xl border border-slate-200 bg-white p-6 dark:border-slate-800 dark:bg-slate-900">
              <h1 className="text-xl font-semibold">History is off</h1>
              <p className="mt-2 text-slate-600 dark:text-slate-300">
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
