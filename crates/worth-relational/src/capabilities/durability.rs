use std::sync::Arc;

use crate::durability::data::{DurabilityError, DurableCheckpoint, DurableStore};
use crate::history::data::PositionedCanonicalCommit;
use crate::runtime::RelationalRuntime;

/// Durable truth read through shared ownership: each accessor carries its
/// answer out of the durability lock rather than lending a borrow into it.
pub(crate) trait DurabilityRead {
    fn durable_checkpoints(&self) -> Vec<Arc<DurableCheckpoint>>;
    fn durable_store(&self) -> Option<Arc<DurableStore>>;
    fn durable_log(&self) -> Vec<Arc<PositionedCanonicalCommit>>;
}

impl DurabilityRead for RelationalRuntime {
    fn durable_checkpoints(&self) -> Vec<Arc<DurableCheckpoint>> {
        self.durability.checkpoints()
    }

    fn durable_store(&self) -> Option<Arc<DurableStore>> {
        self.durability.store()
    }

    fn durable_log(&self) -> Vec<Arc<PositionedCanonicalCommit>> {
        self.durability.log()
    }
}

pub(crate) trait DurabilityWrite {
    fn append_durable_envelope(
        &self,
        authority: crate::durability::authority::DurableAppendAuthority,
        commit: &PositionedCanonicalCommit,
        bytes: Option<&worth_execution::ExecutionImmutableBytes>,
    ) -> Result<(), DurabilityError>;
}

impl DurabilityWrite for RelationalRuntime {
    fn append_durable_envelope(
        &self,
        authority: crate::durability::authority::DurableAppendAuthority,
        commit: &PositionedCanonicalCommit,
        bytes: Option<&worth_execution::ExecutionImmutableBytes>,
    ) -> Result<(), DurabilityError> {
        self.durability_authority()
            .append_commit(authority, commit, bytes)
    }
}
