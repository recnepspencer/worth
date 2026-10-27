use super::*;
use crate::data::temporal::{TemporalFrontierSnapshot, TemporalWakeSummary};

impl<D, I, T> SignalBranchCellState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(in crate::branch::owner_services) fn owned_async_wake_frontier_for_test(
        &mut self,
        node: ResourceNodeId,
    ) -> Result<
        (TemporalWakeSummary, TemporalFrontierSnapshot),
        SignalConditionalServiceExecutionDenial,
    > {
        self.state.owned_async_wake_frontier_for_test(node)
    }
}
