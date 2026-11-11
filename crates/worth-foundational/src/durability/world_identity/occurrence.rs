use super::super::{identity_seal, DurableIdentity};
use super::DurableWorldOwnerIdentity;

/// Recorded World occurrence data. Its ordinal is never a live issuance grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableProductBranchIncarnation {
    owner: DurableWorldOwnerIdentity,
    ordinal: u64,
}
impl DurableProductBranchIncarnation {
    pub const fn from_recorded(owner: DurableWorldOwnerIdentity, ordinal: u64) -> Self {
        Self { owner, ordinal }
    }
    pub const fn owner(self) -> DurableWorldOwnerIdentity {
        self.owner
    }
    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }
}
impl identity_seal::Sealed for DurableProductBranchIncarnation {}
impl DurableIdentity for DurableProductBranchIncarnation {}

/// Recorded World occurrence data. Its ordinal is never a live issuance grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableCompositeCommitIdentity {
    owner: DurableWorldOwnerIdentity,
    ordinal: u64,
}
impl DurableCompositeCommitIdentity {
    pub const fn from_recorded(owner: DurableWorldOwnerIdentity, ordinal: u64) -> Self {
        Self { owner, ordinal }
    }
    pub const fn owner(self) -> DurableWorldOwnerIdentity {
        self.owner
    }
    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }
}
impl identity_seal::Sealed for DurableCompositeCommitIdentity {}
impl DurableIdentity for DurableCompositeCommitIdentity {}

/// Recorded World occurrence data. Its ordinal is never a live issuance grant.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct DurableCompositePublicationAttemptIdentity {
    owner: DurableWorldOwnerIdentity,
    ordinal: u64,
}
impl DurableCompositePublicationAttemptIdentity {
    pub const fn from_recorded(owner: DurableWorldOwnerIdentity, ordinal: u64) -> Self {
        Self { owner, ordinal }
    }
    pub const fn owner(self) -> DurableWorldOwnerIdentity {
        self.owner
    }
    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }
}
impl identity_seal::Sealed for DurableCompositePublicationAttemptIdentity {}
impl DurableIdentity for DurableCompositePublicationAttemptIdentity {}

/// Recorded World occurrence data. Its ordinal is never a live issuance grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableWorldBootstrapAttemptIdentity {
    owner: DurableWorldOwnerIdentity,
    ordinal: u64,
}
impl DurableWorldBootstrapAttemptIdentity {
    pub const fn from_recorded(owner: DurableWorldOwnerIdentity, ordinal: u64) -> Self {
        Self { owner, ordinal }
    }
    pub const fn owner(self) -> DurableWorldOwnerIdentity {
        self.owner
    }
    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }
}
impl identity_seal::Sealed for DurableWorldBootstrapAttemptIdentity {}
impl DurableIdentity for DurableWorldBootstrapAttemptIdentity {}

/// Recorded World occurrence data. Its ordinal is never a live issuance grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DurableProductUnpublishedOwnerEffectsIdentity {
    owner: DurableWorldOwnerIdentity,
    ordinal: u64,
}
impl DurableProductUnpublishedOwnerEffectsIdentity {
    pub const fn from_recorded(owner: DurableWorldOwnerIdentity, ordinal: u64) -> Self {
        Self { owner, ordinal }
    }
    pub const fn owner(self) -> DurableWorldOwnerIdentity {
        self.owner
    }
    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }
}
impl identity_seal::Sealed for DurableProductUnpublishedOwnerEffectsIdentity {}
impl DurableIdentity for DurableProductUnpublishedOwnerEffectsIdentity {}
