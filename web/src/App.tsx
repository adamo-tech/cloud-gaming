import { useState, type FormEvent, type ReactNode } from "react";
import {
  AdamoProvider,
  connectLocalRobot,
  type LocalConnection,
  type NativeRuntimeLike,
} from "adamo-react";
import { createZenohWebTransport } from "@adamo-tech/zenoh-webtransport";
import { Player } from "./Player";
import { enterPlay, leavePlay } from "./play";

const FIELD_CLASS =
  "w-full rounded-lg border border-white/10 bg-white/[0.04] px-3 py-2.5 text-sm text-white placeholder:text-navy-400 focus:border-white/20 focus:outline-none";
const LABEL_CLASS = "mb-1.5 block text-xs text-zinc-200";

type RelayConnection = { runtime: NativeRuntimeLike; hostName: string; apiKey: string };

function errorMessage(reason: unknown): string {
  return reason instanceof Error ? reason.message : String(reason);
}

export function App() {
  const [address, setAddress] = useState(() => localStorage.getItem("address") ?? "");
  const [pairingKey, setPairingKey] = useState(() => localStorage.getItem("pairingKey") ?? "");
  const [isThroughRelay, setIsThroughRelay] = useState(
    () => localStorage.getItem("isThroughRelay") === "true",
  );
  const [hostName, setHostName] = useState(() => localStorage.getItem("hostName") ?? "");
  const [apiKey, setApiKey] = useState(() => localStorage.getItem("apiKey") ?? "");
  const [isConnecting, setIsConnecting] = useState(false);
  const [connection, setConnection] = useState<LocalConnection | null>(null);
  const [relayConnection, setRelayConnection] = useState<RelayConnection | null>(null);
  const [error, setError] = useState<string | null>(null);

  const play = (event: FormEvent) => {
    event.preventDefault();
    localStorage.setItem("address", address);
    localStorage.setItem("pairingKey", pairingKey);
    localStorage.setItem("isThroughRelay", String(isThroughRelay));
    localStorage.setItem("hostName", hostName);
    localStorage.setItem("apiKey", apiKey);
    setError(null);
    void enterPlay();
    setIsConnecting(true);
    const runtime = createZenohWebTransport({
      receiveBufferBytes: 8 * 1024 * 1024,
      maxPendingSends: 96,
    });
    if (isThroughRelay) {
      runtime.then(
        (runtime) => setRelayConnection({ runtime, hostName, apiKey }),
        (reason) => setError(errorMessage(reason)),
      );
    } else {
      runtime
        .then((runtime) => connectLocalRobot(runtime, address, pairingKey))
        .then(setConnection, (reason) => setError(errorMessage(reason)));
    }
  };

  const exit = () => {
    leavePlay();
    void connection?.close();
    setIsConnecting(false);
    setConnection(null);
    setRelayConnection(null);
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
          <ConnectionPicker isThroughRelay={isThroughRelay} onChange={setIsThroughRelay} />
          <label className={LABEL_CLASS} htmlFor="gaming-pc">
            Gaming PC
          </label>
          <input
            id="gaming-pc"
            value={isThroughRelay ? hostName : address}
            onChange={(event) =>
              isThroughRelay ? setHostName(event.target.value) : setAddress(event.target.value)
            }
            placeholder={isThroughRelay ? "cloud-gaming" : "192.168.1.20:17447"}
            className={`${FIELD_CLASS} mb-4`}
          />
          <label className={LABEL_CLASS} htmlFor="key">
            {isThroughRelay ? "API key" : "Pairing key"}
          </label>
          <input
            id="key"
            type="password"
            autoComplete="off"
            value={isThroughRelay ? apiKey : pairingKey}
            onChange={(event) =>
              isThroughRelay ? setApiKey(event.target.value) : setPairingKey(event.target.value)
            }
            placeholder={isThroughRelay ? "From your Adamo organization" : "Printed by the host"}
            className={`${FIELD_CLASS} mb-8`}
          />
          <button
            type="submit"
            disabled={isThroughRelay ? !hostName || !apiKey : !address || !pairingKey}
            className="w-full rounded-lg bg-white py-3 px-4 font-medium text-gray-800 hover:bg-gray-100 disabled:opacity-40 disabled:cursor-not-allowed transition-colors"
          >
            Play
          </button>
        </form>
      </Centered>
    );
  }
  if (relayConnection)
    return (
      <AdamoProvider
        apiKey={relayConnection.apiKey}
        nativeRuntime={relayConnection.runtime}
        p2p
        p2pRobotId={relayConnection.hostName}
        onSessionOpenFailed={(reason) => setError(errorMessage(reason))}
      >
        <Player robot={relayConnection.hostName} onExit={exit} />
      </AdamoProvider>
    );
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

function ConnectionPicker({
  isThroughRelay,
  onChange,
}: {
  isThroughRelay: boolean;
  onChange: (isThroughRelay: boolean) => void;
}) {
  return (
    <div className="mb-4 flex rounded-lg border border-white/10 bg-white/[0.04] p-0.5">
      {[
        { label: "Local network", isRelayOption: false },
        { label: "Adamo relay", isRelayOption: true },
      ].map(({ label, isRelayOption }) => (
        <button
          key={label}
          type="button"
          onClick={() => onChange(isRelayOption)}
          className={`flex-1 rounded-md py-1.5 text-xs font-medium transition-colors ${
            isRelayOption === isThroughRelay ? "bg-white/[0.14] text-white" : "text-navy-400 hover:text-zinc-200"
          }`}
        >
          {label}
        </button>
      ))}
    </div>
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
