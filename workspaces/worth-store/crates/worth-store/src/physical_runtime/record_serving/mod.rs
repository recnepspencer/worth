mod access;
mod access_policy;
mod admission;
mod canonical_read_execution;
mod evidence;
mod identity;
mod lifecycle;
mod mutation_work_port;
mod planning;
mod publication;
mod read_work_port;
mod record_queue_policy;
pub(in crate::physical_runtime) mod residency;
pub(in crate::physical_runtime) mod work_semantics;

#[cfg(test)]
pub(in crate::physical_runtime) use access::locate::assert_actual_lifecycle_manifest_denial_maps_without_damage;
#[cfg(feature = "certification-test-authority")]
pub use access::locate::CertificationPhysicalReadCallPauseGate;
pub use access::locate::{PhysicalRecordReader, RecordReadCancellation, RecordReadSession};
pub use access::read_observation::{
    RecordReadDenial, RecordReadError, RecordReadLimits, RecordReadObservation,
    RecordReadWorkDenial, StalePhysicalRecordPlacement,
};
pub use access::readmission::{
    PhysicalLocatorReadmissionDenial, PhysicalLocatorReadmissionOutcome,
};
pub use access::record_chunk_view::{PhysicalRecordChunkBasis, PhysicalRecordChunkView};
pub use access::scan::{
    PhysicalRecordScanSession, RecordScanBatch, RecordScanDenial, RecordScanOutcome,
    RecordScanRequest, ScannedPhysicalRecord,
};
pub use access::scan_observation::{
    CompletedRecordScan, RecordScanCounterSnapshot, RecordScanError,
};
pub use access::scan_readmission::ExternalRecordScanCursor;
pub use access_policy::{
    AdmittedRecordAccessPolicy, PhysicalRecordAccessPolicy, PhysicalRecordAccessPolicyBuilder,
    PhysicalRecordAccessPolicyDenial,
};
pub use admission::admission_outcome::{
    RecordServingAdmissionDeferred, RecordServingAdmissionInspectionRequired,
    RecordServingAdmissionOutcome, RecordServingAdmissionRebindRequired,
    RecordServingAdmissionStale, RecordStoreInitializationDenial, RecordStoreInitializationOutcome,
    RecordStoreOpenDenial, RecordStoreOpenOutcome,
};
pub use admission::bootstrap::{
    BootstrapCatalogReadLimits, PhysicalRecordFormatMismatch, RecordBootstrapDenial,
    RecordBootstrapFailure, RecordServingRebindReason, RecordServingStaleReason,
    UnsupportedPhysicalRecordFormat,
};
pub use admission::format_admission::AdmittedPhysicalRecordFormat;
pub use admission::request::{PhysicalRecordInitialization, PhysicalRecordOpen};
pub use admission::residency_policy::{
    AdmittedPhysicalRecordResidencyPolicy, PhysicalOperationAllocationScope,
    PhysicalRecordResidencyPolicy, PhysicalRecordResidencyPolicyBuilder,
    PhysicalRecordResidencyPolicyDenial, PhysicalRecordResidencyPolicyOutcome,
    PhysicalResidencyDimension, PhysicalSpeculativeWorkKind,
};
use canonical_read_execution::PreparedCanonicalMetadataRead;
pub(in crate::physical_runtime) use canonical_read_execution::PreparedCanonicalRecordRead;
#[cfg(feature = "certification-test-authority")]
pub use evidence::canonical_evidence::{
    lower_record_publication_canonical_basis, PhysicalRecordPublicationSummary,
    RecordCanonicalObservationDenial, RecordTopologyCanonicalBasisOutcome,
};
#[cfg(feature = "certification-test-authority")]
pub use evidence::performance_evidence::{
    lower_record_operation_performance_receipt, PhysicalRecordAccessSummary,
    PhysicalRecordPerformanceContract, RecordAppendPerformanceExpectation,
    RecordLocatePerformanceExpectation, RecordManifestPerformanceExpectation,
    RecordPerformanceEvidenceDenial, RecordScanPerformanceExpectation,
    RecordTransferPerformanceExpectation, StoreRecordPerformanceReceipt,
};
pub use identity::{ExternalPhysicalRecordLocator, PhysicalRecordId};
pub use lifecycle::record_lifecycle::RecordServingCounterSnapshot;
pub(in crate::physical_runtime) use lifecycle::record_lifecycle::RecordServingOwner;
pub use lifecycle::record_observation::{PhysicalRecordObservation, PhysicalRecordObserver};
pub(in crate::physical_runtime) use lifecycle::serving_health::ServingHealth;
pub use lifecycle::serving_runtime::ServingPhysicalRuntime;
pub use lifecycle::serving_shutdown::{
    RecordServingOwnerDisposition, RecordServingTerminalObservation, RecordServingTerminalPosture,
    ServingShutdownOutcome,
};
pub(in crate::physical_runtime) use mutation_work_port::{
    CanonicalRecordMutationFailure, CanonicalRecordMutationPort, CanonicalRecordMutationSettlement,
};
pub use mutation_work_port::{
    PhysicalRecordMutationFailureCause, PhysicalRecordMutationFailureEvidence,
};
pub use planning::placement_policy::{
    AdmittedRecordPlacementPolicy, PhysicalRecordPlacementPolicy,
    PhysicalRecordPlacementPolicyBuilder, PhysicalRecordPlacementPolicyDenial,
};
pub use planning::policy_units::{
    ManifestEntryCapacity, PageFillPercent, RecordByteLimit, RecordCountLimit, SegmentPageCount,
};
pub use planning::SettledRootProjectionMergeDenial;
pub(in crate::physical_runtime) use planning::{
    PreparedPhysicalRootProjection, RejectedSettledRootProjections,
};
pub use publication::append::{RecordAppendDenial, RecordAppendError, RecordPlacementClass};
pub use publication::append_observation::{RecordAppendObservation, RecordRootPlanningObservation};
pub use publication::batch::{RecordAppendBatch, RecordAppendBatchBuilder};
pub use publication::publication_residue::RecordPublicationResidueObservation;
pub use publication::streaming::{
    RecordStreamFailure, RecordStreamFailureKind, RecordWriteSource, RecordWriteSourceError,
};
#[cfg(feature = "certification-test-authority")]
pub use publication::CertificationPhysicalRecordSubmission;
pub use publication::RecordPublicationStage;
pub use publication::{
    PhysicalManifestCapacityTransition, PhysicalMutationAdmissionDisposition,
    PhysicalMutationPreparationDeferred, PhysicalMutationPreparationDenial,
    PhysicalMutationPreparationFailure, PhysicalMutationPreparationOutcome,
    PhysicalMutationPreparationRebindRequired, PhysicalMutationPreparationStale,
    PhysicalMutationPreparationSuccess, PhysicalMutationResourceShape,
    PhysicalPreSealCancellationDenial, PhysicalPreSealCancellationOutcome,
    PhysicalRecordSubmission, PreparedPhysicalMutation, RootPublicationCandidatePlan,
    RootPublicationPlanningMembers,
};
pub(in crate::physical_runtime) use publication::{
    PlannedPhysicalMutationParts, PreparedPhysicalMutationContext, PreparedPhysicalRootCandidate,
    PreparedRecordCompletionProjection, RecordPublicationDirector, RecordPublicationFoundation,
    WrittenRootPublicationCandidate,
};
pub(in crate::physical_runtime) use publication::{
    RootCandidateWriteFailureKind, RootCandidateWriteFailurePosture,
};
pub(in crate::physical_runtime) use read_work_port::{
    CanonicalRecordReadFailure, CanonicalRecordReadPort,
};
pub(in crate::physical_runtime) use record_queue_policy::{
    admit_checkpoint_background_policy, admit_record_queue_policy,
    admit_wal_reclamation_background_policy,
};
pub use residency::candidate_frame_residency::CandidateFrameContractViolation;
pub(in crate::physical_runtime) use residency::candidate_frame_residency::{
    CandidateFrameEffectSettlement, CandidateFrameEffectSource,
};
pub(in crate::physical_runtime) use residency::frame_ports::RecordFramePorts;
#[cfg(feature = "certification-test-authority")]
pub use residency::frame_ports::{FramePortCounterObserver, FramePortCounterSnapshot};
pub use residency::scheduled_writeback::PhysicalScheduledWritebackAdmissionDenial;
#[cfg(feature = "certification-test-authority")]
pub use residency::{
    AdmittedDirtyFrame, AdmittedPhysicalWriteback, CertificationFrameFaultCause,
    CertificationFrameReadFailure, CertificationFrameWorkFailure, CertificationResidentFrame,
    CertificationScopeAdmissionFailure, CertificationScopePressure, CertificationScopedAllocation,
    PhysicalDirtyTransitionFailure, PhysicalResidencyCertification, PhysicalWritebackExecution,
    PhysicalWritebackInspectionRequired, PhysicalWritebackSettlement,
    PhysicalWritebackTransitionFailure, PreparedPhysicalWriteback, ReadyPhysicalWriteback,
    RetryablePhysicalWriteback,
};
pub use residency::{
    BlobPhysicalAllocation, MaintenancePhysicalAllocation, PhysicalFrameFaultCause,
    PhysicalFrameReadFailure, PhysicalFrameWorkFailure, PhysicalPrefetchIntent,
    PhysicalPrefetchOutcome, PhysicalReadAheadBatch, PhysicalReadAheadFrameOutcome,
    PhysicalReadAheadIntent, PhysicalReadAheadIntentDenial, PhysicalReadAheadOutcome,
    PhysicalRecordPressureBasis, PhysicalRecordPressureEvidence, PhysicalRecordResidencyFailure,
    PhysicalRecordResidencyFailureKind, PhysicalRecordResidencyFailureReason,
    PhysicalRecordWritebackFailureCause, PhysicalRecordWritebackFailureEvidence,
    PhysicalResidencyAllocationEventSnapshot, PhysicalResidencyAllocationSnapshot,
    PhysicalResidencyCounterSnapshot, PhysicalResidencyObservation, PhysicalResidencyRetryPosture,
    PhysicalScopedAllocationAdmission, PhysicalScopedAllocationFailure,
    PhysicalSpeculativeReadDrop, PhysicalSpeculativeReadFailure, PhysicalWritebackCounterSnapshot,
    PhysicalWritebackFailureCause, RecoveryPhysicalAllocation, ScrubPhysicalAllocation,
    VerificationPhysicalAllocation,
};
#[cfg(feature = "certification-test-authority")]
pub use residency::{
    PhysicalResidencyAllocationBoundaryEvent, PhysicalResidencyAllocationBoundaryKind,
    PhysicalResidencyAllocationTrace,
};
pub(in crate::physical_runtime) use work_semantics::{RecordReadPartition, RecordWorkAdmission};
pub use worth_store_physical_format::{
    PhysicalPageSizeClass, PhysicalRecordByteOrder, PhysicalRecordFormatDeclaration,
    PhysicalRecordFormatDeclarationBuilder, PhysicalRecordFormatDenial,
    PhysicalRecordFormatVersion, PhysicalRecordIntegrity, PhysicalRecordRootProtocol,
};

pub(in crate::physical_runtime) use admission::bootstrap::RecordServingState;
pub(in crate::physical_runtime) use admission::{initialize, open};
pub(super) use planning::allocation_frontier::RecordAllocationFrontier;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_placement_and_access_rules_do_not_collapse() {
        let format = AdmittedPhysicalRecordFormat::admit(
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        );
        assert!(matches!(
            PhysicalRecordPlacementPolicy::builder()
                .extent_threshold(RecordByteLimit::new(16_384).unwrap())
                .admit(format),
            Err(PhysicalRecordPlacementPolicyDenial::ExtentThresholdCannotFitPage)
        ));
        assert!(RecordCountLimit::new(0).is_none());
        assert!(RecordByteLimit::new(0).is_none());
        assert!(SegmentPageCount::new(0).is_none());
        assert!(ManifestEntryCapacity::new(0).is_none());
        assert!(ManifestEntryCapacity::new(1).is_none());
        assert_eq!(ManifestEntryCapacity::new(2).unwrap().get(), 2);
        assert!(matches!(
            PhysicalRecordPlacementPolicy::builder()
                .manifest_capacity(ManifestEntryCapacity(1))
                .admit(format),
            Err(PhysicalRecordPlacementPolicyDenial::ManifestCapacityCannotBranch)
        ));
        assert!(PageFillPercent::new(0).is_none());
        assert!(PageFillPercent::new(101).is_none());
        assert!(matches!(
            PhysicalRecordPlacementPolicy::builder()
                .manifest_capacity(ManifestEntryCapacity::new(342).unwrap())
                .admit(format),
            Err(PhysicalRecordPlacementPolicyDenial::ManifestCapacityCannotFitPage)
        ));
        assert!(matches!(
            PhysicalRecordAccessPolicy::builder()
                .transfer_limit(RecordByteLimit::new(4096).unwrap())
                .admit(format),
            Err(PhysicalRecordAccessPolicyDenial::TransferSmallerThanPage)
        ));
        assert!(matches!(
            PhysicalRecordAccessPolicy::builder()
                .scan_record_limit(RecordCountLimit::new(u32::MAX).unwrap())
                .admit(format),
            Err(PhysicalRecordAccessPolicyDenial::ScanMetadataExceedsScratch)
        ));
        assert!(PhysicalRecordPlacementPolicy::builder()
            .admit(format)
            .is_ok());
        let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
        let bootstrap_limits = BootstrapCatalogReadLimits::for_format(format, access);
        assert_eq!(bootstrap_limits.catalog_bytes(), 82);
        assert_eq!(bootstrap_limits.current_root_bytes().get(), 16_384);
        assert_eq!(bootstrap_limits.current_root_entries(), 185);
        let permissive = PhysicalRecordAccessPolicy::builder()
            .transfer_limit(RecordByteLimit::new(u32::MAX).unwrap())
            .scratch_limit(RecordByteLimit::new(u32::MAX).unwrap())
            .admit(format)
            .unwrap();
        let fixed_limits = BootstrapCatalogReadLimits::for_format(format, permissive);
        assert_eq!(fixed_limits.current_root_bytes().get(), 16_384);
        assert_eq!(fixed_limits.current_root_entries(), 185);
    }
}
pub use read_work_port::PhysicalIntegrityScrubReadDeferral;
pub(in crate::physical_runtime) use record_queue_policy::admit_scrub_background_policy;
