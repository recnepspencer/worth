use super::super::{identity_seal, DurableIdentity};
use super::DurableSignalOwnerIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableSignalBranchIdentity {
    owner: DurableSignalOwnerIdentity,
    ordinal: u64,
}
impl DurableSignalBranchIdentity {
    pub const fn from_recorded(owner: DurableSignalOwnerIdentity, ordinal: u64) -> Self {
        Self { owner, ordinal }
    }
    pub const fn owner(self) -> DurableSignalOwnerIdentity {
        self.owner
    }
    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }
}
impl identity_seal::Sealed for DurableSignalBranchIdentity {}
impl DurableIdentity for DurableSignalBranchIdentity {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableSignalCellIncarnation {
    branch: DurableSignalBranchIdentity,
    ordinal: u64,
}
impl DurableSignalCellIncarnation {
    pub const fn from_recorded(branch: DurableSignalBranchIdentity, ordinal: u64) -> Self {
        Self { branch, ordinal }
    }
    pub const fn branch(self) -> DurableSignalBranchIdentity {
        self.branch
    }
    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }
}
impl identity_seal::Sealed for DurableSignalCellIncarnation {}
impl DurableIdentity for DurableSignalCellIncarnation {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableSignalGeneration {
    cell: DurableSignalCellIncarnation,
    ordinal: u64,
}
impl DurableSignalGeneration {
    pub const fn from_recorded(cell: DurableSignalCellIncarnation, ordinal: u64) -> Self {
        Self { cell, ordinal }
    }
    pub const fn cell(self) -> DurableSignalCellIncarnation {
        self.cell
    }
    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }
}
impl identity_seal::Sealed for DurableSignalGeneration {}
impl DurableIdentity for DurableSignalGeneration {}
