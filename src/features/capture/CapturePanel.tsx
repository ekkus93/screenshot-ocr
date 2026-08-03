import type { ReactNode } from "react";
import type { AppController } from "../../app/controllerTypes";

interface Props {
  controller: AppController;
}

const LABELS = {
  idle: "Ready to capture",
  preparing: "Preparing capture",
  selecting: "Select a region; OCR continues after selection",
  processing: "Recognizing text",
  cancelling: "Cancelling capture",
  reviewing: "Review recognized text",
  copied: "Text copied",
  cancelled: "Capture cancelled",
  error: "Capture needs attention",
} as const;

/** Viewfinder-style corner brackets — the visual signature tying the capture
 * button and error state back to the literal act of scanning the screen. */
function ScanFrame({
  tone,
  active = false,
  children,
}: {
  tone: "phosphor" | "alert";
  active?: boolean;
  children: ReactNode;
}) {
  const border = tone === "alert" ? "border-alert-500" : "border-phosphor-500";
  const corner = `absolute h-3 w-3 ${border} ${active ? "animate-pulse" : ""}`;
  return (
    <div className="relative p-1.5">
      <span aria-hidden className={`${corner} left-0 top-0 border-l-2 border-t-2`} />
      <span aria-hidden className={`${corner} right-0 top-0 border-r-2 border-t-2`} />
      <span aria-hidden className={`${corner} bottom-0 left-0 border-b-2 border-l-2`} />
      <span aria-hidden className={`${corner} bottom-0 right-0 border-b-2 border-r-2`} />
      {children}
    </div>
  );
}

function ConfidenceMeter({ value }: { value: number | null }) {
  if (value === null) {
    return (
      <span className="font-mono text-xs text-steel-500 dark:text-steel-400">
        confidence unavailable
      </span>
    );
  }
  const pct = Math.max(0, Math.min(100, Math.round(value)));
  return (
    <div className="flex items-center gap-2">
      <div
        role="meter"
        aria-valuenow={pct}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label="Recognition confidence"
        className="h-1.5 w-24 overflow-hidden rounded-full bg-steel-200 dark:bg-ink-700"
      >
        <div className="h-full bg-signal-500" style={{ width: `${pct.toString()}%` }} />
      </div>
      <span className="font-mono text-xs text-steel-600 dark:text-steel-400">{pct}%</span>
    </div>
  );
}

export function CapturePanel({ controller }: Props) {
  const busy = ["preparing", "selecting", "processing", "cancelling"].includes(controller.status);
  const cancelling = controller.status === "cancelling";

  return (
    <div className="flex h-full flex-col gap-1.5">
      <div className="flex shrink-0 flex-wrap items-center gap-4">
        <ScanFrame tone="phosphor" active={busy}>
          <button
            type="button"
            disabled={busy}
            aria-label={busy ? undefined : "Capture text from screen"}
            className="rounded-md bg-phosphor-500 px-3 py-1.5 font-mono text-sm font-semibold text-ink-950 hover:bg-phosphor-600 disabled:cursor-not-allowed disabled:opacity-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-phosphor-500"
            onClick={() => {
              void controller.capture();
            }}
          >
            {busy ? "Scanning…" : "Capture"}
          </button>
        </ScanFrame>

        {busy && (
          <button
            type="button"
            disabled={cancelling}
            className="rounded-md border border-steel-200 px-3 py-1.5 font-mono text-sm font-medium hover:bg-paper-100 disabled:cursor-not-allowed disabled:opacity-50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-phosphor-500 dark:border-ink-700 dark:hover:bg-ink-800"
            onClick={() => {
              void controller.cancel();
            }}
          >
            {cancelling ? "Cancelling…" : "Cancel capture"}
          </button>
        )}

        <span className="font-mono text-xs text-steel-500 dark:text-steel-400">
          Shortcut{" "}
          <kbd className="rounded border border-steel-200 px-1.5 py-0.5 dark:border-ink-700">
            Super + Shift + O
          </kbd>
        </span>

        <span className="ml-auto font-mono text-sm text-steel-600 dark:text-steel-400">
          {LABELS[controller.status]}
        </span>
      </div>

      {controller.error !== null && (
        <ScanFrame tone="alert">
          <div
            role="alert"
            className="block w-full shrink-0 rounded-lg border border-alert-500/40 bg-alert-500/5 p-1.5 dark:bg-alert-500/10"
          >
            <p className="font-mono text-sm font-semibold text-alert-700 dark:text-alert-500">
              {controller.error.message}
            </p>
            <p className="text-xs text-steel-600 dark:text-steel-400">
              {controller.error.guidance}{" "}
              <code className="font-mono">[ {controller.error.code} ]</code>
            </p>
          </div>
        </ScanFrame>
      )}

      <div className="flex min-h-0 flex-1 flex-col rounded-lg border border-steel-200 bg-white p-2 dark:border-ink-700 dark:bg-ink-900">
        <div className="flex shrink-0 flex-wrap items-center justify-between gap-3">
          <h2 className="font-mono text-sm font-semibold uppercase tracking-widest text-steel-600 dark:text-steel-400">
            Recognized text
          </h2>
          <ConfidenceMeter value={controller.result?.meanConfidence ?? null} />
        </div>
        {controller.result?.warnings.map((warning) => (
          <p
            key={warning.code}
            className="mt-2 shrink-0 rounded-md bg-alert-500/10 p-2 text-sm text-alert-700 dark:text-alert-500"
          >
            {warning.message}
          </p>
        ))}
        <label htmlFor="ocr-output" className="sr-only">
          Recognized text editor
        </label>
        <textarea
          id="ocr-output"
          rows={3}
          value={controller.editorText}
          onChange={(event) => {
            controller.setEditorText(event.target.value);
          }}
          placeholder="Captured text will appear here."
          className="mt-1.5 w-full flex-1 resize-y rounded-lg border border-steel-200 bg-transparent p-2 font-mono text-base focus-visible:outline focus-visible:outline-2 focus-visible:outline-phosphor-500 dark:border-ink-700"
        />
        <div className="mt-1.5 flex shrink-0 flex-wrap justify-end gap-2">
          <button
            type="button"
            disabled={busy}
            className="rounded-md border border-steel-200 px-3 py-1.5 font-mono text-sm disabled:cursor-not-allowed disabled:opacity-50 dark:border-ink-700"
            onClick={controller.clear}
          >
            Clear
          </button>
          <button
            type="button"
            disabled={busy}
            className="rounded-md bg-phosphor-500 px-3 py-1.5 font-mono text-sm font-semibold text-ink-950 hover:bg-phosphor-600 disabled:cursor-not-allowed disabled:opacity-50"
            onClick={() => {
              void controller.copy();
            }}
          >
            Copy text
          </button>
        </div>
      </div>
    </div>
  );
}
