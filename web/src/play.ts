type KeyboardLockNavigator = Navigator & { keyboard?: { lock(): Promise<void> } };

export async function enterPlay(): Promise<void> {
  const fullscreen = document.fullscreenElement ? Promise.resolve() : document.documentElement.requestFullscreen().then(lockKeyboard);
  await lockPointer().catch(() => fullscreen.then(lockPointer));
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
