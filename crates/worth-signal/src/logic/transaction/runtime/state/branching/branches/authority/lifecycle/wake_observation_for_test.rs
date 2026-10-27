use super::*;
use crate::data::temporal::{TemporalFrontierSnapshot, TemporalWakeSummary};

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn owned_async_wake_frontier_for_test(
        &mut self,
        node: ResourceNodeId,
    ) -> Result<
        (TemporalWakeSummary, TemporalFrontierSnapshot),
        SignalConditionalServiceExecutionDenial,
    > {
        self.admit_owned_async_source(node)?;
        Ok((
            self.derived.temporal.wake_summary(),
            self.derived.temporal.frontier_snapshot(),
        ))
    }
}
