import { useState, type FormEvent, type ReactNode } from "react";
import { AdamoProvider, connectLocalRobot, type LocalConnection } from "adamo-react";
import { createZenohWebTransport } from "@adamo-tech/zenoh-webtransport";
import { Player } from "./Player";
import { enterPlay, leavePlay } from "./play";

const FIELD_CLASS =
  "w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2.5 text-sm text-white placeholder:text-navy-400 focus:border-white/20 focus:outline-none";
const LABEL_CLASS = "mb-1.5 block text-xs text-zinc-200";

export function App() {
  const [address, setAddress] = useState(() => localStorage.getItem("address") ?? "");
  const [pairingKey, setPairingKey] = useState(() => localStorage.getItem("pairingKey") ?? "");
  const [isConnecting, setIsConnecting] = useState(false);
  const [connection, setConnection] = useState<LocalConnection | null>(null);
  const [error, setError] = useState<string | null>(null);

  const play = (event: FormEvent) => {
    event.preventDefault();
    localStorage.setItem("address", address);
    localStorage.setItem("pairingKey", pairingKey);
    setError(null);
    void enterPlay();
    setIsConnecting(true);
    createZenohWebTransport({ receiveBufferBytes: 8 * 1024 * 1024, maxPendingSends: 96 })
      .then((runtime) => connectLocalRobot(runtime, address, pairingKey))
      .then(setConnection, (reason) =>
        setError(reason instanceof Error ? reason.message : String(reason)),
      );
  };

  const exit = () => {
    leavePlay();
    void connection?.close();
    setIsConnecting(false);
    setConnection(null);
    setError(null);
  };

  if (error) {
    return (
      <Centered>
        <h1 className="font-serif text-2xl font-bold text-white mb-2">Could not connect</h1>
        <p className="text-sm text-danger break-words mb-6">{error}</p>
        <button
          onClick={exit}
          className="w-full rounded-lg bg-white/[0.1] px-4 py-2.5 text-sm font-medium text-white hover:bg-white/[0.15] transition-colors"
        >
          Back
        </button>
      </Centered>
    );
  }
  if (!isConnecting) {
    return (
      <Centered>
        <form onSubmit={play}>
          <h1 className="font-serif text-2xl font-bold text-white mb-2">Adamo Cloud Gaming</h1>
          <p className="text-zinc-200 mb-8">Play the games on your own PC from anywhere.</p>
          <label className={LABEL_CLASS} htmlFor="address">
            Gaming PC address
          </label>
          <input
            id="address"
            value={address}
            onChange={(event) => setAddress(event.target.value)}
            placeholder="192.168.1.20"
            className={`${FIELD_CLASS} mb-4`}
          />
          <label className={LABEL_CLASS} htmlFor="pairing-key">
            Pairing key
          </label>
          <input
            id="pairing-key"
            type="password"
            autoComplete="off"
            value={pairingKey}
            onChange={(event) => setPairingKey(event.target.value)}
            className={`${FIELD_CLASS} mb-8`}
          />
          <button
            type="submit"
            disabled={!address || !pairingKey}
            className="w-full rounded-lg bg-white py-3 px-4 font-medium text-gray-800 hover:bg-gray-100 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
          >
            Play
          </button>
        </form>
      </Centered>
    );
  }
  if (!connection)
    return (
      <Centered>
        <p className="text-zinc-200">Connecting…</p>
      </Centered>
    );
  return (
    <AdamoProvider
      session={connection.session}
      ctrlSession={connection.session}
      org={connection.organization}
    >
      <Player robot={connection.robot} audioSession={connection.ctrlSession} onExit={exit} />
    </AdamoProvider>
  );
}

function Centered({ children }: { children: ReactNode }) {
  return (
    <div className="relative min-h-screen flex items-center justify-center overflow-hidden px-4">
      <div className="relative z-10 bg-navy-900/40 backdrop-blur-md p-8 rounded-2xl border border-white/10 max-w-md w-full">
        {children}
      </div>
    </div>
  );
}
