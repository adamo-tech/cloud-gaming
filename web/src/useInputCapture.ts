import { useEffect, useState } from "react";
import { Reliability, usePublisher, useTimeSync } from "adamo-react";
import { MouseTotals, controlKey, encodeMouseMessage } from "adamo/teleop";

const MOUSE_SEND_INTERVAL_MS = 2;
const WHEEL_V120_PER_LINE = 40;
const WHEEL_V120_PER_PIXEL = 1.2;
const POINTER_MOVE_EVENT = "onpointerrawupdate" in window ? "pointerrawupdate" : "pointermove";

export function usePointerLocked(): boolean {
  const [isLocked, setIsLocked] = useState(() => document.pointerLockElement !== null);
  useEffect(() => {
    const onLockChange = () => setIsLocked(document.pointerLockElement !== null);
    document.addEventListener("pointerlockchange", onLockChange);
    return () => document.removeEventListener("pointerlockchange", onLockChange);
  }, []);
  return isLocked;
}

export function useInputCapture(org: string, robot: string): void {
  const timeSync = useTimeSync();
  const keyboard = usePublisher(org ? controlKey(org, robot, "json/keyboard") : "", {
    reliability: Reliability.RELIABLE,
  });
  const mouse = usePublisher(org ? controlKey(org, robot, "json/mouse") : "");

  useEffect(() => {
    if (!keyboard || !mouse || !timeSync) return;
    const totals = new MouseTotals();
    const pressedCodes = new Set<string>();
    let sendTimer = 0;
    let lastMouseSend = 0;

    const isCaptured = () => document.pointerLockElement !== null;

    const sendKey = (key: string, code: string, action: "down" | "up") => {
      const message = { key, code, action, stamp: timeSync.fabricNowMs() };
      void keyboard.put(new TextEncoder().encode(JSON.stringify(message)));
    };

    const sendMouse = () => {
      sendTimer = 0;
      lastMouseSend = performance.now();
      void mouse.put(encodeMouseMessage(totals.message(timeSync.fabricNowMs())));
    };

    const scheduleMouse = () => {
      if (sendTimer) return;
      const wait = MOUSE_SEND_INTERVAL_MS - (performance.now() - lastMouseSend);
      if (wait <= 0) sendMouse();
      else sendTimer = window.setTimeout(sendMouse, wait);
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
      scheduleMouse();
    };

    const onButtons = (event: MouseEvent) => {
      if (!isCaptured()) return;
      totals.buttonMask = event.buttons;
      scheduleMouse();
    };

    const onWheel = (event: WheelEvent) => {
      if (!isCaptured()) return;
      event.preventDefault();
      const scale =
        event.deltaMode === WheelEvent.DOM_DELTA_LINE ? WHEEL_V120_PER_LINE : WHEEL_V120_PER_PIXEL;
      totals.scroll(event.deltaY * scale, event.deltaX * scale);
      scheduleMouse();
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
}
