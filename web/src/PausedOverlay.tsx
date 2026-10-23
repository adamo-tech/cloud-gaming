import type { StreamStats } from "adamo-react";
import { enterPlay } from "./play";

type PausedOverlayProps = {
  robot: string;
  isOnline: boolean | undefined;
  stats: StreamStats | null;
  onExit: () => void;
};

export function PausedOverlay({ robot, isOnline, stats, onExit }: PausedOverlayProps) {
  const status = isOnline === undefined ? "Connecting…" : isOnline ? "Online" : "Offline";
  const statusColor =
    isOnline === undefined ? "bg-navy-400" : isOnline ? "bg-success" : "bg-danger";
  return (
    <>
      <div className="absolute inset-x-0 top-0 flex items-center justify-between gap-4 bg-gradient-to-b from-black/80 to-transparent px-5 py-4 text-sm">
        <div className="flex items-center gap-3">
          <span className={`h-2 w-2 rounded-full ${statusColor}`} />
          <span className="font-serif text-lg text-white">{robot}</span>
          <span className="text-zinc-300">{status}</span>
        </div>
        <div className="flex items-center gap-4">
          {stats && (
            <span className="font-mono text-xs text-zinc-300">
              {stats.frameWidth}×{stats.frameHeight} · {Math.round(stats.fps)} fps ·{" "}
              {(stats.kbps / 1000).toFixed(1)} Mbps · {Math.round(stats.rttMs)} ms
            </span>
          )}
          <button
            onClick={onExit}
            className="rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-white transition-colors hover:border-white/20 hover:bg-white/[0.08]"
          >
            Disconnect
          </button>
        </div>
      </div>
      <button
        onClick={enterPlay}
        className="absolute inset-0 top-16 flex cursor-pointer items-center justify-center"
      >
        <span className="rounded-2xl border border-white/10 bg-navy-900/40 px-8 py-6 text-center backdrop-blur-md">
          <span className="mb-1 block font-serif text-2xl font-bold text-white">Click to play</span>
          <span className="block text-sm text-zinc-200">
            Captures mouse and keyboard in fullscreen. Press Esc to release.
          </span>
        </span>
      </button>
    </>
  );
}
