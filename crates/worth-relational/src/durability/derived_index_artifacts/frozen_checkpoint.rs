//! One observation of the resident index roster and its original cold member.
use super::cold_checkpoint::DeferredIndexImage;
use crate::runtime::state::subsystems::indexing::FrozenCheckpointIndexes;
use std::sync::Arc;

pub(crate) struct FrozenIndexCheckpoint {
    indexes: FrozenCheckpointIndexes,
    original: Option<Arc<DeferredIndexImage>>,
}

impl FrozenIndexCheckpoint {
    pub(crate) fn new(
        indexes: FrozenCheckpointIndexes,
        original: Option<Arc<DeferredIndexImage>>,
    ) -> Self {
        Self { indexes, original }
    }

    pub(crate) fn into_parts(self) -> (FrozenCheckpointIndexes, Option<Arc<DeferredIndexImage>>) {
        (self.indexes, self.original)
    }
}
