/**
 * Live worker-first root diagnostics when no import context is active yet.
 * Snapshot fields stay sync for React attach; refresh pulls from the bridge
 * after standalone mutations so subscribe listeners see real updates.
 */
import {
  createDiagnosticsSubscriptionHandle,
  emptyRootDiagnosticsSnapshot,
  emptyWebPerformanceSummary,
} from "./worker_first_root_empty_diagnostics.js";

export function createWorkerFirstRootLiveDiagnostics(bridge) {
  let snapshot = emptyRootDiagnosticsSnapshot();
  const listeners = new Set();

  function notify() {
    for (const listener of [...listeners]) {
      try {
        listener();
      } catch {
        // One faulty subscriber must not silence sibling diagnostics delivery
        // or abort terminate/reset notification.
      }
    }
  }

  return Object.freeze({
    peek() {
      return snapshot;
    },
    subscribe(listener) {
      if (typeof listener !== "function") {
        throw new TypeError(
          "diagnostics.subscribe(...) requires a listener function",
        );
      }
      listeners.add(listener);
      return createDiagnosticsSubscriptionHandle(() => {
        listeners.delete(listener);
      });
    },
    notify,
    adopt(committedSnapshot) {
      snapshot = Object.freeze(committedSnapshot);
      notify();
    },
    reset() {
      snapshot = emptyRootDiagnosticsSnapshot();
      notify();
    },
    async refresh() {
      const [
        health,
        latestFlow,
        latestObservation,
        performanceSummary,
        latestFailure,
        latestRollback,
        latestInvalidationPlanningEstimate,
        latestInvalidationTraceRecords,
        recentHistory,
        diagnosticsSummaryPacket,
        diagnosticsHistoryPacket,
      ] = await Promise.all([
        bridge.health(),
        bridge.latestFlow(),
        bridge.latestObservation(),
        bridge.performanceSummary(),
        bridge.latestFailure(),
        bridge.latestRollback(),
        bridge.latestInvalidationPlanningEstimate(),
        bridge.latestInvalidationTraceRecords(),
        bridge.recentHistory(),
        bridge.readDiagnosticsSummary().catch(() => null),
        bridge.readDiagnosticsHistory().catch(() => null),
      ]);
      snapshot = Object.freeze({
        health,
        diagnosticsSummary: diagnosticsSummaryPacket?.summary ?? null,
        diagnosticsHistory: diagnosticsHistoryPacket?.history ?? null,
        latestFlow,
        latestObservation,
        performanceSummary: performanceSummary ?? emptyWebPerformanceSummary(),
        latestFailure,
        latestRollback,
        latestInvalidationPlanningEstimate,
        latestInvalidationTraceRecords: Object.freeze(
          latestInvalidationTraceRecords ?? [],
        ),
        recentHistory: Object.freeze(recentHistory ?? []),
      });
      notify();
      return snapshot;
    },
  });
}
