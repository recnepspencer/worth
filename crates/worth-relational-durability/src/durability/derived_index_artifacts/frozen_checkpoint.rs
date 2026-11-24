//! One observation of the resident index roster and its original cold member.
use super::cold_checkpoint::DeferredIndexImage;
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::checkpoint_inputs::FrozenCheckpointIndexes;
use std::sync::Arc;

pub struct FrozenIndexCheckpoint {
    indexes: FrozenCheckpointIndexes,
    original: Option<Arc<DeferredIndexImage>>,
}

impl FrozenIndexCheckpoint {
    pub fn new(
        indexes: FrozenCheckpointIndexes,
        original: Option<Arc<DeferredIndexImage>>,
    ) -> Self {
        Self { indexes, original }
    }

    pub(crate) fn into_parts(self) -> (FrozenCheckpointIndexes, Option<Arc<DeferredIndexImage>>) {
        (self.indexes, self.original)
    }
}
