import type {
  BranchStateProofReport, LineageSummary, MergePlanArtifact, MergePlanProofEnvelope,
  MergePolicyPreviewRequest, MergeResultArtifact, MergeResultProofEnvelope,
  ReplayArtifactProofInput, ReplayArtifactProofReport, ReplayParityProofReport, ReplaySummary,
  RuntimeBranchHandle, RuntimeDefinitionEnvelope, RuntimeEnvelope, HostCapabilityTransportReport,
  RuntimeProofReport, RuntimeSnapshotArtifact, RuntimeSnapshotEnvelope,
} from "../diagnostics.js";

export interface CallableSignalAdapters {
  exportDefinitions(): RuntimeDefinitionEnvelope;
  exportRuntimeEnvelope(): RuntimeEnvelopeArtifact;
  replaceRuntimeEnvelope(envelope: RuntimeEnvelope): Promise<void>;
  restoreExactRuntimeEnvelope(envelope: RuntimeEnvelopeArtifact): Promise<void>;
  /** Release the artifact's pending exact restore token; `false` when it is no longer pending. */
  discardExactRuntimeEnvelope(envelope: RuntimeEnvelopeArtifact): boolean | Promise<boolean>;
  runtimeProofReport(): RuntimeProofReport;
  hostCapabilityTransportReport(envelope?: RuntimeEnvelope): HostCapabilityTransportReport;
  free(): void;
  [Symbol.dispose](): void;
}

export type CallableBranchId = number | bigint;

export interface RuntimeEnvelopeArtifact extends RuntimeEnvelope {
  runtimeEnvelopeRestoreToken: string;
  runtimeEnvelopeRestoreMode: "SameRuntimeExact";
  runtimeEnvelopePortableWire: string;
}

export interface RuntimeSnapshotEnvelopeArtifact extends RuntimeSnapshotEnvelope {
  snapshotEnvelopeRestoreToken: string;
  snapshotEnvelopeRestoreMode: "SameRuntimeExact";
  snapshotEnvelopePortableWire: string;
}

export interface RuntimeSnapshotArtifactWithWire extends RuntimeSnapshotArtifact {
  snapshotRestoreToken: string;
  snapshotRestoreMode: "SameRuntimeExact";
  snapshotPortableWire: string;
}

export interface CallableSignalHistory {
  replay_for(id: string): ReplaySummary;
  lineage_for(id: string): LineageSummary;
  snapshot(): RuntimeSnapshotEnvelopeArtifact;
  restore_snapshot(snapshot: RuntimeSnapshotEnvelope): void | Promise<void>;
  restore_exact_snapshot(snapshot: RuntimeSnapshotEnvelopeArtifact): void | Promise<void>;
  /** Release the artifact's pending exact restore token; `false` when it is no longer pending. */
  discard_exact_artifact(
    artifact: RuntimeSnapshotEnvelopeArtifact | RuntimeSnapshotArtifactWithWire,
  ): boolean | Promise<boolean>;
  current_branch(): RuntimeBranchHandle;
  branches(): ReadonlyArray<RuntimeBranchHandle>;
  create_branch(name: string): RuntimeBranchHandle | Promise<RuntimeBranchHandle>;
  switch_branch(branchId: CallableBranchId): void | Promise<void>;
  replay_for_branch(branchId: CallableBranchId): ReplaySummary;
  branch_snapshot(branchId: CallableBranchId): RuntimeSnapshotArtifactWithWire;
  branch_snapshot_id(branchId: CallableBranchId): bigint;
  branch_snapshot_envelope(branchId: CallableBranchId): RuntimeSnapshotEnvelopeArtifact;
  restore_branch_snapshot(
    branchId: CallableBranchId,
    snapshot: RuntimeSnapshotArtifact,
  ): void | Promise<void>;
  restore_exact_branch_snapshot(
    branchId: CallableBranchId,
    snapshot: RuntimeSnapshotArtifactWithWire,
  ): void | Promise<void>;
  restore_branch_snapshot_by_id(
    branchId: CallableBranchId,
    snapshotId: number | bigint,
  ): void | Promise<void>;
  merge_branches(
    sourceBranchId: CallableBranchId,
    targetBranchId: CallableBranchId,
  ): MergeResultArtifact | Promise<MergeResultArtifact>;
  merge_branches_with_proof(
    sourceBranchId: CallableBranchId,
    targetBranchId: CallableBranchId,
  ): MergeResultProofEnvelope | Promise<MergeResultProofEnvelope>;
  plan_merge_branches(
    sourceBranchId: CallableBranchId,
    targetBranchId: CallableBranchId,
  ): MergePlanArtifact | Promise<MergePlanArtifact>;
  plan_merge_branches_with_proof(
    sourceBranchId: CallableBranchId,
    targetBranchId: CallableBranchId,
  ): MergePlanProofEnvelope | Promise<MergePlanProofEnvelope>;
  plan_merge_policy_preview(
    request: MergePolicyPreviewRequest,
  ): MergePlanArtifact | Promise<MergePlanArtifact>;
  plan_merge_policy_preview_with_proof(
    request: MergePolicyPreviewRequest,
  ): MergePlanProofEnvelope | Promise<MergePlanProofEnvelope>;
  merge_branches_policy_preview(
    request: MergePolicyPreviewRequest,
  ): MergeResultArtifact | Promise<MergeResultArtifact>;
  merge_branches_policy_preview_with_proof(
    request: MergePolicyPreviewRequest,
  ): MergeResultProofEnvelope | Promise<MergeResultProofEnvelope>;
  branch_state_proof(branchId: CallableBranchId): BranchStateProofReport;
  replay_parity_proof(
    expectedBranchId: CallableBranchId,
    replayedBranchId: CallableBranchId,
  ): ReplayParityProofReport;
  replay_artifact_proof(
    expected: ReplayArtifactProofInput,
    replayedBranchId: CallableBranchId,
  ): ReplayArtifactProofReport;
  subscribe(listener: () => void): () => void;
  free(): void;
  [Symbol.dispose](): void;
}
