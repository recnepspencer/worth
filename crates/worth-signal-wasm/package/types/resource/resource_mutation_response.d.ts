import type {
  ResourceMutationResponseSubmittedTargetBasis,
  ResourceMutationResponseTargetStaleness,
} from "./resource_mutation_response_target_basis.js";
import type {
  ResourceMutationResponseLifecycleProof,
  ResourceMutationResponseTargetEffectProof,
} from "./resource_mutation_response_lifecycle_proof.js";
import type {
  ResourceMutationResponseAnyIdentityTargetDeclaration,
  ResourceMutationResponseAnyCreateTargetDeclaration,
  ResourceMutationResponseAnyTargetDeclaration,
  ResourceMutationResponseAtomicity,
  ResourceMutationResponseCollectionReconciliationDeclaration,
  ResourceMutationResponseDetailReconciliationDeclaration,
  ResourceMutationResponseDiagnosticDeclaration,
  ResourceMutationResponseFallbackTargetDeclaration,
  ResourceMutationResponseIdentityAtomicity,
  ResourceMutationResponseIdentityDeclaration,
  ResourceMutationResponseIdentityDetailChildTargetScope,
  ResourceMutationResponseIdentitySelectionTargetScope,
  ResourceMutationResponseIdentitySummaryTargetScope,
  ResourceMutationResponseIdentityTargetDeclaration,
  ResourceMutationResponseResidentLineIdentityTargetDeclaration,
  ResourceMutationResponseSelectionIdentityTargetDeclaration,
  ResourceMutationResponseSummaryIdentityTargetDeclaration,
  ResourceMutationResponseSummaryReconciliationDeclaration,
  ResourceMutationResponseTargetDeclaration,
  ResourceMutationResponseTargetFamily,
  ResourceMutationResponseTargetFamilyKind,
  ResourceMutationResponseTargetFamilyParams,
} from "./resource_mutation_response_authoring.js";
import type {
  ResourceMutationResponseExecutionArtifact,
  ResourceMutationResponseExactCollectionDeleteExecutionArtifact,
  ResourceMutationResponseExactCollectionInsertExecutionArtifact,
  ResourceMutationResponseExactCollectionItemExecutionArtifact,
  ResourceMutationResponseExactCollectionTombstoneExecutionArtifact,
  ResourceMutationResponseExactDetailExecutionArtifact,
  ResourceMutationResponseExactDetailInvalidationExecutionArtifact,
  ResourceMutationResponseExactSummaryExecutionArtifact,
  ResourceMutationResponseFallbackExecutionArtifact,
} from "./resource_mutation_response_execution_artifacts.js";
export type {
  ResourceMutationResponseSubmittedTargetBasis,
  ResourceMutationResponseTargetStaleness,
} from "./resource_mutation_response_target_basis.js";
export type {
  ResourceMutationResponseLifecycleProof,
  ResourceMutationResponseLifecycleProofEntry,
  ResourceMutationResponseMergeRebaseProof,
  ResourceMutationResponseRollbackProof,
  ResourceMutationResponseTargetEffectProof,
} from "./resource_mutation_response_lifecycle_proof.js";
export type {
  ResourceMutationResponseAnyIdentityTargetDeclaration,
  ResourceMutationResponseAnyCreateTargetDeclaration,
  ResourceMutationResponseAnyTargetDeclaration,
  ResourceMutationResponseAtomicity,
  ResourceMutationResponseCollectionReconciliationDeclaration,
  ResourceMutationResponseDetailReconciliationDeclaration,
  ResourceMutationResponseDiagnosticDeclaration,
  ResourceMutationResponseFallbackTargetDeclaration,
  ResourceMutationResponseIdentityAtomicity,
  ResourceMutationResponseIdentityDeclaration,
  ResourceMutationResponseIdentityDetailChildTargetScope,
  ResourceMutationResponseIdentitySelectionTargetScope,
  ResourceMutationResponseIdentitySummaryTargetScope,
  ResourceMutationResponseIdentityTargetDeclaration,
  ResourceMutationResponseResidentLineIdentityTargetDeclaration,
  ResourceMutationResponseSelectionIdentityTargetDeclaration,
  ResourceMutationResponseSummaryIdentityTargetDeclaration,
  ResourceMutationResponseSummaryReconciliationDeclaration,
  ResourceMutationResponseTargetDeclaration,
  ResourceMutationResponseTargetFamily,
  ResourceMutationResponseTargetFamilyKind,
  ResourceMutationResponseTargetFamilyParams,
} from "./resource_mutation_response_authoring.js";
export type {
  ResourceMutationResponseExecutionArtifact,
  ResourceMutationResponseExactCollectionDeleteExecutionArtifact,
  ResourceMutationResponseExactCollectionInsertExecutionArtifact,
  ResourceMutationResponseExactCollectionItemExecutionArtifact,
  ResourceMutationResponseExactCollectionTombstoneExecutionArtifact,
  ResourceMutationResponseExactDetailExecutionArtifact,
  ResourceMutationResponseExactDetailInvalidationExecutionArtifact,
  ResourceMutationResponseExactSummaryExecutionArtifact,
  ResourceMutationResponseFallbackExecutionArtifact,
} from "./resource_mutation_response_execution_artifacts.js";

export interface ResourceMutationResponseLineDigest {
  readonly familyId: string;
  readonly runtimeLineId: string;
  readonly canonicalKey: string;
}

export interface ResourceMutationResponseRequestDigest {
  readonly correlationId: string | null;
  readonly branchId: string | number | null;
  readonly basisId: string | null;
  readonly requestPath: string | null;
  readonly url: string | null;
}

export interface ResourceMutationResponsePayloadDigest {
  readonly topology: string;
  readonly readResponseLensSource: string;
  readonly readResponseLensDigest: string;
  readonly mutationResponseLensDigest: string;
  readonly declaredFieldDigest: string;
  readonly unknownFieldPosture: {
    readonly kind: "notApplicable" | "none" | "present";
    readonly fields: readonly string[];
    readonly digest: string;
  };
  readonly payloadDigest: string;
}

export interface ResourceMutationResponseCounters {
  readonly planningBreadth: number;
  readonly responseExtractionBreadth: number;
  readonly payloadFieldExtractionBreadth: number;
  readonly targetLookupBreadth: number;
  readonly targetFanoutBreadth: number;
  readonly topologyTraversalBreadth: number;
  readonly reconstructionBreadth: number;
  readonly fallbackBreadth: number;
  readonly executionBreadth: number;
  readonly diagnosticExtractionBreadth: number;
  readonly confirmationClassificationBreadth: number;
  readonly lifecycleProofBreadth: number;
  readonly targetBasisSnapshotBreadth: number;
  readonly staleTargetDenialBreadth: number;
  readonly partialPolicyBreadth?: number;
  readonly identityResponseExtractionBreadth?: number;
  readonly identityMigrationTargetFanoutBreadth?: number;
  readonly identityMigrationStaleDenialBreadth?: number;
  readonly identityMigrationExecutionBreadth?: number;
  readonly identityMigrationLifecycleProofBreadth?: number;
  readonly appliedTargetBreadth?: number;
}

export type ResourceMutationResponseFallbackKind =
  | "identityMigrationUnavailable"
  | "deletionUnavailable"
  | "placementUnavailable"
  | "refetchRequired"
  | "deliveryAwaited"
  | "partialReconciliation"
  | "unsupportedTarget";

export type ResourceMutationResponseIdentityFallbackKind =
  | "identityMigrationUnavailable"
  | "refetchRequired"
  | "deliveryAwaited"
  | "partialReconciliation";

export interface ResourceMutationResponseTargetLineDigest {
  readonly familyKind: "detail" | "collection" | "paged";
  readonly familyId: string;
  readonly canonicalKey: string;
  readonly runtimeLineId: string | null;
  readonly residency: "declared" | "resident";
}

export interface ResourceMutationResponseTargetFallbackDigest {
  readonly kind: ResourceMutationResponseFallbackKind;
  readonly detail: string;
}

export interface ResourceMutationResponseTargetReconciliationDigest {
  readonly kind:
    | "replace"
    | "invalidate"
    | "field"
    | "region"
    | "jsonPath"
    | "item"
    | "delete"
    | "insert"
    | "summary";
  readonly itemId: string | null;
  readonly placement: "append" | "prepend" | null;
  readonly field: string | null;
  readonly region: string | null;
  readonly path: string | null;
  readonly summary: string | null;
  readonly targetDigest: string;
}

export interface ResourceMutationResponseTargetDigest {
  readonly targetId: string;
  readonly family: {
    readonly kind: "detail" | "collection" | "paged";
    readonly familyId: string;
  };
  readonly line: ResourceMutationResponseTargetLineDigest;
  readonly fallback: ResourceMutationResponseTargetFallbackDigest;
  readonly reconciliation: ResourceMutationResponseTargetReconciliationDigest | null;
  readonly submittedTarget: ResourceMutationResponseSubmittedTargetBasis | null;
  readonly execution:
    | ResourceMutationResponseExecutionArtifact;
  readonly targetDigest: string;
}

export interface ResourceMutationResponseTargetOutcomeSummary {
  readonly targetId: string;
  readonly familyKind: "detail" | "collection" | "paged";
  readonly familyId: string;
  readonly canonicalKey: string;
  readonly residency: "declared" | "resident";
  readonly outcomeKind: "exact" | "fallback";
  readonly executionKind: ResourceMutationResponseExecutionArtifact["kind"];
  readonly scope:
    | "line"
    | "field"
    | "region"
    | "jsonPath"
    | "item"
    | "summary"
    | "invalidate"
    | null;
  readonly fallbackKind: ResourceMutationResponseFallbackKind | null;
  readonly partialKind: "missingResponseField" | "duplicateDeliverySuppressed" | null;
  readonly partialField: string | null;
  readonly staleReason: ResourceMutationResponseTargetStaleness["reason"] | null;
  readonly locus: string | null;
  readonly targetDigest: string;
}

export interface ResourceMutationResponseDiagnosticFact {
  readonly diagnosticId: string;
  readonly kind: "validation" | "warnings";
  readonly field: string;
  readonly value: unknown;
  readonly valueDigest: string;
}

export interface ResourceMutationResponseDiagnosticsDigest {
  readonly entries: readonly ResourceMutationResponseDiagnosticFact[];
  readonly count: number;
  readonly digest: string;
}

export type ResourceMutationResponseConfirmationKind =
  | "preservedOptimisticTruth"
  | "consumedCanonicalTruth"
  | "partialCanonicalTruth"
  | "refetchRequired"
  | "deliveryAwaited";

export interface ResourceMutationResponseConfirmationClassification {
  readonly kind: ResourceMutationResponseConfirmationKind;
  readonly detail: string;
  readonly exactTargetCount: number;
  readonly fallbackTargetCount: number;
  readonly diagnosticCount: number;
  readonly fallbackKinds: readonly ResourceMutationResponseFallbackKind[];
  readonly digest: string;
}

export interface ResourceMutationResponseIdentityNoopExecutionDigest {
  readonly kind: "noMigrationRequired";
  readonly detail: string;
}
export interface ResourceMutationResponseIdentityFallbackExecutionDigest {
  readonly kind: "fallback";
  readonly fallback: ResourceMutationResponseIdentityFallbackKind;
  readonly detail: string;
}
export interface ResourceMutationResponseIdentityExactResidentLineExecutionDigest {
  readonly kind: "exactResidentLine";
  readonly previousCanonicalKey: string;
  readonly nextCanonicalKey: string;
  readonly previousRuntimeLineId: string | null;
  readonly nextRuntimeLineId: string | null;
  readonly basisId: string | null;
  readonly requestPath: string | null;
  readonly outcomeKind: "applied" | null;
  readonly detail: string;
}
export interface ResourceMutationResponseIdentityExactDetailChildRegionExecutionDigest {
  readonly kind: "exactDetailChildRegion";
  readonly region: string;
  readonly packetId: string;
  readonly effectId: string | null;
  readonly effectProof: ResourceMutationResponseTargetEffectProof | null;
  readonly outcomeKind: "applied" | null;
  readonly targetVisibleValueVersion: number | null;
  readonly detail: string;
}
export interface ResourceMutationResponseIdentityMigrationTargetDigest {
  readonly targetId: string;
  readonly family: ResourceMutationResponseTargetDigest["family"];
  readonly scope:
    | { readonly kind: "residentLine" }
    | ResourceMutationResponseIdentitySummaryTargetScope
    | ResourceMutationResponseIdentitySelectionTargetScope
    | ResourceMutationResponseIdentityDetailChildTargetScope;
  readonly line: ResourceMutationResponseTargetLineDigest;
  readonly fallback: ResourceMutationResponseIdentityFallbackKind;
  readonly submittedTarget: ResourceMutationResponseSubmittedTargetBasis | null;
  readonly staleness: ResourceMutationResponseTargetStaleness | null;
  readonly outcome:
    | "noMigrationRequired"
    | "fallback"
    | "exactResidentLine"
    | "exactDetailChildRegion";
  readonly detail: string;
  readonly execution:
    | ResourceMutationResponseIdentityNoopExecutionDigest
    | ResourceMutationResponseIdentityFallbackExecutionDigest
    | ResourceMutationResponseIdentityExactResidentLineExecutionDigest
    | ResourceMutationResponseIdentityExactDetailChildRegionExecutionDigest;
  readonly targetDigest: string;
}

export interface ResourceMutationResponseIdentityMigrationDigest {
  readonly declarationDigest: string;
  readonly atomicity: ResourceMutationResponseIdentityAtomicity;
  readonly partialAdmission: "notNeeded" | "admitted" | "denied";
  readonly submittedIdentity: string;
  readonly submittedIdentityDigest: string;
  readonly responseIdentity: string | null;
  readonly responseIdentityDigest: string;
  readonly canonicalIdentity: string;
  readonly canonicalIdentityDigest: string;
  readonly migrationNeeded: boolean;
  readonly exactTargetCount: number;
  readonly targets: readonly ResourceMutationResponseIdentityMigrationTargetDigest[];
  readonly targetCount: number;
  readonly targetDigest: string;
  readonly fallbackDigest: string;
  readonly fallbackKinds: readonly ResourceMutationResponseIdentityFallbackKind[];
  readonly executionDigest: string;
  readonly counters: {
    readonly responseIdentityExtractionBreadth: number;
    readonly canonicalIdentityBreadth: number;
    readonly targetFanoutBreadth: number;
    readonly targetBasisSnapshotBreadth: number;
    readonly staleTargetDenialBreadth: number;
    readonly exactTargetCount: number;
    readonly requestDescriptorRewriteBreadth: number;
    readonly lifecycleProofBreadth: number;
    readonly partialPolicyBreadth: number;
  };
  readonly digest: string;
}

export interface ResourceMutationResponsePlan {
  readonly version: "resource-mutation-response-plan-v1";
  readonly source: string;
  readonly planId: string;
  readonly route: string;
  readonly method: "POST" | "PUT" | "PATCH" | "DELETE";
  readonly line: ResourceMutationResponseLineDigest;
  readonly request: ResourceMutationResponseRequestDigest;
  readonly submittedTargets: readonly ResourceMutationResponseSubmittedTargetBasis[];
  readonly response: ResourceMutationResponsePayloadDigest;
  readonly confirmation: ResourceMutationResponseConfirmationClassification;
  readonly lifecycleProof: ResourceMutationResponseLifecycleProof;
  readonly diagnostics: ResourceMutationResponseDiagnosticsDigest;
  readonly identityMigration: ResourceMutationResponseIdentityMigrationDigest | null;
  readonly targets: readonly ResourceMutationResponseTargetDigest[];
  readonly targetCount: number;
  readonly atomicity: "zeroTargets" | "singleTarget" | "allOrNone";
  readonly reconciliationAtomicity: ResourceMutationResponseAtomicity;
  readonly partialAdmission: "notNeeded" | "admitted" | "denied";
  readonly targetDigest: string;
  readonly fallbackDigest: string;
  readonly executionArtifacts: readonly ResourceMutationResponseExecutionArtifact[];
  readonly executionDigest: string;
  readonly counters: ResourceMutationResponseCounters;
}
