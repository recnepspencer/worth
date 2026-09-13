#![forbid(unsafe_code)]

mod access;
mod access_planning;
mod artifact_family;
mod backup_verification;
mod blob_basis;
pub mod bootstrap;
mod catalog;
pub mod compaction_projection;
pub mod customization;
pub mod declarations;
pub mod evolution;
mod facade;
pub mod integrity;
mod keyspace;
mod maintenance;
pub mod materialization;
pub mod observation;
mod planning;
mod read;
mod recovery;
mod strategy;
pub mod strategy_declarations;

pub use access::execution::{
    btree_lookup_readiness_cases, degraded_scan_readiness_cases, AccessPathCounterSnapshot,
    BTreeLookupReadinessCaseId, BTreeLookupReadinessOutcome, BTreeLookupReadinessView,
    BTreeLookupReady, CounterEnvelopeViolation, DegradedScanAdmissionDenied,
    DegradedScanCounterReceipt, DegradedScanExecution, DegradedScanLoweringBasis,
    DegradedScanReadinessCaseId, DegradedScanReadinessOutcome, DegradedScanReadinessView,
    DegradedScanReady, DegradedScanRebindAdmission, DegradedScanRebindTrace,
    ExecutedLayoutOperation, LoweredBTreeLookup, LoweredDegradedExactScan,
    PhysicalDegradedExecutionDenial, PlannedCounterObservation, StaleDegradedExactScan,
};
pub use access::execution::{
    layout_degraded_scan_runtime, DegradedExactScanExecutionDenied,
    DegradedExactScanExecutionRequest, LayoutDegradedScanRuntime,
};
pub use access::shape::DegradedExactScanRequest;
pub use access::AdmittedAccessIntent;
pub use artifact_family::{
    artifact_family_admission_cases, AdmittedPhysicalArtifactFamily, ArtifactFamilyAdmissionCaseId,
    ArtifactFamilyAdmissionOutcome, ArtifactFamilyAdmissionView,
};
pub use backup_verification::{
    verify_bounded_layout_index_artifact, verify_bounded_layout_index_artifact_from_reader,
    BoundedLayoutIndexDenial, BoundedLayoutIndexObservation, BoundedLayoutIndexVerificationRequest,
    LayoutIndexBackupFormat,
};
pub use blob_basis::{BlobGenerationBasis, BlobIdentityKeyBasis};
pub use bootstrap::{
    bootstrap_catalog, bootstrap_catalog_read_cases, BootstrapCatalogAccess,
    BootstrapCatalogReadAdmission, BootstrapCatalogReadCaseId, BootstrapCatalogReadCounterSnapshot,
    BootstrapCatalogReadOutcome, BootstrapCatalogReadOutcomeView, BootstrapLayoutCatalog,
    BootstrapOnlyAccessDenied, BootstrapOnlyAccessPath, MinimalRootDiscoveryLayout,
};
pub use catalog::system_families::io_scheduler::{
    project_background_pacing, project_foreground_interference, project_scheduler_reservation,
    BackgroundPacingInterferencePosture, BackgroundPacingLayoutReport,
    ForegroundInterferenceAccessBudget, ForegroundInterferenceLayoutReport,
    ForegroundInterferencePosture, SchedulerReservationInterferencePosture,
    SchedulerReservationLayoutReport,
};
pub use catalog::system_families::offline_verifier::{
    project_offline_custody_capsule, project_offline_export_bundle,
    project_offline_repair_blast_radius, OfflineVerifierAccessShape,
    OfflineVerifierAuthorityPosture, OfflineVerifierEvidenceKind, OfflineVerifierLayoutProjection,
};
pub use facade::access_planning;
pub use keyspace::AdmittedConcretePhysicalKey;
pub use keyspace::{
    physical_key_domain_admission_cases, AdmittedPhysicalKeyDomain,
    PhysicalKeyDomainAdmissionCaseId, PhysicalKeyDomainAdmissionOutcome,
    PhysicalKeyDomainAdmissionView,
};
pub use maintenance::{
    derived_index_parity_cases, derived_index_rebuild_admission_cases,
    derived_index_rebuild_execution_cases, layout_lsm_maintenance, layout_mutation_admission,
    layout_mutation_admission_cases, layout_parity_verification, layout_rebuild_admission,
    layout_rebuild_candidate_readmission, layout_rebuild_execution, live_maintenance_posture,
    live_maintenance_posture_cases, lsm_maintenance_owner_case_inventory,
    AdvisoryMaintenanceCapability, DeferredMaintenanceWitness, DerivedIndexCandidateDeclaration,
    DerivedIndexCandidateReadmissionReceipt, DerivedIndexCostEnvelopeParity,
    DerivedIndexCounterShapeParity, DerivedIndexCoverageParity, DerivedIndexIdentityParity,
    DerivedIndexOrderingParity, DerivedIndexParityBasis, DerivedIndexParityCaseId,
    DerivedIndexParityCounterSnapshot, DerivedIndexParityDenied, DerivedIndexParityOutcome,
    DerivedIndexParityRow, DerivedIndexParityWitness, DerivedIndexPartialKeySpace,
    DerivedIndexRebuildAdmissionCaseId, DerivedIndexRebuildAdmissionOutcome,
    DerivedIndexRebuildAdmissionView, DerivedIndexRebuildCounterSnapshot,
    DerivedIndexRebuildDenied, DerivedIndexRebuildExecutionCaseId, DerivedIndexRebuildOutcome,
    DerivedIndexRebuildPlan, DerivedIndexRebuildReceipt, DerivedIndexRebuildRequest,
    DerivedIndexRebuildScope, DerivedIndexRebuildSourceInput, DerivedIndexRepairExecutionDenial,
    DerivedIndexRepairPlan, DerivedIndexRepairReceipt, DerivedIndexRepairRequest,
    DerivedIndexResultIdentity, IndexLagWitness, IndexMaintenanceFailureOutcome,
    IndexMaintenanceMode, IndexPublicationProtocol, LayoutLsmMaintenance, LayoutMutationAdmission,
    LayoutMutationAdmissionCaseId, LayoutMutationAdmissionOutcome, LayoutMutationAdmissionView,
    LayoutMutationPlan, LayoutOperationalRepairOwner, LayoutParityVerification,
    LayoutRebuildAdmission, LayoutRebuildCandidateReadmission, LayoutRebuildExecution,
    LazyMaintenanceCapability, LiveMaintenancePosture, LiveMaintenancePostureAdmission,
    LiveMaintenancePostureCaseId, LiveMaintenancePostureOutcome, LiveMaintenancePostureView,
    LiveMaintenanceRequest, LsmCompactionAdmissionRequest,
    LsmCompactionMaintenanceAdmissionOutcome, LsmCompactionMaintenanceAdmissionView,
    LsmMaintenanceAdmissionDenialKind, LsmMaintenanceAdmissionDenied, LsmMaintenanceDisposition,
    LsmMaintenanceOperation, LsmMaintenanceOwnerCaseDeclaration, LsmMaintenanceOwnerCaseId,
    LsmMaintenanceOwnerCaseObservation, LsmReplayAdmissionRequest,
    LsmReplayMaintenanceAdmissionOutcome, LsmReplayMaintenanceAdmissionView,
    LsmRunPublicationAdmissionOutcome, LsmRunPublicationAdmissionRequest,
    LsmRunPublicationAdmissionView, MigrationMaintenanceCapability,
    RebuildOnlyMaintenanceCapability, VerifierMaintenanceCapability,
};
pub use materialization::{
    AdmittedCoverageBasis, AdmittedLayoutMaterialization,
    BTreeLookupMaterializationAdmissionOutcome, BTreeLookupMaterializationAdmissionView,
    BTreePublicationMaterializationAdmissionOutcome, BTreePublicationMaterializationAdmissionView,
    BTreeReplayMaterializationAdmissionOutcome, BTreeReplayMaterializationAdmissionView,
    CatalogRootMaterializationAdmissionOutcome, CatalogRootMaterializationAdmissionView,
    CurrentLayoutMaterialization, CurrentMaterializationFrontier,
    ImportedBlobMaterializationAdmissionOutcome, ImportedBlobMaterializationAdmissionView,
    ImportedBlobMaterializationSourceIdentity, LayoutMaterializationSourceIdentity,
    LayoutMaterializationSourceKind, LsmLookupMaterializationAdmissionOutcome,
    LsmLookupMaterializationAdmissionView, LsmPublicationMaterializationAdmissionOutcome,
    LsmPublicationMaterializationAdmissionView, LsmReplayMaterializationAdmissionOutcome,
    LsmReplayMaterializationAdmissionView, MaterializationDenial, MaterializationFreshness,
    StaleLayoutMaterialization,
};
pub use observation::{LayoutAccessPerformanceReceipt, ObserveOwnerCase, OwnerCaseObservation};
pub use planning::{
    access_plan_selection_cases, imported_blob_read_admission_cases, AccessPlanCostClass,
    AccessPlanCostDenial, AccessPlanCostEstimate, AccessPlanIdentity, AccessPlanSelectionCaseId,
    AccessPlanSelectionDenied, AccessPlanSelectionOutcome, AccessPlanSelectionView,
    AccessPlanSelector, AdmittedPhysicalMutationRequest, AdmittedPhysicalReadRequest,
    AdmittedPhysicalRecoveryRequest, BTreeLookupOperation, ImportedBlobReadAdmissionCaseId,
    ImportedBlobReadAdmissionOutcome, ImportedBlobReadAdmissionView,
    PhysicalAccessRequestAdmissionDenied, SelectedBTreeLookup, SelectedBTreeReplayRecovery,
    SelectedDegradedExactScan, SelectedLsmCompaction, SelectedLsmLookup, SelectedLsmReplayRecovery,
    SelectedLsmRunPublication, SelectionCandidateAudit, SelectionCandidateOutcome,
    SelectionCandidateRejection, SelectionCandidateRejectionCase,
};
pub use read::{
    layout_read_runtime, LayoutReadAdmissionDenied, LayoutReadRuntime, PageLookupRequest,
    WalLookupRequest,
};
pub use recovery::{
    btree_replay_cases, layout_btree_recovery, AdmittedBTreeReplayPhysicalSource,
    AdmittedBTreeReplaySource, BTreeReplayCaseId, BTreeReplayDenialKind, BTreeReplayDenied,
    BTreeReplayLocation, BTreeReplayOutcome, BTreeReplayPhysicalSource,
    BTreeReplayPhysicalSourceIdentity, BTreeReplayRequest, BTreeReplayRootAgreement,
    BTreeReplaySourceDenial, BTreeReplayView, LayoutBTreeRecovery,
};
pub use strategy::btree::execution::{
    btree_lookup_execution_cases, btree_replay_runtime,
    decode_leaf_record as decode_baseline_btree_leaf_record,
    decode_root_record as decode_baseline_btree_root_record,
    encode_leaf_record as encode_baseline_btree_leaf_record,
    encode_root_record as encode_baseline_btree_root_record, BTreeLookupExecutionCaseId,
    BTreeLookupExecutionOutcome, BTreeLookupExecutionView, BTreeReplayReady, BTreeReplayRuntime,
    BTreeSeparatorPartitionDenial, BaselineBTreeCorruptionMarker, BaselineBTreeExactCounterWitness,
    BaselineBTreeExecutionDenial, BaselineBTreeExecutionDenialKind, BaselineBTreeExecutionWitness,
    BaselineBTreeLeafRecord, BaselineBTreeLookupAbsence, BaselineBTreeLookupAdmission,
    BaselineBTreeLookupBranch, BaselineBTreeLookupCounterReceipt, BaselineBTreeLookupExecution,
    BaselineBTreeReadPreflight, BaselineBTreeReadShape, BaselineBTreeReadSource,
    BaselineBTreeReplayAdmission, BaselineBTreeReplayRecoveryExecution, BaselineBTreeRootNode,
    StableBTreeLookupExecution,
};

#[cfg(test)]
pub(crate) use access::execution::degraded_scan_runtime;
pub use access::shape::{
    access_shapes, full_declared_scan_cases, AccessLaneClassification, AccessShapeContract,
    AccessShapeUnsupportedDenial, FullDeclaredScanBasis, FullDeclaredScanCaseId,
    FullDeclaredScanOutcome, FullDeclaredScanView,
};
pub(crate) use catalog::{
    ArtifactFamilyAuthorityWitness, ArtifactFamilyDenial, ArtifactFamilyLifecycleAdmission,
    PhysicalArtifactFamily, PhysicalArtifactFamilyDeclaration,
};
#[cfg(test)]
pub(crate) use integrity::LayoutCorruptionView;
pub(crate) use keyspace::{CanonicalKeyBytes, PhysicalKeyDomain, PhysicalKeyDomainWitness};
pub(crate) use maintenance::LayoutCorruptionClassification;
pub use maintenance::PhysicalMutationShape;
pub(crate) use materialization::LayoutCoverageWitness;
#[cfg(test)]
pub(crate) use materialization::LayoutMaterializationState;
#[cfg(test)]
pub(crate) use materialization::MaterializationStateClass;
pub(crate) use strategy::btree::execution::btree_lookup_runtime;

// Unit tests live beside their owners but compile as one crate. Keep this
// convenience vocabulary crate-private and absent from production builds.
#[cfg(test)]
pub(crate) use access::shape::{
    AccessAuthorityPosture, AccessShapeDetail, AccessStaleDisposition, BoundedScanBasis,
    DegradedExactScanBasis, ExpectedCounterClass, GroupedPrefixBasis, MaintenanceReadBasis,
    ManifestGraphWalkBasis, MultiRangeBasis, MutationAccessBasis, PrefixBasis, RangeBasis,
    StreamingContinuationBasis, StreamingReadBasis,
};
#[cfg(test)]
pub(crate) use catalog::ArtifactFamilyAccessLane;
#[cfg(test)]
pub(crate) use catalog::ArtifactScopePartitionWitness;
#[cfg(test)]
pub(crate) use evolution::{
    LayoutBindingWitness, LayoutCompatibilityWindow, LayoutEvolutionDeclaration,
    LayoutInterruptionPolicy, LayoutReadCompatibilityPosture, LayoutVersion,
    LayoutWriteCompatibilityPosture,
};
#[cfg(test)]
pub(crate) use keyspace::{CompositeKeyField, HashCollisionBehavior};
pub use strategy::{
    baseline_lsm_lookup_admission_cases, baseline_lsm_lookup_cases, lsm_compaction_runtime,
    lsm_execution_owner_case_inventory, lsm_lookup_runtime, lsm_physical_compaction_runtime,
    lsm_publication_runtime, lsm_replay_runtime, lsm_strategy, AdmittedLsmCompactionDemand,
    BaselineLsmCompactionAdmission, BaselineLsmCompactionKeyIdentity, BaselineLsmCompactionPlan,
    BaselineLsmCompactionPublicationReceipt, BaselineLsmCompactionRecordIdentity,
    BaselineLsmCompactionRecordKind, BaselineLsmCompactionTransition,
    BaselineLsmCounterObservation, BaselineLsmExecutionAdmissionDenial,
    BaselineLsmExecutionAdmissionDenialKind, BaselineLsmLookupAbsence, BaselineLsmLookupAdmission,
    BaselineLsmLookupAdmissionCaseId, BaselineLsmLookupAdmissionOutcome,
    BaselineLsmLookupAdmissionView, BaselineLsmLookupCaseId, BaselineLsmLookupCounterReceipt,
    BaselineLsmLookupDisposition, BaselineLsmLookupExecution, BaselineLsmLookupSource,
    BaselineLsmLookupView, BaselineLsmManifestPublicationExecution,
    BaselineLsmMembershipObservation, BaselineLsmReplayAdmission, BaselineLsmReplayExecution,
    BaselineLsmRunIdentity, BaselineLsmRunPublicationAdmission, InterlockedLsmCompaction,
    LayoutStrategyRegistrySnapshot, LsmCompactionPreparationOutcome, LsmCompactionPreparationView,
    LsmCompactionPublicationOutcome, LsmCompactionPublicationView, LsmCompactionRuntime,
    LsmExecutionDisposition, LsmExecutionOperation, LsmExecutionOwnerCaseDeclaration,
    LsmExecutionOwnerCaseId, LsmExecutionOwnerCaseObservation, LsmLookupRuntime,
    LsmMembershipActivationOutcome, LsmMembershipActivationView,
    LsmPhysicalCompactionBindingOutcome, LsmPhysicalCompactionBindingView,
    LsmPhysicalCompactionIntent, LsmPhysicalCompactionRuntime, LsmPublicationRuntime,
    LsmReplayExecutionOutcome, LsmReplayExecutionView, LsmReplayRuntime, LsmStrategy,
    PreparedLsmCompaction, PublishedLsmCompaction,
};
#[cfg(test)]
pub(crate) use strategy::{
    BTreeCorruptionRegion, BTreeLookupBranch, LayoutStrategyFamily, StrategyAmplificationProfile,
    StrategyDenial, StrategyLocalityProfile, StrategyLookupInvariant, StrategyPublicationInvariant,
    StrategyRebuildSourceRequirement,
};

pub(crate) use declarations::layout_declarations;
mod operational_repair;
pub use operational_repair::{
    LayoutRepairConsequence, LayoutRepairConsequenceDenial, LayoutRepairConsequenceOwner,
    LayoutRepairConsequencePlan, LayoutRepairConsequenceReceipt, LayoutRepairRegionObservation,
};
