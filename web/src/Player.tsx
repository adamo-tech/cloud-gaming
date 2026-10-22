import { useEffect, useState } from "react";
import { AudioStream, GamepadController, Reliability, Stream, useOrg, usePublisher, useRobotOnline, useSubscription, useTimeSync, type StreamStats } from "adamo-react";
import { MouseTotals, controlKey, decodeRumbleMessage, encodeMouseMessage, playRumble, rumbleKey } from "adamo/teleop";
import { enterPlay } from "./play";
import { Cursor } from "./Cursor";

const GAMEPAD_INDICES = [0, 1, 2, 3];
const MOUSE_SEND_INTERVAL_MS = 4;
const POINTER_MOVE_EVENT = "onpointerrawupdate" in window ? "pointerrawupdate" : "pointermove";

export function Player({ robot, onExit }: { robot: string; onExit: () => void }) {
  const org = useOrg() ?? "";
  const timeSync = useTimeSync();
  const isOnline = useRobotOnline(robot);
  const keyboard = usePublisher(org ? controlKey(org, robot, "json/keyboard") : "", { reliability: Reliability.RELIABLE });
  const mouse = usePublisher(org ? controlKey(org, robot, "json/mouse") : "");
  const [isLocked, setIsLocked] = useState(() => document.pointerLockElement !== null);
  const [stats, setStats] = useState<StreamStats | null>(null);

  useSubscription(org ? rumbleKey(org, robot) : "", (sample) => playRumble(decodeRumbleMessage(sample.payload().toBytes())));

  useEffect(() => {
    const onLockChange = () => setIsLocked(document.pointerLockElement !== null);
    document.addEventListener("pointerlockchange", onLockChange);
    return () => document.removeEventListener("pointerlockchange", onLockChange);
  }, []);

  useEffect(() => {
    if (!keyboard || !mouse || !timeSync) return;
    const totals = new MouseTotals();
    const pressedCodes = new Set<string>();
    let isMouseDirty = false;
    let sendTimer = 0;

    const isCaptured = () => document.pointerLockElement !== null;
    const sendKey = (key: string, code: string, action: "down" | "up") => void keyboard.put(new TextEncoder().encode(JSON.stringify({ key, code, action, stamp: timeSync.fabricNowMs() })));
    const sendMouse = () => {
      isMouseDirty = false;
      void mouse.put(encodeMouseMessage(totals.message(timeSync.fabricNowMs())));
    };
    const markMouseDirty = () => {
      if (isMouseDirty) return;
      isMouseDirty = true;
      sendTimer = window.setTimeout(sendMouse, MOUSE_SEND_INTERVAL_MS);
    };

    const onKey = (event: KeyboardEvent) => {
      if (!isCaptured() || event.repeat) return;
      event.preventDefault();
      const action = event.type === "keydown" ? "down" : "up";
      if (action === "down") pressedCodes.add(event.code);
      else pressedCodes.delete(event.code);
      sendKey(event.key, event.code, action);
    };
    const onPointerMove = (event: PointerEvent) => {
      if (!isCaptured()) return;
      totals.move(event.movementX, event.movementY);
      markMouseDirty();
    };
    const onButtons = (event: MouseEvent) => {
      if (!isCaptured()) return;
      totals.buttonMask = event.buttons;
      markMouseDirty();
    };
    const onWheel = (event: WheelEvent) => {
      if (!isCaptured()) return;
      event.preventDefault();
      const scale = event.deltaMode === WheelEvent.DOM_DELTA_LINE ? 40 : 1.2;
      totals.scroll(event.deltaY * scale, event.deltaX * scale);
      markMouseDirty();
    };
    const onLockChange = () => {
      if (isCaptured()) return;
      for (const code of pressedCodes) sendKey("", code, "up");
      pressedCodes.clear();
      totals.buttonMask = 0;
      sendMouse();
    };

    window.addEventListener("keydown", onKey);
    window.addEventListener("keyup", onKey);
    window.addEventListener("wheel", onWheel, { passive: false });
    document.addEventListener(POINTER_MOVE_EVENT, onPointerMove as EventListener);
    document.addEventListener("mousedown", onButtons);
    document.addEventListener("mouseup", onButtons);
    document.addEventListener("pointerlockchange", onLockChange);
    return () => {
      clearTimeout(sendTimer);
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("keyup", onKey);
      window.removeEventListener("wheel", onWheel);
      document.removeEventListener(POINTER_MOVE_EVENT, onPointerMove as EventListener);
      document.removeEventListener("mousedown", onButtons);
      document.removeEventListener("mouseup", onButtons);
      document.removeEventListener("pointerlockchange", onLockChange);
    };
  }, [keyboard, mouse, timeSync]);

  return (
    <div className="fixed inset-0 bg-black">
      <Stream robot={robot} track="screen" videoCodec="av1" jitterMaxMs={0} robotOnline={isOnline} onStats={setStats} style={{ width: "100%", height: "100%" }} />
      <AudioStream robot={robot} track="audio" />
      <Cursor org={org} robot={robot} />
      {org && GAMEPAD_INDICES.map((index) => (
        <GamepadController key={index} topic={`joy/${index}`} config={{ org, robotName: robot, deviceId: index, serializationFormat: "msgpack" }} />
      ))}
      {!isLocked && (
        <>
          <div className="absolute inset-x-0 top-0 flex items-center justify-between gap-4 bg-gradient-to-b from-black/80 to-transparent px-5 py-4 text-sm">
            <div className="flex items-center gap-3">
              <span className={`h-2 w-2 rounded-full ${isOnline ? "bg-success" : isOnline === false ? "bg-danger" : "bg-navy-400"}`} />
              <span className="font-serif text-lg text-white">{robot}</span>
              <span className="text-zinc-300">{isOnline ? "Online" : isOnline === false ? "Offline" : "Connecting…"}</span>
            </div>
            <div className="flex items-center gap-4">
              {stats && (
                <span className="font-mono text-xs text-zinc-300">
                  {stats.frameWidth}×{stats.frameHeight} · {Math.round(stats.fps)} fps · {(stats.kbps / 1000).toFixed(1)} Mbps · {Math.round(stats.rttMs)} ms
                </span>
              )}
              <button onClick={onExit} className="rounded-lg border border-white/10 bg-white/[0.04] px-3 py-1.5 text-white hover:bg-white/[0.08] hover:border-white/20 transition-colors">Disconnect</button>
            </div>
          </div>
          <button onClick={enterPlay} className="absolute inset-0 top-16 flex items-center justify-center cursor-pointer">
            <span className="bg-navy-900/40 backdrop-blur-md px-8 py-6 rounded-2xl border border-white/10 text-center">
              <span className="block font-serif text-2xl font-bold text-white mb-1">Click to play</span>
              <span className="block text-sm text-zinc-200">Captures mouse and keyboard in fullscreen. Press Esc to release.</span>
            </span>
          </button>
        </>
      )}
    </div>
  );
}
