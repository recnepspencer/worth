export interface ResourceMutationResponseRollbackProof {
  readonly kind:
    | "exactBranchRestoreAvailable"
    | "compactInverseAvailable"
    | "unavailable"
    | "identityMigrationUnavailable"
    | "notApplicable"
    | "fallbackUnavailable"
    | "awaitingExecution";
  readonly mode: "SameRuntimeBranchExact" | "CompactInversePatch" | null;
  readonly branchId: number | null;
  readonly snapshotId: number | null;
  readonly inverseKind: string | null;
  readonly detail: string;
}

export interface ResourceMutationResponseMergeRebaseProof {
  readonly kind:
    | "unavailable"
    | "identityMigrationUnavailable"
    | "notApplicable"
    | "fallbackUnavailable"
    | "awaitingExecution";
  readonly granularity: string;
  readonly locusKind?: string;
  readonly locusProofDigest?: string | null;
  readonly detail: string;
}

export interface ResourceMutationResponseReplayExactProof {
  readonly kind:
    | "available"
    | "identityMigrationUnavailable"
    | "fallbackUnavailable"
    | "awaitingExecution";
  readonly mode: "SameRuntimeSignalExact" | null;
  readonly detail: string;
}

export interface ResourceMutationResponseRestoreExactProof {
  readonly kind:
    | "available"
    | "identityMigrationUnavailable"
    | "fallbackUnavailable"
    | "awaitingExecution";
  readonly mode: "SameRuntimeBranchExact" | null;
  readonly detail: string;
}

export interface ResourceMutationResponseTargetEffectProof {
  readonly effectId: string;
  readonly authorityDigest: string;
  readonly rollback: ResourceMutationResponseRollbackProof;
  readonly mergeRebase: ResourceMutationResponseMergeRebaseProof;
  readonly branchLifecycleKind: string;
  readonly optimisticKind: string;
  readonly locusKind: string;
  readonly locusProofDigest: string | null;
  readonly digest: string;
}

export interface ResourceMutationResponseLifecycleProofEntry {
  readonly entryKind: "reconciliation" | "identityMigration";
  readonly targetId: string;
  readonly effectId: string | null;
  readonly authorityDigest: string | null;
  readonly replayExact: ResourceMutationResponseReplayExactProof;
  readonly restoreExact: ResourceMutationResponseRestoreExactProof;
  readonly rollback: ResourceMutationResponseRollbackProof;
  readonly mergeRebase: ResourceMutationResponseMergeRebaseProof;
  readonly digest: string;
}

export interface ResourceMutationResponseLifecycleProof {
  readonly entries: readonly ResourceMutationResponseLifecycleProofEntry[];
  readonly count: number;
  readonly replayExactDigest: string;
  readonly restoreExactDigest: string;
  readonly rollbackDigest: string;
  readonly mergeRebaseDigest: string;
  readonly digest: string;
}
