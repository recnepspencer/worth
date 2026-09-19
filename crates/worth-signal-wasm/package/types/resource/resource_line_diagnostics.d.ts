import type {
  ResourceLineDownloadDiagnostics,
  ResourceLineOperation,
  ResourceLineProcessing,
  ResourceLineUploadDiagnostics,
} from "./resource_lifecycle.js";
import type { ResourceEffectEnvelope } from "./resource_effect_envelope.js";
import type {
  ResourcePolicyProfileName,
  ResourceRequestDiagnostics,
} from "./resource_postures.js";
import type { ResourceMutationResponsePlan } from "./resource_mutation_response.js";
import type { ResourceLineIdentityMigrationHistoryDigest } from "./resource_line_history.js";

export interface ResourceLineBasisDiagnostics {
  readonly currentBasisId: string | null;
  readonly advanceCount: number;
  readonly lastAdvanceFromBasisId: string | null;
  readonly lastAdvanceToBasisId: string | null;
}

export type ResourceLineVisibleSelection =
  | {
      readonly kind: "unavailable";
      readonly source: "initialLoad";
      readonly effectId: null;
      readonly branchId: string | number | null;
      readonly snapshotId: null;
      readonly basisId: string | null;
      readonly detail: string;
    }
  | {
      readonly kind: "committed";
      readonly source:
        | "initialLoad"
        | "refresh"
        | "revalidate"
        | "localPatch"
        | "optimismUnavailable";
      readonly effectId: string | null;
      readonly branchId: string | number | null;
      readonly snapshotId: null;
      readonly basisId: string | null;
      readonly unavailableReason?: string;
      readonly detail: string;
    }
  | {
      readonly kind: "speculative";
      readonly source: "localPatch";
      readonly effectId: string;
      readonly branchId: number;
      readonly snapshotId: number;
      readonly basisId: string | null;
      readonly rollbackKind: ResourceEffectEnvelope["optimistic"]["rollback"]["kind"];
      readonly detail: string;
    }
  | {
      readonly kind: "confirmed";
      readonly source: "delivery";
      readonly effectId: string;
      readonly branchId: string | number | null;
      readonly snapshotId: null;
      readonly basisId: string | null;
      readonly confirmationKind: string;
      readonly previousEffectId: string | null;
      readonly detail: string;
    }
  | {
      readonly kind: "restored";
      readonly source: "historyRestore" | "exactBranchRestore" | "compactInverse";
      readonly effectId: string | null;
      readonly branchId: string | number | null;
      readonly snapshotId: number | null;
      readonly basisId: string | null;
      readonly rollbackKind: ResourceEffectEnvelope["optimistic"]["rollback"]["kind"] | null;
      readonly detail: string;
    };

export interface ResourceLineDiagnostics {
  readonly policyProfileName: ResourcePolicyProfileName;
  readonly continuity: "preserveVisibleValue";
  readonly freshnessPolicy: ResourcePolicyProfileName;
  readonly request: ResourceRequestDiagnostics;
  readonly basis: ResourceLineBasisDiagnostics;
  readonly processing: ResourceLineProcessing;
  readonly upload: ResourceLineUploadDiagnostics;
  readonly download: ResourceLineDownloadDiagnostics;
  readonly visibleSelection: ResourceLineVisibleSelection;
  readonly lastOperation: ResourceLineOperation;
  readonly lastOutcome: "fulfilled" | "rejected" | "pending" | "timedOut";
  readonly pendingOperation: ResourceLineOperation | null;
  readonly refreshCount: number;
  readonly revalidateCount: number;
  readonly retryAttemptCount: number;
  readonly rejectionCount: number;
  readonly timeoutCount: number;
  readonly supersessionCount: number;
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
  readonly preservedVisibleValueOnLastRejection: boolean;
  readonly lastTimeoutOperation: ResourceLineOperation | null;
  readonly lastErrorMessage: string | null;
  readonly visibleValueVersion: number;
  readonly lastMutationResponsePlan?: ResourceMutationResponsePlan;
  readonly mutationResponsePlanCount?: number;
  readonly identityMigrationCount?: number;
  readonly lastIdentityMigration?: ResourceLineIdentityMigrationHistoryDigest;
}
