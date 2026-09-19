mod advance_completion;
pub use advance_completion::SignalBranchAdvanceCompletion;
mod advancement;
mod authority;
mod basis;
mod basis_admission_identity;
mod basis_registry;
mod descriptor;
mod fork;
mod identity;
mod lifecycle;
mod managed_reference;
mod merge;
pub(crate) mod owner_services;
mod readmission;
mod reference;
mod restoration;
mod retention;
mod snapshot_capture;
mod snapshot_reconstruction;
mod target;

pub use advancement::{
    SignalBranchAdvanceDenial, SignalBranchAdvanceEngineDenial, SignalBranchAdvanceOutcome,
};
pub(crate) use authority::{mint_signal_branch_authority, signal_branch_basis_proof};
pub use authority::{
    SignalBranchBasisAuthority, SignalBranchBasisAuthorityMarker, SignalBranchBasisOwnerProof,
    SignalBranchBasisProof,
};
#[cfg(test)]
pub(crate) use basis::admit_runtime_signal_branch_observation;
pub use basis::AdmittedSignalBranchBasis;
pub use basis_admission_identity::SignalBranchBasisAdmissionIdentity;
pub(crate) use basis_registry::SignalBranchBasisRegistry;
pub use descriptor::{SignalBranchBasisDescriptor, SIGNAL_BRANCH_BASIS_DESCRIPTOR_SCHEMA_VERSION};
pub use fork::{SignalBranchForkOperationDenial, SignalBranchForkOutcome};
pub use identity::{
    signal_branch_identity, SignalBranchIdentity, SignalBranchIdentityConstructionDenial,
};
pub use identity::{validate_signal_branch_name, ValidatedSignalBranchName};
pub use lifecycle::{
    PlannedSignalBranchRetirement, PlannedSignalBranchRetirementBatch,
    SignalBranchBasisLifecyclePosture, SignalBranchRetirementBatchDenial,
    SignalBranchRetirementBatchReceipt, SignalBranchRetirementDenial, SignalBranchRetirementReason,
    SignalBranchRetirementReceipt,
};
pub use managed_reference::{
    ManagedSignalBranchReference, ManagedSignalBranchReferenceAdmissionDenial,
};
pub use merge::{SignalBranchMergeDenial, SignalBranchMergeOutcome};
#[allow(
    unused_imports,
    reason = "Phase 4 port signatures consume the Phase 3 cancellation vocabulary"
)]
pub use owner_services::{
    AdmittedSignalConditionalDefinitionPublication, SignalBranchBasisPort,
    SignalBranchForkReservation, SignalBranchLifecyclePort, SignalBranchMutationPort,
    SignalCommittedPatchDeliveryCompletion, SignalCommittedPatchDeliveryDenial,
    SignalCommittedPatchDeliveryRequest, SignalCommittedPatchTarget,
    SignalConditionalDefinitionAdvanceBinding, SignalConditionalDefinitionPublicationOperation,
    SignalConditionalDefinitionPublicationPort, SignalConditionalEvaluationAdmission,
    SignalConditionalEvaluationBindingEvidence, SignalConditionalEvaluationReadmission,
    SignalConditionalEvaluationReadmissionCounters, SignalConditionalEvaluationReadmissionDenial,
    SignalConditionalEvaluationReadmissionRequest, SignalConditionalEvaluationSourceEvidence,
    SignalConditionalExecutionPort, SignalConditionalInstallationChangeDenial,
    SignalConditionalInstallationCustody, SignalConditionalInstallationExtensionCompletion,
    SignalConditionalInstallationExtensionDenial, SignalConditionalInstallationExtensionRequest,
    SignalConditionalReconstitutionReport, SignalConditionalRetentionObservation,
    SignalConditionalRetirementCompletion, SignalConditionalServiceCompletion,
    SignalConditionalServiceExecutionDenial, SignalConditionalServiceExecutionRequest,
    SignalConditionalServiceIssuanceDenial, SignalConditionalSuccessorTransition,
    SignalConditionalTemporalPartition, SignalConditionalTemporalPartitionDenial,
    SignalConditionalTemporalPromotion, SignalConditionalTemporalPromotionView,
    SignalOwnedAsyncRequestAdmission, SignalOwnedAsyncRetryAdmission,
    SignalOwnedAsyncRetrySchedule, SignalOwnedAsyncRevalidationAdmission,
    SignalOwnedAsyncSourceBinding, SignalOwnedAsyncTimeoutAdmission, SignalOwnerCancellationSource,
    SignalOwnerCancellationToken, SignalOwnerLifecycleObservation, SignalOwnerServiceCostSnapshot,
    SignalOwnerServiceIssuanceDenial, SignalOwnerServicePorts, SignalOwnerUnavailable,
    SignalPreparedConditionalInstallationExtension,
};
#[cfg(feature = "test-operation-control")]
pub use owner_services::{
    SignalOwnerOperationBoundary, SignalOwnerOperationControl, SignalOwnerOperationPause,
};
pub use readmission::{
    SignalBranchBasisCompatibilityDenial, SignalBranchBasisObservationDenial,
    SignalBranchBasisReadmissionDenial, SignalBranchRetainedReadmissionDenial,
};
pub use reference::{
    signal_branch_observation, SignalBranchComparisonBasis, SignalBranchForkBasis,
    SignalBranchObservation, SignalBranchObservationConstructionDenial,
};
pub use restoration::SignalBranchRestoreDenial;
pub(crate) use retention::{
    SignalBranchAdmissionLease, SignalBranchRetentionBinding,
    SignalBranchRetentionOwnerRelationship, SignalBranchRetentionRegistry,
};
pub use retention::{
    SignalBranchRetentionAcquisitionDenial, SignalBranchRetentionLease,
    SignalBranchRetentionOwnerPosture, SignalBranchRetentionReleaseDenial,
    SignalBranchRetentionReleaseOutcome, SignalBranchRetentionReleaseReceipt,
    SignalBranchRetentionTerminalCounts, SignalBranchRetentionTerminalOutcome,
};
pub use snapshot_capture::{
    AdmittedSignalBranchSnapshot, SignalBranchSnapshotCaptureDenial,
    SignalBranchSnapshotCaptureOutcome,
};
pub use snapshot_reconstruction::{
    SignalBranchSnapshotReconstructionDenial, SignalBranchSnapshotReconstructionOutcome,
};
pub use target::{SignalBranchTarget, SignalBranchTargetConstructionDenial};
