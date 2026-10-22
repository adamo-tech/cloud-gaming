import { Component, type ReactNode } from "react";

export class ErrorScreen extends Component<{ children: ReactNode }, { error: Error | null }> {
  state = { error: null as Error | null };

  static getDerivedStateFromError(error: Error) {
    return { error };
  }

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <div className="min-h-screen flex items-center justify-center px-4">
        <div className="bg-navy-900/40 backdrop-blur-md p-8 rounded-2xl border border-white/10 max-w-2xl w-full">
          <h1 className="font-serif text-2xl font-bold text-white mb-2">Something broke</h1>
          <pre className="font-mono text-xs text-danger whitespace-pre-wrap break-words">{this.state.error.stack ?? this.state.error.message}</pre>
        </div>
      </div>
    );
  }
}
