use std::ffi::{OsStr, OsString};
use worth_store_physical_format::store_namespace::{NamespaceEntryType, StableStoreIdentity};

#[derive(Debug, PartialEq, Eq)]
pub struct ObservedWalArtifact {
    pub(super) store: StableStoreIdentity,
    pub(super) observation: RecoveryWalObservationIdentity,
    pub(super) name: OsString,
    pub(super) entry_type: NamespaceEntryType,
    pub(super) bytes: Option<Vec<u8>>,
}

/// One C.4 WAL read within one admitted recovery-media generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecoveryWalObservationIdentity {
    pub(super) media_generation: super::super::PhysicalRecoveryMediaGeneration,
    pub(super) discovery_incarnation: u64,
    pub(super) sequence: u64,
}

impl ObservedWalArtifact {
    pub const fn store_identity(&self) -> StableStoreIdentity {
        self.store
    }
    pub const fn observation_identity(&self) -> RecoveryWalObservationIdentity {
        self.observation
    }
    pub fn matches_media_generation(
        &self,
        generation: super::super::PhysicalRecoveryMediaGeneration,
    ) -> bool {
        self.observation.media_generation == generation
    }
    pub fn name(&self) -> &OsStr {
        &self.name
    }
    pub fn name_heap_bytes(&self) -> usize {
        self.name.capacity()
    }
    pub const fn entry_type(&self) -> NamespaceEntryType {
        self.entry_type
    }
    pub fn bytes(&self) -> Option<&[u8]> {
        self.bytes.as_deref()
    }
}
