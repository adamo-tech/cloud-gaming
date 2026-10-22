type KeyboardLockNavigator = Navigator & { keyboard?: { lock(): Promise<void> } };

export function enterPlay(): void {
  if (!document.fullscreenElement) void document.documentElement.requestFullscreen().then(() => (navigator as KeyboardLockNavigator).keyboard?.lock());
  void Promise.resolve(document.body.requestPointerLock({ unadjustedMovement: true })).catch(() => document.body.requestPointerLock());
}

export function leavePlay(): void {
  document.exitPointerLock();
  if (document.fullscreenElement) void document.exitFullscreen();
}
