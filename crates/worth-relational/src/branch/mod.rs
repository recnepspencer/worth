mod record_advance;
pub use basis::{
    RelationalBranchBasisRetirement, RelationalBranchBasisRetirementDenial,
    RelationalBranchBasisRetirementProgress, RetiredRelationalBranchBasis,
};
pub(crate) mod authority;
mod custody_witness;
pub use custody_witness::{RelationalBasisCustodyReadDenial, RelationalBasisCustodyWitness};
pub(crate) mod basis;
pub(crate) mod basis_admission;
pub(crate) mod basis_admission_identity;
mod basis_axis_validation;
pub(crate) mod basis_counters;
mod basis_currentness;
mod basis_currentness_runtime;
pub(crate) use worth_relational_contracts::internal_port::branch::basis_denial;
#[path = "envelope/basis_descriptor.rs"]
pub(crate) mod basis_descriptor;
mod basis_descriptor_resolution;
mod basis_identity_validation;
mod basis_observation;
pub(crate) mod basis_observation_inputs;
mod basis_readmission;
mod basis_readmission_inputs;
pub(crate) mod basis_registry;
pub(crate) mod basis_retention;
mod basis_retention_runtime;
mod canonical_runtime_frame;
pub(crate) mod checkpoint_visibility;
mod commit_change;
pub(crate) mod coordination;
pub(crate) mod fork;
mod fork_materialization;
mod fork_port;
mod fork_source_basis;
pub(crate) mod head_visibility_inputs;
pub(crate) mod historical_selection;
pub(crate) mod home_program_step;
pub mod maintenance;
#[path = "envelope/retained_allocation.rs"]
pub(crate) mod retained_allocation;
mod retained_fork_admission;
mod retained_fork_source;
pub(crate) use worth_relational_contracts::internal_port::branch::root_capture_denial;
mod root_snapshot_custody;
pub use retained_fork_source::AdmittedRelationalRetainedForkSource;
pub(crate) use worth_relational_contracts::internal_port::branch::identity;
pub(crate) mod lifecycle;
pub(crate) mod managed_source_affinity;
pub(crate) mod materialization;
pub(crate) mod model_ownership;
mod owner_services;
pub(crate) mod recorded_basis;
pub use model_ownership::current_publications::{
    RelationalOwnedModelOutputRoute, RelationalOwnedModelPublicationRead,
    RelationalOwnedModelPublishedRoute,
};
pub use model_ownership::{
    observe_candidate_custody_reads, RelationalCandidateCustodyReads, RelationalModelOwnership,
    RelationalModelOwnershipConflict, RelationalModelOwnershipDenial,
    RelationalModelOwnershipDomain, RelationalOwnedModelEntityBoundary,
    RelationalOwnedModelForeignRelation, RelationalOwnedModelGraphSelection,
    RelationalOwnedModelSelectionDenial,
};
pub(crate) mod reference;
pub use record_advance::{
    RelationalRecordAdvanceDenial, RelationalRecordAdvanceLimits,
    RelationalRecordAdvanceObservation,
};
pub use recorded_basis::{
    AdmittedRelationalMemoryComponent, ObservedRelationalMemoryComponent,
    RecordedRelationalCanonicalComponent, RecordedRelationalComponentReference,
    RecordedRelationalComponentTarget, RelationalRecordedComponentRefusal,
};
pub(crate) use worth_relational_contracts::internal_port::branch::reference_checkpoint_data;
pub(crate) mod reference_publication_cell;
mod reference_state;
pub(crate) mod registry;
pub(crate) use registry::RelationalSourceRootSlot;
pub(crate) mod projection_view;
pub(crate) mod root;
pub(crate) mod root_checkpoint;
mod root_execution_payload;
mod root_partition_access;
mod root_region;
pub(crate) mod root_regions;
pub(crate) mod structural_state;
pub(crate) use root::{
    retained_compilation_tree_bytes, RelationalBranchVisibilityCommitment,
    RelationalOriginalRootCommitEvidence, RelationalRootAllocationVisitor,
    RelationalRootAuthoritativeAllocationObservation, RelationalRootCommitEvidence,
    RelationalRootRegionObservation, RetainedRelationalAspectPlans,
    RetainedRelationalSchemaRegistry,
};
pub(crate) use worth_relational_contracts::internal_port::branch::result_snapshot_outcome;
mod root_runtime_access;
pub(crate) mod root_selection;
mod root_selection_admission;
pub(crate) mod selected_read;
pub(crate) use selected_read::SelectedRelationalBranchRead;
mod sharing_cost_cell;
pub(crate) use worth_relational_contracts::internal_port::branch::target;
pub(crate) mod transition;
pub(crate) use worth_relational_contracts::internal_port::branch::version;

pub(crate) use authority::issue_relational_branch_mutation_authority;
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
pub(crate) use basis_admission::{
    descriptor_for_cell, issue_admitted_relational_branch_basis_with_retention,
};
pub use basis_admission_identity::RelationalBranchBasisAdmissionIdentity;
pub use basis_counters::RelationalBranchBasisCostCounters;
pub use basis_denial::{RelationalBranchBasisDenial, RelationalBranchBasisMismatchAxis};
pub(crate) use canonical_runtime_frame::{CANONICAL_RUNTIME_INSTANCE_ID, CANONICAL_TRANSACTION_ID};
pub use coordination::RelationalBranchCoordinationCellId;
pub use fork::{RelationalAssociatedFork, RelationalForkDenial, RelationalForkOutcome};
pub use fork_port::RelationalForkPort;
pub use fork_source_basis::{AdmittedRelationalForkSourceBasis, RelationalForkSourceDescriptor};
pub use identity::{RelationalBranchIdentity, RelationalBranchIdentityDenial};
pub use lifecycle::{
    ArchivedRelationalBranch, DeletedRelationalBranch, RelationalBranchArchiveDenial,
    RelationalBranchDeleteDenial, RelationalBranchDeletionOutcome, RelationalBranchDeletionPending,
    RelationalBranchLifecyclePosture,
};
pub use materialization::completion::{
    PreparedRelationalMaterializationSuspension, PreparedRelationalRematerialization,
    RelationalMaterializationError, RelationalMaterializationSuspension,
    RelationalMaterializationSuspensionCompletion, RelationalRematerializationCompletion,
    RelationalRematerializationFailure,
};
pub use materialization::{
    RelationalEntityMaterialization, RelationalMaterializationCustody,
    RelationalMaterializationRecord, RelationalRelationMaterialization,
    SuspendRelationalMaterialization,
};
pub use owner_services::{
    RelationalBranchBasisPort, RelationalBranchLifecyclePort,
    RelationalBranchTransactionAdmissionPort, RelationalCommitResultSnapshotCleanup,
    RelationalCommitResultSnapshotCleanupDenial, RelationalCommitResultSnapshotCleanupPending,
    RelationalCommitResultSnapshotCleanupProgress, RelationalCommitResultSnapshotRelease,
    RelationalCommitResultSnapshotReleaseDenial, RelationalMaterializationPort,
    RelationalOwnerLifecycleObservation, RelationalOwnerServicePorts,
    RelationalOwnerServiceRetirementDenial, RetiredRelationalOwnerServicePorts,
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
pub(crate) use root_regions::{RelationalRegionSourceWalk, RelationalRegionSourceWalkProgress};
pub(crate) use root_selection::SelectedRelationalBranchState;
pub use target::{RelationalBranchRootDescriptor, RelationalBranchTarget};
pub use transition::{
    PositionedBranchTransition, RelationalBranchTransition, RelationalBranchTransitionSequence,
    RelationalBranchTransitionStop, RelationalCommittedTransitionEvidence,
};
pub(crate) use transition::{
    RelationalBranchTransitionDurableStep, RelationalTransitionPublication,
    ReplayedBranchTransitionDivergence, ReplayedBranchTransitionStep,
};
pub use version::RelationalBranchVersion;

#[cfg(test)]
#[path = "basis_admission_tests.rs"]
pub(crate) mod basis_admission_tests;

#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
pub(crate) mod symbol_publication;
pub(crate) mod symbol_read;

#[cfg(test)]
mod pinned_walk_request_tests;
