use super::super::{identity_seal, DurableIdentity};
use super::DurableSignalOwnerIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SignalDefinitionTransitionIdentity {
    owner: DurableSignalOwnerIdentity,
    ordinal: u64,
}
impl SignalDefinitionTransitionIdentity {
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
impl identity_seal::Sealed for SignalDefinitionTransitionIdentity {}
impl DurableIdentity for SignalDefinitionTransitionIdentity {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SignalBranchAdvanceIdentity {
    owner: DurableSignalOwnerIdentity,
    ordinal: u64,
}
impl SignalBranchAdvanceIdentity {
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
impl identity_seal::Sealed for SignalBranchAdvanceIdentity {}
impl DurableIdentity for SignalBranchAdvanceIdentity {}

/// Content-addressed reference to one exact Signal definition publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SignalDefinitionTransitionReference {
    identity: SignalDefinitionTransitionIdentity,
    digest: [u8; 32],
}
impl SignalDefinitionTransitionReference {
    pub const fn from_recorded(
        identity: SignalDefinitionTransitionIdentity,
        digest: [u8; 32],
    ) -> Self {
        Self { identity, digest }
    }
    pub const fn identity(self) -> SignalDefinitionTransitionIdentity {
        self.identity
    }
    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}
impl identity_seal::Sealed for SignalDefinitionTransitionReference {}
impl DurableIdentity for SignalDefinitionTransitionReference {}
