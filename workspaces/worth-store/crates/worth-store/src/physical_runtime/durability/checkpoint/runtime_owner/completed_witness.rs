use std::num::NonZeroU64;

use super::PhysicalCheckpointRuntimeOwner;

/// Minted only from a namespace-durable checkpoint selected at reopen or
/// completed by this runtime. Issued candidate sequences are not witnesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct CompletedDurableCheckpointWitness(NonZeroU64);

impl CompletedDurableCheckpointWitness {
    pub(in crate::physical_runtime) const fn sequence(self) -> NonZeroU64 {
        self.0
    }
}

impl PhysicalCheckpointRuntimeOwner {
    /// Unfinished and indeterminate checkpoint attempts never advance it.
    pub(in crate::physical_runtime) fn selected_completed_checkpoint(
        &self,
    ) -> Option<CompletedDurableCheckpointWitness> {
        let sequence = self
            .state()
            .latest_publication
            .as_ref()
            .map(|publication| publication.basis().identity().sequence().get())
            .unwrap_or(self.selected_checkpoint_sequence);
        NonZeroU64::new(sequence).map(CompletedDurableCheckpointWitness)
    }
}
