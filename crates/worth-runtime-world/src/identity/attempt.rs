use super::RuntimeWorldOwnerIdentity;

/// Owner-issued identity of one bounded composite publication attempt.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompositePublicationAttemptIdentity {
    owner: RuntimeWorldOwnerIdentity,
    ordinal: u64,
}

impl CompositePublicationAttemptIdentity {
    pub(super) const fn issued(owner: RuntimeWorldOwnerIdentity, ordinal: u64) -> Self {
        Self { owner, ordinal }
    }

    pub const fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }

    /// Descriptive attempt ordinal; this value cannot admit publication.
    pub const fn ordinal(&self) -> u64 {
        self.ordinal
    }
}
