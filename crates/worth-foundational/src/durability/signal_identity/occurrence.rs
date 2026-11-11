use super::super::{identity_seal, DurableIdentity};
use super::DurableSignalOwnerIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableSignalSnapshotIdentity {
    owner: DurableSignalOwnerIdentity,
    ordinal: u64,
}
impl DurableSignalSnapshotIdentity {
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
impl identity_seal::Sealed for DurableSignalSnapshotIdentity {}
impl DurableIdentity for DurableSignalSnapshotIdentity {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableSignalAdmissionIdentity {
    owner: DurableSignalOwnerIdentity,
    ordinal: u64,
}
impl DurableSignalAdmissionIdentity {
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
impl identity_seal::Sealed for DurableSignalAdmissionIdentity {}
impl DurableIdentity for DurableSignalAdmissionIdentity {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableSignalConditionalOccurrence {
    owner: DurableSignalOwnerIdentity,
    ordinal: u64,
}
impl DurableSignalConditionalOccurrence {
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
impl identity_seal::Sealed for DurableSignalConditionalOccurrence {}
impl DurableIdentity for DurableSignalConditionalOccurrence {}
