use std::sync::Arc;

use crate::durability::data::DurableCheckpoint;
#[cfg(test)]
use crate::history::data::PositionedCanonicalCommit;
use crate::runtime::RelationalRuntime;

/// Durable truth read through shared ownership: each accessor carries its
/// answer out of the durability lock rather than lending a borrow into it.
impl RelationalRuntime {
    pub(crate) fn durable_checkpoints(&self) -> Vec<Arc<DurableCheckpoint>> {
        self.durability.checkpoints()
    }

    #[cfg(test)]
    pub(crate) fn durable_log(&self) -> Vec<Arc<PositionedCanonicalCommit>> {
        self.durability.log()
    }
}
