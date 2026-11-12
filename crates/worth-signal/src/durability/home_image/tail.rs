//! A selected image prefix and one protected original tail supply one source.
use super::*;
use crate::durability::SignalDurableMediumDenial as M;
struct ImageTail {
    image: SignalAuthoredHomeImage,
    tail: Box<dyn SignalDurableReadSource>,
    bounds: SignalRecordedIdentityContribution,
    _memory: Option<ExecutionMemoryReservation>,
}
impl std::fmt::Debug for ImageTail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalImageTail")
            .field("owner", &self.image.owner())
            .finish()
    }
}
impl SignalAuthoredHomeImage {
    pub fn with_original_tail(
        &self,
        tail: Box<dyn SignalDurableReadSource>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn SignalDurableReadSource>, M> {
        if tail.owner() != self.owner() {
            return Err(M::ForeignOwner);
        }
        for journal in [
            SignalDurableJournal::Definitions,
            SignalDurableJournal::Advances,
        ] {
            let observed = tail.frontier(journal, execution)?;
            let checkpoint = self.covered_through(journal);
            if observed < checkpoint {
                return Err(M::CheckpointSourceBehind {
                    journal,
                    checkpoint,
                    observed,
                });
            }
        }
        let mut bounds = self.source.bounds.clone();
        bounds
            .merge(&tail.identity_contribution(), execution)
            .map_err(|cause| M::CheckpointBounds(Box::new(cause)))?;
        let memory = execution
            .reserve_retained_memory(std::mem::size_of::<ImageTail>() as u64)
            .map_err(M::Request)?;
        Ok(Box::new(ImageTail {
            image: self.clone(),
            tail,
            bounds,
            _memory: memory,
        }))
    }
}
impl SignalDurableReadSource for ImageTail {
    fn owner(&self) -> DurableSignalOwnerIdentity {
        self.image.owner()
    }
    fn identity_contribution(&self) -> SignalRecordedIdentityContribution {
        self.bounds.clone()
    }
    fn frontier(
        &self,
        journal: SignalDurableJournal,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<u64, M> {
        self.tail.frontier(journal, execution)
    }
    fn read(
        &self,
        journal: SignalDurableJournal,
        position: u64,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ExecutionImmutableBytes, M> {
        if position <= self.image.covered_through(journal) {
            self.image.read(journal, position, execution)
        } else {
            self.tail.read(journal, position, execution)
        }
    }
}
