mod helpers;
mod key_registry;
mod patch_buffer;
mod runtime;
#[cfg(test)]
mod tests;

pub(crate) use runtime::admit_signal_observation_request;
pub(crate) use runtime::RuntimeObservationRegistry;
pub(crate) use runtime::SignalCanonicalCallerUnwind;
pub(crate) use runtime::TemporalRuntimeState;
pub use runtime::{branch_state_proof_report, canonical_digest};
#[allow(unused_imports)]
pub use runtime::{
    replay_artifact_proof_report, replay_parity_proof_report, runtime_proof_report,
    temporal_certification_builder, temporal_certification_bundle,
    temporal_certification_bundle_parity_report, temporal_certification_record,
    temporal_replay_parity_report, AdvisoryRecord, BatchChangeSession, BoundedJournalSegment,
    BranchMutationLedger, BranchStateDenseGridProofBasis, BranchStateProofBasis,
    BranchStateProofReport, BranchTargetedTransactionDenial,
    BranchTargetedTransactionExecutionOutcome, BranchTargetedTransactionRequest,
    CheckpointBoundary, CheckpointRecord, CommittedObservationEventSummary, DecisionDetail,
    DecisionLog, DecisionRecord, DecisionSummary, DefinedComputation, DefinedKeyedComputation,
    DependencyIndexRebuildProof, EvaluationSummary, ExecutedBranchTargetedTransactionReceipt,
    IntegrityMarkers, JournalSegment, LoweredBranchTargetedTransactionPlan, MatchingObserverSet,
    ObservationBoundaryOutcome, ObservationBoundarySummary, ObservationDeliveryMode,
    ObservationHandle, ObservationHandleId, ObservationListener, ObservationNotice,
    ObservationPolicy, ObservationReadContext, ObservationRegistrySummary, ObservationTrigger,
    ObservedDemandSummary, ObservedNodeSet, ObserverId, PlannedSignalBranchRetirement,
    PlannedSignalBranchRetirementBatch, Recipe, ReconstructabilityProof, ReconstructabilityRecord,
    ReplayArtifactProofInput, ReplayArtifactProofReport, ReplayMismatchClass,
    ReplayParityProofReport, ReplaySuffixRebuildProof, RequiredDerivedRebuildSet,
    RuntimeExecutionRequest, RuntimeHistory, RuntimeMaterializer, RuntimeObserver,
    RuntimeProofReport, SignalBranchBasis, SignalBranchBasisArtifact, SignalBranchBasisAuthority,
    SignalBranchBasisDenial, SignalBranchBasisIdentity,
    SignalBranchBasisReady, SignalBranchForkDenial,
    SignalBranchForkReceipt, SignalBranchForkRequest, SignalBranchForkRequestBasis,
    SignalBranchHeadPosture, SignalBranchRestorePosture, SignalBranchRetirementBatchDenial,
    SignalBranchRetirementBatchReceipt, SignalBranchRetirementDenial, SignalBranchRetirementReason,
    SignalBranchRetirementReceipt, SignalBranchTransactionHead, SignalObservationAdmissionDenial,
    SignalObservationCompletion, SignalObservationRequest, SignalObservationSession,
    SignalObservationSurface, SignalRuntime, SignalRuntimeBuilder, SignalRuntimeConfig,
    SignalTransaction, TemporalCertificationBuilder,
    TemporalCertificationBundle, TemporalCertificationBundleMismatchClass,
    TemporalCertificationBundleParityReport, TemporalCertificationFailure,
    TemporalCertificationFamily, TemporalCertificationRecord, TemporalCertificationSummary,
    TemporalEligibilityFact, TemporalReconstructabilityArtifact, TemporalReplayMismatchClass,
    TemporalReplayParityReport, TemporalStateRebuildProof, TemporalTransactionEvidence,
    TransactionExecutionRequest, TransactionOutcome, TransactionReplayEntry, TransactionResult,
    TransactionTiming, ValidatedBranchTargetedTransactionRequest, BRANCH_STATE_PROOF_BASIS_VERSION,
    BRANCH_STATE_PROOF_SCHEMA_VERSION, REQUIRED_TEMPORAL_CERTIFICATION_FAMILIES,
    SIGNAL_BRANCH_BASIS_SCHEMA_VERSION, TEMPORAL_CERTIFICATION_BUNDLE_PARITY_SCHEMA_VERSION,
    TEMPORAL_CERTIFICATION_BUNDLE_SCHEMA_VERSION, TEMPORAL_REPLAY_PARITY_SCHEMA_VERSION,
};
pub(crate) use runtime::{
    BranchState, SignalOwnerMetadataCloseBatch, SignalOwnerMetadataState, SignalOwnerPartition,
    SignalOwnerRetirementCleanup, SignalOwnerSnapshotReservationDenial, SnapshotBranchState,
    SnapshotStatePacket,
};
pub(crate) use runtime::{
    SignalObservationCaptureGate, SignalObservationDropCleanup, SignalObservationSessionState,
};

#[cfg(test)]
pub use runtime::{
    bridge_signal_branch_basis_trust_boundary, BoundaryBridgedSignalBranchBasisArtifact,
};
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

#[cfg(test)]
pub use runtime::{SignalBranchBasisCompactExplanation, SignalBranchBasisValidationOutcome, StaleSignalBranchBasisArtifact};
