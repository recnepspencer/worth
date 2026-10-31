use super::super::{identity_seal, DurableIdentity};
use super::DurableWorldOwnerIdentity;
/// Exact branch spelling, scoped to its durable owner. No normalization occurs.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableProductBranchIdentity {
    owner: DurableWorldOwnerIdentity,
    name: String,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DurableProductBranchNameDenial {
    Empty,
    TooLong,
}
impl DurableProductBranchIdentity {
    pub fn from_recorded(
        owner: DurableWorldOwnerIdentity,
        name: String,
    ) -> Result<Self, DurableProductBranchNameDenial> {
        if name.trim().is_empty() {
            return Err(DurableProductBranchNameDenial::Empty);
        }
        if name.len() > 256 {
            return Err(DurableProductBranchNameDenial::TooLong);
        }
        Ok(Self { owner, name })
    }
    pub const fn owner(&self) -> DurableWorldOwnerIdentity {
        self.owner
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}
impl identity_seal::Sealed for DurableProductBranchIdentity {}
impl DurableIdentity for DurableProductBranchIdentity {}
/// A recorded generation within an explicitly identified product branch.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableProductReferenceGeneration(u64);
impl DurableProductReferenceGeneration {
    pub const fn from_recorded(value: u64) -> Self {
        Self(value)
    }
    pub const fn get(self) -> u64 {
        self.0
    }
}
impl identity_seal::Sealed for DurableProductReferenceGeneration {}
impl DurableIdentity for DurableProductReferenceGeneration {}
