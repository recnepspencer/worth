import type {
  LineageSummary,
  ReplaySummary,
} from "../diagnostics.js";
import type { ResourceLineVerificationPackage } from "./resource_verification.js";
import type { ResourceEffectEnvelope } from "./resource_effect_envelope.js";
import type { ResourceEffectSettlementResult } from "./resource_effect_branch_dag.js";
import type {
  ResourceLineContinuity,
  ResourceLineFreshness,
  ResourceLineOperation,
  ResourceLineStatus,
} from "./resource_lifecycle.js";
import type { ResourceLineVisibleSelection } from "./resource_line_diagnostics.js";
import type { ResourceMutationResponsePlan } from "./resource_mutation_response.js";
import type { ResourceMutationResponseTargetOutcomeSummary } from "./resource_mutation_response.js";

export type ResourceLineHistoryEvent =
  | "materialized"
  | "identityMigrated"
  | "mutationResponsePlanned"
  | "pending"
  | "superseded"
  | "patched"
  | "delivered"
  | "replayed"
  | "restored"
  | "fulfilled"
  | "rejected"
  | "timedOut"
  | "invalidated";

export interface ResourceLineIdentityMigrationHistoryDigest {
  readonly previousCanonicalKey: string;
  readonly nextCanonicalKey: string;
  readonly previousRuntimeLineId: string | null;
  readonly nextRuntimeLineId: string | null;
  readonly basisId: string | null;
  readonly requestPath: string | null;
}

export interface ResourceLineHistoryEntry {
  readonly sequence: number;
  readonly event: ResourceLineHistoryEvent;
  readonly status: ResourceLineStatus;
  readonly freshness: ResourceLineFreshness;
  readonly lastOperation: ResourceLineOperation;
  readonly lastOutcome: "fulfilled" | "rejected" | "pending" | "timedOut";
  readonly pendingOperation: ResourceLineOperation | null;
  readonly statusContinuity: ResourceLineContinuity | null;
  readonly retryAttemptCount: number;
  readonly rejectionCount: number;
  readonly timeoutCount: number;
  readonly supersessionCount: number;
  readonly supersededOperation: ResourceLineOperation | null;
  readonly invalidationCount: number;
  readonly patchCount: number;
  readonly deliveryCount: number;
  readonly lastSupersededOperation: ResourceLineOperation | null;
  readonly lastInvalidationCause:
    | "deliveryInvalidate"
    | "manualLineInvalidate"
    | "manualFamilyInvalidate"
    | "manualFamilyInvalidateAll"
    | "manualRuntimeInvalidateAll"
    | null;
  readonly lastInvalidationScope: "line" | "familyMember" | "familyAll" | "runtimeAll" | null;
  readonly lastPatchKind:
    | "replace"
    | "field"
    | "region"
    | "jsonPath"
    | "item"
    | "delete"
    | "insert"
    | "itemAspect"
    | "summary"
    | null;
  readonly lastPatchScope:
    | "line"
    | "field"
    | "region"
    | "jsonPath"
    | "item"
    | "aspect"
    | "summary"
    | null;
  readonly lastPatchedItemId: string | null;
  readonly lastPatchedField: string | null;
  readonly lastPatchedRegion: string | null;
  readonly lastPatchedPath: string | null;
  readonly lastPatchedAspect: string | null;
  readonly lastPatchedSummary: string | null;
  readonly lastDeliveryKind:
    | "replace"
    | "patch"
    | "invalidate"
    | "basisRefresh"
    | null;
  readonly lastDeliveryScope:
    | "line"
    | "field"
    | "region"
    | "jsonPath"
    | "item"
    | "aspect"
    | "summary"
    | "basis"
    | "invalidate"
    | null;
  readonly lastDeliveryPacketId: string | null;
  readonly lastDeliveryBasisId: string | null;
  readonly lastEffect: ResourceEffectEnvelope | null;
  readonly visibleSelection: ResourceLineVisibleSelection;
  readonly currentBasisId: string | null;
  readonly basisAdvanceCount: number;
  readonly lastBasisAdvanceFromId: string | null;
  readonly lastBasisAdvanceToId: string | null;
  readonly downloadCount: number;
  readonly readyDownloadCount: number;
  readonly unavailableDownloadCount: number;
  readonly incompatibleDownloadCount: number;
  readonly preservedVisibleValueOnLastRejection: boolean;
  readonly lastTimeoutOperation: ResourceLineOperation | null;
  readonly lastErrorMessage: string | null;
  readonly visibleValueVersion: number;
  readonly mutationResponsePlan?: ResourceMutationResponsePlan;
  readonly mutationResponsePlanCount?: number;
  readonly mutationResponseTargetCount?: number;
  readonly mutationResponseExactTargetCount?: number;
  readonly mutationResponseFallbackTargetCount?: number;
  readonly mutationResponseTargetLookupBreadth?: number;
  readonly mutationResponseTargetFanoutBreadth?: number;
  readonly mutationResponsePayloadFieldExtractionBreadth?: number;
  readonly mutationResponseTopologyTraversalBreadth?: number;
  readonly mutationResponseReconstructionBreadth?: number;
  readonly mutationResponseFallbackBreadth?: number;
  readonly mutationResponseFallbackReasonDigest?: string;
  readonly mutationResponseFallbackAffectedTargetDigest?: string;
  readonly mutationResponseStaleTargetReasonDigest?: string;
  readonly mutationResponseStaleTargetAffectedTargetDigest?: string;
  readonly mutationResponseFreshnessPostureDigest?: string;
  readonly mutationResponseDeliveryAwaitedDigest?: string;
  readonly mutationResponseRefetchRequiredDigest?: string;
  readonly mutationResponsePartialReconciliationDigest?: string;
  readonly mutationResponseUnsupportedTargetDigest?: string;
  readonly mutationResponseNoHiddenMutationDigest?: string;
  readonly mutationResponseTargetOutcomeDigest?: string;
  readonly mutationResponseTargetOutcomes?: readonly ResourceMutationResponseTargetOutcomeSummary[];
  readonly mutationResponseReplayExactDigest?: string;
  readonly mutationResponseRestoreExactDigest?: string;
  readonly identityMigration?: ResourceLineIdentityMigrationHistoryDigest;
}

export interface ResourceLineBranchSummary {
  readonly id: number;
  readonly name: string;
  readonly parentBranchId: number | null;
  readonly headSnapshotId: number | null;
}

export interface ResourceLineBasisAdvance {
  readonly sequence: number;
  readonly event: ResourceLineHistoryEvent;
  readonly operation: ResourceLineOperation;
  readonly deliveryKind:
    | "replace"
    | "patch"
    | "invalidate"
    | "basisRefresh"
    | null;
  readonly deliveryScope:
    | "line"
    | "field"
    | "region"
    | "jsonPath"
    | "item"
    | "aspect"
    | "summary"
    | "invalidate"
    | null;
  readonly deliveryPacketId: string | null;
  readonly deliveryBasisId: string | null;
  readonly fromBasisId: string | null;
  readonly toBasisId: string | null;
  readonly currentBasisId: string | null;
}

export interface ResourceLineBasisHistory {
  readonly currentBasisId: string | null;
  readonly advanceCount: number;
  readonly lastAdvanceFromId: string | null;
  readonly lastAdvanceToId: string | null;
  readonly advances: readonly ResourceLineBasisAdvance[];
}

export interface ResourceLineHistoryArtifactAvailable {
  readonly kind: "available";
}

export interface ResourceLineHistoryArtifactUnavailable {
  readonly kind: "unavailable";
  readonly reason: "unsupportedByRuntime" | "runtimeRejected";
  readonly detail: string;
}

export type ResourceLineHistoryArtifactAvailability =
  | ResourceLineHistoryArtifactAvailable
  | ResourceLineHistoryArtifactUnavailable;

export interface ResourceLineReplayAvailable {
  readonly kind: "available";
  readonly mode: "SameRuntimeSignalExact";
  readonly signalId: string;
}

export interface ResourceLineReplayUnavailable {
  readonly kind: "unavailable";
  readonly reason:
    | "unsupportedByRuntime"
    | "runtimeRejected"
    | "identityMigrationUnavailable";
  readonly detail: string;
}

export type ResourceLineReplayAvailability =
  | ResourceLineReplayAvailable
  | ResourceLineReplayUnavailable;

export interface ResourceLineRestoreAvailable {
  readonly kind: "available";
  readonly mode: "SameRuntimeBranchExact";
  readonly branchId: number;
  readonly snapshotId: number;
}

export interface ResourceLineRestoreUnavailable {
  readonly kind: "unavailable";
  readonly reason:
    | "unsupportedByRuntime"
    | "branchHeadUnavailable"
    | "runtimeRejected"
    | "identityMigrationUnavailable";
  readonly detail: string;
}

export type ResourceLineRestoreAvailability =
  | ResourceLineRestoreAvailable
  | ResourceLineRestoreUnavailable;

export interface ResourceLineExactRestoreResultRestored {
  readonly kind: "restored";
  readonly mode: "SameRuntimeBranchExact";
  readonly branchId: number;
  readonly snapshotId: number;
  readonly basisCurrentId: string | null;
  readonly basisAdvanceCount: number;
  readonly reloadStatus: ResourceLineStatus;
}

export interface ResourceLineExactRestoreResultUnavailable {
  readonly kind: "unavailable";
  readonly reason:
    | "unsupportedByRuntime"
    | "branchHeadUnavailable"
    | "runtimeRejected"
    | "identityMigrationUnavailable";
  readonly detail: string;
  readonly basisCurrentId: string | null;
  readonly basisAdvanceCount: number;
}

export type ResourceLineExactRestoreResult =
  | ResourceLineExactRestoreResultRestored
  | ResourceLineExactRestoreResultUnavailable;

export interface ResourceLineEffectRollbackResultUnavailable {
  readonly kind: "unavailable";
  readonly reason:
    | "noOpenEffect"
    | "unknownEffect"
    | "effectAlreadySettled";
  readonly detail: string;
  readonly effectId: string | null;
  readonly basisCurrentId: string | null;
  readonly basisAdvanceCount: number;
}

export type ResourceLineEffectRollbackResult<TValue = unknown> =
  | ResourceEffectSettlementResult<TValue>
  | ResourceLineEffectRollbackResultUnavailable;

export interface ResourceLineExactReplayResultReplayed {
  readonly kind: "replayed";
  readonly mode: "SameRuntimeSignalExact";
  readonly signalId: string;
  readonly basisCurrentId: string | null;
  readonly basisAdvanceCount: number;
  readonly reloadStatus: ResourceLineStatus;
}

export interface ResourceLineExactReplayResultUnavailable {
  readonly kind: "unavailable";
  readonly reason:
    | "unsupportedByRuntime"
    | "runtimeRejected"
    | "identityMigrationUnavailable";
  readonly detail: string;
  readonly basisCurrentId: string | null;
  readonly basisAdvanceCount: number;
}

export type ResourceLineExactReplayResult =
  | ResourceLineExactReplayResultReplayed
  | ResourceLineExactReplayResultUnavailable;

export interface ResourceLineHistoryAvailability {
  readonly replay: ResourceLineHistoryArtifactAvailability;
  readonly replayExact: ResourceLineReplayAvailability;
  readonly lineage: ResourceLineHistoryArtifactAvailability;
  readonly branch: ResourceLineHistoryArtifactAvailability;
  readonly restoreExact: ResourceLineRestoreAvailability;
}

export interface ResourceLineHistory {
  readonly replay: ReplaySummary | null;
  readonly lineage: LineageSummary | null;
  readonly branch: ResourceLineBranchSummary | null;
  readonly basis: ResourceLineBasisHistory;
  readonly availability: ResourceLineHistoryAvailability;
  readonly lifecycle: readonly ResourceLineHistoryEntry[];
  replayExact(): ResourceLineExactReplayResult;
  restoreExact(): ResourceLineExactRestoreResult;
  rollbackEffect(effectId: string): Promise<ResourceLineEffectRollbackResult>;
  rollbackLastEffect(): Promise<ResourceLineEffectRollbackResult>;
  verificationPackage(): ResourceLineVerificationPackage;
}
