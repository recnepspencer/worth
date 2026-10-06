import type {
  HostCapabilityReadSummary,
  HostCapabilityTransportArtifact,
} from "./host_capability.js";

export interface WhySummary {
  id: string;
  node: string;
  apiFamily: string | null;
  recipeFamily: string | null;
  state: string;
  upstream: ReadonlyArray<string>;
  changedRegions: ReadonlyArray<string>;
  propagationSuppressed: boolean;
  outputChange: string | null;
  outputIdentity: string | null;
  callback: CallbackWhySummary | null;
}

export interface CallbackWhySummary {
  purityPosture: string;
  currentReads: ReadonlyArray<string>;
  hostCapabilityReads: ReadonlyArray<HostCapabilityReadSummary>;
  registered: boolean;
  unavailableReason?: string;
  tokenSlot?: number;
  tokenGeneration?: number;
  lastRuntimeReadBreadth: number;
  lastDependencyPatch: CallbackDependencyPatchSummary | null;
  lastFailure: CallbackFailureSummary | null;
}

export interface CallbackRuntimeNodeSummary {
  id: string;
  node: string;
  apiFamily: string | null;
  recipeFamily: string | null;
  purityPosture: string;
  currentReads: ReadonlyArray<string>;
  hostCapabilityReads: ReadonlyArray<HostCapabilityReadSummary>;
  registered: boolean;
  unavailableReason?: string;
  tokenSlot?: number;
  tokenGeneration?: number;
  lastRuntimeReadBreadth: number;
  lastDependencyPatch: CallbackDependencyPatchSummary | null;
  lastFailure: CallbackFailureSummary | null;
}

export interface CallbackDependencyPatchSummary {
  previousReads: ReadonlyArray<string>;
  currentReads: ReadonlyArray<string>;
  addedCount: number;
  removedCount: number;
  retainedCount: number;
  runtimeReadBreadth: number;
}

export interface CallbackFailureSummary {
  class: string;
  message: string;
  code: string | null;
}

export interface UnavailableCallbackArtifact {
  id: string;
  signalKind: string;
  reason: string;
  currentReads: ReadonlyArray<string>;
  hostCapabilityReads: ReadonlyArray<HostCapabilityReadSummary>;
  hostCapabilityTransports: ReadonlyArray<HostCapabilityTransportArtifact>;
}
