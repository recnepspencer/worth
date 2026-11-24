pub(crate) use recovery::RecoveredCodecPublicationCustody;
pub(crate) use runtime_rebuild::root_inventory::RecoveredRootInventory;
pub mod associated_effects;
pub mod authority_continuity;
pub use worth_relational_branch::internal_port::durability::authority::associated_effects::observed_memory_commit::ObservedRelationalMemoryCommit;
pub use associated_effects::ReadmittedRelationalAssociatedEffect;
pub use worth_relational_branch::internal_port::durability::authority::associated_effects::created_row_read::ReadmittedRelationalCreatedEntity;
pub use worth_relational_contracts::internal_port::durability::authority::associated_effects::refusal::RelationalAssociatedEffectRefusal;
pub use associated_effects::RelationalAssociatedEffectSettlement;
pub use associated_effects::RelationalAssociatedEffectSubject;
pub mod checkpoint_capture;
pub mod checkpoint_complete;
pub use checkpoint_complete::{
    CapturedNativeInterimAdmission, CapturedNativeInterimComponent, CapturedNativeInterimOrigin,
    CapturedRelationalCheckpointAdmissionImage, CompleteRelationalNativeCheckpoint,
};
pub mod checkpoint_components;
pub mod checkpoint_image;
pub mod checkpoint_lineage_basis;
pub mod checkpoint_member_carry;
pub mod checkpoint_memory_demand;
pub mod checkpoint_native_capture;
pub mod checkpoint_region_consistency;

pub mod checkpoint_transition;
pub mod checkpointing;
pub mod diagnostics;
mod lineage_allocation;

#[doc(hidden)]
pub trait RecoveredRelationalBranchBasisOwnerPort: Sized {
    fn from_readmission(basis: AdmittedRelationalBranchBasis) -> Self;
    fn readmitted_basis(&self) -> &AdmittedRelationalBranchBasis;
}

impl RecoveredRelationalBranchBasisOwnerPort for RecoveredRelationalBranchBasis {
    fn from_readmission(basis: AdmittedRelationalBranchBasis) -> Self {
        Self { basis }
    }
    fn readmitted_basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.basis
    }
}

#[cfg(test)]
pub mod lineage_allocation_fixtures;
#[cfg(test)]
pub mod lineage_allocation_tests;
pub mod recorded_component_readmission;

pub mod recovered_publication;
#[doc(hidden)]
pub use recovered_publication::RelationalRetentionGateTrialForTest;
pub use recovered_publication::{
    RefusedRelationalPublicationReadmission, RelationalRecordedSuccessorRefusal,
    RelationalRecoveredPublicationAdmissionCause, RelationalRecoveredPublicationCustody,
    RelationalRecoveredPublicationRetirement, RetiredRelationalRecoveredPublication,
};
pub mod fresh_source;
pub mod recovery;
pub use recorded_component_readmission::{
    RefusedRelationalRecordedComponent, RelationalRecordedComponentAdmissionCause,
};
pub use recovery::{
    PreparedRelationalHomeRecovery, ReadmittedRelationalAssociatedFork,
    ReadmittedRelationalHomeProgramCatalogs, RefusedRelationalForkReadmission,
    RelationalForkReadmissionCause, RelationalHomeProgramPlanningRoot,
    RelationalHomeRecoveryPreparationDenial,
};
pub mod runtime_rebuild;

use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;

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
