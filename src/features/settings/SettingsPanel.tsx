import type { AppSettings } from "../../lib/types";
import type { ReturnTypeOfController } from "../../test/types";

interface Props {
  controller: ReturnTypeOfController;
}

export function SettingsPanel({ controller }: Props) {
  const patch = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    controller.changeSettings({ ...controller.settings, [key]: value });
  };

  return (
    <div className="space-y-5">
      <header>
        <h1 className="text-xl font-semibold">Settings</h1>
        <p className="mt-1 text-sm text-slate-600 dark:text-slate-300">
          Defaults are optimized for terminals and source code.
        </p>
      </header>
      <div className="grid gap-5 rounded-2xl border border-slate-200 bg-white p-5 dark:border-slate-800 dark:bg-slate-900 sm:grid-cols-2">
        <label className="space-y-2">
          <span className="font-medium">OCR language</span>
          <select
            value={controller.settings.language}
            onChange={(event) => patch("language", event.target.value as "eng")}
            className="w-full rounded-xl border border-slate-300 bg-transparent px-3 py-2 dark:border-slate-700"
          >
            <option value="eng">English</option>
          </select>
        </label>
        <label className="space-y-2">
          <span className="font-medium">Text mode</span>
          <select
            value={controller.settings.textMode}
            onChange={(event) => patch("textMode", event.target.value as AppSettings["textMode"])}
            className="w-full rounded-xl border border-slate-300 bg-transparent px-3 py-2 dark:border-slate-700"
          >
            <option value="terminal">Terminal and source code</option>
            <option value="document">Normal document</option>
            <option value="singleLine">Single line</option>
          </select>
        </label>
        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={controller.settings.previewBeforeCopy}
            onChange={(event) => patch("previewBeforeCopy", event.target.checked)}
          />{" "}
          Preview before copying
        </label>
        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={controller.settings.preserveWhitespace}
            onChange={(event) => patch("preserveWhitespace", event.target.checked)}
          />{" "}
          Preserve indentation and blank lines
        </label>
        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={controller.settings.notifyAfterCopy}
            onChange={(event) => patch("notifyAfterCopy", event.target.checked)}
          />{" "}
          Notify after copy
        </label>
        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={controller.settings.startAtLogin}
            onChange={(event) => patch("startAtLogin", event.target.checked)}
          />{" "}
          Start at login
        </label>
        <label className="flex items-center gap-3">
          <input
            type="checkbox"
            checked={controller.settings.closeToTray}
            onChange={(event) => patch("closeToTray", event.target.checked)}
          />{" "}
          Keep running when window closes
        </label>
        <label className="space-y-2">
          <span className="font-medium">Capture backend</span>
          <select
            value={controller.settings.captureBackend}
            onChange={(event) =>
              patch("captureBackend", event.target.value as AppSettings["captureBackend"])
            }
            className="w-full rounded-xl border border-slate-300 bg-transparent px-3 py-2 dark:border-slate-700"
          >
            <option value="auto">Auto</option>
            <option value="gnome">GNOME screenshot helper</option>
            <option value="portal">Portal (capability required)</option>
          </select>
        </label>
      </div>
      <div className="rounded-2xl border border-slate-200 bg-white p-5 dark:border-slate-800 dark:bg-slate-900">
        <h2 className="font-medium">GNOME shortcut</h2>
        <p className="mt-2 text-sm text-slate-600 dark:text-slate-300">
          Create a custom shortcut for <code className="font-mono">screenshot-ocr capture</code> and
          assign <kbd>Super+Shift+O</kbd>.
        </p>
      </div>
      <div className="flex flex-wrap justify-end gap-2">
        <button
          type="button"
          className="rounded-xl border border-slate-300 px-4 py-2 dark:border-slate-700"
          onClick={() => void controller.restoreSettings()}
        >
          Reset settings
        </button>
        <button
          type="button"
          disabled={!controller.settingsDirty}
          className="rounded-xl bg-indigo-600 px-4 py-2 font-medium text-white disabled:opacity-50"
          onClick={() => void controller.saveSettings()}
        >
          Save settings
        </button>
      </div>
    </div>
  );
}
