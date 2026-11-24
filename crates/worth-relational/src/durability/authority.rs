pub(crate) use recovery::{AdmittedRecoveredCodecPublication, RecoveredCodecPublicationCustody};
pub(crate) use runtime_rebuild::RecoveredRootInventory;
pub(crate) mod associated_effects;
mod authority_continuity;
pub use associated_effects::{
    ObservedRelationalMemoryCommit, ReadmittedRelationalAssociatedEffect,
    ReadmittedRelationalCreatedEntity, RelationalAssociatedEffectRefusal,
    RelationalAssociatedEffectSettlement, RelationalAssociatedEffectSubject,
};
pub(crate) mod checkpoint_capture;
pub(crate) mod checkpoint_complete;
pub use checkpoint_complete::{
    CapturedNativeInterimAdmission, CapturedNativeInterimComponent, CapturedNativeInterimOrigin,
    CapturedRelationalCheckpointAdmissionImage, CompleteRelationalNativeCheckpoint,
};
pub(in crate::durability) mod checkpoint_components;
mod checkpoint_image;
#[cfg(test)]
pub(crate) use checkpoint_image::{
    recompute_branch_root_digests_for_test, recompute_branch_root_image_digest_for_test,
};
mod checkpoint_lineage_basis;
mod checkpoint_member_carry;
mod checkpoint_memory_demand;
mod checkpoint_native_capture;
mod checkpoint_region_consistency;
mod checkpoint_runtime;
pub(crate) mod checkpoint_transition;
mod checkpointing;
mod diagnostics;
pub(crate) mod lineage_allocation;
#[cfg(test)]
mod lineage_allocation_fixtures;
#[cfg(test)]
mod lineage_allocation_tests;
mod recorded_component_readmission;
mod recovered_associated_effects;
mod recovered_publication;
#[doc(hidden)]
pub use recovered_publication::RelationalRetentionGateTrialForTest;
pub use recovered_publication::{
    RefusedRelationalPublicationReadmission, RelationalRecordedSuccessorRefusal,
    RelationalRecoveredPublicationAdmissionCause, RelationalRecoveredPublicationCustody,
    RelationalRecoveredPublicationRetirement, RetiredRelationalRecoveredPublication,
};
pub(crate) mod fresh_source;
pub(crate) mod recovery;
pub use recorded_component_readmission::{
    RefusedRelationalRecordedComponent, RelationalRecordedComponentAdmissionCause,
};
pub use recovery::{
    PreparedRelationalHomeRecovery, ReadmittedRelationalAssociatedFork,
    ReadmittedRelationalHomeProgramCatalogs, RefusedRelationalForkReadmission,
    RelationalForkReadmissionCause, RelationalHomeProgramPlanningRoot,
    RelationalHomeRecoveryPreparationDenial,
};
mod runtime_rebuild;
pub(crate) use runtime_rebuild::replay_commit_stop;
pub(crate) use runtime_rebuild::RecordedReplayResultSchema;
pub(crate) use runtime_rebuild::RecoveredIndexCatalog;

use crate::branch::basis::AdmittedRelationalBranchBasis;

pub(crate) mod recovered_runtime;
pub(crate) mod runtime_access;

/// An operational branch basis proven to belong to a successfully recovered
/// Relational runtime. Only recovery authority can construct this wrapper.
#[derive(Debug)]
pub struct RecoveredRelationalBranchBasis {
    basis: AdmittedRelationalBranchBasis,
}

impl RecoveredRelationalBranchBasis {
    pub fn into_basis(self) -> AdmittedRelationalBranchBasis {
        self.basis
    }
}

pub(crate) use recovery::versioned_artifact::install_recovered_versioned_artifact;
