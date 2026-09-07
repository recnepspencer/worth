#![forbid(unsafe_code)]

mod cleanup;

mod entry;
mod handoff;
mod integrity_ingress;
mod observation;
mod orchestration;
mod progression;

pub use cleanup::{
    PerformedRecoveryCleanupRemoval, PhysicalRecoveryCleanupCancellation,
    RecoveryCleanupDeferralReason, RecoveryCleanupDisposition, RecoveryCleanupDispositionKind,
    RecoveryCleanupEligibility, RecoveryCleanupTarget,
};
pub use entry::{
    PhysicalManifestObservationDenial, PhysicalRecoveryAdmissionCounters, PhysicalRecoveryBlock,
    PhysicalRecoveryBlockEvidence, PhysicalRecoveryBlockKind,
    PhysicalRecoveryCheckpointIntegrityDenial, PhysicalRecoveryEntryBindingDrift,
    PhysicalRecoveryIntegrityObservations, PhysicalRecoveryLimitDeclaration,
    PhysicalRecoveryLimitDenial, PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure,
    PhysicalRecoveryLimits, PhysicalRecoveryMediaObservationFailure, PhysicalRecoveryOpenRequest,
    PhysicalRecoveryOutcome, PhysicalRecoveryPageAdmissionDenial, PhysicalRecoveryPlanningDenial,
    PhysicalRecoveryPlatformAdmissionError, PhysicalRecoveryPlatformAuthority,
    PhysicalRecoveryPublicationCounters, PhysicalRecoveryPublicationDenial,
    PhysicalRecoveryPublicationIndeterminate, PhysicalRecoveryPublicationSettlement,
    PhysicalRecoveryPublicationSettlementLedger, PhysicalRecoveryRefusal,
    PhysicalRecoveryRefusalKind, PhysicalRecoveryReopenCounters, PhysicalRecoveryReopenFailure,
    PhysicalRecoveryRootProtocolArtifact, PhysicalRecoveryRootProtocolCounters,
    PhysicalRecoveryRootProtocolDenial, PhysicalRecoverySessionIdentity,
    PhysicalRecoverySourceDenial, PhysicalRecoveryStagingCounters, PhysicalRecoveryStagingDenial,
    PhysicalRecoveryStagingSettlement, PhysicalRecoveryStagingSettlementLedger,
    PhysicalRecoveryStaticConfiguration, PhysicalRecoverySuccessorCandidateDenial,
    PhysicalRecoverySuccessorCandidateMismatch, PhysicalRecoveryWalIntegrityDenial,
    PhysicalRecoveryWalIntegrityObservation, PhysicalRecoveryWalIntegrityObservationOutcome,
};
pub use handoff::{
    RecoveredPhysicalRuntimeHandoff, RecoveryCleanupCounters, RecoveryCleanupDeferralEvidence,
    RecoveryCleanupEvidence, RecoveryCleanupPosture, RecoveryOperationFateSet,
};
pub use integrity_ingress::{
    RecoveryIntegrityIngressObservation as PhysicalRecoveryIntegrityObservation,
    RecoveryIntegrityIngressObservationOutcome as PhysicalRecoveryIntegrityObservationOutcome,
    RecoveryIntegrityIngressRejection as PhysicalRecoveryIntegrityRejection,
};
pub use observation::{
    RecoveryReportBlockCause, RecoveryReportCounters, RecoveryReportDecodeDenial,
    RecoveryReportDenialCause, RecoveryReportEnvelope, RecoveryReportOutcome,
    RecoveryReportRefusalCause, RECOVERY_REPORT_COMPATIBILITY_WINDOW, RECOVERY_REPORT_PROTOCOL,
    RECOVERY_REPORT_VERSION,
};
#[cfg(feature = "certification-test-authority")]
pub use progression::{complete_recovery, RecoveryCompletionDenial};
pub use progression::{
    AdmittedPhysicalRecovery, ClosedRecoveryStagingGeneration, DiscoveredPhysicalRecovery,
    NamespaceDurablePhysicalRecovery, PhysicalRecoveryDiscoveryCounters,
    PhysicalRecoveryStagingCancellation, PlannedPhysicalRecovery, RecoveryBaseImageAction,
    RecoveryBaseImagePlan, RecoveryCompletion, RecoveryPayloadManifestAction,
    RecoveryPublicationAction, RecoveryPublicationCandidateArtifact,
    RecoveryPublicationExpectation, RecoveryPublicationPlan, RecoveryQuiescencePlan,
    RecoverySegmentRoutingAction, RecoveryStagingAction, RecoveryStagingCommandPlan,
    RecoveryStagingLayoutPlan, RecoveryStagingRedoStep, ReopenedPhysicalRecovery,
    SelectedPhysicalRecovery, StagedPhysicalRecovery,
};

/// The single production composition facade for one fresh-process physical
/// recovery attempt.
pub struct WorthStoreRecovery {
    _private: (),
}

impl WorthStoreRecovery {
    pub fn recover(request: PhysicalRecoveryOpenRequest) -> PhysicalRecoveryOutcome {
        orchestration::recover(request, None)
    }

    pub fn recover_with_process_yieldpoint(
        request: PhysicalRecoveryOpenRequest,
        yieldpoint: worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint,
    ) -> PhysicalRecoveryOutcome {
        orchestration::recover(request, Some(yieldpoint))
    }
}
pub use integrity_ingress::RecoveryIntegrityIngressCounters as PhysicalRecoveryIntegrityCounters;
