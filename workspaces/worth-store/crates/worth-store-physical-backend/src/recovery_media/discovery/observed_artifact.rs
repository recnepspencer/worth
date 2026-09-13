use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::RecoveryDiscoveryArtifact;

/// One bounded C4 read, including the actual owner, locator, and file offset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedRecoveryArtifact {
    store: StableStoreIdentity,
    artifact: RecoveryDiscoveryArtifact,
    offset: u64,
    bytes: Option<Vec<u8>>,
}

impl ObservedRecoveryArtifact {
    pub(super) fn new(
        store: StableStoreIdentity,
        artifact: RecoveryDiscoveryArtifact,
        offset: u64,
        bytes: Option<Vec<u8>>,
    ) -> Self {
        Self {
            store,
            artifact,
            offset,
            bytes,
        }
    }

    pub const fn store_identity(&self) -> StableStoreIdentity {
        self.store
    }

    pub const fn artifact(&self) -> &RecoveryDiscoveryArtifact {
        &self.artifact
    }

    pub const fn offset(&self) -> u64 {
        self.offset
    }

    pub fn bytes(&self) -> Option<&[u8]> {
        self.bytes.as_deref()
    }

    pub fn into_bytes(self) -> Option<Vec<u8>> {
        self.bytes
    }
}
