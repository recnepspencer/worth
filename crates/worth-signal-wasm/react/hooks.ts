import { useMemo, useRef, useSyncExternalStore } from "react";

import { useReactSignalsStore } from "./context.js";

import type {
  OptionalResourceLineResult,
  OptionalSignalValueResult,
  ReactSignalsStore,
  ResourceLineReactLike,
  SignalHandleLike,
  SignalsDiagnosticsSnapshot,
} from "./model.js";

export function useSignalValue<T = unknown>(
  signal: SignalHandleLike,
  store: ReactSignalsStore,
): T {
  const subscribe = useMemo(
    () => (listener: () => void) => store.subscribeSignal(signal, listener),
    [signal, store],
  );
  const getSnapshot = useMemo(
    () => () => store.getSignalSnapshot(signal) as T,
    [signal, store],
  );
  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}

export function useOutputValue<T = unknown>(
  output: SignalHandleLike,
  store: ReactSignalsStore,
): T {
  const subscribe = useMemo(
    () => (listener: () => void) => store.subscribeSignal(output, listener),
    [output, store],
  );
  const getSnapshot = useMemo(
    () => () => store.getSignalSnapshot(output) as T,
    [output, store],
  );
  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}

export function useOptionalSignalValue<TValue = unknown, TInactive = undefined>(
  signal: SignalHandleLike | null | undefined,
  store: ReactSignalsStore,
  options?: {
    inactiveValue?: TInactive;
  },
): OptionalSignalValueResult<TValue, TInactive> {
  const inactiveValue = options?.inactiveValue as TInactive;
  const inactiveSnapshot = useMemo(
    () => Object.freeze({ kind: "inactive", value: inactiveValue }),
    [inactiveValue],
  );
  const subscribe = useMemo(
    () => (listener: () => void) =>
      signal
        ? store.subscribeSignal(signal, listener)
        : () => {},
    [signal, store],
  );
  const getSnapshot = useMemo(
    () => () => (signal ? store.getSignalSnapshot(signal) : inactiveSnapshot),
    [inactiveSnapshot, signal, store],
  );
  const snapshot = useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
  return useMemo(() => {
    if (snapshot === inactiveSnapshot) {
      return Object.freeze<OptionalSignalValueResult<TValue, TInactive>>({
        kind: "inactive",
        reason: "authorInactive",
        value: inactiveValue,
      });
    }
    return Object.freeze({
      kind: "active",
      value: snapshot as TValue,
    });
  }, [inactiveSnapshot, inactiveValue, snapshot]);
}

export function useOptionalResourceLine<
  TValue = unknown,
  TInactive = undefined,
  TParams = unknown,
  TLine extends ResourceLineReactLike<TValue, TParams> = ResourceLineReactLike<TValue, TParams>,
>(
  line: TLine | null | undefined,
  store: ReactSignalsStore,
  options?: {
    inactiveValue?: TInactive;
  },
): OptionalResourceLineResult<TLine, TValue, TInactive> {
  const signalState = useOptionalSignalValue<TValue, TInactive>(
    line?.signal(),
    store,
    options,
  );
  const summaryState = useOptionalSignalValue<ReturnType<TLine["summary"]>>(
    line?.summarySignal(),
    store,
  );

  return useMemo(
    () => {
      if (
        !line
        || signalState.kind === "inactive"
        || summaryState.kind === "inactive"
      ) {
        return Object.freeze({
          kind: "inactive",
          reason: "authorInactive",
          line: null,
          value: signalState.value as TInactive,
          summary: null,
          status: null,
          freshness: null,
          diagnosticsSummary: null,
        }) as OptionalResourceLineResult<TLine, TValue, TInactive>;
      }

      const summary = summaryState.value;
      return Object.freeze({
        kind: "active",
        line,
        value: signalState.value,
        summary,
        status: summary.current.status,
        freshness: summary.current.freshness,
        diagnosticsSummary: summary.diagnostics,
      }) as OptionalResourceLineResult<TLine, TValue, TInactive>;
    },
    [line, signalState.kind, signalState.value, summaryState],
  );
}

export function useSignalsDiagnostics(
  store?: ReactSignalsStore,
): SignalsDiagnosticsSnapshot {
  const resolvedStore = store ?? useReactSignalsStore();
  return useSyncExternalStore(
    resolvedStore.subscribeDiagnostics,
    resolvedStore.getDiagnosticsSnapshot,
    resolvedStore.getDiagnosticsSnapshot,
  );
}

export function useSignalsDiagnosticsValue<TValue>(
  selector: (snapshot: SignalsDiagnosticsSnapshot) => TValue,
  store?: ReactSignalsStore,
): TValue {
  const resolvedStore = store ?? useReactSignalsStore();
  const cacheRef = useRef<{
    diagnosticsSnapshot: SignalsDiagnosticsSnapshot | null;
    hasValue: boolean;
    value: TValue | undefined;
  }>({
    diagnosticsSnapshot: null,
    hasValue: false,
    value: undefined,
  });
  const getSnapshot = useMemo(
    () => () => {
      const diagnosticsSnapshot = resolvedStore.getDiagnosticsSnapshot();
      const cache = cacheRef.current;
      if (cache.diagnosticsSnapshot === diagnosticsSnapshot && cache.hasValue) {
        return cache.value as TValue;
      }
      const nextValue = selector(diagnosticsSnapshot);
      if (cache.hasValue && Object.is(cache.value, nextValue)) {
        cache.diagnosticsSnapshot = diagnosticsSnapshot;
        return cache.value as TValue;
      }
      cache.diagnosticsSnapshot = diagnosticsSnapshot;
      cache.hasValue = true;
      cache.value = nextValue;
      return nextValue;
    },
    [resolvedStore, selector],
  );
  return useSyncExternalStore(
    resolvedStore.subscribeDiagnostics,
    getSnapshot,
    getSnapshot,
  );
}
