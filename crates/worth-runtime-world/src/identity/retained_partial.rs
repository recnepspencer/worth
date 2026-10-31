use super::RuntimeWorldOwnerIdentity;

/// Identity of one retained owner-effect record whose product reference did
/// not move.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProductUnpublishedOwnerEffectsIdentity {
    owner: RuntimeWorldOwnerIdentity,
    ordinal: u64,
}

impl ProductUnpublishedOwnerEffectsIdentity {
    pub(crate) const fn ordinal(&self) -> u64 {
        self.ordinal
    }
    pub(super) const fn issued(owner: RuntimeWorldOwnerIdentity, ordinal: u64) -> Self {
        Self { owner, ordinal }
    }

    pub const fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }
}
