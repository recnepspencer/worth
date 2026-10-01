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
    pub(in crate::recovery_media) fn new(
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

    /// Heap backing retained by this observation, excluding the inline wrapper.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.bytes
            .as_ref()
            .map_or(Some(0), |bytes| u64::try_from(bytes.capacity()).ok())
    }

    pub fn into_bytes(self) -> Option<Vec<u8>> {
        self.bytes
    }
}
