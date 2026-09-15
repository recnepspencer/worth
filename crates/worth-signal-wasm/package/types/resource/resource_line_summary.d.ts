import type {
  ResourceRequestDescriptor,
  ResourceRequestDiagnostics,
} from "./resource_postures.js";
import type { ResourceEffectEnvelope } from "./resource_effect_envelope.js";
import type {
  ResourceLineDownload,
  ResourceLineDownloadDiagnostics,
  ResourceLineFreshness,
  ResourceLineProcessing,
  ResourceLineStatus,
  ResourceLineUpload,
  ResourceLineUploadDiagnostics,
} from "./resource_lifecycle.js";
import type { ResourcePolicyProfileName } from "./resource_postures.js";
import type { ResourceLineHistoryAvailability } from "./resource_line_history.js";
import type { ResourceLineIdentityMigrationHistoryDigest } from "./resource_line_history.js";
import type { ResourceLineVisibleSelection } from "./resource_line_diagnostics.js";
import type { ResourceMutationResponseTargetOutcomeSummary } from "./resource_mutation_response.js";

export interface ResourceLineDiagnosticsCurrentSummary {
  readonly status: ResourceLineStatus;
  readonly freshness: ResourceLineFreshness;
  readonly hasVisibleValue: boolean;
  readonly visibleValueVersion: number;
  readonly visibleSelection: ResourceLineVisibleSelection;
}

export interface ResourceLineDiagnosticsActivitySummary {
  readonly lastOperation: import("./resource_lifecycle.js").ResourceLineOperation;
  readonly lastOutcome: "fulfilled" | "rejected" | "pending" | "timedOut";
  readonly pendingOperation: import("./resource_lifecycle.js").ResourceLineOperation | null;
  readonly continuity: "preserveVisibleValue";
  readonly freshnessPolicy: ResourcePolicyProfileName;
}

export interface ResourceLineDiagnosticsChangeCountSummary {
  readonly refreshCount: number;
  readonly revalidateCount: number;
  readonly retryAttemptCount: number;
  readonly rejectionCount: number;
  readonly timeoutCount: number;
  readonly supersessionCount: number;
  readonly invalidationCount: number;
  readonly patchCount: number;
  readonly deliveryCount: number;
  readonly basisAdvanceCount: number;
}

export interface ResourceLineDiagnosticsLatestChangeSummary {
  readonly invalidationCause:
    | "deliveryInvalidate"
    | "manualLineInvalidate"
    | "manualFamilyInvalidate"
    | "manualFamilyInvalidateAll"
    | "manualRuntimeInvalidateAll"
    | null;
  readonly invalidationScope: "line" | "familyMember" | "familyAll" | "runtimeAll" | null;
  readonly patchKind:
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
  readonly patchScope:
    | "line"
    | "field"
    | "region"
    | "jsonPath"
    | "item"
    | "aspect"
    | "summary"
    | null;
  readonly patchedItemId: string | null;
  readonly patchedField: string | null;
  readonly patchedRegion: string | null;
  readonly patchedPath: string | null;
  readonly patchedAspect: string | null;
  readonly patchedSummary: string | null;
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
    | "basis"
    | "invalidate"
    | null;
  readonly deliveryPacketId: string | null;
  readonly deliveryBasisId: string | null;
  readonly basisCurrentId: string | null;
  readonly basisAdvanceFromId: string | null;
  readonly basisAdvanceToId: string | null;
  readonly effect: ResourceEffectEnvelope | null;
  readonly supersededOperation: import("./resource_lifecycle.js").ResourceLineOperation | null;
  readonly timeoutOperation: import("./resource_lifecycle.js").ResourceLineOperation | null;
  readonly errorMessage: string | null;
  readonly preservedVisibleValueOnLastRejection: boolean;
  readonly identityMigrationCount?: number;
  readonly lastIdentityMigration?: ResourceLineIdentityMigrationHistoryDigest;
  readonly mutationResponsePlanId?: string;
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
  readonly mutationResponseConfirmationKind?: import("./resource_mutation_response.js").ResourceMutationResponseConfirmationKind;
  readonly mutationResponseConfirmationDigest?: string;
  readonly mutationResponseReplayExactDigest?: string;
  readonly mutationResponseRestoreExactDigest?: string;
  readonly mutationResponseRollbackDigest?: string;
  readonly mutationResponseMergeRebaseDigest?: string;
  readonly mutationResponseExecutionDigest?: string;
  readonly mutationResponseDiagnosticCount?: number;
  readonly mutationResponseDiagnosticDigest?: string;
  readonly mutationResponsePlanCount?: number;
  readonly mutationResponseIdentityMigrationDigest?: string;
  readonly mutationResponseIdentityMigrationNeeded?: boolean;
  readonly mutationResponseIdentityMigrationPartialAdmission?:
    | "notNeeded"
    | "admitted"
    | "denied";
  readonly mutationResponseIdentityMigrationTargetCount?: number;
  readonly mutationResponseIdentityMigrationExactTargetCount?: number;
  readonly mutationResponseIdentityMigrationExecutionDigest?: string;
  readonly mutationResponseIdentityMigrationFallbackDigest?: string;
}

export interface ResourceLineDiagnosticsSummary {
  readonly current: ResourceLineDiagnosticsCurrentSummary;
  readonly activity: ResourceLineDiagnosticsActivitySummary;
  readonly counts: ResourceLineDiagnosticsChangeCountSummary;
  readonly latest: ResourceLineDiagnosticsLatestChangeSummary;
  readonly request: ResourceRequestDiagnostics;
  readonly processing: ResourceLineProcessing;
  readonly upload: ResourceLineUploadDiagnostics;
  readonly download: ResourceLineDownloadDiagnostics;
  readonly explainability: ResourceLineHistoryAvailability;
}

export interface ResourceLineSummary<TParams = unknown> {
  readonly current: ResourceLineDiagnosticsCurrentSummary;
  readonly request: ResourceRequestDescriptor<TParams>;
  readonly processing: ResourceLineProcessing;
  readonly upload: ResourceLineUpload;
  readonly download: ResourceLineDownload;
  readonly diagnostics: ResourceLineDiagnosticsSummary;
  readonly explainability: ResourceLineHistoryAvailability;
}
