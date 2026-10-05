/**
 * Empty-root worker-first diagnostics: honest null/zero snapshots when no
 * imported graph is active yet (React attach and empty app shells).
 */
export function emptyWebPerformanceSummary() {
  return Object.freeze({
    activeHandleCount: 0,
    activeCallbackCount: 0,
    activeComputeCallbackCount: 0,
    activeComputeCollectorCount: 0,
    matchedWatcherBreadth: 0,
    deliveredObservationCount: 0,
    rollbackSuppressedDeliveryCount: 0,
    serialExecutorUsageCount: 0,
    executionReport: null,
    outputSerializationCount: 0,
    outputSerializationBreadth: 0,
    jsCallbackInvocationCount: 0,
    jsCallbackFailureCount: 0,
    observationCallbackRegistrationCount: 0,
    observationCallbackDisposalCount: 0,
    observationCallbackGenerationMismatchDenialCount: 0,
    observationCallbackAllocationCount: 0,
    observationCallbackReuseCount: 0,
    computeCallbackRegistrationCount: 0,
    computeCallbackDisposalCount: 0,
    computeCallbackInvocationCount: 0,
    computeCallbackFailureCount: 0,
    computeCallbackGenerationMismatchDenialCount: 0,
    computeCallbackSelfReadDenialCount: 0,
    computeCallbackDynamicCycleDenialCount: 0,
    computeCallbackPromiseReturnDenialCount: 0,
    computeCallbackInvalidReturnDenialCount: 0,
    computeCallbackCollectorInstallationCount: 0,
    computeCallbackCaptureCount: 0,
    computeCallbackCapturedReadCount: 0,
    computeCallbackReturnSerializationBreadth: 0,
    computeCallbackAllocationCount: 0,
    computeCallbackReuseCount: 0,
    computeCallbackDependencyPatchCount: 0,
    computeCallbackDependencyPatchAddedCount: 0,
    computeCallbackDependencyPatchRemovedCount: 0,
    computeCallbackDependencyPatchRetainedCount: 0,
    computeCallbackRuntimeReadBreadth: 0,
    computeCallbackConstantNoSignalReadClassificationCount: 0,
    computeCallbackSignalTrackedClassificationCount: 0,
    computeCallbackMissingUnavailabilityCount: 0,
    compatibilityReadCount: 0,
    compatibilityReadBreadth: 0,
  });
}

export function emptyRootDiagnosticsSnapshot() {
  return Object.freeze({
    health: null,
    diagnosticsSummary: null,
    diagnosticsHistory: null,
    latestFlow: null,
    latestObservation: null,
    performanceSummary: emptyWebPerformanceSummary(),
    latestFailure: null,
    latestRollback: null,
    latestInvalidationPlanningEstimate: null,
    latestInvalidationTraceRecords: Object.freeze([]),
    recentHistory: Object.freeze([]),
  });
}

export function createDiagnosticsSubscriptionHandle(onFree = null) {
  let active = true;
  const free = () => {
    if (!active) {
      return;
    }
    active = false;
    if (typeof onFree === "function") {
      onFree();
    }
  };
  return Object.freeze({
    free,
    [Symbol.dispose]() {
      free();
    },
  });
}
