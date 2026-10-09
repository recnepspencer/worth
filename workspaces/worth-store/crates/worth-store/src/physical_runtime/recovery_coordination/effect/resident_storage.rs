//! Heap retained by physical recovery occurrences, excluding inline evidence.

use super::{ActionMarker, PerformedRecoveryPhysicalEffect, RecoveryPhysicalEffectOccurrence};

impl<Action: ActionMarker> PerformedRecoveryPhysicalEffect<Action> {
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        self.occurrence().owned_heap_bytes()
    }
}

impl RecoveryPhysicalEffectOccurrence {
    fn owned_heap_bytes(&self) -> Option<u64> {
        match self {
            Self::StagingWrite(value) => value.physical.owned_heap_bytes(),
            Self::StagingSynchronization(value) => value.physical.owned_heap_bytes(),
            Self::PublicationCandidateMaterialization(value) => value.physical.owned_heap_bytes(),
            Self::PublicationCandidateSynchronization(value) => value.physical.owned_heap_bytes(),
            Self::FreshReopen(value) => value
                .selector
                .owned_heap_bytes()?
                .checked_add(value.root.owned_heap_bytes()?),
            Self::CleanupRemoval(_) => Some(0),
            Self::RootProtocolReplacement(value) | Self::RecordNamespaceSynchronization(value) => {
                value.physical.owned_heap_bytes()
            }
        }
    }
}
