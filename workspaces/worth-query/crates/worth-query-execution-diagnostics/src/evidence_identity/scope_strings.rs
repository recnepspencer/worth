use super::consumer_scope_strings::consumer_kit_evidence_scope_as_str;
use super::graph_application_scope_strings::graph_application_evidence_scope_as_str;
use super::installed_domain_scope_strings::{
    installed_domain_evidence_scope_as_str, installed_domain_evidence_scopes,
};
use super::scope::WorthQueryEvidenceScope;

pub(crate) fn evidence_scope_as_str(scope: WorthQueryEvidenceScope) -> &'static str {
    use WorthQueryEvidenceScope as S;
    match scope {
        WorthQueryEvidenceScope::RuntimePublicSupportMatrixRow => {
            "runtime-public-support-matrix-row"
        }
        WorthQueryEvidenceScope::RuntimePublicSupportMatrix => "runtime-public-support-matrix",
        WorthQueryEvidenceScope::RuntimePublicApiFamilyContract => {
            "runtime-public-api-family-contract"
        }
        WorthQueryEvidenceScope::RuntimePublicApiContract => "runtime-public-api-contract",
        WorthQueryEvidenceScope::RuntimePublicApiTranscriptEvidence => {
            "runtime-public-api-transcript-evidence"
        }
        WorthQueryEvidenceScope::RuntimeSubscriptionBudget => "runtime-subscription-budget",
        WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact => {
            "runtime-hostile-certification-artifact"
        }
        WorthQueryEvidenceScope::RuntimeStateSnapshot => "runtime-state-snapshot",
        WorthQueryEvidenceScope::AsyncResourceRequestIdentity => "async-resource-request-identity",
        WorthQueryEvidenceScope::WorkflowContextBinding => "workflow-context-binding",
        WorthQueryEvidenceScope::WorkflowMutationLowering => "workflow-mutation-lowering",
        WorthQueryEvidenceScope::SubscriptionActivationReceipt => "subscription-activation-receipt",
        WorthQueryEvidenceScope::SignalInvalidationRoutingReceipt => {
            "signal-invalidation-routing-receipt"
        }
        WorthQueryEvidenceScope::LowerRuntimeCapabilitySubject => {
            "lower-runtime-capability-subject"
        }
        WorthQueryEvidenceScope::LowerRuntimeRouteSubject => "lower-runtime-route-subject",
        WorthQueryEvidenceScope::LowerRuntimeCapabilityRequest => {
            "lower-runtime-capability-request"
        }
        WorthQueryEvidenceScope::LowerRuntimeCapabilityEligibility => {
            "lower-runtime-capability-eligibility"
        }
        WorthQueryEvidenceScope::LowerRuntimeRoutePlan => "lower-runtime-route-plan",
        WorthQueryEvidenceScope::PreviewPromotionContinuation => "preview-promotion-continuation",
        WorthQueryEvidenceScope::LowerRuntimeReadmissionReceipt => {
            "lower-runtime-readmission-receipt"
        }
        WorthQueryEvidenceScope::LowerRuntimeBoundaryExecutionReceipt => {
            "lower-runtime-boundary-execution-receipt"
        }
        WorthQueryEvidenceScope::LowerRuntimeBoundaryAuthority => {
            "lower-runtime-boundary-authority"
        }
        WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence => "lower-runtime-boundary-evidence",
        WorthQueryEvidenceScope::LowerRuntimeBoundaryEnvelope => "lower-runtime-boundary-envelope",
        WorthQueryEvidenceScope::DeclarationBridgeRoutingDigest => {
            "declaration-bridge-routing-digest"
        }
        WorthQueryEvidenceScope::DeclarationBridgeLoweringIdentity => {
            "declaration-bridge-lowering-identity"
        }
        WorthQueryEvidenceScope::ContinuationExecutionReadmissionEvidence => {
            "continuation-execution-readmission-evidence"
        }
        WorthQueryEvidenceScope::ContinuationLinkedArtifacts => "continuation-linked-artifacts",
        WorthQueryEvidenceScope::ContinuationPreparedDigest => "continuation-prepared-digest",
        WorthQueryEvidenceScope::ContinuationExecutionTranscript => {
            "continuation-execution-transcript"
        }
        WorthQueryEvidenceScope::ContinuationExecutionDigest => "continuation-execution-digest",
        WorthQueryEvidenceScope::ViewShapePlanDigest => "view-shape-plan-digest",
        WorthQueryEvidenceScope::BasisDigest => "basis-digest",
        WorthQueryEvidenceScope::BridgeGroupedTruthViewDigest => "bridge-grouped-truth-view-digest",
        WorthQueryEvidenceScope::ReadGraphDigest => "read-graph-digest",
        WorthQueryEvidenceScope::SessionLabelIdentity => "session-label-identity",
        WorthQueryEvidenceScope::ResolvedSnapshotBasis => "resolved-snapshot-basis",
        WorthQueryEvidenceScope::BasisAdmissionEvidenceRow => "basis-admission-evidence-row",
        WorthQueryEvidenceScope::PreviewBasisAdmission => "preview-basis-admission",
        WorthQueryEvidenceScope::BranchBasisAdmission => "branch-basis-admission",
        WorthQueryEvidenceScope::RawBasisIntent => "raw-basis-intent",
        WorthQueryEvidenceScope::QueryContextCompatibilityBasisLabel => {
            "query-context-compatibility-basis-label"
        }
        WorthQueryEvidenceScope::BridgeLowerRuntimeEvidenceReference => {
            "bridge-lower-runtime-evidence-reference"
        }
        WorthQueryEvidenceScope::BridgeLowerRuntimeBasisBinding => {
            "bridge-lower-runtime-basis-binding"
        }
        WorthQueryEvidenceScope::ContinuationReadmissionBasis => "continuation-readmission-basis",
        WorthQueryEvidenceScope::ContinuationReadmissionLowerRuntimeBinding => {
            "continuation-readmission-lower-runtime-binding"
        }
        WorthQueryEvidenceScope::ContinuationReadmissionSourceBasis => {
            "continuation-readmission-source-basis"
        }
        WorthQueryEvidenceScope::SharedReadGeneration => "shared-read-generation",
        WorthQueryEvidenceScope::PreviewIntentAdmission => "preview-intent-admission",
        WorthQueryEvidenceScope::PreviewIntentReceipt => "preview-intent-receipt",
        WorthQueryEvidenceScope::IntentExecutionProvenanceChain => {
            "intent-execution-provenance-chain"
        }
        WorthQueryEvidenceScope::AuthoritativeIntentReceipt => "authoritative-intent-receipt",
        WorthQueryEvidenceScope::EffectIntentReceipt => "effect-intent-receipt",
        WorthQueryEvidenceScope::WriteReceiptCommitIdentity => "write-receipt-commit-identity",
        WorthQueryEvidenceScope::JournalPositionIdentity => "journal-position-identity",
        WorthQueryEvidenceScope::JournalSegmentIdentity => "journal-segment-identity",
        WorthQueryEvidenceScope::JournalReplayOutcome => "journal-replay-outcome",
        WorthQueryEvidenceScope::WriteReceiptSnapshotIdentity => "write-receipt-snapshot-identity",
        WorthQueryEvidenceScope::WriteReceiptEntityIdentity => "write-receipt-entity-identity",
        WorthQueryEvidenceScope::AuthoredCommandEntityIdentity => {
            "authored-command-entity-identity"
        }
        WorthQueryEvidenceScope::ExistingTruthResolvedTargetIdentity => {
            "existing-truth-resolved-target-identity"
        }
        WorthQueryEvidenceScope::ProjectionConsumptionIdentity => "projection-consumption-identity",
        WorthQueryEvidenceScope::ProjectionConsumptionCertificationIdentity => {
            "projection-consumption-certification-identity"
        }
        WorthQueryEvidenceScope::ConsumerInvalidationDelta => "consumer-invalidation-delta",
        S::DomainCapabilityIdentity => "domain-capability-identity",
        S::DomainCapabilityCertificationIdentity => "domain-capability-certification-identity",
        installed_domain_evidence_scopes!() => installed_domain_evidence_scope_as_str(scope),
        WorthQueryEvidenceScope::ProjectionConsumedContinuityAuthorityIdentity => {
            "projection-consumed-continuity-authority-identity"
        }
        WorthQueryEvidenceScope::RuntimeBridgeWritebackAuthority => {
            "runtime-bridge-writeback-authority"
        }
        WorthQueryEvidenceScope::MutationEvidenceAuthorityIdentity => {
            "mutation-evidence-authority-identity"
        }
        WorthQueryEvidenceScope::MutationEvidenceTargetCollectionIdentity => {
            "mutation-evidence-target-collection-identity"
        }
        WorthQueryEvidenceScope::MutationEvidenceSymbolIdentity => {
            "mutation-evidence-symbol-identity"
        }
        WorthQueryEvidenceScope::MutationEvidenceSourceDigest => "mutation-evidence-source-digest",
        WorthQueryEvidenceScope::MutationEvidenceAggregateDigest => {
            "mutation-evidence-aggregate-digest"
        }
        WorthQueryEvidenceScope::EffectTriggerCommitIdentity => "effect-trigger-commit-identity",
        WorthQueryEvidenceScope::PreviewIntentBasisEvidence => "preview-intent-basis-evidence",
        WorthQueryEvidenceScope::PreviewIntentReceiptInspectionBasis => {
            "preview-intent-receipt-inspection-basis"
        }
        WorthQueryEvidenceScope::PreviewIntentReceiptInspection => {
            "preview-intent-receipt-inspection"
        }
        WorthQueryEvidenceScope::IntentInspectionDeliveryCounters => {
            "intent-inspection-delivery-counters"
        }
        WorthQueryEvidenceScope::IntentReceiptInspection => "intent-receipt-inspection",
        WorthQueryEvidenceScope::IntentDenialInspection => "intent-denial-inspection",
        WorthQueryEvidenceScope::EffectIntentReceiptPhase => "effect-intent-receipt-phase",
        WorthQueryEvidenceScope::EffectIntentReceiptInspection => {
            "effect-intent-receipt-inspection"
        }
        WorthQueryEvidenceScope::FeedbackPhaseGraph => "feedback-phase-graph",
        WorthQueryEvidenceScope::FeedbackPhaseGraphInspection => "feedback-phase-graph-inspection",
        WorthQueryEvidenceScope::BranchIntentReceiptInspectionBasis => {
            "branch-intent-receipt-inspection-basis"
        }
        WorthQueryEvidenceScope::BranchIntentReceiptInspection => {
            "branch-intent-receipt-inspection"
        }
        WorthQueryEvidenceScope::GenericInspectionIntentSeed => "generic-inspection-intent-seed",
        WorthQueryEvidenceScope::AuthoritativeMutationIntentSeed => {
            "authoritative-mutation-intent-seed"
        }
        WorthQueryEvidenceScope::AuthoritativeMutationBatchIntentSeed => {
            "authoritative-mutation-batch-intent-seed"
        }
        WorthQueryEvidenceScope::AuthoritativeMutationExecutionHandoff => {
            "authoritative-mutation-execution-handoff"
        }
        WorthQueryEvidenceScope::BranchIntentAdmission => "branch-intent-admission",
        WorthQueryEvidenceScope::BranchIntentReceipt => "branch-intent-receipt",
        WorthQueryEvidenceScope::IntentDenialEvidence => "intent-denial-evidence",
        WorthQueryEvidenceScope::IntentExecutionFailureEvidence => {
            "intent-execution-failure-evidence"
        }
        WorthQueryEvidenceScope::PreviewCloseoutEvidence => "preview-closeout-evidence",
        WorthQueryEvidenceScope::PreviewPromotionDenialEvidence => {
            "preview-promotion-denial-evidence"
        }
        WorthQueryEvidenceScope::PreviewExecutionEvidence => "preview-execution-evidence",
        WorthQueryEvidenceScope::PreviewPromotionRebinding => "preview-promotion-rebinding",
        WorthQueryEvidenceScope::PreviewWriteReceiptIdentity => "preview-write-receipt-identity",
        WorthQueryEvidenceScope::WriteReceiptInspectionArtifact => {
            "write-receipt-inspection-artifact"
        }
        WorthQueryEvidenceScope::WriteReceiptDeclaredAspectOperation => {
            "write-receipt-declared-aspect-operation"
        }
        WorthQueryEvidenceScope::WriteReceiptMutationMetadataEntry => {
            "write-receipt-mutation-metadata-entry"
        }
        WorthQueryEvidenceScope::BatchWriteReceipt => "batch-write-receipt",
        WorthQueryEvidenceScope::BatchWriteReceiptInspectionArtifact => {
            "batch-write-receipt-inspection-artifact"
        }
        WorthQueryEvidenceScope::BatchWriteReceiptComponent => "batch-write-receipt-component",
        WorthQueryEvidenceScope::BatchWriteReceiptSymbolicAspectResolution => {
            "batch-write-receipt-symbolic-aspect-resolution"
        }
        WorthQueryEvidenceScope::BatchWriteReceiptGraphResolution => {
            "batch-write-receipt-graph-resolution"
        }
        WorthQueryEvidenceScope::RetainedExistingTruthAssertionEvidence => {
            "retained-existing-truth-assertion-evidence"
        }
        WorthQueryEvidenceScope::LiveArtifactBundle => "live-artifact-bundle",
        WorthQueryEvidenceScope::GroupedExecutionSurfaceArtifact => {
            "grouped-execution-surface-artifact"
        }
        WorthQueryEvidenceScope::DerivedMaterializationBundle => "derived-materialization-bundle",
        WorthQueryEvidenceScope::PreviewBindingInspectionArtifact => {
            "preview-binding-inspection-artifact"
        }
        WorthQueryEvidenceScope::PreviewOutcomeInspectionArtifact => {
            "preview-outcome-inspection-artifact"
        }
        WorthQueryEvidenceScope::CausalObservationReceipt => "causal-observation-receipt",
        WorthQueryEvidenceScope::CausalObservationQuery => "causal-observation-query",
        WorthQueryEvidenceScope::CausalObservationBasis => "causal-observation-basis",
        WorthQueryEvidenceScope::CausalObservationTarget => "causal-observation-target",
        WorthQueryEvidenceScope::CausalResultShapeContext => "causal-result-shape-context",
        WorthQueryEvidenceScope::CausalQueryObservationReceipt => {
            "causal-query-observation-receipt"
        }
        WorthQueryEvidenceScope::CausalObservationAnchor => "causal-observation-anchor",
        WorthQueryEvidenceScope::CausalObservationAnchorCounters => {
            "causal-observation-anchor-counters"
        }
        WorthQueryEvidenceScope::CausalObservationAnchorFailure => {
            "causal-observation-anchor-failure"
        }
        WorthQueryEvidenceScope::CausalEvidenceReference => "causal-evidence-reference",
        WorthQueryEvidenceScope::CausalEvidenceReferenceReceipt => {
            "causal-evidence-reference-receipt"
        }
        WorthQueryEvidenceScope::CausalEvidenceReferenceResolutionCounters => {
            "causal-evidence-reference-resolution-counters"
        }
        WorthQueryEvidenceScope::CausalEvidenceReferenceResolutionDenial => {
            "causal-evidence-reference-resolution-denial"
        }
        WorthQueryEvidenceScope::CausalEvidenceReferenceIndex => "causal-evidence-reference-index",
        WorthQueryEvidenceScope::CausalEvidenceReferenceIndexRecord => {
            "causal-evidence-reference-index-record"
        }
        WorthQueryEvidenceScope::CausalEvidenceReferenceIndexError => {
            "causal-evidence-reference-index-error"
        }
        WorthQueryEvidenceScope::CausalInspectionTarget => "causal-inspection-target",
        WorthQueryEvidenceScope::CausalInspectionRequest => "causal-inspection-request",
        WorthQueryEvidenceScope::CausalInspectionRequestFailure => {
            "causal-inspection-request-failure"
        }
        WorthQueryEvidenceScope::CausalInspectionAdmissionSubject => {
            "causal-inspection-admission-subject"
        }
        WorthQueryEvidenceScope::CausalInspectionAdmissionDecision => {
            "causal-inspection-admission-decision"
        }
        WorthQueryEvidenceScope::CausalInspectionDecisionTraceRow => {
            "causal-inspection-decision-trace-row"
        }
        WorthQueryEvidenceScope::CausalInspectionDecisionTraceIndex => {
            "causal-inspection-decision-trace-index"
        }
        WorthQueryEvidenceScope::CausalInspectionAdmissionCounters => {
            "causal-inspection-admission-counters"
        }
        WorthQueryEvidenceScope::CausalInspectionAdmissionReceipt => {
            "causal-inspection-admission-receipt"
        }
        WorthQueryEvidenceScope::CausalInspectionOutcome => "causal-inspection-outcome",
        WorthQueryEvidenceScope::CausalInspectionMaterializedDetail => {
            "causal-inspection-materialized-detail"
        }
        WorthQueryEvidenceScope::CausalInspectionDeniedArtifactDetail => {
            "causal-inspection-denied-artifact-detail"
        }
        WorthQueryEvidenceScope::CausalInspectionArtifact => "causal-inspection-artifact",
        WorthQueryEvidenceScope::CausalInspectionArtifactIdentity => {
            "causal-inspection-artifact-identity"
        }
        WorthQueryEvidenceScope::CausalInspectionPerformanceSnapshot => {
            "causal-inspection-performance-snapshot"
        }
        WorthQueryEvidenceScope::CausalInspectionPerformanceSlope => {
            "causal-inspection-performance-slope"
        }
        WorthQueryEvidenceScope::CausalInspectionPerformanceScaleSlope => {
            "causal-inspection-performance-scale-slope"
        }
        WorthQueryEvidenceScope::CausalInspectionPerformanceCertificationBundle => {
            "causal-inspection-performance-certification-bundle"
        }
        WorthQueryEvidenceScope::CausalInspectionCertificationError => {
            "causal-inspection-certification-error"
        }
        WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence => {
            "causal-inspection-certification-failure-evidence"
        }
        WorthQueryEvidenceScope::RuntimePublicApiNamingRow => "runtime-public-api-naming-row",
        WorthQueryEvidenceScope::RuntimePublicApiNamingContract => {
            "runtime-public-api-naming-contract"
        }
        WorthQueryEvidenceScope::ConsumerEvidenceReportField
        | WorthQueryEvidenceScope::ConsumerEvidenceReport
        | WorthQueryEvidenceScope::ConsumerEvidenceReportFieldInventory
        | WorthQueryEvidenceScope::ConsumerEvidenceReportDigestParticipation
        | WorthQueryEvidenceScope::ConsumerSupportSnapshotSchema
        | WorthQueryEvidenceScope::ConsumerSupportSnapshotRow
        | WorthQueryEvidenceScope::ConsumerSupportSnapshotDocument
        | WorthQueryEvidenceScope::ConsumerProjectionContractDenial
        | WorthQueryEvidenceScope::ConsumerSupportPinContractSchema
        | WorthQueryEvidenceScope::ConsumerSupportPinVocabulary
        | WorthQueryEvidenceScope::ConsumerSupportPinRequirement
        | WorthQueryEvidenceScope::ConsumerSupportPinObservedRow
        | WorthQueryEvidenceScope::ConsumerSupportPinContract
        | WorthQueryEvidenceScope::ConsumerSupportPinContractDocument
        | WorthQueryEvidenceScope::ConsumerSupportPinFinding
        | WorthQueryEvidenceScope::ConsumerSupportPinReport => {
            consumer_kit_evidence_scope_as_str(scope)
        }
        WorthQueryEvidenceScope::GraphCompositionDomainInvariantDenial
        | WorthQueryEvidenceScope::GraphCompositionInvariantViolation
        | WorthQueryEvidenceScope::GraphTouchDescriptor
        | WorthQueryEvidenceScope::GraphTouchDescriptorRow
        | WorthQueryEvidenceScope::ApplicationSupportSectionPosture
        | WorthQueryEvidenceScope::ApplicationSupportReport => {
            graph_application_evidence_scope_as_str(scope)
        }
    }
}
