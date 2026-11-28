import { stableValueDigest } from "../values/value_paths.js";
import { sourceCompatibilityBlockers } from "../sources/source_compatibility.js";

export function noteResourceDrift(store, observation) {
  if (observation.resourceSource === null) {
    return null;
  }
  return store.observe(classifyResourceDriftObservation(observation));
}

export function readResourceDriftReport(store, context) {
  const history = store.history().map((artifact) => normalizeArtifact(artifact, context));
  const current = normalizeArtifact(store.current(), context);
  const active = current && current.stale !== true && current.resolved !== true
    ? current
    : null;
  const summary = Object.freeze({
    status: active?.status ?? "ready",
    stale: current?.stale ?? false,
    resolved: current?.resolved ?? false,
    hadLocalDraft: active?.hadLocalDraft ?? false,
    blockerCount: active?.blockers.length ?? 0,
    messageCount: active?.messages.length ?? 0,
  });
  const counters = Object.freeze({
    costBasis: "resourceDriftHistoryScan",
    incrementalStatus: "notIncremental",
    observedChanges: history.length,
    preservedChanges: history.filter((entry) => entry.status === "preserved").length,
    rebasedChanges: history.filter((entry) => entry.status === "rebased").length,
    blockedChanges: history.filter((entry) => entry.status === "blocked").length,
    conflictedChanges: history.filter((entry) => entry.status === "conflict").length,
    staleChanges: history.filter((entry) => entry.stale === true).length,
    resolvedChanges: history.filter((entry) => entry.resolved === true).length,
    blockers: active?.blockers.length ?? 0,
    messages: active?.messages.length ?? 0,
  });
  return Object.freeze({
    current,
    history,
    summary,
    counters,
    digest: stableValueDigest({
      current,
      history,
      summary,
      counters,
    }),
  });
}

function classifyResourceDriftObservation(observation) {
  const draftDigest = stableValueDigest(observation.draft);
  const effectiveDigest = stableValueDigest(observation.effective);
  const hadLocalDraft = draftDigest !== "{}";
  const sourceCompatibilityBlockerList = sourceCompatibilityBlockers(observation.sourceCompatibility);
  if (sourceCompatibilityBlockerList.length > 0) {
    return Object.freeze({
      sourceKind: "resourceLine",
      currentSourceDigest: observation.currentSourceDigest,
      status: "blocked",
      hadLocalDraft,
      draftDigest,
      effectiveDigest,
      sourceCompatibilityPosture: observation.sourceCompatibility.posture,
      visibleSelectionKind: observation.resourceSource.visibleSelection.kind,
      blockers: sourceCompatibilityBlockerList,
      messages: Object.freeze([]),
      reason: observation.sourceCompatibility.reason
        ?? "remote resource source drift blocked local draft truth because schema compatibility is unavailable",
    });
  }
  return Object.freeze({
    sourceKind: "resourceLine",
    currentSourceDigest: observation.currentSourceDigest,
    status: hadLocalDraft && observation.resourceSource.visibleSelection.rebaseProof.admitted
      ? "rebased"
      : "preserved",
    hadLocalDraft,
    draftDigest,
    effectiveDigest,
    sourceCompatibilityPosture: observation.sourceCompatibility.posture,
    visibleSelectionKind: observation.resourceSource.visibleSelection.kind,
    blockers: Object.freeze([]),
    messages: Object.freeze([]),
    reason: hadLocalDraft
      ? observation.resourceSource.visibleSelection.rebaseProof.admitted
        ? "remote resource source drift rebased effective truth over the preserved local draft"
        : "remote resource source drift preserved the local draft without admitted resource branch rebase proof"
      : "remote resource source drift preserved source truth because no local draft was active",
  });
}

function normalizeArtifact(artifact, context) {
  if (artifact === null) {
    return null;
  }
  const stale = artifact.currentSourceDigest !== context.currentSourceDigest;
  const resolved = context.latestCanonicalSourceDigest !== null
    && artifact.currentSourceDigest === context.latestCanonicalSourceDigest;
  if (stale === false && resolved === false) {
    return Object.freeze({
      ...artifact,
      stale: false,
      resolved: false,
      resultDigest: stableValueDigest({
        ...artifact,
        stale: false,
        resolved: false,
      }),
    });
  }
  return Object.freeze({
    ...artifact,
    stale,
    resolved,
    resultDigest: stableValueDigest({
      ...artifact,
      stale,
      resolved,
    }),
  });
}
