#![forbid(unsafe_code)]
//! Physical isolation authority must consume executed lower evidence.
//!
//! A compaction read verdict cannot be minted from an admitted plan alone:
//!
//! ```compile_fail
//! let _shortcut =
//!     worth_store_physical_isolation::execute_admitted_compaction_rewrite_for_plan;
//! ```
//!
//! A byte-guard scope derives its protected reference from the Store chunk.
//! A caller cannot pair an unrelated reference with borrowed bytes:
//!
//! ```compile_fail
//! use worth_store::physical_runtime::PhysicalRecordChunkView;
//! use worth_store_physical_isolation::{
//!     CurrentGenerationPhysicalReference, PhysicalByteGuardScope,
//! };
//!
//! fn pair_unrelated_reference(
//!     reference: CurrentGenerationPhysicalReference,
//!     chunk: &PhysicalRecordChunkView<'_>,
//! ) {
//!     let _ = PhysicalByteGuardScope::for_record_chunk(reference, chunk);
//! }
//! ```
//!
//! Replica bootstrap cannot lease a caller-built raw source request. It must
//! consume the cut resolved by Recovery Physics from independently reopened
//! media:
//!
//! ```compile_fail
//! use worth_store_physical_isolation::{
//!     RecoverySourceLeaseRegistry, RecoverySourceLeaseRequest,
//! };
//!
//! fn raw_bootstrap_source_is_insufficient(
//!     registry: &RecoverySourceLeaseRegistry,
//!     raw: RecoverySourceLeaseRequest,
//! ) {
//!     let _ = registry.admit_bootstrap_source_cut(raw);
//! }
//! ```

extern crate self as worth_store_physical_isolation;

mod backup_cut;
mod blob_orphan_reclaim;
mod byte_guard;
mod checkpoint_interlock;
mod compaction_interlock;
mod epoch;
mod executed_isolation_evidence;
mod free_reuse_fence;
mod generation;
mod hazard_lease;
mod latch;
mod movable_stability;
mod physical_read_plan;
mod physical_semantic_boundary;
mod publication;
mod readiness;
mod reclaim_reachability;
mod recovery_source_lease;
mod root_protocol;
mod security_scope_propagation;
mod stable_read_execution;

pub use backup_cut::{
    abandon_backup_cut, prepare_backup_cut_abandonment, AdmittedBackupCut, BackupArtifactCoverage,
    BackupArtifactFamily, BackupArtifactReference, BackupCutAbandonmentReceipt,
    BackupCutAdmissionAuthority, BackupCutAdmissionDenial, BackupCutAdmissionRequest,
    BackupCutCoordinates, BackupCutManifest, BackupCutManifestDenial, BackupCutReadmissionDenial,
    BackupCutRecoveryDenial, BackupCutRecoveryRecord, BackupCutReleaseMismatch,
    BackupCutStoragePosture, BackupCutStoragePostureDenial, BackupLeaseOverlap,
    BackupReachabilityLease, BackupReachabilityLeaseHolderId, BackupReachabilityLeaseIndexSnapshot,
    BackupReachabilityLeasePersistenceRecord, BackupReachabilityLeaseRecoveryDenial,
    BackupReachabilityLeaseRegistry, BackupReachabilityLeaseRegistryDenial,
    BackupReachabilityLeaseReleaseRecord, InvalidBackupReachabilityLeaseReleaseRecord,
    PendingBackupLeaseAdmission, PendingBackupLeaseRelease, PersistedBackupReachabilityLease,
    PreparedBackupCutAbandonment, ReleasedBackupReachabilityLease, UntrustedBackupArtifactClaim,
};
pub use blob_orphan_reclaim::{
    BlobOrphanReclaimBarrier, BlobOrphanReclaimCounterSnapshot, BlobOrphanReclaimCoverage,
    BlobOrphanReclaimDenial, BlobOrphanReclaimIdentity, BlobOrphanReclaimProof,
    BlobPartialChunkOrphan,
};
pub use byte_guard::{
    ByteGuardReleaseReceipt, PhysicalByteGuard, PhysicalByteGuardDenial, PhysicalByteGuardScope,
};
#[cfg(any(test, feature = "certification-authority"))]
pub use checkpoint_interlock::read_during_checkpoint_verdict_for_certification_test;
pub use checkpoint_interlock::{
    checkpoint_flush_scheduler_demand, reject_copied_checkpoint_report_as_checkpoint_interlock,
    reject_same_run_self_comparison_as_checkpoint_interlock, CheckpointInterlockEvidenceOrigin,
    CheckpointInterlockFoundationalEvidence, CheckpointPublicationReadmission,
    CheckpointPublicationStabilityProof, CheckpointReadInterlockCounters,
    CheckpointReadInterlockDenial, CheckpointReadInterlockPlan, CheckpointRootEpochTransition,
    ReadDuringCheckpointVerdict,
};
pub use compaction_interlock::{
    compaction_owner_case_inventory, compaction_rewrite_scheduler_demand,
    execute_read_during_compaction_cutover, CompactionCandidateRangeSet, CompactionCutoverDelta,
    CompactionCutoverStabilityProof, CompactionCutoverState, CompactionDeferredReclaimQueue,
    CompactionInterlockFoundationalEvidence, CompactionMutationLaneOrigin,
    CompactionMutationLaneReceipt, CompactionMutationLaneReceiptKind,
    CompactionOwnerCaseDeclaration, CompactionOwnerCaseId, CompactionOwnerCaseObservation,
    CompactionProtectedReferenceSet, CompactionReadInterlockCounters,
    CompactionReadInterlockDenial, CompactionReadInterlockPlan, CompactionRecoveryEvidence,
    CompactionRewritePublication, CompactionSourceIntegrityAdmission,
    CompactionSourceIntegrityAdmissionDenial, CompactionSourceIntegrityEvidence,
    DrainedCompactionReclaim, ReadDuringCompactionVerdict,
};
#[cfg(any(test, feature = "certification-authority"))]
pub use epoch::next_root_epoch_for_certification;
pub use epoch::{
    compare_physical_epoch_vectors_with_evidence, required_physical_isolation_ordering_contracts,
    ChunkEpoch, EpochComparisonScope, EpochComparisonScopeMismatch, EpochRetryDecision,
    EpochStabilityScopeKind, ExtentEpoch, ExtentPublicationEpochBasis,
    FutureChunkPublicationEpochBasis, ManifestEpoch, PageEpoch, PagePublicationEpochBasis,
    PhysicalEpochComparisonEvidence, PhysicalEpochComparisonEvidenceDenial, PhysicalEpochDriftKind,
    PhysicalEpochFreshness, PhysicalEpochFreshnessBasis, PhysicalEpochFreshnessProofArtifact,
    PhysicalEpochFreshnessProofEvidence, PhysicalEpochFreshnessProofPhase, PhysicalEpochVector,
    PhysicalEpochVectorBuilder, PhysicalEpochVectorDenial, PhysicalOrderingContract,
    PhysicalOrderingContractDenial, PhysicalOrderingSite, PhysicalOrderingStrength, RootEpoch,
    SegmentEpoch, SegmentPublicationEpochBasis, StalePhysicalReadPlanDenial,
};
pub use executed_isolation_evidence::{
    reject_foundational_projection_as_physical_isolation_store_authority,
    reject_log_or_json_projection_as_physical_isolation_store_authority,
    reject_planned_or_support_projection_as_physical_isolation_store_authority,
    reject_projection_as_latch_order_proof_authority,
    reject_projection_as_physical_epoch_basis_authority,
    reject_projection_as_reclaim_eligibility_proof_authority,
    reject_projection_as_stable_physical_read_plan_authority,
    reject_proof_projection_as_physical_isolation_store_authority,
    PhysicalIsolationEvidenceProfile, ProjectionArtifactKind, ProjectionAuthorityDenial,
    S5IsolationEvidenceRichness, StorePhysicalAuthoritySurface,
};
pub use executed_isolation_evidence::{
    ExecutedIsolationBasis, ExecutedIsolationCounterKind, ExecutedIsolationEvidence,
    ExecutedIsolationEvidenceDenial, ExecutedIsolationReceipts, IsolationInterferenceCounterName,
    IsolationInterferenceSnapshot, IsolationInterferenceSnapshotRow,
    PhysicalIsolationCounterSnapshot,
};
pub use free_reuse_fence::{
    AllocatorPublicationReceipt, CrashStableReclaimReuseFence, FreeReuseFenceDenial,
    GenerationAdvanceReceipt,
};
pub use generation::{
    CurrentGenerationPhysicalReference, GenerationCountedPhysicalReference,
    GenerationCountedReferenceDenial, PhysicalReferenceGenerationMismatch,
    PhysicalReferenceGenerationMismatchKind,
};
pub use hazard_lease::{
    ActiveHazardLease, HazardLeaseCounterSnapshot, HazardLeaseDenial,
    HazardLeaseEpochIndexSnapshot, HazardLeaseGeneration, HazardLeaseKind, HazardLeaseOverlap,
    HazardLeaseReleaseReceipt, HazardLeaseSlot, HazardLeaseTable, HazardLeaseTableCapacity,
    LeaseExpiryPosture, OwnedCopyStableReadReceipt, ProtectedReferenceLease,
    ReadHandleRevocationReceipt,
};
pub use latch::{
    latch_counter_backed_performance_receipt, lower_latch_acquisition_plan,
    pre_wait_denial_for_execution_time_latch_discovery, pre_wait_denial_for_hierarchy_inversion,
    pre_wait_denial_for_unauthorized_latch_upgrade, pre_wait_denial_for_unordered_latch_set,
    CanonicalLatchAcquisitionOrder, DeadlockDetectionReport, DeadlockPreventionDenial,
    LatchAcquisitionDenial, LatchAcquisitionPlan, LatchAcquisitionRequest, LatchAcquisitionStep,
    LatchCounterEvidenceDenial, LatchCounterPerformanceReceipt, LatchDeniedBeforeWaitEvidence,
    LatchOrderProof, LatchUpgradeAuthority, LatchWaitCounterSnapshot, LatchWaitForGraph,
    LatchWaitForGraphAdmissionDenial, LatchWaitForGraphDenial, PhysicalLatchClass,
    PhysicalLatchDeadlockPolicy, PhysicalLatchFamilyDeadlockPolicy, PhysicalLatchKey,
    PhysicalLatchMode, PhysicalLatchWaitEdge,
};
#[cfg(any(test, feature = "certification-authority"))]
pub use movable_stability::physical_placement_movement_execution_for_certification_test;
pub use movable_stability::{
    tier_movement_stability_capability, ChunkMigrationReadInterlockPlan,
    FoundationalTierMovementNonClaimEvidence, FutureBlobMigrationNonClaim,
    FutureBlobMigrationNonClaimReport, FutureChunkStabilityBasis, FutureChunkStabilityRecipe,
    MovablePhysicalRef, MovablePhysicalRefKind, PhysicalChunkStabilityPlaceholder,
    PhysicalPlacementMovementExecutionReceipt, TierMovementAdmissionLabel,
    TierMovementReadInterlockPlan, TierMovementStabilityCapability,
    TierMovementStabilityCounterSnapshot, TierMovementStabilityDenial,
    TierMovementStabilityVerdict, UnsupportedTierMovementClaim, UnsupportedTierMovementRequest,
};
pub use physical_read_plan::{
    admit_seed_stable_read_plan, physical_epoch_vector_for_current_root,
    CompactProtectedReferenceSet, PhysicalReadPlanAdmissionDenial, PhysicalReadPlanFootprint,
    PhysicalReadPlanReleaseReceipt, PhysicalReadPlanReleaseSemantics, PhysicalReadPlanRetryPosture,
    PhysicalReadProtectedFootprintBasis, PhysicalReadReachabilityBarrier,
    PostProtectionPhysicalReadObservation, ProtectedPhysicalReference,
    ProtectedPhysicalReferenceSet, ProtectedReferenceRange, ProtectedReferenceRangeSet,
    ProtectedRootObservation, PublishedReaderHazard, ReadPlanAdmissionScratchArena,
    ReadPlanCounterSnapshot, ReadPlanScratchUsage, SeedStableReadPlan, StablePhysicalReadHandle,
    StablePhysicalReadPlan, StablePhysicalReadPlanAdmission, StepwiseStableReadCursor,
    TraversalAdmissionGuard, TraversalAdmissionReceipt, UnprotectedReadIntent,
    ValidatedRootObservation,
};
#[cfg(any(test, feature = "certification-authority"))]
pub use physical_semantic_boundary::physical_read_stability_authority_for_certification_test;
pub use physical_semantic_boundary::{
    admit_post_compaction_read_stability_authority,
    admit_post_publication_read_stability_authority,
    correlate_semantic_visibility_with_physical_snapshot,
    deny_semantic_visibility_as_physical_stability, PhysicalReadStabilityAuthority,
    PhysicalReadStabilityCorrelationBasis, PhysicalSemanticBoundaryDenial,
    PhysicalSemanticBoundaryOutcome, PhysicalSemanticBoundaryRoleEvidence,
    PhysicalSnapshotCorrelation, SemanticCorrelationCapability,
    SemanticVisibilityCannotMintPhysicalStability, SemanticVisibilityReference,
    SemanticVisibilityReferenceKind,
};
pub use publication::{
    AllocatorPublicationFence, AtomicPhysicalRootSwap, CopyOnWritePublicationBinding,
    CopyOnWritePublicationPlan, CrashStableFreeReusePosture, ExecutedPublicationRecoveryReceipt,
    LoweredCopyOnWritePublicationPlan, ManifestPublicationEpoch, NewRootPublicationProof,
    OldReachabilityPreservation, PhysicalIdentityReuse, PhysicalPublicationCounterSnapshot,
    PhysicalPublicationDenial, PhysicalPublicationFoundationalEvidence, PhysicalPublicationIntent,
    PhysicalPublicationIntentKind, PhysicalPublicationReadiness, PhysicalPublicationReceipt,
    PhysicalPublicationReleasePosture, PublicationCrashRecoveryOutcome, PublicationCrashStage,
    PublicationEpochPair, PublicationEpochReadiness, PublicationLatchReadiness,
    PublicationRecoveryReplayInput, PublicationRootCandidate, PublicationRootSuccessorOwner,
    RecoveredPublicationStructure, RecoveredPublicationStructureKind, ReleasedOldReachability,
    RootPublicationEpoch, RootSwapOrderingContract, ValidatedPhysicalPublicationIntent,
};
pub use readiness::{
    PhysicalIsolationAdmittedEntryRecipe, PhysicalIsolationEntryDenial,
    PhysicalIsolationEntryEvidence, PhysicalIsolationEntryFoundationalEvidence,
    PhysicalIsolationEntryIdentity, PhysicalIsolationEntryProofProgression,
    PhysicalIsolationEntryProofRequest, PhysicalIsolationEntryRebindRequired,
    PhysicalIsolationLoweredEntryRecipe, PhysicalIsolationResolvedEntryRecipe,
    PhysicalIsolationRootEpochBasis, RecoveryReadinessBasis,
};
pub use reclaim_reachability::{
    reject_backend_residue_as_reclaim_authority,
    reject_copied_read_plan_fields_as_reclaim_authority,
    reject_current_root_absence_as_reclaim_authority, reject_lease_expiry_as_reclaim_authority,
    reject_raw_reader_handle_scan_as_reclaim_authority, BlockedReclaimReport, DeferredReclaimQueue,
    DeferredReclaimReceipt, ExecutedReachabilityEvidence, ReclaimCandidateSet,
    ReclaimCounterSnapshot, ReclaimDecision, ReclaimDenial, ReclaimEligibilityProof,
    ReclaimReachabilityRemovalReceipt, S6ReclaimReachabilityRemovalEvidence,
    S6ReclaimReachabilityRemovalEvidenceDenial,
};
pub use recovery_source_lease::{
    AdmittedPitrSourceCut, AdmittedRollbackSourceCut, BootstrapReachabilityLease,
    BootstrapSourceArtifact, BootstrapSourceArtifactFamily, BootstrapSourceEvidenceBinding,
    BootstrapSourceFrontier, BootstrapSourceResolutionCounters, BootstrapSourceResolutionDenial,
    BootstrapSourceResolutionRequest, PhysicalIsolationBootstrapSourceOwner, PitrReachabilityLease,
    RecoveredRecoverySourceLease, RecoverySourceLeaseDenial, RecoverySourceLeaseKind,
    RecoverySourceLeaseRegistry, RecoverySourceLeaseReleaseReceipt, RecoverySourceLeaseRequest,
    ResolvedBootstrapRecoverySourceCut, ResolvedBootstrapSourceCut, RollbackReachabilityLease,
};
pub use root_protocol::{
    readmit_current_root_for_read_plan, reject_checkpoint_root_as_current_read_authority,
    reject_manifest_locator_root_as_current_read_authority,
    reject_recovery_root_as_current_read_authority, CheckpointPublicationIdentity,
    CheckpointPublicationRoot, CheckpointPublicationRootBasis, CurrentPhysicalRoot,
    CurrentPhysicalRootBasis, ManifestLocatorRoot, ManifestLocatorRootBasis, RecoveryRoot,
    RecoveryRootBasis, RootKindMismatchDenial,
};
pub use security_scope_propagation::{
    preserve_secure_io_stable_read_scope, LogicalDecodeSecurityScopeEntry,
    SecureIoStableReadDenial, SecureIoStableReadPreservation, StableReadObservedSecurityScope,
    StableReadSecurityScopeCarrierBasis, StableReadSecurityScopePropagation,
    StableReadSecurityScopePropagationCounters, StableReadSecurityScopePropagationDenial,
    StableReadSecurityScopePropagationInput,
};
#[cfg(any(test, feature = "certification-authority"))]
pub use stable_read_execution::{
    stable_physical_read_plan_for_certification_seed,
    stable_physical_read_plan_for_certification_test,
    stable_physical_read_receipt_for_certification_test,
};
pub use stable_read_execution::{
    ByteGuardedPhysicalRead, EpochRetryReceipt, PhysicalByteGuardAdmission,
    PhysicalReadExecutionDenial, PhysicalReadIoAttempt, PhysicalReadIoPosture,
    StablePhysicalReadEpochFreshnessOutcome, StablePhysicalReadExecution,
    StablePhysicalReadExecutionCounters, StablePhysicalReadExecutionOutcome,
    StablePhysicalReadFoundationalEvidence, StablePhysicalReadReceipt,
};
