mod canonical_digest;
pub use branching::branch_mutation_journal::BranchMutationLedger;
pub use branching::replay_artifact_proof::{
    replay_artifact_proof_report, runtime_proof_report, ReplayArtifactProofInput,
    ReplayArtifactProofReport, RuntimeProofReport,
};
pub use branching::state_proof::{
    branch_state_proof_report, replay_parity_proof_report, BranchStateDenseGridProofBasis,
    BranchStateProofBasis, BranchStateProofReport, ReplayMismatchClass, ReplayParityProofReport,
    BRANCH_STATE_PROOF_BASIS_VERSION, BRANCH_STATE_PROOF_SCHEMA_VERSION,
};
pub use canonical_digest::canonical_digest;

mod async_capability;
mod branching;
mod builder;

mod guided;

mod mutation;
mod observation;
mod observer;
mod reconstructability;
mod resource;
mod resource_observation;
mod retention_omission;
mod runtime_observation;
mod runtime_state;
mod temporal;
pub(crate) use temporal::TemporalRuntimeState;

pub(crate) use crate::observation::session::admit as admit_signal_observation_request;
pub(crate) use crate::observation::session::{
    SignalObservationCaptureGate, SignalObservationDropCleanup, SignalObservationSessionState,
};
pub(crate) use branching::{
    BranchState, SignalCanonicalCallerUnwind, SignalOwnerMetadataCloseBatch,
    SignalOwnerMetadataState, SignalOwnerPartition, SignalOwnerRetirementCleanup,
    SignalOwnerSnapshotReservationDenial, SnapshotBranchState, SnapshotStatePacket,
};
pub use branching::{
    BranchTargetedTransactionDenial, BranchTargetedTransactionExecutionOutcome,
    BranchTargetedTransactionRequest, ExecutedBranchTargetedTransactionReceipt,
    LoweredBranchTargetedTransactionPlan, PlannedSignalBranchRetirement,
    PlannedSignalBranchRetirementBatch, SignalBranchBasis, SignalBranchBasisArtifact,
    SignalBranchBasisAuthority, SignalBranchBasisCompactExplanation, SignalBranchBasisDenial,
    SignalBranchBasisIdentity, SignalBranchBasisReady, SignalBranchBasisValidationOutcome,
    SignalBranchForkDenial, SignalBranchForkReceipt, SignalBranchForkRequest,
    SignalBranchForkRequestBasis, SignalBranchHeadPosture, SignalBranchRestorePosture,
    SignalBranchRetirementBatchDenial, SignalBranchRetirementBatchReceipt,
    SignalBranchRetirementDenial, SignalBranchRetirementReason, SignalBranchRetirementReceipt,
    SignalBranchTransactionHead, StaleSignalBranchBasisArtifact,
    ValidatedBranchTargetedTransactionRequest, SIGNAL_BRANCH_BASIS_SCHEMA_VERSION,
};
pub use builder::SignalRuntimeBuilder;

pub use guided::RuntimeHistory;

#[allow(unused_imports)]
pub use observation::{
    SignalObservationAdmissionDenial, SignalObservationCompletion, SignalObservationRequest,
    SignalObservationSession, SignalObservationSurface,
};
pub use observer::{RuntimeMaterializer, RuntimeObserver};
#[allow(unused_imports)]
pub use reconstructability::{
    temporal_certification_builder, temporal_certification_bundle,
    temporal_certification_bundle_parity_report, temporal_certification_record,
    temporal_replay_parity_report, BoundedJournalSegment, CheckpointBoundary,
    DependencyIndexRebuildProof, MergeSupportRebuildProof, ReconstructabilityProof,
    ReplaySuffixRebuildProof, RequiredDerivedRebuildSet, TemporalCertificationBuilder,
    TemporalCertificationBundle, TemporalCertificationBundleMismatchClass,
    TemporalCertificationBundleParityReport, TemporalCertificationFailure,
    TemporalCertificationFamily, TemporalCertificationRecord, TemporalCertificationSummary,
    TemporalReconstructabilityArtifact, TemporalReplayMismatchClass, TemporalReplayParityReport,
    TemporalStateRebuildProof, REQUIRED_TEMPORAL_CERTIFICATION_FAMILIES,
    TEMPORAL_CERTIFICATION_BUNDLE_PARITY_SCHEMA_VERSION,
    TEMPORAL_CERTIFICATION_BUNDLE_SCHEMA_VERSION, TEMPORAL_REPLAY_PARITY_SCHEMA_VERSION,
};
#[allow(unused_imports)]
pub use reconstructability::{CheckpointRecord, JournalSegment, ReconstructabilityRecord};
pub(in crate::logic::transaction::runtime) use resource::ResourceRuntimeState;
pub(crate) use runtime_observation::RuntimeObservationRegistry;
pub use runtime_observation::{
    MatchingObserverSet, ObservationDeliveryMode, ObservationHandle, ObservationHandleId,
    ObservationListener, ObservationNotice, ObservationPolicy, ObservationReadContext,
    ObservationRegistrySummary, ObservationTrigger, ObservedNodeSet, ObserverId,
};
pub use runtime_state::SignalRuntime;

#[cfg(test)]
pub use branching::{
    bridge_signal_branch_basis_trust_boundary, BoundaryBridgedSignalBranchBasisArtifact,
};
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
