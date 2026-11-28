mod async_keyed;
mod computation;
mod config;
mod execution;
mod state;
mod transaction;
pub(crate) use state::TemporalRuntimeState;

pub use computation::{DefinedComputation, DefinedKeyedComputation, Recipe};
pub use config::SignalRuntimeConfig;
pub use execution::{ObservedDemandSummary, RuntimeExecutionRequest, TransactionExecutionRequest};
pub(crate) use state::admit_signal_observation_request;
pub(crate) use state::RuntimeObservationRegistry;
pub(crate) use state::SignalCanonicalCallerUnwind;
pub use state::{branch_state_proof_report, canonical_digest};
#[allow(unused_imports)]
pub use state::{replay_artifact_proof_report, replay_parity_proof_report};
pub use state::{
    runtime_proof_report, BranchMutationLedger, BranchStateDenseGridProofBasis,
    BranchStateProofBasis, BranchStateProofReport, BranchTargetedTransactionDenial,
    BranchTargetedTransactionExecutionOutcome, BranchTargetedTransactionRequest,
    ExecutedBranchTargetedTransactionReceipt, LoweredBranchTargetedTransactionPlan,
    PlannedSignalBranchRetirement, PlannedSignalBranchRetirementBatch, ReplayArtifactProofInput,
    ReplayArtifactProofReport, ReplayMismatchClass, ReplayParityProofReport, RuntimeProofReport,
    SignalBranchBasis, SignalBranchBasisArtifact, SignalBranchBasisAuthority,
    SignalBranchBasisDenial, SignalBranchBasisIdentity,
    SignalBranchBasisReady, SignalBranchForkDenial,
    SignalBranchForkReceipt, SignalBranchForkRequest, SignalBranchForkRequestBasis,
    SignalBranchHeadPosture, SignalBranchRestorePosture, SignalBranchRetirementBatchDenial,
    SignalBranchRetirementBatchReceipt, SignalBranchRetirementDenial, SignalBranchRetirementReason,
    SignalBranchRetirementReceipt, SignalBranchTransactionHead,
    ValidatedBranchTargetedTransactionRequest, BRANCH_STATE_PROOF_BASIS_VERSION,
    BRANCH_STATE_PROOF_SCHEMA_VERSION, SIGNAL_BRANCH_BASIS_SCHEMA_VERSION,
};
pub use state::{
    temporal_certification_builder, temporal_certification_bundle,
    temporal_certification_bundle_parity_report, temporal_certification_record,
    temporal_replay_parity_report, BoundedJournalSegment, CheckpointBoundary, CheckpointRecord,
    DependencyIndexRebuildProof, JournalSegment, MatchingObserverSet, ObservationDeliveryMode,
    ObservationHandle, ObservationHandleId, ObservationListener, ObservationNotice,
    ObservationPolicy, ObservationReadContext, ObservationRegistrySummary, ObservationTrigger,
    ObservedNodeSet, ObserverId, ReconstructabilityProof, ReconstructabilityRecord,
    ReplaySuffixRebuildProof, RequiredDerivedRebuildSet, RuntimeHistory, RuntimeMaterializer,
    RuntimeObserver, SignalObservationAdmissionDenial, SignalObservationCompletion,
    SignalObservationRequest, SignalObservationSession, SignalObservationSurface, SignalRuntime,
    SignalRuntimeBuilder, TemporalCertificationBuilder, TemporalCertificationBundle,
    TemporalCertificationBundleMismatchClass, TemporalCertificationBundleParityReport,
    TemporalCertificationFailure, TemporalCertificationFamily, TemporalCertificationRecord,
    TemporalCertificationSummary, TemporalReconstructabilityArtifact, TemporalReplayMismatchClass,
    TemporalReplayParityReport, TemporalStateRebuildProof,
    REQUIRED_TEMPORAL_CERTIFICATION_FAMILIES, TEMPORAL_CERTIFICATION_BUNDLE_PARITY_SCHEMA_VERSION,
    TEMPORAL_CERTIFICATION_BUNDLE_SCHEMA_VERSION, TEMPORAL_REPLAY_PARITY_SCHEMA_VERSION,
};
pub(crate) use state::{
    BranchState, SignalOwnerMetadataCloseBatch, SignalOwnerMetadataState, SignalOwnerPartition,
    SignalOwnerRetirementCleanup, SignalOwnerSnapshotReservationDenial, SnapshotBranchState,
    SnapshotStatePacket,
};
pub(crate) use state::{
    SignalObservationCaptureGate, SignalObservationDropCleanup, SignalObservationSessionState,
};
#[allow(unused_imports)]
pub use transaction::{
    AdvisoryRecord, BatchChangeSession, CommittedObservationEventSummary, DecisionDetail,
    DecisionLog, DecisionRecord, DecisionSummary, EvaluationSummary, IntegrityMarkers,
    ObservationBoundaryOutcome, ObservationBoundarySummary, SignalTransaction,
    TemporalEligibilityFact, TemporalTransactionEvidence, TransactionOutcome,
    TransactionReplayEntry, TransactionResult, TransactionTiming,
};

#[cfg(test)]
pub use state::{
    bridge_signal_branch_basis_trust_boundary, BoundaryBridgedSignalBranchBasisArtifact,
};
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

#[cfg(test)]
pub use state::{SignalBranchBasisCompactExplanation, SignalBranchBasisValidationOutcome, StaleSignalBranchBasisArtifact};
