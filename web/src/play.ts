type KeyboardLockNavigator = Navigator & { keyboard?: { lock(): Promise<void> } };

export function enterPlay(): void {
  if (!document.fullscreenElement) {
    void document.documentElement.requestFullscreen().then(lockKeyboard);
  }
  void lockPointer();
}

export function leavePlay(): void {
  document.exitPointerLock();
  if (document.fullscreenElement) void document.exitFullscreen();
}

async function lockPointer(): Promise<void> {
  try {
    await document.body.requestPointerLock({ unadjustedMovement: true });
  } catch {
    await document.body.requestPointerLock();
  }
}

function lockKeyboard(): Promise<void> | undefined {
  return (navigator as KeyboardLockNavigator).keyboard?.lock();
}
