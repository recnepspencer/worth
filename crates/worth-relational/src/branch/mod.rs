mod authority;
mod basis;
mod basis_admission_identity;
mod basis_axis_validation;
mod basis_counters;
mod basis_currentness;
mod basis_denial;
mod basis_descriptor_resolution;
mod basis_identity_validation;
mod basis_observation;
mod basis_readmission;
mod basis_registry;
#[cfg(test)]
pub(crate) use basis_registry::{
    assert_original_registry_busy, assert_pruned_original_registration,
    assert_successor_registration_preserved,
};
mod basis_retention;
mod canonical_runtime_frame;
mod coordination;
mod fork;
mod fork_port;
mod fork_source_basis;
mod identity;
mod lifecycle;
mod materialization;
mod owner_services;
mod recorded_basis;
mod reference;
pub use recorded_basis::{
    RecordedRelationalCanonicalComponent, RecordedRelationalComponentReference,
    RecordedRelationalComponentTarget, RelationalRecordedComponentRefusal,
};
pub(crate) use reference::CheckpointComponentCellSource;
mod reference_publication_cell;
mod reference_state;
mod registry;
pub(crate) use registry::RelationalSourceRootSlot;
mod root;
mod root_checkpoint;
mod root_partition_access;
mod root_region;
mod root_regions;
pub(crate) use root::{
    RelationalRootAllocationVisitor, RelationalRootAuthoritativeAllocationObservation,
    RelationalRootRegionObservation,
};
pub(crate) use root_regions::RelationalPartitionVisit;
mod root_runtime_access;
mod root_selection;
mod selected_read;
pub(crate) use selected_read::SelectedRelationalBranchRead;
mod sharing_cost_cell;
mod target;
mod transition;
mod version;

pub(crate) use authority::issue_relational_branch_mutation_authority;
pub(crate) use authority::issue_relational_branch_publication_authority;
pub use authority::{
    RelationalBranchMutationAuthority, RelationalBranchMutationAuthorityMarker,
    RelationalBranchObservationAuthority, RelationalBranchObservationAuthorityMarker,
    RelationalBranchPublicationAuthority, RelationalBranchPublicationAuthorityMarker,
    RelationalForkSourceAuthority, RelationalForkSourceAuthorityMarker,
};
pub use basis::{
    AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor, RelationalBranchBasisPosture,
    ResolvedRelationalBasisDescriptor, RELATIONAL_BRANCH_BASIS_DESCRIPTOR_VERSION,
};
pub(crate) use basis::{AdmittedRelationalBranchBasisInner, CheckpointComponentDescriptorView};
pub use basis_admission_identity::RelationalBranchBasisAdmissionIdentity;
pub use basis_counters::RelationalBranchBasisCostCounters;
pub use basis_denial::{RelationalBranchBasisDenial, RelationalBranchBasisMismatchAxis};
pub(crate) use basis_observation::{
    descriptor_for_cell, issue_admitted_relational_branch_basis_with_retention,
};
pub(crate) use basis_registry::{
    RelationalBasisRegistryRetirementCustody, RelationalBranchBasisRegistry,
    RelationalBranchBasisRegistryMetrics,
};
pub(crate) use canonical_runtime_frame::{CANONICAL_RUNTIME_INSTANCE_ID, CANONICAL_TRANSACTION_ID};
pub use coordination::RelationalBranchCoordinationCellId;
pub use fork::{RelationalForkDenial, RelationalForkOutcome};
pub use fork_port::RelationalForkPort;
pub use fork_source_basis::{AdmittedRelationalForkSourceBasis, RelationalForkSourceDescriptor};
pub use identity::{RelationalBranchIdentity, RelationalBranchIdentityDenial};
pub(crate) use lifecycle::RelationalBranchDeletionKind;
pub use lifecycle::{
    ArchivedRelationalBranch, DeletedRelationalBranch, RelationalBranchArchiveDenial,
    RelationalBranchDeleteDenial, RelationalBranchDeletionOutcome, RelationalBranchDeletionPending,
    RelationalBranchLifecyclePosture,
};
pub use materialization::{
    PreparedRelationalMaterializationSuspension, PreparedRelationalRematerialization,
    RelationalEntityMaterialization, RelationalMaterializationCustody,
    RelationalMaterializationError, RelationalMaterializationRecord,
    RelationalMaterializationSuspension, RelationalMaterializationSuspensionCompletion,
    RelationalRelationMaterialization, RelationalRematerializationCompletion,
    RelationalRematerializationFailure, SuspendRelationalMaterialization,
};
pub use owner_services::{
    RelationalBranchBasisPort, RelationalBranchLifecyclePort,
    RelationalBranchTransactionAdmissionPort, RelationalCommitResultSnapshotCleanup,
    RelationalCommitResultSnapshotCleanupDenial, RelationalCommitResultSnapshotCleanupPending,
    RelationalCommitResultSnapshotCleanupProgress, RelationalCommitResultSnapshotRelease,
    RelationalCommitResultSnapshotReleaseDenial, RelationalMaterializationPort,
    RelationalOwnerLifecycleObservation, RelationalOwnerServicePorts,
};
pub use reference::RelationalBranchCellDenial;
pub use reference::{
    relational_branch_observation, RelationalBranchComparisonBasis, RelationalBranchForkBasis,
    RelationalBranchObservationConstructionDenial, RelationalBranchReferenceObservation,
};
pub(crate) use reference::{
    RelationalBranchCellCheckpoint, RelationalBranchReferenceCell,
    RelationalBranchReferenceMutableState,
};
pub(crate) use reference_publication_cell::RelationalBranchPublicationCell;
pub use reference_state::RelationalBranchReferenceState;
pub use registry::RelationalForkTargetReservation;
pub(crate) use registry::{
    CheckpointBranchMetadata, CheckpointBranchMetadataCaptureDenial,
    CheckpointBranchMetadataFields, RelationalBranchReferenceRegistry,
    RelationalForkTargetReservationDenial,
};
pub use root::RelationalRootCorrectnessIndex;
pub(crate) use root::{
    PreparedRelationalBranchRootCapture, RelationalBranchRoot, RelationalBranchRootCaptureDenial,
    RelationalBranchRootIdentityIssuer, RelationalBranchRootSchemaAuthority,
    RelationalBranchRootState, RelationalRootAuthoritativeAllocationKind,
};
pub(crate) use root_checkpoint::{RelationalBranchRootCheckpoint, RelationalRootRegionCheckpoint};
pub(crate) use root_regions::{
    RelationalPersistentRegionAllocationKind, RelationalPersistentRegionSet,
    RelationalRegionSourceWalk, RelationalRegionSourceWalkProgress,
};
pub(crate) use root_selection::SelectedRelationalBranchState;
pub(crate) use sharing_cost_cell::RelationalBranchSharingCostCell;
pub use target::{RelationalBranchRootDescriptor, RelationalBranchTarget};
pub(crate) use transition::{
    LiveBranchTransitionStep, RelationalBranchTransitionDenial,
    RelationalBranchTransitionDurableStep, RelationalTransitionPublication,
    ReplayedBranchTransitionDivergence, ReplayedBranchTransitionStep,
};
pub use transition::{
    PositionedBranchTransition, RelationalBranchTransition, RelationalBranchTransitionSequence,
    RelationalBranchTransitionStop,
};
pub use version::RelationalBranchVersion;

#[cfg(test)]
#[path = "basis_admission_tests.rs"]
mod basis_admission_tests;
#[cfg(test)]
#[path = "basis_registry_scale_tests.rs"]
mod basis_registry_scale_tests;

mod symbol_read;
pub(crate) use symbol_read::SymbolAdmittedRootRead;
