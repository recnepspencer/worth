use worth_store_physical_format::PersistedRecordIdentity;

use super::{PhysicalCurrentRootOwner, PhysicalMutationIdentity, ReclaimPhase};

impl PhysicalCurrentRootOwner {
    pub(in crate::physical_runtime) fn reclaim_drops_for(
        &self,
        mutation: PhysicalMutationIdentity,
    ) -> Option<Vec<PersistedRecordIdentity>> {
        let fence = self.lock_reclaim();
        fence.as_ref().and_then(|fence| {
            (fence.drop_mutation == Some(mutation)
                && matches!(
                    fence.phase,
                    ReclaimPhase::ReservePublished | ReclaimPhase::DropEffect
                )
                && !fence.drop_records.is_empty())
            .then(|| fence.drop_records.clone())
        })
    }

    /// Copies the bounded, pre-admitted set only for the registered descriptor
    /// member. The source is the post-reservation root that the drop displaces.
    pub(in crate::physical_runtime) fn note_reclaim_displaced_batch(
        &self,
        mutation: PhysicalMutationIdentity,
    ) -> bool {
        let fence = self.lock_reclaim();
        let Some(fence) = fence.as_ref().filter(|fence| {
            fence.drop_mutation == Some(mutation)
                && matches!(
                    fence.phase,
                    ReclaimPhase::ReservePublished | ReclaimPhase::DropEffect
                )
                && !fence.drop_records.is_empty()
        }) else {
            return false;
        };
        let mut displaced = Vec::new();
        if displaced.try_reserve_exact(fence.displaced.len()).is_err() {
            return false;
        }
        displaced.extend(fence.displaced.iter().map(|entry| {
            crate::physical_runtime::durability::DisplacedArtifact {
                source_root: fence.expected_root.generation().get(),
                artifact: entry.artifact,
                bytes: entry.bytes,
            }
        }));
        let mut notes = self
            .displaced
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if notes.contains_key(&mutation) || notes.try_reserve(1).is_err() {
            return false;
        }
        notes.insert(mutation, displaced);
        true
    }
}
