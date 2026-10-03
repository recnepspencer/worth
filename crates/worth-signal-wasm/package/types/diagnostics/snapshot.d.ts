import type { KeyedRecipeFamilySpec, KeyedSourceFamilySpec, RecipeSpec, SignalValue, SourceSpec } from "../model.js";
import type {
  ArtifactRetentionPolicy,
  RuntimePolicySpec,
  SignalRuntimePolicySummary,
} from "./policy.js";
import type {
  UnavailableCallbackArtifact,
} from "./callback.js";
import type {
  HostCapabilityReadSummary,
} from "./host_capability.js";

export interface RuntimeDefinitionEnvelope {
  policy: RuntimePolicySpec;
  sources: ReadonlyArray<SourceSpec>;
  recipes: ReadonlyArray<RecipeSpec>;
  sourceFamilies: ReadonlyArray<KeyedSourceFamilySpec>;
  recipeFamilies: ReadonlyArray<KeyedRecipeFamilySpec>;
  unavailableCallbacks: ReadonlyArray<UnavailableCallbackArtifact>;
}

export interface RuntimeSnapshotAspectVersionSummary {
  aspect: number;
  version: number;
}

export interface StoredSourceSnapshot {
  id: string;
  value: SignalValue;
  version: number;
  producesAspects?: ReadonlyArray<number>;
  aspectVersions: ReadonlyArray<RuntimeSnapshotAspectVersionSummary>;
}

export interface StoredCallbackRecipeSnapshot {
  tokenSlot: number;
  tokenGeneration: number;
  reads: ReadonlyArray<string>;
  hostCapabilityReads: ReadonlyArray<HostCapabilityReadSummary>;
}

export interface StoredRecipeSnapshot {
  id: string;
  value: SignalValue;
  version: number;
  producesAspects?: ReadonlyArray<number>;
  aspectVersions: ReadonlyArray<RuntimeSnapshotAspectVersionSummary>;
  initialized: boolean;
  outputIdentity: string | null;
  callback?: StoredCallbackRecipeSnapshot | null;
}

export interface RuntimeStoreSnapshot {
  sources: ReadonlyArray<StoredSourceSnapshot>;
  recipes: ReadonlyArray<StoredRecipeSnapshot>;
}

export interface RuntimeSnapshotArtifactRetentionPolicy {
  explanation_retention: ArtifactRetentionPolicy;
  provenance_retention: ArtifactRetentionPolicy;
}

export interface RuntimeCheckpointImageArtifact extends Readonly<Record<string, unknown>> {}
export interface RuntimeDiagnosticGraphArtifact extends Readonly<Record<string, unknown>> {}
export interface RuntimeSnapshotDiagnosticsArtifact extends Readonly<Record<string, unknown>> {}
export interface RuntimeTelemetryArtifact extends Readonly<Record<string, unknown>> {}
export interface RuntimeReconstructabilityArtifact extends Readonly<Record<string, unknown>> {}

export interface RuntimeSnapshotMeta {
  schema_version: number;
  snapshot_id: number;
  branch_id: number;
  branch_name: string;
  core_storage_profile: string;
  replay_head: number | null;
  runtime_policy: SignalRuntimePolicySummary;
  artifact_retention: RuntimeSnapshotArtifactRetentionPolicy;
}

export interface RuntimeSnapshotArtifact {
  meta: RuntimeSnapshotMeta;
  checkpoint_image: RuntimeCheckpointImageArtifact;
  diagnostic_graph: RuntimeDiagnosticGraphArtifact;
  diagnostics: RuntimeSnapshotDiagnosticsArtifact;
  graph_telemetry: RuntimeTelemetryArtifact;
  runtime_telemetry?: RuntimeTelemetryArtifact;
  reconstructability?: RuntimeReconstructabilityArtifact;
}

export interface RuntimeSnapshotEnvelope {
  snapshot: RuntimeSnapshotArtifact;
  state: RuntimeStoreSnapshot;
}

export interface RuntimeEnvelope {
  definitions: RuntimeDefinitionEnvelope;
  snapshot: RuntimeSnapshotEnvelope;
}
