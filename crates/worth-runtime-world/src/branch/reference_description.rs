//! Transported reference coordinates, admitted only by their live World owner.
use super::{ProductBranchNameDenial, ProductBranchObservation, RuntimeWorldBranchAdmissionDenial};

pub const PRODUCT_BRANCH_REFERENCE_DESCRIPTION_VERSION: u16 = 2;

/// Comparison data containing no observation, component basis or retention.
/// Fields are intentionally editable data; only owner readmission issues custody.
/// Owner ordinals identify live owners within this process, not across restarts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductBranchReferenceDescription {
    pub format_version: u16,
    pub owner_ordinal: u64,
    pub home_owner: Option<worth_foundational::facade::DurableWorldOwnerIdentity>,
    pub branch_name: String,
    pub lifecycle_ordinal: u64,
    pub reference_generation: u64,
    pub composite_commit_ordinal: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductBranchReferenceReadmissionDenial {
    UnsupportedDescriptionVersion { supported: u16, actual: u16 },
    InvalidName(ProductBranchNameDenial),
    IncarnationChanged,
    Branch(RuntimeWorldBranchAdmissionDenial),
}

impl ProductBranchReferenceDescription {
    pub fn from_observation(observation: &ProductBranchObservation) -> Self {
        Self {
            format_version: PRODUCT_BRANCH_REFERENCE_DESCRIPTION_VERSION,
            owner_ordinal: observation.owner_identity().get(),
            home_owner: observation.owner_identity().durable(),
            branch_name: observation.branch_identity().name().as_str().to_owned(),
            lifecycle_ordinal: observation.lifecycle_incarnation().ordinal(),
            reference_generation: observation.reference_generation().get(),
            composite_commit_ordinal: observation.selected_commit().ordinal(),
        }
    }

    pub(crate) fn matches_owner(&self, owner: crate::identity::RuntimeWorldOwnerIdentity) -> bool {
        match (self.home_owner, owner.durable()) {
            (Some(expected), Some(actual)) => expected == actual,
            (None, None) => self.owner_ordinal == owner.get(),
            (Some(_), None) | (None, Some(_)) => false,
        }
    }

    pub(crate) fn matches_observation(&self, observation: &ProductBranchObservation) -> bool {
        self.matches_owner(observation.owner_identity())
            && self.branch_name == observation.branch_identity().name().as_str()
            && self.lifecycle_ordinal == observation.lifecycle_incarnation().ordinal()
            && self.reference_generation == observation.reference_generation().get()
            && self.composite_commit_ordinal == observation.selected_commit().ordinal()
    }
}
