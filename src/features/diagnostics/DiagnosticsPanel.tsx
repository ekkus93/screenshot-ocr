import type { AppController } from "../../app/controllerTypes";

interface Props {
  controller: AppController;
}

function formatKey(key: string): string {
  return key.replace(/([a-z0-9])([A-Z])/g, "$1 $2");
}

export function DiagnosticsPanel({ controller }: Props) {
  const rows = controller.diagnostics === null ? [] : Object.entries(controller.diagnostics);
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-end gap-2">
        <button
          type="button"
          className="rounded-md border border-steel-200 px-2 py-1 font-mono text-xs dark:border-ink-700"
          onClick={() => void controller.refreshDiagnostics()}
        >
          Refresh
        </button>
      </div>
      <dl className="divide-y divide-steel-200 rounded-lg border border-steel-200 bg-white px-2 dark:divide-ink-700 dark:border-ink-700 dark:bg-ink-900">
        {rows.map(([key, value]) => (
          <div key={key} className="grid grid-cols-[13rem_1fr] gap-1 py-px">
            <dt className="font-mono text-xs uppercase tracking-normal text-steel-500 dark:text-steel-400">
              {formatKey(key)}
            </dt>
            <dd className="break-words font-mono text-xs leading-tight text-ink-950 dark:text-paper-50">
              {Array.isArray(value) ? value.join(", ") : String(value ?? "none")}
            </dd>
          </div>
        ))}
      </dl>
    </div>
  );
}
