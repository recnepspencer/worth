import type {
  CallbackRuntimeNodeSummary,
} from "./callback.js";

export interface RuntimeProofReport {
  proofSchemaVersion: string;
  schemaRegistryDigest: string;
  mergeStrategyRegistryDigest: string;
  mergeBaseStrategyRegistryDigest: string;
  aspectMergePolicyRegistryDigest: string;
  conflictIsolationRegistryDigest: string;
  conflictPolicyRegistryDigest: string;
  identityMatcherRegistryDigest: string;
  sourceOnlyPolicyRegistryDigest: string;
  deletionPolicyRegistryDigest: string;
  registryBundleDigest: string;
}

export interface RuntimeBranchHandle {
  id: number;
  name: string;
  parent_branch_id: number | null;
  head_snapshot_id: number | null;
}

export interface ReplayFrameSummary {
  cursor: number;
  kind: string;
  branchId: number;
  snapshotId: number | null;
  node: string | null;
  detail: string | null;
  callback?: CallbackRuntimeNodeSummary | null;
}

export interface ReplaySummary {
  frames: ReadonlyArray<ReplayFrameSummary>;
}

export interface LineageEventSummary {
  sequence: number;
  label: string;
  emittedOnBranchId: number;
  node: string | null;
  subjectArtifactId: number | null;
  parentArtifactId: number | null;
  snapshotId: number | null;
  callback?: CallbackRuntimeNodeSummary | null;
}

export interface LineageSummary {
  events: ReadonlyArray<LineageEventSummary>;
}

export type ReplayMismatchClass =
  | "LegacyMergeArtifactUnsupported"
  | "ProofSchemaVersionMismatch"
  | "MissingRegistryBundleDigest"
  | "RegistryBundleDigestMismatch"
  | "MissingLoweredStrategyBundleDigest"
  | "LoweredStrategyBundleDigestMismatch"
  | "MissingMergePlanDigest"
  | "MergePlanDigestMismatch"
  | "MissingMergeResultDigest"
  | "MergeResultDigestMismatch"
  | "MissingLineageDigest"
  | "LineageDigestMismatch"
  | "BranchStateDigestMismatch";

export interface BranchStateProofReport {
  proofSchemaVersion: string;
  branchId: number;
  branchName: string;
  snapshotId: number | null;
  stateDigest: string;
}

export interface ReplayArtifactProofInput {
  proofSchemaVersion: string;
  registryBundleDigest: string | null;
  loweredStrategyBundleDigest: string | null;
  mergePlanDigest: string | null;
  mergeResultDigest: string | null;
  lineageDigest: string | null;
  branchStateDigest: string;
}

export interface ReplayParityProofReport {
  proofSchemaVersion: string;
  expectedBranchId: number;
  expectedBranchName: string;
  expectedSnapshotId: number | null;
  expectedStateDigest: string;
  replayedBranchId: number;
  replayedBranchName: string;
  replayedSnapshotId: number | null;
  replayedStateDigest: string;
  parity: boolean;
  mismatchClasses: ReadonlyArray<ReplayMismatchClass>;
}

export interface ReplayArtifactProofReport {
  proofSchemaVersion: string;
  expected: ReplayArtifactProofInput;
  replayed: ReplayArtifactProofInput;
  parity: boolean;
  mismatchClasses: ReadonlyArray<ReplayMismatchClass>;
}
