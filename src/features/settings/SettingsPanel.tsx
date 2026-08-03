import { useState } from "react";
import type { AppController } from "../../app/controllerTypes";
import type { AppSettings } from "../../lib/types";

interface Props {
  controller: AppController;
}

type Section = "general" | "reserved";

function ReservedCheckbox({ checked, label }: { checked: boolean; label: string }) {
  return (
    <label className="flex items-start gap-2 text-steel-500 dark:text-steel-400">
      <input type="checkbox" checked={checked} disabled className="mt-0.5" />
      <span className="text-sm">
        <span className="font-medium text-steel-600 dark:text-steel-400">{label}</span>{" "}
        <span className="text-xs">— Not implemented in this pre-release build.</span>
      </span>
    </label>
  );
}

export function SettingsPanel({ controller }: Props) {
  const [section, setSection] = useState<Section>("general");
  const patch = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    controller.changeSettings({ ...controller.settings, [key]: value });
  };

  return (
    <div className="space-y-1">
      {controller.settingsWarning !== null && (
        <div
          role="alert"
          className="rounded-lg border border-phosphor-500/40 bg-phosphor-500/10 p-1.5"
        >
          <p className="font-mono text-sm font-semibold text-phosphor-700 dark:text-phosphor-500">
            {controller.settingsWarning.message}
          </p>
          <p className="text-xs text-steel-600 dark:text-steel-400">
            {controller.settingsWarning.guidance}{" "}
            <code className="font-mono">[ {controller.settingsWarning.code} ]</code>
          </p>
        </div>
      )}

      {controller.error !== null && (
        <div role="alert" className="rounded-lg border border-alert-500/40 bg-alert-500/5 p-1.5">
          <p className="font-mono text-sm font-semibold text-alert-700 dark:text-alert-500">
            {controller.error.message}
          </p>
          <p className="text-xs text-steel-600 dark:text-steel-400">
            {controller.error.guidance}{" "}
            <code className="font-mono">[ {controller.error.code} ]</code>
          </p>
        </div>
      )}

      <div className="flex gap-1 border-b border-steel-200 dark:border-ink-700">
        {(
          [
            { id: "general", label: "General" },
            { id: "reserved", label: "Reserved" },
          ] as const
        ).map((item) => (
          <button
            key={item.id}
            type="button"
            className={`-mb-px border-b-2 px-2 py-0.5 font-mono text-xs tracking-wide ${
              section === item.id
                ? "border-phosphor-500 text-ink-950 dark:text-paper-50"
                : "border-transparent text-steel-500 hover:text-ink-950 dark:text-steel-400 dark:hover:text-paper-50"
            }`}
            onClick={() => {
              setSection(item.id);
            }}
          >
            {item.label}
          </button>
        ))}
      </div>

      {section === "general" && (
        <>
          <div className="grid grid-cols-2 gap-1 rounded-lg border border-steel-200 bg-white p-1.5 dark:border-ink-700 dark:bg-ink-900">
            <label className="space-y-1">
              <span className="text-sm font-medium">OCR language</span>
              <select
                value={controller.settings.language}
                onChange={(event) => {
                  patch("language", event.target.value as "eng");
                }}
                className="w-full rounded-md border border-steel-200 bg-transparent px-2 py-1 focus-visible:outline focus-visible:outline-2 focus-visible:outline-phosphor-500 dark:border-ink-700"
              >
                <option value="eng">English</option>
              </select>
            </label>
            <label className="space-y-1">
              <span className="text-sm font-medium">Text mode</span>
              <select
                value={controller.settings.textMode}
                onChange={(event) => {
                  patch("textMode", event.target.value as AppSettings["textMode"]);
                }}
                className="w-full rounded-md border border-steel-200 bg-transparent px-2 py-1 focus-visible:outline focus-visible:outline-2 focus-visible:outline-phosphor-500 dark:border-ink-700"
              >
                <option value="terminal">Terminal and source code</option>
                <option value="document">Normal document</option>
                <option value="singleLine">Single line</option>
              </select>
            </label>
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={controller.settings.previewBeforeCopy}
                onChange={(event) => {
                  patch("previewBeforeCopy", event.target.checked);
                }}
              />
              Preview before copying
            </label>
            <label className="space-y-1">
              <span className="text-sm font-medium">Capture backend</span>
              <select
                value={controller.settings.captureBackend}
                onChange={(event) => {
                  patch("captureBackend", event.target.value as AppSettings["captureBackend"]);
                }}
                className="w-full rounded-md border border-steel-200 bg-transparent px-2 py-1 focus-visible:outline focus-visible:outline-2 focus-visible:outline-phosphor-500 dark:border-ink-700"
              >
                <option value="auto">Auto</option>
                <option value="gnome">GNOME screenshot helper</option>
                <option value="portal">Portal (capability required)</option>
              </select>
            </label>
          </div>

          <p className="text-sm text-steel-600 dark:text-steel-400">
            GNOME shortcut: assign{" "}
            <kbd className="rounded border border-steel-200 px-1.5 py-0.5 font-mono text-xs dark:border-ink-700">
              Super+Shift+O
            </kbd>{" "}
            to <code className="font-mono">screenshot-ocr capture</code>.
          </p>

          <p className="font-mono text-xs text-steel-500 dark:text-steel-600">
            Local processing only — no network calls.
          </p>

          <div className="flex justify-end gap-2">
            <button
              type="button"
              className="rounded-md border border-steel-200 px-3 py-1.5 font-mono text-sm dark:border-ink-700"
              onClick={() => {
                void controller.restoreSettings();
              }}
            >
              Reset settings
            </button>
            <button
              type="button"
              disabled={!controller.settingsDirty}
              className="rounded-md bg-phosphor-500 px-3 py-1.5 font-mono text-sm font-semibold text-ink-950 hover:bg-phosphor-600 disabled:opacity-50"
              onClick={() => {
                void controller.saveSettings();
              }}
            >
              Save settings
            </button>
          </div>
        </>
      )}

      {section === "reserved" && (
        <div className="space-y-2 rounded-lg border border-steel-200 bg-white p-2 dark:border-ink-700 dark:bg-ink-900">
          <label className="flex items-start gap-2 text-steel-500 dark:text-steel-400">
            <input
              type="checkbox"
              checked={controller.settings.preserveWhitespace}
              disabled
              className="mt-0.5"
            />
            <span className="text-sm">
              <span className="font-medium text-steel-600 dark:text-steel-400">
                Preserve indentation and blank lines
              </span>{" "}
              <span className="text-xs">— always on in this pre-release build.</span>
            </span>
          </label>
          <ReservedCheckbox
            checked={controller.settings.notifyAfterCopy}
            label="Notify after copy"
          />
          <ReservedCheckbox checked={controller.settings.startAtLogin} label="Start at login" />
          <ReservedCheckbox
            checked={controller.settings.closeToTray}
            label="Keep running when window closes"
          />
        </div>
      )}
    </div>
  );
}
