use std::sync::Arc;

use crate::durability::data::DurableCheckpoint;
use crate::history::data::PositionedCanonicalCommit;
use crate::runtime::RelationalRuntime;

/// Durable truth read through shared ownership: each accessor carries its
/// answer out of the durability lock rather than lending a borrow into it.
pub(crate) trait DurabilityRead {
    fn durable_checkpoints(&self) -> Vec<Arc<DurableCheckpoint>>;
    fn durable_log(&self) -> Vec<Arc<PositionedCanonicalCommit>>;
}

impl DurabilityRead for RelationalRuntime {
    fn durable_checkpoints(&self) -> Vec<Arc<DurableCheckpoint>> {
        self.durability.checkpoints()
    }

    fn durable_log(&self) -> Vec<Arc<PositionedCanonicalCommit>> {
        self.durability.log()
    }
}
