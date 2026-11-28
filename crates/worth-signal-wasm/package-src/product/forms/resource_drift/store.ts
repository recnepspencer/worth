import { stableValueDigest } from "../values/value_paths.js";

export function createResourceDriftStore() {
  let nextArtifactId = 1;
  let lastObservedSourceDigest = null;
  let activePreviousSourceDigest = null;
  let activeCurrentSourceDigest = null;
  let latestObservationDigest = null;
  let current = null;
  const history = [];

  return Object.freeze({
    observe(snapshot) {
      if (lastObservedSourceDigest === null) {
        lastObservedSourceDigest = snapshot.currentSourceDigest;
        return null;
      }
      if (lastObservedSourceDigest !== snapshot.currentSourceDigest) {
        activePreviousSourceDigest = lastObservedSourceDigest;
        activeCurrentSourceDigest = snapshot.currentSourceDigest;
        latestObservationDigest = null;
      }
      lastObservedSourceDigest = snapshot.currentSourceDigest;
      if (activePreviousSourceDigest === null || activeCurrentSourceDigest === null) {
        return null;
      }
      const artifact = Object.freeze({
        kind: "resourceDriftObservation",
        artifactId: nextArtifactId++,
        observedAtMs: Date.now(),
        previousSourceDigest: activePreviousSourceDigest,
        currentSourceDigest: activeCurrentSourceDigest,
        ...snapshot,
      });
      const observationDigest = stableValueDigest({
        previousSourceDigest: artifact.previousSourceDigest,
        currentSourceDigest: artifact.currentSourceDigest,
        status: artifact.status,
        hadLocalDraft: artifact.hadLocalDraft,
        draftDigest: artifact.draftDigest,
        effectiveDigest: artifact.effectiveDigest,
        sourceCompatibilityPosture: artifact.sourceCompatibilityPosture,
        visibleSelectionKind: artifact.visibleSelectionKind,
        blockers: artifact.blockers,
        messages: artifact.messages,
        reason: artifact.reason,
      });
      if (observationDigest === latestObservationDigest) {
        return current;
      }
      latestObservationDigest = observationDigest;
      current = artifact;
      history.push(artifact);
      return artifact;
    },
    current() {
      return current;
    },
    history() {
      return Object.freeze([...history]);
    },
  });
}
