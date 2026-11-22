//! Actual heap backing retained by immutable recovery plans.
//! Inline plan values and transient construction buffers are not charged here.

use super::{
    RecoveryBaseImagePlan, RecoveryPublicationExpectation, RecoveryPublicationPlan,
    RecoveryReleaseTopologyProof, RecoveryStagingLayoutPlan,
};

fn boxed_bytes<T>(entries: &[T]) -> Option<u64> {
    u64::try_from(entries.len())
        .ok()?
        .checked_mul(u64::try_from(std::mem::size_of::<T>()).ok()?)
}

impl RecoveryBaseImagePlan {
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        let body = self.body();
        let mut bytes = boxed_bytes(&self.inner)?
            .checked_add(boxed_bytes(&body.selected_root_topology)?)?
            .checked_add(boxed_bytes(&body.actions)?)?
            .checked_add(boxed_bytes(&body.segment_updates)?)?
            .checked_add(boxed_bytes(&body.manifests)?)?
            .checked_add(boxed_bytes(&body.root_states)?)?
            .checked_add(boxed_bytes(&body.source_artifacts)?)?;
        for (_, block) in body.selected_root_topology.iter() {
            bytes = bytes.checked_add(block.owned_heap_bytes()?)?;
        }
        for state in body.root_states.iter() {
            bytes = bytes.checked_add(state.owned_heap_bytes()?)?;
        }
        if let Some(replay) = &body.release_head_replay {
            bytes = bytes.checked_add(replay.owned_heap_bytes()?)?;
        }
        Some(bytes)
    }
}

impl RecoveryStagingLayoutPlan {
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        let mut bytes = self
            .base
            .owned_heap_bytes()?
            .checked_add(boxed_bytes(&self.actions)?)?
            .checked_add(boxed_bytes(&self.commands)?)?
            .checked_add(boxed_bytes(&self.source_copies)?)?
            .checked_add(boxed_bytes(&self.allocated_targets)?)?;
        for action in self.actions.iter() {
            bytes = bytes.checked_add(boxed_bytes(&action.steps)?)?;
        }
        for command in self.commands.iter() {
            bytes = bytes.checked_add(boxed_bytes(&command.bytes)?)?;
        }
        // PersistedExtentCopyRecipe is Copy and has no nested heap backing.
        Some(bytes)
    }
}

impl RecoveryReleaseTopologyProof {
    fn owned_heap_bytes(&self) -> Option<u64> {
        self.transition.owned_heap_bytes()
    }
}

impl RecoveryPublicationPlan {
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        let body = self.body();
        let mut bytes = boxed_bytes(&self.inner)?
            .checked_add(body.expectation_backing.backing_bytes()?)?
            .checked_add(boxed_bytes(&body.actions)?)?
            .checked_add(boxed_bytes(&body.referenced_artifacts)?)?
            .checked_add(boxed_bytes(&body.candidates)?)?
            .checked_add(boxed_bytes(&body.created_artifacts)?)?;
        for candidate in body.candidates.iter() {
            bytes = bytes.checked_add(boxed_bytes(&candidate.bytes)?)?;
        }
        if let Some(release) = &body.release_topology {
            bytes = bytes.checked_add(release.owned_heap_bytes()?)?;
        }
        Some(bytes)
    }
}

impl RecoveryPublicationExpectation {
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        let mut bytes = self
            .backing
            .backing_bytes()?
            .checked_add(boxed_bytes(&self.payload().referenced_artifacts)?)?
            .checked_add(boxed_bytes(&self.payload().created_artifacts)?)?;
        if let Some(release) = &self.payload().release_topology {
            bytes = bytes.checked_add(release.owned_heap_bytes()?)?;
        }
        Some(bytes)
    }
}
