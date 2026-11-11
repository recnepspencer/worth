//! A stable node coordinate belongs to one Home-scoped Signal branch.
use super::super::{identity_seal, DurableIdentity};
use super::DurableSignalBranchIdentity;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableSignalNodeIdentity {
    branch: DurableSignalBranchIdentity,
    index: u32,
    generation: u32,
}
impl DurableSignalNodeIdentity {
    pub const fn from_recorded(
        branch: DurableSignalBranchIdentity,
        index: u32,
        generation: u32,
    ) -> Self {
        Self {
            branch,
            index,
            generation,
        }
    }
    pub const fn branch(self) -> DurableSignalBranchIdentity {
        self.branch
    }
    pub const fn index(self) -> u32 {
        self.index
    }
    pub const fn generation(self) -> u32 {
        self.generation
    }
}
impl identity_seal::Sealed for DurableSignalNodeIdentity {}
impl DurableIdentity for DurableSignalNodeIdentity {}
