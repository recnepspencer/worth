import type { FormReadinessBlocker } from "./core.js";
import type { FormResourceVisibleSelectionKind } from "./resource_source.js";
import type { FormMessageArtifact } from "./validation.js";

export type FormResourceDriftStatus =
  | "preserved"
  | "rebased"
  | "blocked"
  | "conflict";

export interface FormResourceDriftArtifact {
  readonly kind: "resourceDriftObservation";
  readonly artifactId: number;
  readonly observedAtMs: number;
  readonly sourceKind: "resourceLine";
  readonly previousSourceDigest: string;
  readonly currentSourceDigest: string;
  readonly status: FormResourceDriftStatus;
  readonly stale: boolean;
  readonly resolved: boolean;
  readonly hadLocalDraft: boolean;
  readonly draftDigest: string;
  readonly effectiveDigest: string;
  readonly sourceCompatibilityPosture:
    | "notDeclared"
    | "current"
    | "compatible"
    | "migrated"
    | "unavailable";
  readonly visibleSelectionKind: FormResourceVisibleSelectionKind;
  readonly blockers: ReadonlyArray<FormReadinessBlocker>;
  readonly messages: ReadonlyArray<FormMessageArtifact>;
  readonly reason: string;
  readonly resultDigest: string;
}

export interface FormResourceDriftReport {
  readonly current: FormResourceDriftArtifact | null;
  readonly history: ReadonlyArray<FormResourceDriftArtifact>;
  readonly summary: {
    readonly status: FormResourceDriftStatus | "ready";
    readonly stale: boolean;
    readonly resolved: boolean;
    readonly hadLocalDraft: boolean;
    readonly blockerCount: number;
    readonly messageCount: number;
  };
  readonly counters: {
    readonly costBasis: "resourceDriftHistoryScan";
    readonly incrementalStatus: "notIncremental";
    readonly observedChanges: number;
    readonly preservedChanges: number;
    readonly rebasedChanges: number;
    readonly blockedChanges: number;
    readonly conflictedChanges: number;
    readonly staleChanges: number;
    readonly resolvedChanges: number;
    readonly blockers: number;
    readonly messages: number;
  };
  readonly digest: string;
}
