const SIGNAL_CLEANUP_CALLBACKS = new WeakMap();

export function runRegisteredCleanup(signal) {
  const cleanups = SIGNAL_CLEANUP_CALLBACKS.get(signal);
  if (!cleanups || cleanups.length === 0) {
    return;
  }
  SIGNAL_CLEANUP_CALLBACKS.delete(signal);
  for (const cleanup of cleanups.splice(0)) {
    cleanup();
  }
}

export function registerSignalCleanup(signal, cleanup) {
  if (typeof cleanup !== "function") {
    throw new TypeError("registerSignalCleanup expects a cleanup function");
  }
  const cleanups = SIGNAL_CLEANUP_CALLBACKS.get(signal) ?? [];
  cleanups.push(cleanup);
  SIGNAL_CLEANUP_CALLBACKS.set(signal, cleanups);
}
