//! Signal branch contracts.
//!
//! Owner-issued managed branch and owner-service contracts are composition-facing. Descriptor-only
//! readmission and portable snapshot reconstruction remain owner-root compatibility surfaces; their
//! exports here do not make them composition-port contracts.

pub use crate::branch::{
    signal_branch_identity, signal_branch_observation, validate_signal_branch_name,
    AdmittedSignalBranchBasis, AdmittedSignalBranchSnapshot,
    AdmittedSignalConditionalDefinitionPublication, ManagedSignalBranchReference,
    ManagedSignalBranchReferenceAdmissionDenial, PlannedSignalBranchRetirement,
    PlannedSignalBranchRetirementBatch, SignalBranchAdvanceCompletion, SignalBranchAdvanceDenial,
    SignalBranchAdvanceEngineDenial, SignalBranchAdvanceOutcome,
    SignalBranchBasisAdmissionIdentity, SignalBranchBasisAuthority,
    SignalBranchBasisAuthorityMarker, SignalBranchBasisLifecyclePosture,
    SignalBranchBasisObservationDenial, SignalBranchBasisOwnerProof, SignalBranchBasisPort,
    SignalBranchBasisProof, SignalBranchBasisReadmissionDenial, SignalBranchComparisonBasis,
    SignalBranchForkBasis, SignalBranchForkOperationDenial, SignalBranchForkOutcome,
    SignalBranchForkReservation, SignalBranchIdentity, SignalBranchIdentityConstructionDenial,
    SignalBranchLifecyclePort, SignalBranchMergeDenial, SignalBranchMergeOutcome,
    SignalBranchMutationPort, SignalBranchObservation, SignalBranchObservationConstructionDenial,
    SignalBranchRestoreDenial, SignalBranchRetainedReadmissionDenial,
    SignalBranchRetentionAcquisitionDenial, SignalBranchRetentionLease,
    SignalBranchRetentionOwnerPosture, SignalBranchRetentionReleaseDenial,
    SignalBranchRetentionReleaseOutcome, SignalBranchRetentionReleaseReceipt,
    SignalBranchRetentionTerminalCounts, SignalBranchRetentionTerminalOutcome,
    SignalBranchRetirementBatchDenial, SignalBranchRetirementBatchReceipt,
    SignalBranchRetirementDenial, SignalBranchRetirementReason, SignalBranchRetirementReceipt,
    SignalBranchSnapshotCaptureDenial, SignalBranchSnapshotCaptureOutcome, SignalBranchTarget,
    SignalBranchTargetConstructionDenial, SignalCommittedPatchDeliveryCompletion,
    SignalCommittedPatchDeliveryDenial, SignalCommittedPatchDeliveryRequest,
    SignalCommittedPatchTarget, SignalConditionalDefinitionAdvanceBinding,
    SignalConditionalDefinitionPublicationOperation, SignalConditionalDefinitionPublicationPort,
    SignalConditionalEvaluationAdmission, SignalConditionalEvaluationBindingEvidence,
    SignalConditionalEvaluationReadmission, SignalConditionalEvaluationReadmissionCounters,
    SignalConditionalEvaluationReadmissionDenial, SignalConditionalEvaluationReadmissionRequest,
    SignalConditionalEvaluationSourceEvidence, SignalConditionalExecutionPort,
    SignalConditionalInstallationChangeDenial, SignalConditionalInstallationCustody,
    SignalConditionalInstallationExtensionCompletion, SignalConditionalInstallationExtensionDenial,
    SignalConditionalInstallationExtensionRequest, SignalConditionalReconstitutionReport,
    SignalConditionalRetentionObservation, SignalConditionalRetirementCompletion,
    SignalConditionalServiceCompletion, SignalConditionalServiceExecutionDenial,
    SignalConditionalServiceExecutionRequest, SignalConditionalServiceIssuanceDenial,
    SignalConditionalSuccessorTransition, SignalConditionalTemporalPartition,
    SignalConditionalTemporalPartitionDenial, SignalConditionalTemporalPromotion,
    SignalConditionalTemporalPromotionView, SignalOwnedAsyncRequestAdmission,
    SignalOwnedAsyncRetryAdmission, SignalOwnedAsyncRetrySchedule,
    SignalOwnedAsyncRevalidationAdmission, SignalOwnedAsyncSourceBinding,
    SignalOwnedAsyncTimeoutAdmission, SignalOwnerCancellationSource, SignalOwnerCancellationToken,
    SignalOwnerLifecycleObservation, SignalOwnerServiceCostSnapshot,
    SignalOwnerServiceIssuanceDenial, SignalOwnerServicePorts, SignalOwnerUnavailable,
    SignalPreparedConditionalInstallationExtension, ValidatedSignalBranchName,
};

/// Owner-root compatibility exports, not composition-port inputs or operations.
pub use crate::branch::{
    SignalBranchBasisCompatibilityDenial, SignalBranchBasisDescriptor,
    SignalBranchSnapshotReconstructionDenial, SignalBranchSnapshotReconstructionOutcome,
    SIGNAL_BRANCH_BASIS_DESCRIPTOR_SCHEMA_VERSION,
};

#[cfg(feature = "test-operation-control")]
pub use crate::branch::{
    SignalOwnerOperationBoundary, SignalOwnerOperationControl, SignalOwnerOperationPause,
};
