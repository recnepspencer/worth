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
mod basis_retention;
mod coordination;
mod fork;
mod fork_port;
mod fork_source_basis;
mod identity;
mod lifecycle;
mod owner_services;
mod reference;
mod reference_publication_cell;
mod reference_state;
mod registry;
mod root;
mod root_checkpoint;
mod root_partition_access;
mod root_region;
mod root_regions;
mod root_runtime_access;
mod root_selection;
mod sharing_cost_cell;
mod target;
mod version;

pub(crate) use authority::issue_relational_branch_mutation_authority;
pub(crate) use authority::issue_relational_branch_publication_authority;
pub use authority::{
    RelationalBranchMutationAuthority, RelationalBranchMutationAuthorityMarker,
    RelationalBranchObservationAuthority, RelationalBranchObservationAuthorityMarker,
    RelationalBranchPublicationAuthority, RelationalBranchPublicationAuthorityMarker,
    RelationalForkSourceAuthority, RelationalForkSourceAuthorityMarker,
};
pub(crate) use basis::AdmittedRelationalBranchBasisInner;
pub use basis::{
    AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor, RelationalBranchBasisPosture,
    ResolvedRelationalBasisDescriptor, RELATIONAL_BRANCH_BASIS_DESCRIPTOR_VERSION,
};
pub use basis_admission_identity::RelationalBranchBasisAdmissionIdentity;
pub use basis_counters::RelationalBranchBasisCostCounters;
pub use basis_denial::{RelationalBranchBasisDenial, RelationalBranchBasisMismatchAxis};
pub(crate) use basis_observation::{
    descriptor_for_cell, issue_admitted_relational_branch_basis_with_retention,
};
pub(crate) use basis_registry::{
    RelationalBranchBasisRegistry, RelationalBranchBasisRegistryMetrics,
};
pub use coordination::RelationalBranchCoordinationCellId;
pub use fork::{RelationalForkDenial, RelationalForkOutcome};
pub use fork_port::RelationalForkPort;
pub use fork_source_basis::{AdmittedRelationalForkSourceBasis, RelationalForkSourceDescriptor};
pub use identity::{RelationalBranchIdentity, RelationalBranchIdentityDenial};
pub use lifecycle::{
    ArchivedRelationalBranch, DeletedRelationalBranch, RelationalBranchArchiveDenial,
    RelationalBranchDeleteDenial, RelationalBranchDeletionOutcome, RelationalBranchDeletionPending,
    RelationalBranchLifecyclePosture,
};
pub use owner_services::{
    RelationalBranchBasisPort, RelationalBranchLifecyclePort,
    RelationalBranchTransactionAdmissionPort, RelationalOwnerLifecycleObservation,
    RelationalOwnerServicePorts,
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
    RelationalBranchReferenceRegistry, RelationalForkTargetReservationDenial,
};
pub use root::RelationalRootCorrectnessIndex;
pub(crate) use root::{
    PreparedRelationalBranchRootCapture, RelationalBranchRoot, RelationalBranchRootCaptureDenial,
    RelationalBranchRootIdentityIssuer, RelationalBranchRootSchemaAuthority,
    RelationalBranchRootState, RelationalRootAuthoritativeAllocationKind,
};
pub(crate) use root_checkpoint::RelationalBranchRootCheckpoint;
pub(crate) use root_regions::{
    RelationalPersistentRegionAllocationKind, RelationalPersistentRegionSet,
};
pub(crate) use root_selection::SelectedRelationalBranchState;
pub(crate) use sharing_cost_cell::RelationalBranchSharingCostCell;
pub use target::{RelationalBranchRootDescriptor, RelationalBranchTarget};
pub use version::RelationalBranchVersion;

#[cfg(test)]
#[path = "basis_admission_tests.rs"]
mod basis_admission_tests;
#[cfg(test)]
#[path = "basis_registry_scale_tests.rs"]
mod basis_registry_scale_tests;
