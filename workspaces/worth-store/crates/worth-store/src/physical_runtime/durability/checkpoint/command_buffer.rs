//! Pre-admitted command storage, exclusively writable between executor settlements.

use std::sync::Arc;

use crate::physical_runtime::durability::FundedCheckpointBufferPreparation;

pub(in crate::physical_runtime) struct FundedCheckpointCommandBuffer {
    storage: Arc<FundedCheckpointBufferPreparation>,
}

pub(in crate::physical_runtime) struct FundedCheckpointCommandFrame {
    storage: Arc<FundedCheckpointBufferPreparation>,
}

impl FundedCheckpointCommandBuffer {
    /// The preparation owner funds both the byte capacity and Arc allocation.
    pub(in crate::physical_runtime) fn from_prepared(
        preparation: FundedCheckpointBufferPreparation,
    ) -> Self {
        Self {
            storage: Arc::new(preparation),
        }
    }

    pub(in crate::physical_runtime) fn bytes_mut(&mut self) -> Option<&mut Vec<u8>> {
        Arc::get_mut(&mut self.storage).map(FundedCheckpointBufferPreparation::bytes_mut)
    }

    pub(in crate::physical_runtime) fn frame(&self) -> FundedCheckpointCommandFrame {
        FundedCheckpointCommandFrame {
            storage: Arc::clone(&self.storage),
        }
    }
}

impl FundedCheckpointCommandFrame {
    pub(in crate::physical_runtime) fn bytes(&self) -> &[u8] {
        self.storage.bytes()
    }
}
