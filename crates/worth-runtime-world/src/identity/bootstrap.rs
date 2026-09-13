use super::RuntimeWorldOwnerIdentity;

/// The one owner-issued occurrence that may establish a Runtime World root.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RuntimeWorldBootstrapAttemptIdentity {
    owner: RuntimeWorldOwnerIdentity,
    ordinal: u64,
}

impl RuntimeWorldBootstrapAttemptIdentity {
    pub(super) const fn issued(owner: RuntimeWorldOwnerIdentity, ordinal: u64) -> Self {
        Self { owner, ordinal }
    }

    pub const fn owner_identity(&self) -> RuntimeWorldOwnerIdentity {
        self.owner
    }
}
