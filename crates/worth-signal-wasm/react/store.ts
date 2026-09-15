import type {
  DisposableHandleLike,
  ReactPerformanceSummary,
  ReactSignalsStore,
  SignalHandleLike,
  SignalsDiagnosticsSnapshot,
  SignalsLike,
} from "./model.js";
import { sameSignalSnapshot } from "./signal_snapshot_equality.js";

type SignalEntry = {
  id: string;
  target: SignalHandleLike | string;
  version: number;
  snapshotVersion: number;
  snapshot: unknown;
  runtimeHandle: DisposableHandleLike | null;
  notifyQueued: boolean;
  establishingSubscription: boolean;
  listeners: Set<() => void>;
};

function resolveSignalId(target: SignalHandleLike | string): string {
  if (typeof target === "string") {
    return target;
  }
  if (target && typeof target.id === "string") {
    return target.id;
  }
  throw new TypeError("signal target must be a string id or signal handle");
}

function readSignalValue(
  signals: SignalsLike,
  target: SignalHandleLike | string,
): unknown {
  // Always enter root read first so worker-first ownership checks still fire for
  // foreign handles. Then prefer the handle callable: resource line.signal() and
  // similar local-truth views can be honest immediately while read(id) still
  // waits on authored publication catch-up.
  if (typeof target === "function") {
    const callable = target as SignalHandleLike & (() => unknown);
    signals.read(callable);
    return callable();
  }
  return signals.read(target);
}

function createDiagnosticsSnapshot(signals: SignalsLike): SignalsDiagnosticsSnapshot {
  const diagnostics = signals.diagnostics();
  return Object.freeze({
    latestObservation: diagnostics.latestObservation(),
    latestFlow: diagnostics.latestFlow(),
    performanceSummary: diagnostics.performanceSummary(),
  });
}

function createReactPerformanceSummary(
  signalEntries: Map<string, SignalEntry>,
  diagnosticsListeners: Set<() => void>,
): ReactPerformanceSummary {
  let activeReactSubscriberCount = 0;
  let activeRuntimeWatchHandleCount = 0;

  for (const entry of signalEntries.values()) {
    activeReactSubscriberCount += entry.listeners.size;
    if (entry.runtimeHandle) {
      activeRuntimeWatchHandleCount += 1;
    }
  }

  const activeSignalSubscriptionCount = signalEntries.size;
  const sharedFanoutRatio =
    activeRuntimeWatchHandleCount === 0
      ? 0
      : activeReactSubscriberCount / activeRuntimeWatchHandleCount;

  return Object.freeze({
    activeSignalSubscriptionCount,
    activeReactSubscriberCount,
    activeRuntimeWatchHandleCount,
    diagnosticsSubscriberCount: diagnosticsListeners.size,
    sharedFanoutRatio,
  });
}

function notifyAll(listeners: Set<() => void>): void {
  for (const listener of Array.from(listeners)) {
    try {
      listener();
    } catch {
      // One React subscriber fault must not silence sibling delivery.
    }
  }
}

function enqueueMicrotask(callback: () => void): void {
  if (typeof queueMicrotask === "function") {
    queueMicrotask(callback);
    return;
  }
  Promise.resolve().then(callback);
}

export function createReactSignalsStore<TSignals extends SignalsLike>(
  signals: TSignals,
): ReactSignalsStore<TSignals> {
  const signalEntries = new Map<string, SignalEntry>();
  const diagnosticsListeners = new Set<() => void>();
  let diagnosticsRuntimeHandle: DisposableHandleLike | null = null;
  let diagnosticsSnapshot = createDiagnosticsSnapshot(signals);
  let diagnosticsRefreshQueued = false;

  function refreshDiagnosticsSnapshot(): void {
    diagnosticsSnapshot = createDiagnosticsSnapshot(signals);
    notifyAll(diagnosticsListeners);
  }

  function scheduleDiagnosticsRefresh(): void {
    if (diagnosticsRefreshQueued) {
      return;
    }
    diagnosticsRefreshQueued = true;
    enqueueMicrotask(() => {
      diagnosticsRefreshQueued = false;
      refreshDiagnosticsSnapshot();
    });
  }

  function ensureDiagnosticsRuntimeSubscription(): void {
    if (diagnosticsRuntimeHandle) {
      return;
    }
    diagnosticsRuntimeHandle = signals.diagnostics().subscribe(() => {
      scheduleDiagnosticsRefresh();
    });
    // Mutations may have landed before the first React diagnostics subscriber
    // attached; sync immediately so getDiagnosticsSnapshot is not permanently stale.
    refreshDiagnosticsSnapshot();
  }

  function releaseDiagnosticsRuntimeSubscription(): void {
    if (!diagnosticsRuntimeHandle) {
      return;
    }
    diagnosticsRuntimeHandle.free();
    diagnosticsRuntimeHandle = null;
  }

  function ensureSignalEntry(target: SignalHandleLike | string): SignalEntry {
    const id = resolveSignalId(target);
    let entry = signalEntries.get(id);
    if (!entry) {
      entry = {
        id,
        target,
        version: 0,
        snapshotVersion: -1,
        snapshot: undefined,
        runtimeHandle: null,
        notifyQueued: false,
        establishingSubscription: false,
        listeners: new Set(),
      };
      signalEntries.set(id, entry);
    } else if (entry.target !== target) {
      // Ids can collide across worker-first runtimes; never reuse a snapshot
      // keyed only by id when the handle identity changed.
      entry.target = target;
      entry.snapshotVersion = -1;
      entry.snapshot = undefined;
    }
    return entry;
  }

  function subscribeSignal(target: SignalHandleLike | string, listener: () => void): () => void {
    const entry = ensureSignalEntry(target);

    if (!entry.runtimeHandle) {
      entry.establishingSubscription = true;
      entry.runtimeHandle = signals.watch(entry.target, (notice) => {
        if (entry.establishingSubscription) {
          return;
        }
        if (!notice.triggerMatched || !notice.meaningfulChange) {
          return;
        }
        entry.version += 1;
        entry.snapshotVersion = -1;
        if (!entry.notifyQueued) {
          entry.notifyQueued = true;
          enqueueMicrotask(() => {
            entry.notifyQueued = false;
            notifyAll(entry.listeners);
          });
        }
      });
      entry.establishingSubscription = false;
    }

    entry.listeners.add(listener);

    return () => {
      const current = signalEntries.get(entry.id);
      if (!current) {
        return;
      }

      current.listeners.delete(listener);

      if (current.listeners.size === 0) {
        if (current.runtimeHandle) {
          signals.nuke(current.runtimeHandle);
        }
        signalEntries.delete(current.id);
      }
    };
  }

  function getSignalSnapshot(target: SignalHandleLike | string): unknown {
    const entry = ensureSignalEntry(target);
    // Every snapshot read enters root read first. importGraph supersession (and
    // foreign-handle attacks) invalidate authority without a per-signal watch
    // notice; the root read throws for them, so a cached snapshot can never
    // zombie-serve truth.
    if (entry.listeners.size > 0 && entry.snapshotVersion === entry.version) {
      // Subscribed and current: the watch is the change channel, the read is
      // only the authority check, and React keeps the cached reference.
      signals.read(entry.target);
      return entry.snapshot;
    }
    const fresh = readSignalValue(signals, entry.target);
    // Unsubscribed (React reads before it subscribes, twice in development) or
    // invalidated by a watch notice: adopt the fresh value, but keep the cached
    // reference when the value did not change so `useSyncExternalStore` sees a
    // stable snapshot instead of a new object per render.
    if (entry.snapshotVersion !== -1 && sameSignalSnapshot(entry.snapshot, fresh)) {
      entry.snapshotVersion = entry.version;
      return entry.snapshot;
    }
    entry.snapshot = fresh;
    entry.snapshotVersion = entry.version;
    return entry.snapshot;
  }

  function subscribeDiagnostics(listener: () => void): () => void {
    ensureDiagnosticsRuntimeSubscription();
    diagnosticsListeners.add(listener);
    return () => {
      diagnosticsListeners.delete(listener);
      if (diagnosticsListeners.size === 0) {
        releaseDiagnosticsRuntimeSubscription();
      }
    };
  }

  function getDiagnosticsSnapshot(): SignalsDiagnosticsSnapshot {
    return diagnosticsSnapshot;
  }

  function transaction(callback: Parameters<SignalsLike["transaction"]>[0]): unknown {
    return signals.transaction(callback);
  }

  function batch(callback: Parameters<SignalsLike["batch"]>[0]): unknown {
    return signals.batch(callback);
  }

  function refreshDiagnostics(): SignalsDiagnosticsSnapshot {
    refreshDiagnosticsSnapshot();
    return diagnosticsSnapshot;
  }

  function performanceSummary(): ReactPerformanceSummary {
    return createReactPerformanceSummary(signalEntries, diagnosticsListeners);
  }

  function dispose(): void {
    for (const entry of signalEntries.values()) {
      if (entry.runtimeHandle) {
        signals.nuke(entry.runtimeHandle);
      }
    }
    signalEntries.clear();
    diagnosticsListeners.clear();
    releaseDiagnosticsRuntimeSubscription();
  }

  return Object.freeze({
    signals,
    subscribeSignal,
    getSignalSnapshot,
    subscribeDiagnostics,
    getDiagnosticsSnapshot,
    transaction,
    batch,
    refreshDiagnostics,
    performanceSummary,
    dispose,
  });
}
