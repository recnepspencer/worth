use super::RuntimeWorldOwnerIdentity;

/// Owner-issued occurrence identity of one immutable composite commit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompositeCommitIdentity {
    owner: RuntimeWorldOwnerIdentity,
    ordinal: u64,
}

impl CompositeCommitIdentity {
    pub(super) const fn issued(owner: RuntimeWorldOwnerIdentity, ordinal: u64) -> Self {
        Self { owner, ordinal }
    }

    pub const fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }

    /// Descriptive occurrence ordinal; this value cannot issue a commit.
    pub const fn ordinal(&self) -> u64 {
        self.ordinal
    }
}
