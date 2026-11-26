mod admission;
mod checkpoint;
mod closeout;
mod data;
mod evidence_projection;
mod grouping;
mod lifecycle;
mod mutation;
#[cfg(all(
    test,
    feature = "recovery-runtime-owner",
    feature = "certification-test-authority"
))]
pub(in crate::physical_runtime) use mutation::with_conflicting_binding_history;
mod observation;
mod publication;
pub(in crate::physical_runtime) use publication::RetirementClaim;
pub use publication::{ReleaseCertificateCapacityDenial, SelectedReleaseHeadDenial};
mod retention;
pub(in crate::physical_runtime) use retention::PendingPublicationLease;
pub(in crate::physical_runtime) use retention::PhysicalPublicationAdmission;
pub(in crate::physical_runtime) use retention::PhysicalPublicationAdmissionDenial;
pub(in crate::physical_runtime) use retention::PhysicalRetentionProfile;
pub use retention::PhysicalRetirementDenial;
pub(in crate::physical_runtime) use retention::RetainedByteLease;
pub use retention::RetirementReleaseProjection;
pub(in crate::physical_runtime) use retention::WalPublicationReservation;
pub(in crate::physical_runtime) use retention::{
    DisplacedArtifact, RetiredArtifact, RetirementRecord, RetirementRemovalPermit,
    encode_retirement, unresolved_retirement_holds, unresolved_retirements,
};
pub use retention::{PhysicalReclaimBudget, PhysicalReclaimPass, PhysicalReclaimPassEnd};
pub(in crate::physical_runtime) use retention::{
    ProtectedReachSet, ReachWindow, ReclaimCandidate, checkpoint_covers, plan_reclaim,
};
#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) use retention::{decode_retirement, payload_is_retirement};
mod settlement;
mod wal;
pub(in crate::physical_runtime) use wal::DurableMaintenanceReceipt;

pub use admission::{
    AdmittedPhysicalDurabilityPolicy, CheckpointMemoryLimit,
    ConfiguredPhysicalDurabilityDeclaration, GroupCommitDelay, GroupCommitLimit,
    IdempotencyRetentionGenerations, LiveIdempotencyBindingLimit, PendingUnresolvedMutationLimit,
    PhysicalCheckpointPolicy, PhysicalCheckpointStartDeferred, PhysicalCheckpointStartDenial,
    PhysicalCheckpointStartFailure, PhysicalCheckpointStartOutcome,
    PhysicalCheckpointStartRebindRequired, PhysicalCheckpointStartStale,
    PhysicalDurabilityDeclaration, PhysicalDurabilityDeclarationBuilder,
    PhysicalDurabilityPolicyAdmissionOutcome, PhysicalDurabilityPolicyDeferred,
    PhysicalDurabilityPolicyDenial, PhysicalDurabilityPolicyFailure,
    PhysicalDurabilityPolicyIdentity, PhysicalDurabilityPolicyRebindRequired,
    PhysicalDurabilityPolicyStale, PhysicalIdempotencyPolicy, PhysicalWalPolicy,
    RetainedWalTailLimit, WalSegmentByteLimit, WalSegmentInventoryLimit,
};

pub(in crate::physical_runtime) use admission::{
    PhysicalDurabilityRuntimeOwner, PhysicalDurabilityRuntimeRebind,
    ReopenedPhysicalDurabilityRuntimeOwner, bind_policy_to_runtime,
};
pub(in crate::physical_runtime) use checkpoint::{
    CompletedDurableCheckpointWitness, NamespaceDurableCheckpointPublication,
    NamespaceDurablePhysicalBindingCompactionReopen, PhysicalCheckpointCaptureFoundation,
    PhysicalCheckpointRuntimeOwner, PhysicalCheckpointWorkPort, ReopenedPhysicalBindingCompaction,
    reopen_binding_compaction, select_no_release_marker,
};
pub use checkpoint::{
    CompletedPhysicalCheckpoint, ContiguousRetainedWalTail, IndeterminatePhysicalCheckpoint,
    PhysicalBindingCompactionReopenFailure, PhysicalCheckpointCancellationOutcome,
    PhysicalCheckpointCaptureBasis, PhysicalCheckpointCaptureFailureKind,
    PhysicalCheckpointDeadline, PhysicalCheckpointDisposal, PhysicalCheckpointHandle,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome, PhysicalCheckpointPauseGate,
    PhysicalCheckpointPoll, PhysicalCheckpointProgress, PhysicalCheckpointProgressPhase,
    PhysicalCheckpointProvenNoEffectCause, PhysicalCheckpointRequest, PhysicalCheckpointShutdown,
    PhysicalCheckpointStep, PhysicalCheckpointSubmission, ProvenNoEffectPhysicalCheckpoint,
    RetainedWalSegment,
};
pub(in crate::physical_runtime) use closeout::PhysicalIdempotencyCloseoutDenial;
pub use closeout::{
    PhysicalArtifactResidueClassification, PhysicalBackendDurabilityCloseoutEvidence,
    PhysicalDurabilityCloseoutDenial, PhysicalDurabilityCloseoutOutcome,
    PhysicalDurabilityRecoveryHandoff, PhysicalRecoveryAllocationAdmission,
    PhysicalRecoveryAttemptBindingFact, PhysicalRecoveryCheckpointBasis,
    PhysicalRecoveryCompletedMutationFact, PhysicalRecoveryOperationFact,
    PhysicalRecoveryOperationFate, PhysicalRecoveryOperationFateCounts,
    PhysicalRecoveryOperationFates, PhysicalRecoveryRootBasis, PhysicalRecoveryWalAttemptBinding,
    PhysicalRecoveryWalSegment, PhysicalRecoveryWalTail, PhysicalRootNamespaceDurabilityEvidence,
};
pub use data::{
    CertifiedPriorPageBasis, CertifiedPriorPageImage, IndeterminatePhysicalDataDispatch,
    PageWalBasis, PhysicalDataDispatchFailureCause, PhysicalDataDispatchOutcome,
    PhysicalDataEffectSettlement, PhysicalDataEffectSource, PhysicalDataFrameIdentity,
    PhysicalDataFrameKind, PhysicalDataFrameSubject, PhysicalDataSettlementFailureCause,
    PhysicalDataSettlementOutcome, PhysicalExtentCopySettlementObservation, PhysicalRedoLsn,
    PhysicalRedoTargetClaim, SuspendedPhysicalDataDispatch,
};
pub(in crate::physical_runtime) use data::{
    CompletionBoundPhysicalDataSettlement, PhysicalDataPlanBindingDenial,
    PreparedPhysicalDataFrame, PreparedPhysicalDataPlan, WalBoundPhysicalDataFrame,
    WalBoundPhysicalDataPlan, join_dispatched_data,
};
pub use evidence_projection::{
    CheckpointPerformanceExpectation, CloseoutPerformanceExpectation,
    GroupCommitPerformanceExpectation, IdempotencyPerformanceExpectation,
    IndeterminatePhysicalMutationEvidence, PageBasisPerformanceExpectation,
    PhysicalDurabilityPerformanceClaim, PhysicalDurabilityPerformanceContract,
    PhysicalDurabilityPerformanceEvidenceDenial, PhysicalDurabilityPerformanceSummary,
    PhysicalIoPerformanceExpectation, PhysicalMutationExecutedBoundaryEvidence,
    PhysicalMutationPerformanceEvidence, PhysicalQueuePerformanceExpectation,
    PhysicalTrafficPerformanceExpectation, ProvenNoEffectPhysicalMutationEvidence,
    StorePhysicalDurabilityPerformanceReceiptEvidence,
    lower_physical_durability_performance_receipt,
};
#[cfg(test)]
pub(in crate::physical_runtime) use grouping::reopened_membership_digest;
pub(in crate::physical_runtime) use grouping::reopened_membership_digest_fields;
pub use grouping::{
    AdmittedPhysicalDurabilityGroup, AdmittedPhysicalDurabilityGroupMember,
    DataSettledPhysicalMutationMembers, IndeterminatePhysicalWalGroupBarrier,
    PhysicalDataSettledGroupAdmissionOutcome, PhysicalDataSettledGroupDenial,
    PhysicalDurabilityGroupAdmissionDenial, PhysicalDurabilityGroupAdmissionOutcome,
    PhysicalDurabilityGroupBasis, PhysicalDurabilityGroupIdentity,
    PhysicalDurabilityGroupMemberBinding, PhysicalDurabilityGroupSealingDenial,
    PhysicalGroupAppendAmplificationObservation, PhysicalGroupBarrierAmplificationObservation,
    PhysicalGroupMemberOrdinal, PhysicalGroupQueueAdmissionTick, PhysicalGroupRootPublicationPlan,
    PhysicalWalBarrierSettlement, PhysicalWalGroupBarrierDeclaration,
    PhysicalWalGroupBarrierDeclarationDenial, PhysicalWalGroupBarrierFailureCause,
    PhysicalWalGroupBarrierOutcome, PhysicalWalGroupBarrierSettlement,
    RejectedDataSettledPhysicalMutationMembers, RejectedPhysicalDurabilityGroup,
    SealedPhysicalDurabilityGroupMembers, SharedPhysicalRootPublicationPlan, WalBarrierMember,
    WalDurablePhysicalMutationMembers,
};
pub(in crate::physical_runtime) use grouping::{
    CompletionBoundPhysicalWalBarrierSettlement, PhysicalDurabilityGroupSealingFailure,
    PhysicalDurabilityGroupingRuntimeAuthority, PhysicalDurabilityGroupingRuntimeOwner,
    PhysicalWalGroupBarrierPort,
};
pub use lifecycle::PhysicalMutationShutdown;
pub(in crate::physical_runtime) use lifecycle::{
    ExistingPhysicalMutationObservation, PhysicalMutationCostSnapshot,
    PhysicalMutationRuntimeOwner, PhysicalMutationStartPort, PhysicalMutationTerminalState,
};
pub use lifecycle::{PhysicalMutationCheckpoint, PhysicalMutationPauseGate};
#[cfg(feature = "certification-test-authority")]
pub use lifecycle::{
    PhysicalMutationCheckpoint as CertificationPhysicalMutationCheckpoint,
    PhysicalMutationPauseGate as CertificationPhysicalMutationPauseGate,
};
pub(in crate::physical_runtime) use mutation::PhysicalBindingDecodingContext;
pub(in crate::physical_runtime) use mutation::{
    AdmittedPhysicalMutation, AllocatedPhysicalMutationAttemptBinding,
    CompletedPhysicalMutationFact, PersistedPhysicalMutationAttemptBinding,
    PhysicalMutationAttempt, PhysicalMutationBindingCompactionCutover,
    PhysicalMutationBindingCompactionRuntimeAuthority, PhysicalMutationDurabilityRequest,
    PhysicalMutationFingerprintInput, PhysicalMutationGroupSealingBinding,
    PhysicalMutationIdempotencyGroupSealDenial, PhysicalMutationIdempotencyRegistryAdmission,
    PhysicalMutationIdempotencyRegistryAdmissionError, PhysicalMutationIdempotencyRegistryDenial,
    PhysicalMutationIdempotencyRuntimeAuthority, PhysicalMutationIdempotencyRuntimeOwner,
    PhysicalMutationOperationFamily, PhysicalMutationPayloadDigest,
    PhysicalMutationPreSealCancellationDenial, PhysicalMutationRequestScope,
    PhysicalMutationSecurityBasis, PhysicalMutationTerminalFact,
    PhysicalMutationTerminalizationDenial, PhysicalMutationUnresolvedBindingObservation,
    PhysicalOriginalDropCompleted, PhysicalOriginalDropNoEffect,
    PhysicalRecoveredOriginalDropNoDurableEffect, RebuiltPhysicalMutationIdempotency,
    SettledPhysicalMutationBasis, WalRangeReservedPhysicalMutationBasis, rebuild_idempotency,
};
pub use mutation::{
    CompletedPhysicalMutation, DataDispatchedPhysicalMutation, DataSettledPhysicalMutation,
    PhysicalIdempotencyReopenFailure, PhysicalMutationBindingCompaction,
    PhysicalMutationCancellationOutcome, PhysicalMutationDeadline, PhysicalMutationHandle,
    PhysicalMutationIdempotencyIssuanceDenial, PhysicalMutationIdempotencyKey,
    PhysicalMutationIdempotencyKeyIdentity, PhysicalMutationIdempotencyLease,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationIdentity, PhysicalMutationOutcome,
    PhysicalMutationPoll, PhysicalMutationProgress, PhysicalMutationProgressPhase,
    PhysicalMutationRequest, PhysicalMutationRequestFingerprint,
    PhysicalMutationTerminalObservation, PhysicalNamespaceDurableCheckpointGeneration,
    RootNamespaceDurablePhysicalMutationMembers, RootPublicationPhysicalMutationMember,
    RootPublicationPreparedPhysicalMutationMembers, RootReplacedPhysicalMutationMembers,
    WalAppendedPhysicalMutation, WalDurablePhysicalMutation, WalRangeReservedPhysicalMutation,
};
#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) use mutation::{
    DecodedPhysicalMutationBindingRecord, PersistedPhysicalMutationFate,
    PhysicalBindingCompactionRecordDecodeDenial, PhysicalPersistedBindingDecodeDenial,
};
pub(in crate::physical_runtime) use mutation::{
    TerminalHeadNoRetryClaim, TerminalHeadRetryClaimDenial,
};
pub use observation::PhysicalMutationObservation;
pub use observation::{PhysicalDurabilityObservation, PhysicalDurabilityReopenObservation};
pub(in crate::physical_runtime) use observation::{
    PhysicalMutationCancellationClass, PhysicalMutationObservationCounters,
    PhysicalMutationTerminalClass,
};
#[cfg(feature = "recovery-runtime-owner")]
pub(in crate::physical_runtime) use publication::RecoveredReleaseLedgerDenial;
pub(in crate::physical_runtime) use publication::{
    AdmittedFailedIngestDrop, AdmittedManifestResidueRetirement, AdmittedReleasedGenerationDrop,
    CheckpointCertificateFrame, CheckpointCustodyCandidate, CheckpointCustodyDenial,
    CheckpointCustodyOrigin, CleanReopenCheckpointCustody, ManifestResidueDisplacement,
    ManifestResidueProof, NamespaceDurableManifestResidueRoot, NamespaceDurableRetirementRoot,
    PhysicalBlobReclaimAdmissionDenial, PhysicalBlobSessionClaim, PhysicalBlobSessionClaimDenial,
    PhysicalBlobTerminalAdmissionDenial, PhysicalCurrentRootOwner, PhysicalReclaimAttempt,
    PhysicalReconciledReclaimDescriptorFate, PhysicalRootPublicationIdentity,
    PhysicalRootPublicationPreparationFailure, PhysicalRootPublicationPreparationNotStartedCause,
    PhysicalRootPublicationTransition, PhysicalRootPublicationWorkFailure,
    PhysicalRootPublicationWorkPort, PreparedRecoveredCheckpointCustody,
    ReleaseCertificateCapacityLease, ReleaseHeadCapacityCharge, ReleasedDropSourceCaptureDenial,
    RootCandidateSynchronizationFailure, SelectedCheckpointCustodySnapshot,
    SelectedOriginalDropProof, SelectedReleaseHeadBasis, ServingCheckpointCustody,
    publish_manifest_residue_candidate, publish_retirement_candidate, replace_root_candidate,
    synchronize_root_namespace,
};
pub(in crate::physical_runtime) use publication::{
    AdmittedTerminalHeadRetirement, CheckpointAttestedTerminalHead, PublicationStateLockHeld,
    TerminalHeadAttestationDenial, TerminalHeadPublicationExcluded,
    TerminalHeadRetirementAdmissionDenial, TerminalHeadRetirementAuthority,
};
#[cfg(feature = "certification-test-authority")]
pub use publication::{
    CertificationReadRootCapturePauseGate, CertificationReadRootCaptureStage,
    CertificationReleaseHeadObservation,
};
pub use publication::{
    CompletedPhysicalRootPublication, IndeterminatePhysicalCurrentRootAdvance,
    IndeterminatePhysicalRootNamespaceDurability, IndeterminatePhysicalRootPublicationPreparation,
    IndeterminatePhysicalRootReplacement, PhysicalCurrentRootAdvanceFailureCause,
    PhysicalCurrentRootAdvanceOutcome, PhysicalRootCandidateSynchronizationFailureCause,
    PhysicalRootCandidateWriteFailureCause, PhysicalRootCandidateWriteFailurePosture,
    PhysicalRootNamespaceDurabilityFailureCause, PhysicalRootNamespaceDurabilityNotStarted,
    PhysicalRootNamespaceDurabilityOutcome, PhysicalRootPublicationMemberIdentity,
    PhysicalRootPublicationPreparationFailureCause, PhysicalRootPublicationPreparationNotStarted,
    PhysicalRootPublicationPreparationOutcome, PhysicalRootPublicationTransitionDenial,
    PhysicalRootPublicationWorkFailureCause, PhysicalRootReplacementFailureCause,
    PhysicalRootReplacementNotStarted, PhysicalRootReplacementOutcome, RetainedPhysicalRoot,
};
#[cfg(feature = "certification-test-authority")]
pub(in crate::physical_runtime) use publication::{
    NamespaceDurableTierEpochRoot, publish_tier_epoch_candidate,
};
pub use settlement::{
    CompletedUnobservedPhysicalMutation, IndeterminatePhysicalMutation,
    PhysicalMutationAcknowledgment, PhysicalMutationCompletedBreadth,
    PhysicalMutationIndeterminateStage, PhysicalMutationPreSealAdmissionDetail,
    PhysicalMutationProvenNoEffectCause, PhysicalMutationRootPreparationFailure,
    PhysicalRootPreparationEffectPosture, ProvenNoEffectPhysicalMutation,
};
#[cfg(test)]
pub(in crate::physical_runtime) use wal::RetainedWalHistory;
pub use wal::{
    CanonicalRedoRecords, IndeterminatePhysicalWalGroupAppend, PhysicalWalAppendDeclaration,
    PhysicalWalAppendFailureCause, PhysicalWalAppendSettlement, PhysicalWalFrameWriteDisposition,
    PhysicalWalGroupAppendContinuation, PhysicalWalGroupAppendFailureCause,
    PhysicalWalGroupAppendOutcome, PhysicalWalMemberBasis, PhysicalWalMemberIdentity,
    PhysicalWalObservation, PhysicalWalOpenFailure, PhysicalWalReclamationObservation,
    PhysicalWalReclamationReport, PhysicalWalReservationDenial, RedoRecord,
};
pub(in crate::physical_runtime) use wal::{
    CompletionBoundPhysicalWalAppendSettlement, PhysicalWalAppendPort,
    PhysicalWalBindingReopenCutoff, PhysicalWalReclamationFoundation, PhysicalWalReclamationOwner,
    PhysicalWalRuntimeOwner, ReopenedWalPublicationGroup, ReservedPhysicalWalGroupMembers,
    RetainedWalReleaseEvidence, ScheduledMaintenanceDenial, reopen_wal_inventory,
};

pub(in crate::physical_runtime) use mutation::ManagedPublicationRegistration;
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
