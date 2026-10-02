#![forbid(unsafe_code)]

#[cfg(feature = "certification-test-authority")]
mod certification;
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
    HistoricalDropAdmissionStage, PhysicalManifestObservationDenial,
    PhysicalRecoveryAdmissionCounters, PhysicalRecoveryBlock, PhysicalRecoveryBlockEvidence,
    PhysicalRecoveryBlockKind, PhysicalRecoveryCheckpointIntegrityDenial,
    PhysicalRecoveryConfigurationDenial, PhysicalRecoveryEntryBindingDrift,
    PhysicalRecoveryIntegrityObservations, PhysicalRecoveryLimitDeclaration,
    PhysicalRecoveryLimitDenial, PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure,
    PhysicalRecoveryLimits, PhysicalRecoveryMediaObservationFailure, PhysicalRecoveryOpenRequest,
    PhysicalRecoveryOrderedReleaseDenial, PhysicalRecoveryOrderedReleaseJoin,
    PhysicalRecoveryOrderedReleaseStorage, PhysicalRecoveryOutcome,
    PhysicalRecoveryPageAdmissionDenial, PhysicalRecoveryPlanningDenial,
    PhysicalRecoveryPlatformAdmissionError, PhysicalRecoveryPlatformAuthority,
    PhysicalRecoveryPublicationCounters, PhysicalRecoveryPublicationDenial,
    PhysicalRecoveryPublicationIndeterminate, PhysicalRecoveryPublicationSettlement,
    PhysicalRecoveryPublicationSettlementLedger, PhysicalRecoveryRefusal,
    PhysicalRecoveryRefusalKind, PhysicalRecoveryReleaseHeadControlDenial,
    PhysicalRecoveryReleaseHeadReadDenial, PhysicalRecoveryReleaseHeadWalkDenial,
    PhysicalRecoveryReopenCounters, PhysicalRecoveryReopenFailure,
    PhysicalRecoveryRootProtocolArtifact, PhysicalRecoveryRootProtocolCounters,
    PhysicalRecoveryRootProtocolDenial, PhysicalRecoverySelectedRecordReadDenial,
    PhysicalRecoverySelectedReleaseHeadDenial, PhysicalRecoverySessionIdentity,
    PhysicalRecoverySourceDenial, PhysicalRecoverySourceReadAllocationBoundary,
    PhysicalRecoverySourceReadAllocationDenial, PhysicalRecoveryStagingCounters,
    PhysicalRecoveryStagingDenial, PhysicalRecoveryStagingSettlement,
    PhysicalRecoveryStagingSettlementLedger, PhysicalRecoveryStaticConfiguration,
    PhysicalRecoverySuccessorCandidateDenial, PhysicalRecoverySuccessorCandidateMismatch,
    PhysicalRecoveryWalIntegrityDenial, PhysicalRecoveryWalIntegrityObservation,
    PhysicalRecoveryWalIntegrityObservationOutcome, PhysicalRecoveryWalInventoryAllocationBoundary,
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

    /// Runs ordinary C8 ingress and Store's exact same-media rejoin under the
    /// certification-only authority. The pause is after genuine selected
    /// custody mint and before Store re-reads the selected media.
    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_recover_with_custody_pause(
        request: PhysicalRecoveryOpenRequest,
        pause: impl FnOnce(worth_store_physical_format::CurrentPhysicalRecordPlacement) + 'static,
    ) -> PhysicalRecoveryOutcome {
        Self::certification_recover_with_custody_pauses(request, pause, || {})
    }

    /// A second feature-only hook exercises Store's final media reread after
    /// its initial selected checkpoint/control/WAL join.
    #[cfg(feature = "certification-test-authority")]
    #[doc(hidden)]
    pub fn certification_recover_with_custody_pauses(
        request: PhysicalRecoveryOpenRequest,
        after_claim: impl FnOnce(worth_store_physical_format::CurrentPhysicalRecordPlacement) + 'static,
        before_final_rejoin: impl FnOnce() + 'static,
    ) -> PhysicalRecoveryOutcome {
        let _scope = certification::CertificationScope::enter(after_claim, before_final_rejoin);
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
