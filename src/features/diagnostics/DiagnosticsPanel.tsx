import type { ReturnTypeOfController } from "../../test/types";

interface Props {
  controller: ReturnTypeOfController;
}

export function DiagnosticsPanel({ controller }: Props) {
  const rows = controller.diagnostics === null ? [] : Object.entries(controller.diagnostics);
  return (
    <div className="space-y-5">
      <header>
        <h1 className="text-xl font-semibold">Safe diagnostics</h1>
        <p className="mt-1 text-sm text-slate-600 dark:text-slate-300">This report excludes screenshots, recognized text, clipboard contents, and temporary paths.</p>
      </header>
      <dl className="rounded-2xl border border-slate-200 bg-white p-5 dark:border-slate-800 dark:bg-slate-900">
        {rows.map(([key, value]) => (
          <div key={key} className="grid gap-1 border-b border-slate-200 py-3 last:border-b-0 dark:border-slate-800 sm:grid-cols-[14rem_1fr]">
            <dt className="font-medium">{key}</dt>
            <dd className="break-words font-mono text-sm">{Array.isArray(value) ? value.join(", ") : String(value ?? "none")}</dd>
          </div>
        ))}
      </dl>
      <button type="button" className="rounded-xl border border-slate-300 px-4 py-2 dark:border-slate-700" onClick={() => void controller.refreshDiagnostics()}>Refresh diagnostics</button>
    </div>
  );
}
