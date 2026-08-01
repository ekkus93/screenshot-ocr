import type { ReturnTypeOfController } from "../../test/types";

interface Props {
  controller: ReturnTypeOfController;
}

const LABELS = {
  idle: "Ready to capture",
  preparing: "Preparing capture",
  selecting: "Select a screen region",
  processing: "Recognizing text",
  reviewing: "Review recognized text",
  copied: "Text copied",
  cancelled: "Capture cancelled",
  error: "Capture needs attention",
} as const;

export function CapturePanel({ controller }: Props) {
  const busy = ["preparing", "selecting", "processing"].includes(controller.status);
  return (
    <div className="space-y-5">
      <header>
        <h1 className="text-xl font-semibold">{LABELS[controller.status]}</h1>
        <p className="mt-1 text-sm text-slate-600 dark:text-slate-300">
          Select terminal or interface text, recognize it locally, and copy it.
        </p>
      </header>

      <div className="rounded-2xl border border-slate-200 bg-white p-5 dark:border-slate-800 dark:bg-slate-900">
        <button
          type="button"
          disabled={busy}
          className="w-full rounded-xl bg-indigo-600 px-5 py-4 font-medium text-white hover:bg-indigo-700 disabled:cursor-not-allowed disabled:opacity-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-500"
          onClick={() => void controller.capture()}
        >
          {busy ? "Capture in progress…" : "Capture text from screen"}
        </button>
        <div className="mt-4 flex flex-wrap items-center justify-between gap-2 text-sm">
          <span className="text-slate-500">Keyboard shortcut</span>
          <kbd className="rounded-lg border border-slate-300 px-2 py-1 font-mono dark:border-slate-700">
            Super + Shift + O
          </kbd>
        </div>
      </div>

      {controller.error !== null && (
        <div
          role="alert"
          className="rounded-2xl border border-red-300 bg-red-50 p-4 text-red-950 dark:border-red-900 dark:bg-red-950 dark:text-red-100"
        >
          <p className="font-medium">{controller.error.message}</p>
          <p className="mt-1 text-sm">{controller.error.guidance}</p>
          <code className="mt-2 block text-xs">{controller.error.code}</code>
        </div>
      )}

      <div className="rounded-2xl border border-slate-200 bg-white p-5 dark:border-slate-800 dark:bg-slate-900">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h2 className="font-medium">Recognized text</h2>
            <p className="text-sm text-slate-500">Whitespace and line breaks remain editable.</p>
          </div>
          <span className="text-sm text-slate-500">
            {controller.result?.meanConfidence == null
              ? "Confidence unavailable"
              : `${Math.round(controller.result.meanConfidence)}% confidence`}
          </span>
        </div>
        {controller.result?.warnings.map((warning) => (
          <p
            key={warning.code}
            className="mt-3 rounded-lg bg-amber-100 p-3 text-sm text-amber-950 dark:bg-amber-950 dark:text-amber-100"
          >
            {warning.message}
          </p>
        ))}
        <label htmlFor="ocr-output" className="sr-only">
          Recognized text editor
        </label>
        <textarea
          id="ocr-output"
          rows={14}
          value={controller.editorText}
          onChange={(event) => controller.setEditorText(event.target.value)}
          placeholder="Captured text will appear here."
          className="mt-4 w-full resize-y rounded-xl border border-slate-300 bg-transparent p-3 font-mono text-base focus-visible:outline focus-visible:outline-2 focus-visible:outline-indigo-500 dark:border-slate-700"
        />
        <div className="mt-4 flex flex-wrap justify-end gap-2">
          <button
            type="button"
            className="rounded-xl border border-slate-300 px-4 py-2 dark:border-slate-700"
            onClick={controller.clear}
          >
            Clear
          </button>
          <button
            type="button"
            className="rounded-xl border border-slate-300 px-4 py-2 dark:border-slate-700"
            onClick={() => void controller.capture()}
          >
            Capture again
          </button>
          <button
            type="button"
            className="rounded-xl bg-indigo-600 px-4 py-2 font-medium text-white"
            onClick={() => void controller.copy()}
          >
            Copy text
          </button>
        </div>
      </div>
    </div>
  );
}
