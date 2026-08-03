import type { ReactNode } from "react";
import { Component } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  failed: boolean;
}

export class ErrorBoundary extends Component<Props, State> {
  public state: State = { failed: false };

  public static getDerivedStateFromError(): State {
    return { failed: true };
  }

  public componentDidCatch(): void {
    // Captured content and command payloads must never be logged here.
  }

  public render(): ReactNode {
    if (this.state.failed) {
      return (
        <main className="mx-auto max-w-xl bg-paper-50 p-8 font-sans text-ink-950 dark:bg-ink-950 dark:text-paper-50">
          <h1 className="font-mono text-xl font-semibold tracking-tight">
            Screenshot OCR could not display its interface
          </h1>
          <p className="mt-3 text-steel-600 dark:text-steel-400">
            Restart the application. If this continues, use the safe diagnostics report.
          </p>
        </main>
      );
    }
    return this.props.children;
  }
}
